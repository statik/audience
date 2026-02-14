use crate::persistence::config::AppConfig;
use crate::AppState;

/// Get current application settings.
#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

/// Validate a finite f64 value and clamp to range.
fn validate_and_clamp(value: f64, min: f64, max: f64, name: &str) -> Result<f64, String> {
    if !value.is_finite() {
        return Err(format!("{} must be a finite number", name));
    }
    Ok(value.clamp(min, max))
}

/// Update application settings.
#[tauri::command]
pub async fn update_settings(
    state: tauri::State<'_, AppState>,
    click_sensitivity: Option<f64>,
    scroll_sensitivity: Option<f64>,
    overlay_opacity: Option<f64>,
    camera_fov_degrees: Option<f64>,
) -> Result<AppConfig, String> {
    let mut config = state.config.lock().await;

    if let Some(v) = click_sensitivity {
        config.click_sensitivity = validate_and_clamp(v, 0.01, 0.5, "click_sensitivity")?;
    }
    if let Some(v) = scroll_sensitivity {
        config.scroll_sensitivity = validate_and_clamp(v, 0.01, 0.2, "scroll_sensitivity")?;
    }
    if let Some(v) = overlay_opacity {
        config.overlay_opacity = validate_and_clamp(v, 0.1, 0.9, "overlay_opacity")?;
    }
    if let Some(v) = camera_fov_degrees {
        config.camera_fov_degrees = validate_and_clamp(v, 10.0, 180.0, "camera_fov_degrees")?;
    }

    config.save()?;
    Ok(config.clone())
}
