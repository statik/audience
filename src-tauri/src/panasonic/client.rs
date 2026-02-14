use crate::ptz::controller::{PtzController, PtzError};
use crate::ptz::types::PtzPosition;
use async_trait::async_trait;

/// Panasonic AW protocol client using HTTP CGI commands.
/// Supports AW-UE150, AW-UE100, AW-UE70, AW-UE50, AW-UE40, AW-UE20, etc.
pub struct PanasonicClient {
    base_url: String,
    client: reqwest::Client,
}

impl PanasonicClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            base_url: format!("http://{}:{}", host, port),
            client: reqwest::Client::new(),
        }
    }

    async fn send_ptz_command(&self, cmd: &str) -> Result<String, PtzError> {
        let url = format!(
            "{}/cgi-bin/aw_ptz?cmd=%23{}&res=1",
            self.base_url, cmd
        );

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| PtzError::ConnectionFailed(e.to_string()))?;

        let text = response
            .text()
            .await
            .map_err(|e| PtzError::CommandFailed(e.to_string()))?;

        Ok(text)
    }

    /// Convert normalized pan (-1.0 to 1.0) to Panasonic hex value.
    /// Panasonic range: 0x0001 to 0xFFFF, center at 0x8000.
    fn normalize_to_pan_hex(normalized: f64) -> String {
        let clamped = normalized.clamp(-1.0, 1.0);
        let value = ((clamped + 1.0) / 2.0 * 0xFFFE as f64) as u16 + 1;
        format!("{:04X}", value)
    }

    /// Convert normalized tilt (-1.0 to 1.0) to Panasonic hex value.
    /// Panasonic range: 0x0001 to 0xFFFF, center at 0x8000.
    fn normalize_to_tilt_hex(normalized: f64) -> String {
        let clamped = normalized.clamp(-1.0, 1.0);
        let value = ((clamped + 1.0) / 2.0 * 0xFFFE as f64) as u16 + 1;
        format!("{:04X}", value)
    }

    /// Convert normalized zoom (0.0 to 1.0) to Panasonic hex value.
    /// Panasonic range: 0x555 to 0xFFF.
    fn normalize_to_zoom_hex(normalized: f64) -> String {
        let clamped = normalized.clamp(0.0, 1.0);
        let value = (0x555 as f64 + clamped * (0xFFF - 0x555) as f64) as u16;
        format!("{:03X}", value)
    }

    /// Convert normalized speed to Panasonic speed value (01-99, 50=stop).
    fn delta_to_speed(delta: f64) -> String {
        if delta.abs() < 0.01 {
            "50".to_string()
        } else {
            let speed = if delta > 0.0 {
                50.0 + delta.abs() * 49.0
            } else {
                50.0 - delta.abs() * 49.0
            };
            format!("{:02}", speed.clamp(1.0, 99.0) as u8)
        }
    }
}

#[async_trait]
impl PtzController for PanasonicClient {
    async fn move_absolute(&self, pan: f64, tilt: f64, zoom: f64) -> Result<(), PtzError> {
        let pan_hex = Self::normalize_to_pan_hex(pan);
        let tilt_hex = Self::normalize_to_tilt_hex(tilt);
        let zoom_hex = Self::normalize_to_zoom_hex(zoom);

        // Absolute pan/tilt: #APS[pan][tilt][speed]
        let cmd = format!("APS{}{}30", pan_hex, tilt_hex);
        self.send_ptz_command(&cmd).await?;

        // Zoom: #Z[position]
        let zoom_cmd = format!("Z{}", zoom_hex);
        self.send_ptz_command(&zoom_cmd).await?;

        Ok(())
    }

    async fn move_relative(&self, pan_delta: f64, tilt_delta: f64) -> Result<(), PtzError> {
        let pan_speed = Self::delta_to_speed(pan_delta);
        let tilt_speed = Self::delta_to_speed(tilt_delta);

        // Pan speed command: #P[speed]
        let pan_cmd = format!("P{}", pan_speed);
        self.send_ptz_command(&pan_cmd).await?;

        // Tilt speed command: #T[speed]
        let tilt_cmd = format!("T{}", tilt_speed);
        self.send_ptz_command(&tilt_cmd).await?;

        // Brief movement then stop
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Stop: #PTS5050
        self.send_ptz_command("PTS5050").await?;

        Ok(())
    }

    async fn zoom_to(&self, zoom: f64) -> Result<(), PtzError> {
        let zoom_hex = Self::normalize_to_zoom_hex(zoom);
        let cmd = format!("Z{}", zoom_hex);
        self.send_ptz_command(&cmd).await?;
        Ok(())
    }

    async fn recall_preset(&self, preset_index: u8) -> Result<(), PtzError> {
        let cmd = format!("R{:02}", preset_index);
        self.send_ptz_command(&cmd).await?;
        Ok(())
    }

    async fn store_preset(&self, preset_index: u8) -> Result<(), PtzError> {
        let cmd = format!("M{:02}", preset_index);
        self.send_ptz_command(&cmd).await?;
        Ok(())
    }

    async fn get_position(&self) -> Result<PtzPosition, PtzError> {
        // Query current position: #APC returns current pan/tilt
        let response = self.send_ptz_command("APC").await?;

        // Parse response — format: "aPC[pan_hex][tilt_hex]"
        // For now return default; real implementation would parse
        let _ = response;
        Ok(PtzPosition {
            pan: 0.0,
            tilt: 0.0,
            zoom: 0.0,
        })
    }

    async fn test_connection(&self) -> Result<(), PtzError> {
        // Query position as connectivity test
        self.send_ptz_command("APC").await?;
        Ok(())
    }
}
