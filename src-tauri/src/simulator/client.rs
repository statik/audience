use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::ptz::controller::{PtzController, PtzError};
use crate::ptz::types::PtzPosition;

/// Simulated PTZ camera for development and demo use.
///
/// Tracks position and presets in memory with no hardware
/// or network dependencies.
pub struct SimulatedController {
    position: Mutex<PtzPosition>,
    presets: Mutex<HashMap<u8, PtzPosition>>,
}

impl Default for SimulatedController {
    fn default() -> Self {
        Self {
            position: Mutex::new(PtzPosition::default()),
            presets: Mutex::new(HashMap::new()),
        }
    }
}

impl SimulatedController {
    pub fn new() -> Self {
        Self::default()
    }
}

fn clamp_pan_tilt(value: f64) -> f64 {
    value.clamp(-1.0, 1.0)
}

fn clamp_zoom(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[async_trait]
impl PtzController for SimulatedController {
    async fn move_absolute(&self, pan: f64, tilt: f64, zoom: f64) -> Result<(), PtzError> {
        let mut pos = self
            .position
            .lock()
            .map_err(|e| PtzError::CommandFailed(format!("Lock poisoned: {e}")))?;
        pos.pan = clamp_pan_tilt(pan);
        pos.tilt = clamp_pan_tilt(tilt);
        pos.zoom = clamp_zoom(zoom);
        Ok(())
    }

    async fn move_relative(&self, pan_delta: f64, tilt_delta: f64) -> Result<(), PtzError> {
        let mut pos = self
            .position
            .lock()
            .map_err(|e| PtzError::CommandFailed(format!("Lock poisoned: {e}")))?;
        pos.pan = clamp_pan_tilt(pos.pan + pan_delta);
        pos.tilt = clamp_pan_tilt(pos.tilt + tilt_delta);
        Ok(())
    }

    async fn zoom_to(&self, zoom: f64) -> Result<(), PtzError> {
        let mut pos = self
            .position
            .lock()
            .map_err(|e| PtzError::CommandFailed(format!("Lock poisoned: {e}")))?;
        pos.zoom = clamp_zoom(zoom);
        Ok(())
    }

    async fn store_preset(&self, preset_index: u8) -> Result<(), PtzError> {
        let pos = self
            .position
            .lock()
            .map_err(|e| PtzError::CommandFailed(format!("Lock poisoned: {e}")))?;
        let snapshot = pos.clone();
        drop(pos);

        let mut presets = self
            .presets
            .lock()
            .map_err(|e| PtzError::CommandFailed(format!("Lock poisoned: {e}")))?;
        presets.insert(preset_index, snapshot);
        Ok(())
    }

    async fn recall_preset(&self, preset_index: u8) -> Result<(), PtzError> {
        let presets = self
            .presets
            .lock()
            .map_err(|e| PtzError::CommandFailed(format!("Lock poisoned: {e}")))?;
        let stored = presets.get(&preset_index).cloned().ok_or_else(|| {
            PtzError::CommandFailed(format!("No preset stored at index {preset_index}"))
        })?;
        drop(presets);

        let mut pos = self
            .position
            .lock()
            .map_err(|e| PtzError::CommandFailed(format!("Lock poisoned: {e}")))?;
        *pos = stored;
        Ok(())
    }

    async fn get_position(&self) -> Result<PtzPosition, PtzError> {
        let pos = self
            .position
            .lock()
            .map_err(|e| PtzError::CommandFailed(format!("Lock poisoned: {e}")))?;
        Ok(pos.clone())
    }

    async fn test_connection(&self) -> Result<(), PtzError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_controller_starts_at_origin() {
        let ctrl = SimulatedController::new();
        let pos = ctrl.get_position().await.unwrap();
        assert_eq!(pos.pan, 0.0);
        assert_eq!(pos.tilt, 0.0);
        assert_eq!(pos.zoom, 0.0);
    }

    #[tokio::test]
    async fn move_absolute_sets_position() {
        let ctrl = SimulatedController::new();
        ctrl.move_absolute(0.5, -0.3, 0.8).await.unwrap();
        let pos = ctrl.get_position().await.unwrap();
        assert_eq!(pos.pan, 0.5);
        assert_eq!(pos.tilt, -0.3);
        assert_eq!(pos.zoom, 0.8);
    }

    #[tokio::test]
    async fn move_absolute_clamps_values() {
        let ctrl = SimulatedController::new();
        ctrl.move_absolute(2.0, -5.0, 3.0).await.unwrap();
        let pos = ctrl.get_position().await.unwrap();
        assert_eq!(pos.pan, 1.0);
        assert_eq!(pos.tilt, -1.0);
        assert_eq!(pos.zoom, 1.0);
    }

    #[tokio::test]
    async fn move_absolute_clamps_zoom_lower_bound() {
        let ctrl = SimulatedController::new();
        ctrl.move_absolute(0.0, 0.0, -1.0).await.unwrap();
        let pos = ctrl.get_position().await.unwrap();
        assert_eq!(pos.zoom, 0.0);
    }

    #[tokio::test]
    async fn move_relative_adds_deltas() {
        let ctrl = SimulatedController::new();
        ctrl.move_absolute(0.2, 0.3, 0.5).await.unwrap();
        ctrl.move_relative(0.1, -0.2).await.unwrap();
        let pos = ctrl.get_position().await.unwrap();
        assert!((pos.pan - 0.3).abs() < f64::EPSILON);
        assert!((pos.tilt - 0.1).abs() < f64::EPSILON);
        assert_eq!(pos.zoom, 0.5);
    }

    #[tokio::test]
    async fn move_relative_clamps_at_bounds() {
        let ctrl = SimulatedController::new();
        ctrl.move_absolute(0.9, -0.9, 0.0).await.unwrap();
        ctrl.move_relative(0.5, -0.5).await.unwrap();
        let pos = ctrl.get_position().await.unwrap();
        assert_eq!(pos.pan, 1.0);
        assert_eq!(pos.tilt, -1.0);
    }

    #[tokio::test]
    async fn zoom_to_sets_zoom() {
        let ctrl = SimulatedController::new();
        ctrl.zoom_to(0.75).await.unwrap();
        let pos = ctrl.get_position().await.unwrap();
        assert_eq!(pos.zoom, 0.75);
    }

    #[tokio::test]
    async fn zoom_to_clamps() {
        let ctrl = SimulatedController::new();
        ctrl.zoom_to(1.5).await.unwrap();
        assert_eq!(ctrl.get_position().await.unwrap().zoom, 1.0);
        ctrl.zoom_to(-0.5).await.unwrap();
        assert_eq!(ctrl.get_position().await.unwrap().zoom, 0.0);
    }

    #[tokio::test]
    async fn preset_store_and_recall() {
        let ctrl = SimulatedController::new();
        ctrl.move_absolute(0.5, -0.3, 0.8).await.unwrap();
        ctrl.store_preset(1).await.unwrap();

        ctrl.move_absolute(0.0, 0.0, 0.0).await.unwrap();
        ctrl.recall_preset(1).await.unwrap();

        let pos = ctrl.get_position().await.unwrap();
        assert_eq!(pos.pan, 0.5);
        assert_eq!(pos.tilt, -0.3);
        assert_eq!(pos.zoom, 0.8);
    }

    #[tokio::test]
    async fn recall_missing_preset_returns_error() {
        let ctrl = SimulatedController::new();
        let result = ctrl.recall_preset(99).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("99"));
    }

    #[tokio::test]
    async fn multiple_presets_independent() {
        let ctrl = SimulatedController::new();

        ctrl.move_absolute(0.1, 0.2, 0.3).await.unwrap();
        ctrl.store_preset(1).await.unwrap();

        ctrl.move_absolute(0.4, 0.5, 0.6).await.unwrap();
        ctrl.store_preset(2).await.unwrap();

        ctrl.recall_preset(1).await.unwrap();
        let pos = ctrl.get_position().await.unwrap();
        assert_eq!(pos.pan, 0.1);
        assert_eq!(pos.tilt, 0.2);
        assert_eq!(pos.zoom, 0.3);

        ctrl.recall_preset(2).await.unwrap();
        let pos = ctrl.get_position().await.unwrap();
        assert_eq!(pos.pan, 0.4);
        assert_eq!(pos.tilt, 0.5);
        assert_eq!(pos.zoom, 0.6);
    }

    #[tokio::test]
    async fn test_connection_always_succeeds() {
        let ctrl = SimulatedController::new();
        assert!(ctrl.test_connection().await.is_ok());
    }
}
