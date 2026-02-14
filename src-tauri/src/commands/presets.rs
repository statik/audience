use crate::ptz::types::{Preset, PresetProfile};
use crate::AppState;

/// Get all presets from the active profile.
#[tauri::command]
pub async fn get_all_presets(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Preset>, String> {
    let mut profiles = state.profiles.lock().await;
    profiles.ensure_default_profile()?;
    Ok(profiles.get_presets())
}

/// Create a new preset in the active profile.
#[tauri::command]
pub async fn create_preset(
    state: tauri::State<'_, AppState>,
    name: String,
    pan: f64,
    tilt: f64,
    zoom: f64,
    color: String,
) -> Result<Preset, String> {
    if !pan.is_finite() || !tilt.is_finite() || !zoom.is_finite() {
        return Err("Preset values must be finite numbers".to_string());
    }
    let name = name.chars().take(100).collect::<String>();
    if name.trim().is_empty() {
        return Err("Preset name cannot be empty".to_string());
    }
    let preset = Preset {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        pan: pan.clamp(-1.0, 1.0),
        tilt: tilt.clamp(-1.0, 1.0),
        zoom: zoom.clamp(0.0, 1.0),
        color,
    };
    let mut profiles = state.profiles.lock().await;
    profiles.ensure_default_profile()?;
    profiles.create_preset(preset)
}

/// Update an existing preset.
#[tauri::command]
pub async fn update_preset(
    state: tauri::State<'_, AppState>,
    preset: Preset,
) -> Result<Preset, String> {
    let mut profiles = state.profiles.lock().await;
    profiles.update_preset(preset)
}

/// Delete a preset by ID.
#[tauri::command]
pub async fn delete_preset(
    state: tauri::State<'_, AppState>,
    preset_id: String,
) -> Result<(), String> {
    let mut profiles = state.profiles.lock().await;
    profiles.delete_preset(&preset_id)
}

/// Get all profiles.
#[tauri::command]
pub async fn get_profiles(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<PresetProfile>, String> {
    let profiles = state.profiles.lock().await;
    Ok(profiles.get_profiles())
}

/// Save (create or update) a profile.
#[tauri::command]
pub async fn save_profile(
    state: tauri::State<'_, AppState>,
    profile: PresetProfile,
) -> Result<PresetProfile, String> {
    let mut profiles = state.profiles.lock().await;
    profiles.save_profile(profile)
}

/// Load (activate) a profile by ID.
#[tauri::command]
pub async fn load_profile(
    state: tauri::State<'_, AppState>,
    profile_id: String,
) -> Result<(), String> {
    let mut profiles = state.profiles.lock().await;
    profiles.set_active_profile(&profile_id)
}

/// Delete a profile by ID.
#[tauri::command]
pub async fn delete_profile(
    state: tauri::State<'_, AppState>,
    profile_id: String,
) -> Result<(), String> {
    let mut profiles = state.profiles.lock().await;
    profiles.delete_profile(&profile_id)
}
