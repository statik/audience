use crate::video::ndi_source::{self, NdiSource};
use crate::AppState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDevice {
    pub id: String,
    pub label: String,
}

/// List available NDI sources on the network.
#[tauri::command]
pub async fn list_ndi_sources() -> Result<Vec<NdiSource>, String> {
    Ok(ndi_source::discover_sources().await)
}

/// List local video capture devices.
/// Note: Primary device enumeration is done in the frontend via getUserMedia.
/// This command is for devices that require backend capture (FFmpeg fallback).
#[tauri::command]
pub async fn list_local_devices() -> Result<Vec<LocalDevice>, String> {
    // In production, this would enumerate devices via FFmpeg or OS APIs
    // For local getUserMedia devices, the frontend handles enumeration directly
    Ok(Vec::new())
}

/// Start the MJPEG stream server for NDI or fallback capture sources.
/// Returns the port number of the localhost MJPEG server.
#[tauri::command]
pub async fn start_mjpeg_stream(
    state: tauri::State<'_, AppState>,
) -> Result<u16, String> {
    use crate::video::mjpeg_server;
    use std::sync::Arc;

    let mjpeg_state = Arc::new(mjpeg_server::MjpegState::new());
    let port = mjpeg_server::start_server(mjpeg_state).await?;

    *state.mjpeg_port.lock().await = Some(port);
    Ok(port)
}

/// Stop the MJPEG stream server.
#[tauri::command]
pub async fn stop_mjpeg_stream(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    *state.mjpeg_port.lock().await = None;
    // The server task will stop when all references are dropped
    Ok(())
}

/// Get the current MJPEG server port, if running.
#[tauri::command]
pub async fn get_mjpeg_port(
    state: tauri::State<'_, AppState>,
) -> Result<Option<u16>, String> {
    Ok(*state.mjpeg_port.lock().await)
}
