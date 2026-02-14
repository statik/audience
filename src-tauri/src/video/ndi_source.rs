use serde::{Deserialize, Serialize};

/// Represents a discovered NDI source on the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdiSource {
    pub name: String,
    pub url: String,
}

/// Stub for NDI source discovery.
/// Real implementation requires NDI SDK FFI bindings via bindgen.
/// The NDI SDK is proprietary and must be installed separately.
pub async fn discover_sources() -> Vec<NdiSource> {
    // In production, this would:
    // 1. Initialize NDIlib_find_create_t
    // 2. Call NDIlib_find_create_v2
    // 3. Wait for sources via NDIlib_find_wait_for_sources
    // 4. Get sources via NDIlib_find_get_current_sources
    // 5. Map to NdiSource structs
    log::info!("NDI source discovery: NDI SDK not linked — returning empty list");
    Vec::new()
}
