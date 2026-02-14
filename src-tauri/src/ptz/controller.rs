use super::types::PtzPosition;
use async_trait::async_trait;

/// Protocol-agnostic PTZ controller trait.
/// All protocol implementations (NDI, VISCA, Panasonic AW, BirdDog) implement this.
#[async_trait]
pub trait PtzController: Send + Sync {
    /// Move to an absolute position (normalized values).
    async fn move_absolute(&self, pan: f64, tilt: f64, zoom: f64) -> Result<(), PtzError>;

    /// Move relative to current position (normalized deltas).
    async fn move_relative(&self, pan_delta: f64, tilt_delta: f64) -> Result<(), PtzError>;

    /// Set zoom level (normalized 0.0 to 1.0).
    async fn zoom_to(&self, zoom: f64) -> Result<(), PtzError>;

    /// Recall a camera-native preset by index.
    async fn recall_preset(&self, preset_index: u8) -> Result<(), PtzError>;

    /// Store the current position as a camera-native preset.
    async fn store_preset(&self, preset_index: u8) -> Result<(), PtzError>;

    /// Query the current PTZ position from the camera.
    async fn get_position(&self) -> Result<PtzPosition, PtzError>;

    /// Test connectivity to the camera.
    async fn test_connection(&self) -> Result<(), PtzError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PtzError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Not connected")]
    NotConnected,
}

/// Routes PTZ commands to the active protocol-specific controller.
pub struct PtzDispatcher {
    controller: Option<Box<dyn PtzController>>,
}

impl PtzDispatcher {
    pub fn new() -> Self {
        Self { controller: None }
    }

    pub fn set_controller(&mut self, controller: Box<dyn PtzController>) {
        self.controller = Some(controller);
    }

    pub fn clear_controller(&mut self) {
        self.controller = None;
    }

    pub fn has_controller(&self) -> bool {
        self.controller.is_some()
    }

    fn get_controller(&self) -> Result<&dyn PtzController, PtzError> {
        self.controller.as_deref().ok_or(PtzError::NotConnected)
    }

    pub async fn move_absolute(&self, pan: f64, tilt: f64, zoom: f64) -> Result<(), PtzError> {
        self.get_controller()?.move_absolute(pan, tilt, zoom).await
    }

    pub async fn move_relative(&self, pan_delta: f64, tilt_delta: f64) -> Result<(), PtzError> {
        self.get_controller()?
            .move_relative(pan_delta, tilt_delta)
            .await
    }

    pub async fn zoom_to(&self, zoom: f64) -> Result<(), PtzError> {
        self.get_controller()?.zoom_to(zoom).await
    }

    pub async fn recall_preset(&self, preset_index: u8) -> Result<(), PtzError> {
        self.get_controller()?.recall_preset(preset_index).await
    }

    pub async fn store_preset(&self, preset_index: u8) -> Result<(), PtzError> {
        self.get_controller()?.store_preset(preset_index).await
    }

    pub async fn get_position(&self) -> Result<PtzPosition, PtzError> {
        self.get_controller()?.get_position().await
    }

    pub async fn test_connection(&self) -> Result<(), PtzError> {
        self.get_controller()?.test_connection().await
    }
}

impl Default for PtzDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
