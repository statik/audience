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

/// Validate that a host string is a safe IP address or hostname.
/// Rejects values containing path separators, whitespace, or other injection-prone characters.
pub fn validate_host(host: &str) -> Result<(), String> {
    if host.is_empty() {
        return Err("Host cannot be empty".to_string());
    }
    if host.contains('/') || host.contains('\\') || host.contains(' ') || host.contains('@') {
        return Err(format!("Invalid host: '{}'", host));
    }
    // Must look like an IP address or hostname
    let valid = host
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == ':' || c == '-');
    if !valid {
        return Err(format!("Invalid host characters: '{}'", host));
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- PtzPosition tests ---

    #[test]
    fn ptz_position_default_is_origin() {
        let pos = PtzPosition::default();
        assert_eq!(pos.pan, 0.0);
        assert_eq!(pos.tilt, 0.0);
        assert_eq!(pos.zoom, 0.0);
    }

    #[test]
    fn ptz_position_serializes_to_json() {
        let pos = PtzPosition {
            pan: 0.5,
            tilt: -0.3,
            zoom: 0.8,
        };
        let json = serde_json::to_string(&pos).unwrap();
        assert!(json.contains("0.5"));
        assert!(json.contains("-0.3"));
        assert!(json.contains("0.8"));
    }

    #[test]
    fn ptz_position_deserializes_from_json() {
        let json = r#"{"pan":0.5,"tilt":-0.3,"zoom":0.8}"#;
        let pos: PtzPosition = serde_json::from_str(json).unwrap();
        assert_eq!(pos.pan, 0.5);
        assert_eq!(pos.tilt, -0.3);
        assert_eq!(pos.zoom, 0.8);
    }

    // --- PtzCommand tests ---

    #[test]
    fn ptz_command_move_absolute_roundtrips() {
        let cmd = PtzCommand::MoveAbsolute {
            pan: 0.1,
            tilt: 0.2,
            zoom: 0.3,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: PtzCommand = serde_json::from_str(&json).unwrap();
        match decoded {
            PtzCommand::MoveAbsolute { pan, tilt, zoom } => {
                assert_eq!(pan, 0.1);
                assert_eq!(tilt, 0.2);
                assert_eq!(zoom, 0.3);
            }
            _ => panic!("Expected MoveAbsolute"),
        }
    }

    #[test]
    fn ptz_command_recall_preset_roundtrips() {
        let cmd = PtzCommand::RecallPreset { index: 5 };
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: PtzCommand = serde_json::from_str(&json).unwrap();
        match decoded {
            PtzCommand::RecallPreset { index } => assert_eq!(index, 5),
            _ => panic!("Expected RecallPreset"),
        }
    }

    // --- PtzProtocol tests ---

    #[test]
    fn protocol_config_default_is_ndi() {
        let config = ProtocolConfig::default();
        match config {
            ProtocolConfig::Ndi => {}
            _ => panic!("Expected Ndi default"),
        }
    }

    #[test]
    fn protocol_config_visca_roundtrips() {
        let config = ProtocolConfig::Visca {
            host: "192.168.1.100".to_string(),
            port: 1259,
        };
        let json = serde_json::to_string(&config).unwrap();
        let decoded: ProtocolConfig = serde_json::from_str(&json).unwrap();
        match decoded {
            ProtocolConfig::Visca { host, port } => {
                assert_eq!(host, "192.168.1.100");
                assert_eq!(port, 1259);
            }
            _ => panic!("Expected Visca"),
        }
    }

    #[test]
    fn protocol_config_panasonic_with_credentials_roundtrips() {
        let config = ProtocolConfig::PanasonicAw {
            host: "10.0.0.1".to_string(),
            port: 80,
            username: Some("admin".to_string()),
            password: Some("secret".to_string()),
        };
        let json = serde_json::to_string(&config).unwrap();
        let decoded: ProtocolConfig = serde_json::from_str(&json).unwrap();
        match decoded {
            ProtocolConfig::PanasonicAw {
                host,
                port,
                username,
                password,
            } => {
                assert_eq!(host, "10.0.0.1");
                assert_eq!(port, 80);
                assert_eq!(username.as_deref(), Some("admin"));
                assert_eq!(password.as_deref(), Some("secret"));
            }
            _ => panic!("Expected PanasonicAw"),
        }
    }

    // --- validate_host tests ---

    #[test]
    fn validate_host_accepts_ip_address() {
        assert!(validate_host("192.168.1.100").is_ok());
    }

    #[test]
    fn validate_host_accepts_hostname() {
        assert!(validate_host("camera-1.local").is_ok());
    }

    #[test]
    fn validate_host_accepts_ipv6_shorthand() {
        assert!(validate_host("::1").is_ok());
    }

    #[test]
    fn validate_host_rejects_empty() {
        assert!(validate_host("").is_err());
    }

    #[test]
    fn validate_host_rejects_forward_slash() {
        assert!(validate_host("192.168.1.1/24").is_err());
    }

    #[test]
    fn validate_host_rejects_backslash() {
        assert!(validate_host("host\\path").is_err());
    }

    #[test]
    fn validate_host_rejects_spaces() {
        assert!(validate_host("host name").is_err());
    }

    #[test]
    fn validate_host_rejects_at_sign() {
        assert!(validate_host("user@host").is_err());
    }

    #[test]
    fn validate_host_rejects_special_characters() {
        assert!(validate_host("host;rm -rf").is_err());
    }

    // --- Preset tests ---

    #[test]
    fn preset_roundtrips_through_json() {
        let preset = Preset {
            id: "abc-123".to_string(),
            name: "Front Row".to_string(),
            pan: 0.5,
            tilt: -0.3,
            zoom: 0.8,
            color: "#3b82f6".to_string(),
        };
        let json = serde_json::to_string(&preset).unwrap();
        let decoded: Preset = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, "abc-123");
        assert_eq!(decoded.name, "Front Row");
        assert_eq!(decoded.pan, 0.5);
        assert_eq!(decoded.tilt, -0.3);
        assert_eq!(decoded.zoom, 0.8);
        assert_eq!(decoded.color, "#3b82f6");
    }

    // --- CameraEndpoint tests ---

    #[test]
    fn camera_endpoint_roundtrips_through_json() {
        let endpoint = CameraEndpoint {
            id: "ep-1".to_string(),
            name: "Main Camera".to_string(),
            protocol: PtzProtocol::Visca,
            config: ProtocolConfig::Visca {
                host: "10.0.0.50".to_string(),
                port: 1259,
            },
        };
        let json = serde_json::to_string(&endpoint).unwrap();
        let decoded: CameraEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, "ep-1");
        assert_eq!(decoded.name, "Main Camera");
        assert_eq!(decoded.protocol, PtzProtocol::Visca);
    }

    // --- PresetProfile tests ---

    #[test]
    fn preset_profile_with_presets_roundtrips() {
        let profile = PresetProfile {
            id: "prof-1".to_string(),
            name: "Sunday Service".to_string(),
            camera_fov_degrees: 60.0,
            endpoint_id: Some("ep-1".to_string()),
            presets: vec![Preset {
                id: "p1".to_string(),
                name: "Wide".to_string(),
                pan: 0.0,
                tilt: 0.0,
                zoom: 0.0,
                color: "#fff".to_string(),
            }],
        };
        let json = serde_json::to_string(&profile).unwrap();
        let decoded: PresetProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.presets.len(), 1);
        assert_eq!(decoded.presets[0].name, "Wide");
        assert_eq!(decoded.endpoint_id.as_deref(), Some("ep-1"));
    }
}
