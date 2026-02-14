use crate::ptz::types::{CameraEndpoint, ProtocolConfig};
use crate::AppState;

/// Get all configured camera endpoints.
#[tauri::command]
pub async fn get_endpoints(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<CameraEndpoint>, String> {
    let endpoints = state.endpoints.lock().await;
    Ok(endpoints.get_all())
}

/// Create a new camera endpoint.
#[tauri::command]
pub async fn create_endpoint(
    state: tauri::State<'_, AppState>,
    endpoint: CameraEndpoint,
) -> Result<CameraEndpoint, String> {
    let mut endpoints = state.endpoints.lock().await;
    endpoints.create(endpoint)
}

/// Update an existing camera endpoint.
#[tauri::command]
pub async fn update_endpoint(
    state: tauri::State<'_, AppState>,
    endpoint: CameraEndpoint,
) -> Result<CameraEndpoint, String> {
    let mut endpoints = state.endpoints.lock().await;
    endpoints.update(endpoint)
}

/// Delete a camera endpoint by ID.
#[tauri::command]
pub async fn delete_endpoint(
    state: tauri::State<'_, AppState>,
    endpoint_id: String,
) -> Result<(), String> {
    let mut endpoints = state.endpoints.lock().await;
    endpoints.delete(&endpoint_id)
}

/// Set the active camera endpoint.
#[tauri::command]
pub async fn set_active_endpoint(
    state: tauri::State<'_, AppState>,
    endpoint_id: String,
) -> Result<(), String> {
    // Verify endpoint exists
    let endpoints = state.endpoints.lock().await;
    endpoints
        .get(&endpoint_id)
        .ok_or("Endpoint not found")?;
    drop(endpoints);

    *state.active_endpoint_id.lock().await = Some(endpoint_id);
    Ok(())
}

/// Test connectivity to a camera endpoint.
#[tauri::command]
pub async fn test_endpoint_connection(
    config: ProtocolConfig,
) -> Result<String, String> {
    match config {
        ProtocolConfig::Ndi => {
            Ok("NDI connection test: NDI SDK not linked".to_string())
        }
        ProtocolConfig::Visca { host, port } => {
            use crate::ptz::controller::PtzController;
            use crate::visca::client::ViscaClient;
            let client = ViscaClient::new(&host, port)
                .map_err(|e| format!("VISCA init failed: {}", e))?;
            match client.test_connection().await {
                Ok(()) => Ok("VISCA connection successful".to_string()),
                Err(e) => Err(format!("VISCA connection failed: {}", e)),
            }
        }
        ProtocolConfig::PanasonicAw { host, port, .. } => {
            use crate::panasonic::client::PanasonicClient;
            use crate::ptz::controller::PtzController;
            let client = PanasonicClient::new(&host, port)
                .map_err(|e| format!("Panasonic init failed: {}", e))?;
            match client.test_connection().await {
                Ok(()) => Ok("Panasonic AW connection successful".to_string()),
                Err(e) => Err(format!("Panasonic AW connection failed: {}", e)),
            }
        }
        ProtocolConfig::BirdDogRest { host, port } => {
            use crate::birddog::client::BirdDogClient;
            use crate::ptz::controller::PtzController;
            let client = BirdDogClient::new(&host, port)
                .map_err(|e| format!("BirdDog init failed: {}", e))?;
            match client.test_connection().await {
                Ok(()) => Ok("BirdDog connection successful".to_string()),
                Err(e) => Err(format!("BirdDog connection failed: {}", e)),
            }
        }
    }
}
