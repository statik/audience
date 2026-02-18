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

    /// Move to the home/center position.
    async fn home(&self) -> Result<(), PtzError> {
        self.move_absolute(0.0, 0.0, 0.0).await
    }

    /// Start continuous pan/tilt movement at a given velocity.
    /// pan_speed: -1.0 (left) to 1.0 (right), 0 = stop pan.
    /// tilt_speed: -1.0 (down) to 1.0 (up), 0 = stop tilt.
    async fn continuous_move(
        &self,
        _pan_speed: f64,
        _tilt_speed: f64,
    ) -> Result<(), PtzError> {
        Ok(())
    }

    /// Stop all movement.
    async fn stop(&self) -> Result<(), PtzError> {
        Ok(())
    }

    /// Start continuous focus movement. speed: negative = near, positive = far.
    async fn focus_continuous(&self, _speed: f64) -> Result<(), PtzError> {
        Ok(())
    }

    /// Toggle autofocus on or off.
    async fn set_autofocus(&self, _enabled: bool) -> Result<(), PtzError> {
        Ok(())
    }

    /// One-push autofocus trigger.
    async fn autofocus_trigger(&self) -> Result<(), PtzError> {
        Ok(())
    }

    /// Stop focus movement.
    async fn focus_stop(&self) -> Result<(), PtzError> {
        Ok(())
    }
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

    pub async fn home(&self) -> Result<(), PtzError> {
        self.get_controller()?.home().await
    }

    pub async fn continuous_move(
        &self,
        pan_speed: f64,
        tilt_speed: f64,
    ) -> Result<(), PtzError> {
        self.get_controller()?
            .continuous_move(pan_speed, tilt_speed)
            .await
    }

    pub async fn stop(&self) -> Result<(), PtzError> {
        self.get_controller()?.stop().await
    }

    pub async fn focus_continuous(&self, speed: f64) -> Result<(), PtzError> {
        self.get_controller()?.focus_continuous(speed).await
    }

    pub async fn set_autofocus(&self, enabled: bool) -> Result<(), PtzError> {
        self.get_controller()?.set_autofocus(enabled).await
    }

    pub async fn autofocus_trigger(&self) -> Result<(), PtzError> {
        self.get_controller()?.autofocus_trigger().await
    }

    pub async fn focus_stop(&self) -> Result<(), PtzError> {
        self.get_controller()?.focus_stop().await
    }
}

impl Default for PtzDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
