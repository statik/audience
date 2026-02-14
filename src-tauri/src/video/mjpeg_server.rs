use axum::{body::Body, http::header, response::Response, routing::get, Router};
use std::sync::Arc;
use tokio::sync::broadcast;

const BOUNDARY: &str = "mjpeg_boundary";

/// Shared state for the MJPEG server.
pub struct MjpegState {
    pub frame_sender: broadcast::Sender<Vec<u8>>,
}

impl Default for MjpegState {
    fn default() -> Self {
        Self::new()
    }
}

impl MjpegState {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(4); // Small buffer, drop old frames
        Self {
            frame_sender: sender,
        }
    }

    /// Push a JPEG-encoded frame to all connected clients.
    pub fn push_frame(&self, jpeg_data: Vec<u8>) {
        // Ignore send error (no receivers connected)
        let _ = self.frame_sender.send(jpeg_data);
    }
}

/// Handle for the MJPEG stream endpoint.
async fn stream_handler(state: axum::extract::State<Arc<MjpegState>>) -> Response<Body> {
    let mut receiver = state.frame_sender.subscribe();

    let stream = async_stream::stream! {
        loop {
            match receiver.recv().await {
                Ok(frame) => {
                    let part = format!(
                        "--{}\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                        BOUNDARY,
                        frame.len()
                    );
                    yield Ok::<_, std::io::Error>(bytes::Bytes::from(part));
                    yield Ok(bytes::Bytes::from(frame));
                    yield Ok(bytes::Bytes::from("\r\n"));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // Frames were dropped — skip ahead
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    Response::builder()
        .header(
            header::CONTENT_TYPE,
            format!("multipart/x-mixed-replace; boundary={}", BOUNDARY),
        )
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// Start the MJPEG HTTP server on a random available port.
/// Returns the port number and a shutdown sender.
/// Send `true` on the watch channel to gracefully shut down the server.
pub async fn start_server(
    state: Arc<MjpegState>,
) -> Result<(u16, tokio::sync::watch::Sender<bool>), String> {
    let app = Router::new()
        .route("/stream", get(stream_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| e.to_string())?;

    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                // Wait until shutdown signal is received
                while !*shutdown_rx.borrow_and_update() {
                    if shutdown_rx.changed().await.is_err() {
                        break;
                    }
                }
            })
            .await
        {
            log::error!("MJPEG server error on port {}: {}", port, e);
        }
        log::info!("MJPEG server on port {} shut down", port);
    });

    log::info!("MJPEG server started on port {}", port);
    Ok((port, shutdown_tx))
}
