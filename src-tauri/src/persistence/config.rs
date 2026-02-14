use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Application-wide settings persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Multiplier for click-to-pan/tilt adjustments.
    pub click_sensitivity: f64,
    /// Multiplier for scroll-to-zoom adjustments.
    pub scroll_sensitivity: f64,
    /// Overlay opacity (0.1 to 0.9).
    pub overlay_opacity: f64,
    /// Horizontal FOV at 1x zoom in degrees.
    pub camera_fov_degrees: f64,
    /// Currently active profile ID.
    pub active_profile_id: Option<String>,
    /// Currently active video source.
    pub video_source: Option<VideoSourceConfig>,

    #[serde(skip)]
    file_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VideoSourceConfig {
    Local { device_id: String },
    Ndi { source_name: String },
    MjpegFallback { device_path: String },
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            click_sensitivity: 0.1,
            scroll_sensitivity: 0.05,
            overlay_opacity: 0.3,
            camera_fov_degrees: 60.0,
            active_profile_id: None,
            video_source: None,
            file_path: PathBuf::new(),
        }
    }
}

impl AppConfig {
    pub fn load_or_default(data_dir: &Path) -> Self {
        let file_path = data_dir.join("config.json");
        let mut config = if file_path.exists() {
            std::fs::read_to_string(&file_path)
                .ok()
                .and_then(|s| serde_json::from_str::<AppConfig>(&s).ok())
                .unwrap_or_default()
        } else {
            AppConfig::default()
        };
        config.file_path = file_path;
        config
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&self.file_path, json).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ptzcam-test-config-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn default_config_has_expected_values() {
        let config = AppConfig::default();
        assert_eq!(config.click_sensitivity, 0.1);
        assert_eq!(config.scroll_sensitivity, 0.05);
        assert_eq!(config.overlay_opacity, 0.3);
        assert_eq!(config.camera_fov_degrees, 60.0);
        assert!(config.active_profile_id.is_none());
        assert!(config.video_source.is_none());
    }

    #[test]
    fn load_or_default_returns_defaults_for_empty_dir() {
        let dir = temp_dir();
        let config = AppConfig::load_or_default(&dir);
        assert_eq!(config.click_sensitivity, 0.1);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_and_reload_roundtrips() {
        let dir = temp_dir();
        let mut config = AppConfig::load_or_default(&dir);
        config.click_sensitivity = 0.25;
        config.camera_fov_degrees = 90.0;
        config.active_profile_id = Some("prof-1".to_string());
        config.save().unwrap();

        let reloaded = AppConfig::load_or_default(&dir);
        assert_eq!(reloaded.click_sensitivity, 0.25);
        assert_eq!(reloaded.camera_fov_degrees, 90.0);
        assert_eq!(reloaded.active_profile_id.as_deref(), Some("prof-1"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_ignores_corrupt_json() {
        let dir = temp_dir();
        fs::write(dir.join("config.json"), "not valid json!!!").unwrap();
        let config = AppConfig::load_or_default(&dir);
        // Should fall back to defaults
        assert_eq!(config.click_sensitivity, 0.1);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn video_source_config_local_roundtrips() {
        let source = VideoSourceConfig::Local {
            device_id: "dev-0".to_string(),
        };
        let json = serde_json::to_string(&source).unwrap();
        let decoded: VideoSourceConfig = serde_json::from_str(&json).unwrap();
        match decoded {
            VideoSourceConfig::Local { device_id } => assert_eq!(device_id, "dev-0"),
            _ => panic!("Expected Local"),
        }
    }

    #[test]
    fn video_source_config_ndi_roundtrips() {
        let source = VideoSourceConfig::Ndi {
            source_name: "Camera 1".to_string(),
        };
        let json = serde_json::to_string(&source).unwrap();
        let decoded: VideoSourceConfig = serde_json::from_str(&json).unwrap();
        match decoded {
            VideoSourceConfig::Ndi { source_name } => assert_eq!(source_name, "Camera 1"),
            _ => panic!("Expected Ndi"),
        }
    }
}
