use crate::persistence::config::AppConfig;
use crate::AppState;

/// Get current application settings.
#[tauri::command]
pub async fn get_settings(
    state: tauri::State<'_, AppState>,
) -> Result<AppConfig, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
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
        config.click_sensitivity = v;
    }
    if let Some(v) = scroll_sensitivity {
        config.scroll_sensitivity = v;
    }
    if let Some(v) = overlay_opacity {
        config.overlay_opacity = v.clamp(0.1, 0.9);
    }
    if let Some(v) = camera_fov_degrees {
        config.camera_fov_degrees = v;
    }

    config.save()?;
    Ok(config.clone())
}
