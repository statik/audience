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
    pub fn new(host: &str, port: u16) -> Result<Self, PtzError> {
        crate::ptz::types::validate_host(host).map_err(PtzError::ConnectionFailed)?;
        Ok(Self {
            base_url: format!("http://{}:{}", host, port),
            client: reqwest::Client::new(),
        })
    }

    async fn send_ptz_command(&self, cmd: &str) -> Result<String, PtzError> {
        let url = format!("{}/cgi-bin/aw_ptz", self.base_url);
        let cmd_with_prefix = format!("#{}", cmd);

        let response = self
            .client
            .get(&url)
            .query(&[("cmd", &cmd_with_prefix), ("res", &"1".to_string())])
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
            // Map delta to speed where 50=stop, >50=one direction, <50=opposite.
            // Use ranges 51-99 and 1-49 to avoid the stop value (50).
            let speed = if delta > 0.0 {
                51.0 + delta.abs() * 48.0
            } else {
                49.0 - delta.abs() * 48.0
            };
            format!("{:02}", speed.round().clamp(1.0, 99.0) as u8)
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
        let pt_response = self.send_ptz_command("APC").await?;
        let z_response = self.send_ptz_command("GZ").await?;

        // Parse "aPC[PPPPTTTT]" — 4 hex chars pan, 4 hex chars tilt
        let (pan, tilt) = if pt_response.starts_with("aPC") && pt_response.len() >= 11 {
            let pan_hex = &pt_response[3..7];
            let tilt_hex = &pt_response[7..11];
            let pan_val = u16::from_str_radix(pan_hex, 16)
                .map_err(|e| PtzError::ProtocolError(e.to_string()))?;
            let tilt_val = u16::from_str_radix(tilt_hex, 16)
                .map_err(|e| PtzError::ProtocolError(e.to_string()))?;
            // Reverse: val = ((norm+1)/2 * 0xFFFE) + 1
            let pan_norm = (pan_val as f64 - 1.0) / 0xFFFE_u16 as f64 * 2.0 - 1.0;
            let tilt_norm = (tilt_val as f64 - 1.0) / 0xFFFE_u16 as f64 * 2.0 - 1.0;
            (pan_norm, tilt_norm)
        } else {
            return Err(PtzError::ProtocolError(format!(
                "Invalid APC response: {pt_response}"
            )));
        };

        // Parse "gz[ZZZ]" — 3 hex chars zoom
        let zoom = if z_response.starts_with("gz") && z_response.len() >= 5 {
            let zoom_hex = &z_response[2..5];
            let zoom_val = u16::from_str_radix(zoom_hex, 16)
                .map_err(|e| PtzError::ProtocolError(e.to_string()))?;
            (zoom_val as f64 - 0x555_u16 as f64) / (0xFFF_u16 - 0x555_u16) as f64
        } else {
            0.0
        };

        Ok(PtzPosition {
            pan: pan.clamp(-1.0, 1.0),
            tilt: tilt.clamp(-1.0, 1.0),
            zoom: zoom.clamp(0.0, 1.0),
        })
    }

    async fn test_connection(&self) -> Result<(), PtzError> {
        self.send_ptz_command("APC").await?;
        Ok(())
    }

    async fn continuous_move(
        &self,
        pan_speed: f64,
        tilt_speed: f64,
    ) -> Result<(), PtzError> {
        let ps = Self::delta_to_speed(pan_speed);
        let ts = Self::delta_to_speed(tilt_speed);
        let cmd = format!("PTS{}{}", ps, ts);
        self.send_ptz_command(&cmd).await?;
        Ok(())
    }

    async fn stop(&self) -> Result<(), PtzError> {
        self.send_ptz_command("PTS5050").await?;
        Ok(())
    }
}
