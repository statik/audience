use crate::ptz::controller::{PtzController, PtzError};
use crate::ptz::types::PtzPosition;
use async_trait::async_trait;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

use super::commands;

/// VISCA-over-IP client for Sony and compatible PTZ cameras.
pub struct ViscaClient {
    socket: Mutex<Option<UdpSocket>>,
    host: String,
    port: u16,
    sequence: AtomicU32,
}

impl ViscaClient {
    pub fn new(host: &str, port: u16) -> Result<Self, PtzError> {
        crate::ptz::types::validate_host(host)
            .map_err(PtzError::ConnectionFailed)?;
        Ok(Self {
            socket: Mutex::new(None),
            host: host.to_string(),
            port,
            sequence: AtomicU32::new(1),
        })
    }

    async fn ensure_connected(&self) -> Result<(), PtzError> {
        let mut socket = self.socket.lock().await;
        if socket.is_none() {
            let s = UdpSocket::bind("0.0.0.0:0")
                .await
                .map_err(|e| PtzError::ConnectionFailed(e.to_string()))?;
            s.connect(format!("{}:{}", self.host, self.port))
                .await
                .map_err(|e| PtzError::ConnectionFailed(e.to_string()))?;
            *socket = Some(s);
        }
        Ok(())
    }

    async fn send_command(&self, payload: &[u8]) -> Result<Vec<u8>, PtzError> {
        self.ensure_connected().await?;
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        let packet = commands::build_visca_packet(payload, seq);

        let socket = self.socket.lock().await;
        let s = socket.as_ref().ok_or(PtzError::NotConnected)?;

        s.send(&packet)
            .await
            .map_err(|e| PtzError::CommandFailed(e.to_string()))?;

        let mut buf = [0u8; 256];
        let timeout = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            s.recv(&mut buf),
        );

        match timeout.await {
            Ok(Ok(len)) => Ok(buf[..len].to_vec()),
            Ok(Err(e)) => Err(PtzError::CommandFailed(e.to_string())),
            Err(_) => Err(PtzError::Timeout("VISCA response timeout".to_string())),
        }
    }

}

#[async_trait]
impl PtzController for ViscaClient {
    async fn move_absolute(&self, pan: f64, tilt: f64, zoom: f64) -> Result<(), PtzError> {
        let visca_pan = commands::normalize_to_visca_pan(pan);
        let visca_tilt = commands::normalize_to_visca_tilt(tilt);
        let visca_zoom = commands::normalize_to_visca_zoom(zoom);

        let pt_cmd = commands::pan_tilt_absolute(0x0C, 0x0C, visca_pan, visca_tilt);
        self.send_command(&pt_cmd).await?;

        let zoom_cmd = commands::zoom_absolute(visca_zoom);
        self.send_command(&zoom_cmd).await?;

        Ok(())
    }

    async fn move_relative(&self, pan_delta: f64, tilt_delta: f64) -> Result<(), PtzError> {
        // Short-circuit if both deltas are below threshold (nothing to move)
        if pan_delta.abs() < 0.01 && tilt_delta.abs() < 0.01 {
            return Ok(());
        }

        // Determine direction and speed from delta magnitudes
        let pan_speed = ((pan_delta.abs() * 24.0).ceil() as u8).clamp(1, 24);
        let tilt_speed = ((tilt_delta.abs() * 23.0).ceil() as u8).clamp(1, 23);

        let pan_dir = if pan_delta < -0.01 {
            0x01 // left
        } else if pan_delta > 0.01 {
            0x02 // right
        } else {
            0x03 // stop
        };

        let tilt_dir = if tilt_delta > 0.01 {
            0x01 // up
        } else if tilt_delta < -0.01 {
            0x02 // down
        } else {
            0x03 // stop
        };

        let cmd = commands::pan_tilt_relative(pan_speed, tilt_speed, pan_dir, tilt_dir);
        self.send_command(&cmd).await?;

        // Brief movement then stop
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let stop_cmd = commands::pan_tilt_stop();
        self.send_command(&stop_cmd).await?;

        Ok(())
    }

    async fn zoom_to(&self, zoom: f64) -> Result<(), PtzError> {
        let visca_zoom = commands::normalize_to_visca_zoom(zoom);
        let cmd = commands::zoom_absolute(visca_zoom);
        self.send_command(&cmd).await?;
        Ok(())
    }

    async fn recall_preset(&self, preset_index: u8) -> Result<(), PtzError> {
        let cmd = commands::preset_recall(preset_index);
        self.send_command(&cmd).await?;
        Ok(())
    }

    async fn store_preset(&self, preset_index: u8) -> Result<(), PtzError> {
        let cmd = commands::preset_store(preset_index);
        self.send_command(&cmd).await?;
        Ok(())
    }

    async fn get_position(&self) -> Result<PtzPosition, PtzError> {
        // Query pan/tilt and zoom positions
        let _pt_response = self
            .send_command(&commands::pan_tilt_position_inquiry())
            .await?;
        let _zoom_response = self
            .send_command(&commands::zoom_position_inquiry())
            .await?;

        // Parse responses — in production, decode the VISCA nibble format
        // For now return default; real implementation would parse response bytes
        Ok(PtzPosition {
            pan: 0.0,
            tilt: 0.0,
            zoom: 0.0,
        })
    }

    async fn test_connection(&self) -> Result<(), PtzError> {
        self.ensure_connected().await?;
        // Send a position inquiry as a connectivity test
        let cmd = commands::pan_tilt_position_inquiry();
        self.send_command(&cmd).await?;
        Ok(())
    }
}
