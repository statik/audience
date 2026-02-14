use crate::ptz::types::PtzPosition;
use crate::AppState;

/// Move the camera by a relative pan/tilt delta.
#[tauri::command]
pub async fn ptz_move_relative(
    state: tauri::State<'_, AppState>,
    pan_delta: f64,
    tilt_delta: f64,
) -> Result<(), String> {
    let mut pos = state.current_position.lock().await;
    pos.pan = (pos.pan + pan_delta).clamp(-1.0, 1.0);
    pos.tilt = (pos.tilt + tilt_delta).clamp(-1.0, 1.0);
    // In production: dispatch to active PTZ controller
    log::info!("PTZ move relative: pan_delta={}, tilt_delta={}", pan_delta, tilt_delta);
    Ok(())
}

/// Move the camera to an absolute pan/tilt/zoom position.
#[tauri::command]
pub async fn ptz_move_absolute(
    state: tauri::State<'_, AppState>,
    pan: f64,
    tilt: f64,
    zoom: f64,
) -> Result<(), String> {
    let mut pos = state.current_position.lock().await;
    pos.pan = pan.clamp(-1.0, 1.0);
    pos.tilt = tilt.clamp(-1.0, 1.0);
    pos.zoom = zoom.clamp(0.0, 1.0);
    log::info!("PTZ move absolute: pan={}, tilt={}, zoom={}", pan, tilt, zoom);
    Ok(())
}

/// Set zoom level.
#[tauri::command]
pub async fn ptz_zoom(
    state: tauri::State<'_, AppState>,
    zoom: f64,
) -> Result<(), String> {
    let mut pos = state.current_position.lock().await;
    pos.zoom = zoom.clamp(0.0, 1.0);
    log::info!("PTZ zoom: {}", zoom);
    Ok(())
}

/// Recall a preset by its ID, moving the camera to the saved position.
#[tauri::command]
pub async fn ptz_recall_preset(
    state: tauri::State<'_, AppState>,
    preset_id: String,
) -> Result<(), String> {
    let profiles = state.profiles.lock().await;
    let preset = profiles
        .find_preset(&preset_id)
        .ok_or("Preset not found")?;

    let mut pos = state.current_position.lock().await;
    pos.pan = preset.pan;
    pos.tilt = preset.tilt;
    pos.zoom = preset.zoom;

    log::info!(
        "PTZ recall preset '{}': pan={}, tilt={}, zoom={}",
        preset.name, preset.pan, preset.tilt, preset.zoom
    );
    Ok(())
}

/// Store the current camera position as a camera-native preset.
#[tauri::command]
pub async fn ptz_store_preset(
    _state: tauri::State<'_, AppState>,
    preset_index: u8,
) -> Result<(), String> {
    log::info!("PTZ store preset index: {}", preset_index);
    Ok(())
}

/// Get the current PTZ position.
#[tauri::command]
pub async fn ptz_get_position(
    state: tauri::State<'_, AppState>,
) -> Result<PtzPosition, String> {
    let pos = state.current_position.lock().await;
    Ok(pos.clone())
}
