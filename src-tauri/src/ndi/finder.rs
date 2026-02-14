use crate::video::ndi_source::NdiSource;

/// NDI source finder stub.
/// In production, wraps NDIlib_find_instance_t.
pub struct NdiFinder;

impl NdiFinder {
    pub fn new() -> Option<Self> {
        log::warn!("NDI SDK not linked — NdiFinder unavailable");
        None
    }

    pub fn get_sources(&self) -> Vec<NdiSource> {
        Vec::new()
    }
}
