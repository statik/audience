use crate::ptz::types::PtzPosition;
use crate::AppState;

/// Move the camera by a relative pan/tilt delta.
#[tauri::command]
pub async fn ptz_move_relative(
    state: tauri::State<'_, AppState>,
    pan_delta: f64,
    tilt_delta: f64,
) -> Result<(), String> {
    // Update local position tracking
    let mut pos = state.current_position.lock().await;
    pos.pan = (pos.pan + pan_delta).clamp(-1.0, 1.0);
    pos.tilt = (pos.tilt + tilt_delta).clamp(-1.0, 1.0);
    drop(pos);

    // Dispatch to active PTZ controller if connected
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher
            .move_relative(pan_delta, tilt_delta)
            .await
            .map_err(|e| e.to_string())?;
    }

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
    let pan = pan.clamp(-1.0, 1.0);
    let tilt = tilt.clamp(-1.0, 1.0);
    let zoom = zoom.clamp(0.0, 1.0);

    // Update local position tracking
    let mut pos = state.current_position.lock().await;
    pos.pan = pan;
    pos.tilt = tilt;
    pos.zoom = zoom;
    drop(pos);

    // Dispatch to active PTZ controller if connected
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher
            .move_absolute(pan, tilt, zoom)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Set zoom level.
#[tauri::command]
pub async fn ptz_zoom(
    state: tauri::State<'_, AppState>,
    zoom: f64,
) -> Result<(), String> {
    let zoom = zoom.clamp(0.0, 1.0);

    // Update local position tracking
    let mut pos = state.current_position.lock().await;
    pos.zoom = zoom;
    drop(pos);

    // Dispatch to active PTZ controller if connected
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher
            .zoom_to(zoom)
            .await
            .map_err(|e| e.to_string())?;
    }

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

    let pan = preset.pan;
    let tilt = preset.tilt;
    let zoom = preset.zoom;
    let name = preset.name.clone();
    drop(profiles);

    // Update local position tracking
    let mut pos = state.current_position.lock().await;
    pos.pan = pan;
    pos.tilt = tilt;
    pos.zoom = zoom;
    drop(pos);

    // Dispatch absolute move to active PTZ controller
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher
            .move_absolute(pan, tilt, zoom)
            .await
            .map_err(|e| e.to_string())?;
    }

    log::info!(
        "PTZ recall preset '{}': pan={}, tilt={}, zoom={}",
        name, pan, tilt, zoom
    );
    Ok(())
}

/// Store the current camera position as a camera-native preset.
#[tauri::command]
pub async fn ptz_store_preset(
    state: tauri::State<'_, AppState>,
    preset_index: u8,
) -> Result<(), String> {
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher
            .store_preset(preset_index)
            .await
            .map_err(|e| e.to_string())?;
    }

    log::info!("PTZ store preset index: {}", preset_index);
    Ok(())
}

/// Get the current PTZ position.
#[tauri::command]
pub async fn ptz_get_position(
    state: tauri::State<'_, AppState>,
) -> Result<PtzPosition, String> {
    // If we have an active controller, query the camera for its real position
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        match dispatcher.get_position().await {
            Ok(hw_pos) => {
                drop(dispatcher);
                // Update local tracking with hardware position
                let mut pos = state.current_position.lock().await;
                pos.pan = hw_pos.pan;
                pos.tilt = hw_pos.tilt;
                pos.zoom = hw_pos.zoom;
                return Ok(hw_pos);
            }
            Err(e) => {
                log::warn!("Failed to query hardware position, using local: {}", e);
            }
        }
    }
    drop(dispatcher);

    // Fallback to local position tracking
    let pos = state.current_position.lock().await;
    Ok(pos.clone())
}
