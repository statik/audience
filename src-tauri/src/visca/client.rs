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
        crate::ptz::types::validate_host(host).map_err(PtzError::ConnectionFailed)?;
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
        let timeout = tokio::time::timeout(std::time::Duration::from_secs(2), s.recv(&mut buf));

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
        let pt_response = self
            .send_command(&commands::pan_tilt_position_inquiry())
            .await?;
        let zoom_response = self
            .send_command(&commands::zoom_position_inquiry())
            .await?;

        // Strip 8-byte VISCA-over-IP header to get the VISCA payload
        let pt_payload = if pt_response.len() > 8 {
            &pt_response[8..]
        } else {
            &pt_response
        };
        let z_payload = if zoom_response.len() > 8 {
            &zoom_response[8..]
        } else {
            &zoom_response
        };

        let (visca_pan, visca_tilt) = commands::parse_pan_tilt_response(pt_payload)
            .ok_or(PtzError::ProtocolError(
                "Invalid pan/tilt inquiry response".into(),
            ))?;
        let visca_zoom = commands::parse_zoom_response(z_payload)
            .ok_or(PtzError::ProtocolError(
                "Invalid zoom inquiry response".into(),
            ))?;

        Ok(PtzPosition {
            pan: commands::visca_pan_to_normalized(visca_pan),
            tilt: commands::visca_tilt_to_normalized(visca_tilt),
            zoom: commands::visca_zoom_to_normalized(visca_zoom),
        })
    }

    async fn test_connection(&self) -> Result<(), PtzError> {
        self.ensure_connected().await?;
        let cmd = commands::pan_tilt_position_inquiry();
        self.send_command(&cmd).await?;
        Ok(())
    }

    async fn home(&self) -> Result<(), PtzError> {
        self.send_command(&commands::pan_tilt_home()).await?;
        Ok(())
    }

    async fn continuous_move(
        &self,
        pan_speed: f64,
        tilt_speed: f64,
    ) -> Result<(), PtzError> {
        if pan_speed.abs() < 0.01 && tilt_speed.abs() < 0.01 {
            return self.stop().await;
        }
        let ps = ((pan_speed.abs() * 24.0).ceil() as u8).clamp(1, 24);
        let ts = ((tilt_speed.abs() * 23.0).ceil() as u8).clamp(1, 23);
        let pd = if pan_speed < -0.01 {
            0x01
        } else if pan_speed > 0.01 {
            0x02
        } else {
            0x03
        };
        let td = if tilt_speed > 0.01 {
            0x01
        } else if tilt_speed < -0.01 {
            0x02
        } else {
            0x03
        };
        let cmd = commands::pan_tilt_relative(ps, ts, pd, td);
        self.send_command(&cmd).await?;
        Ok(())
    }

    async fn stop(&self) -> Result<(), PtzError> {
        self.send_command(&commands::pan_tilt_stop()).await?;
        Ok(())
    }

    async fn focus_continuous(&self, speed: f64) -> Result<(), PtzError> {
        let cmd = if speed > 0.01 {
            commands::focus_far()
        } else if speed < -0.01 {
            commands::focus_near()
        } else {
            commands::focus_stop()
        };
        self.send_command(&cmd).await?;
        Ok(())
    }

    async fn set_autofocus(&self, enabled: bool) -> Result<(), PtzError> {
        let cmd = if enabled {
            commands::autofocus_on()
        } else {
            commands::autofocus_off()
        };
        self.send_command(&cmd).await?;
        Ok(())
    }

    async fn autofocus_trigger(&self) -> Result<(), PtzError> {
        self.send_command(&commands::autofocus_trigger()).await?;
        Ok(())
    }

    async fn focus_stop(&self) -> Result<(), PtzError> {
        self.send_command(&commands::focus_stop()).await?;
        Ok(())
    }
}
