use crate::ptz::controller::{PtzController, PtzError};
use crate::ptz::types::PtzPosition;
use async_trait::async_trait;

/// NDI PTZ controller stub.
/// In production, wraps NDIlib_recv_ptz_* functions.
pub struct NdiPtzController;

impl NdiPtzController {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NdiPtzController {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PtzController for NdiPtzController {
    async fn move_absolute(&self, _pan: f64, _tilt: f64, _zoom: f64) -> Result<(), PtzError> {
        Err(PtzError::ConnectionFailed("NDI SDK not linked".to_string()))
    }

    async fn move_relative(&self, _pan_delta: f64, _tilt_delta: f64) -> Result<(), PtzError> {
        Err(PtzError::ConnectionFailed("NDI SDK not linked".to_string()))
    }

    async fn zoom_to(&self, _zoom: f64) -> Result<(), PtzError> {
        Err(PtzError::ConnectionFailed("NDI SDK not linked".to_string()))
    }

    async fn recall_preset(&self, _preset_index: u8) -> Result<(), PtzError> {
        Err(PtzError::ConnectionFailed("NDI SDK not linked".to_string()))
    }

    async fn store_preset(&self, _preset_index: u8) -> Result<(), PtzError> {
        Err(PtzError::ConnectionFailed("NDI SDK not linked".to_string()))
    }

    async fn get_position(&self) -> Result<PtzPosition, PtzError> {
        Err(PtzError::ConnectionFailed("NDI SDK not linked".to_string()))
    }

    async fn test_connection(&self) -> Result<(), PtzError> {
        Err(PtzError::ConnectionFailed("NDI SDK not linked".to_string()))
    }
}
