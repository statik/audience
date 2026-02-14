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
