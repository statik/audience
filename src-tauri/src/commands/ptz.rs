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
pub async fn ptz_zoom(state: tauri::State<'_, AppState>, zoom: f64) -> Result<(), String> {
    let zoom = zoom.clamp(0.0, 1.0);

    // Update local position tracking
    let mut pos = state.current_position.lock().await;
    pos.zoom = zoom;
    drop(pos);

    // Dispatch to active PTZ controller if connected
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher.zoom_to(zoom).await.map_err(|e| e.to_string())?;
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
    let preset = profiles.find_preset(&preset_id).ok_or("Preset not found")?;

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
        name,
        pan,
        tilt,
        zoom
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

/// Move the camera to its home/center position.
#[tauri::command]
pub async fn ptz_home(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut pos = state.current_position.lock().await;
    pos.pan = 0.0;
    pos.tilt = 0.0;
    pos.zoom = 0.0;
    drop(pos);

    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher.home().await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Start continuous pan/tilt movement at a given velocity.
#[tauri::command]
pub async fn ptz_continuous_move(
    state: tauri::State<'_, AppState>,
    pan_speed: f64,
    tilt_speed: f64,
) -> Result<(), String> {
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher
            .continuous_move(pan_speed, tilt_speed)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Stop all camera movement.
#[tauri::command]
pub async fn ptz_stop(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher.stop().await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Start continuous focus movement. Negative = near, positive = far.
#[tauri::command]
pub async fn ptz_focus(state: tauri::State<'_, AppState>, speed: f64) -> Result<(), String> {
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher
            .focus_continuous(speed)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Stop focus movement.
#[tauri::command]
pub async fn ptz_focus_stop(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher.focus_stop().await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Toggle autofocus on or off.
#[tauri::command]
pub async fn ptz_set_autofocus(
    state: tauri::State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher
            .set_autofocus(enabled)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// One-push autofocus trigger.
#[tauri::command]
pub async fn ptz_autofocus_trigger(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let dispatcher = state.ptz_dispatcher.lock().await;
    if dispatcher.has_controller() {
        dispatcher
            .autofocus_trigger()
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Get the current PTZ position.
#[tauri::command]
pub async fn ptz_get_position(state: tauri::State<'_, AppState>) -> Result<PtzPosition, String> {
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
