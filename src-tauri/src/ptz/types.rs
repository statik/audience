use serde::{Deserialize, Serialize};

/// Normalized PTZ position: pan/tilt in [-1.0, 1.0], zoom in [0.0, 1.0].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtzPosition {
    pub pan: f64,
    pub tilt: f64,
    pub zoom: f64,
}

impl Default for PtzPosition {
    fn default() -> Self {
        Self {
            pan: 0.0,
            tilt: 0.0,
            zoom: 0.0,
        }
    }
}

/// A PTZ command to send to a camera.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PtzCommand {
    MoveAbsolute { pan: f64, tilt: f64, zoom: f64 },
    MoveRelative { pan_delta: f64, tilt_delta: f64 },
    Zoom { level: f64 },
    RecallPreset { index: u8 },
    StorePreset { index: u8 },
}

/// Supported PTZ control protocols.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum PtzProtocol {
    Ndi,
    Visca,
    PanasonicAw,
    BirdDogRest,
}

/// Protocol-specific connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProtocolConfig {
    Ndi,
    Visca {
        host: String,
        port: u16,
    },
    PanasonicAw {
        host: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    },
    BirdDogRest {
        host: String,
        port: u16,
    },
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self::Ndi
    }
}

/// A camera endpoint definition for PTZ control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraEndpoint {
    pub id: String,
    pub name: String,
    pub protocol: PtzProtocol,
    pub config: ProtocolConfig,
}

/// A single preset definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub pan: f64,
    pub tilt: f64,
    pub zoom: f64,
    pub color: String,
}

/// A named collection of presets for a particular camera setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetProfile {
    pub id: String,
    pub name: String,
    pub camera_fov_degrees: f64,
    pub endpoint_id: Option<String>,
    pub presets: Vec<Preset>,
}
