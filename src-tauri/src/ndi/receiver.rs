/// NDI video receiver stub.
/// In production, wraps NDIlib_recv_instance_t.
pub struct NdiReceiver;

impl NdiReceiver {
    pub fn connect(_source_name: &str) -> Option<Self> {
        log::warn!("NDI SDK not linked — NdiReceiver unavailable");
        None
    }
}
