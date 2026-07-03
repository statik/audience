pub mod commands;
pub mod ndi;
pub mod persistence;
pub mod ptz;
pub mod video;

// Protocol-specific modules
pub mod birddog;
pub mod panasonic;
pub mod simulator;
pub mod visca;

use tauri::{Emitter, Manager};

use persistence::config::AppConfig;
use persistence::profiles::ProfileStore;
use ptz::controller::PtzDispatcher;
use ptz::endpoint_manager::EndpointManager;
use ptz::types::PtzPosition;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared application state accessible from all Tauri commands.
pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub profiles: Arc<Mutex<ProfileStore>>,
    pub endpoints: Arc<Mutex<EndpointManager>>,
    pub current_position: Arc<Mutex<PtzPosition>>,
    pub active_endpoint_id: Arc<Mutex<Option<String>>>,
    pub ptz_dispatcher: Arc<Mutex<PtzDispatcher>>,
    pub mjpeg_port: Arc<Mutex<Option<u16>>>,
    pub mjpeg_shutdown: Arc<Mutex<Option<tokio::sync::watch::Sender<bool>>>>,
}

impl AppState {
    pub fn new(data_dir: std::path::PathBuf) -> Self {
        let config = AppConfig::load_or_default(&data_dir);
        let profiles = ProfileStore::load_or_default(&data_dir);
        let endpoints = EndpointManager::load_or_default(&data_dir);

        Self {
            config: Arc::new(Mutex::new(config)),
            profiles: Arc::new(Mutex::new(profiles)),
            endpoints: Arc::new(Mutex::new(endpoints)),
            current_position: Arc::new(Mutex::new(PtzPosition::default())),
            active_endpoint_id: Arc::new(Mutex::new(None)),
            ptz_dispatcher: Arc::new(Mutex::new(PtzDispatcher::new())),
            mjpeg_port: Arc::new(Mutex::new(None)),
            mjpeg_shutdown: Arc::new(Mutex::new(None)),
        }
    }
}

/// Payload of the `ptz-health` event emitted by the connection heartbeat.
#[derive(Clone, serde::Serialize)]
struct PtzHealthEvent {
    status: &'static str,
    message: Option<String>,
}

const PTZ_HEARTBEAT_INTERVAL_SECS: u64 = 10;

/// Periodically probe the active PTZ controller and push the result to the
/// frontend, so a camera that drops off the network is noticed without
/// waiting for the operator's next command.
fn spawn_ptz_heartbeat(
    app_handle: tauri::AppHandle,
    dispatcher: Arc<Mutex<PtzDispatcher>>,
    active_endpoint_id: Arc<Mutex<Option<String>>>,
) {
    tauri::async_runtime::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(PTZ_HEARTBEAT_INTERVAL_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            interval.tick().await;

            if active_endpoint_id.lock().await.is_none() {
                continue;
            }
            // Holding the dispatcher lock serializes the probe with PTZ
            // commands; probes answer in milliseconds on a healthy camera
            // and time out (2s) only when it is already unreachable.
            let guard = dispatcher.lock().await;
            if !guard.has_controller() {
                continue;
            }
            let result = guard.test_connection().await;
            drop(guard);

            let payload = match result {
                Ok(()) => PtzHealthEvent {
                    status: "ok",
                    message: None,
                },
                Err(e) => PtzHealthEvent {
                    status: "error",
                    message: Some(e.to_string()),
                },
            };
            if let Err(e) = app_handle.emit("ptz-health", &payload) {
                log::warn!("Failed to emit ptz-health event: {}", e);
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");
            std::fs::create_dir_all(&data_dir).expect("Failed to create app data directory");

            let state = AppState::new(data_dir);
            let dispatcher = state.ptz_dispatcher.clone();
            let active_endpoint_id = state.active_endpoint_id.clone();
            app.manage(state);

            spawn_ptz_heartbeat(app.handle().clone(), dispatcher, active_endpoint_id);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::video::list_ndi_sources,
            commands::video::list_local_devices,
            commands::video::start_mjpeg_stream,
            commands::video::stop_mjpeg_stream,
            commands::video::get_mjpeg_port,
            commands::ptz::ptz_move_relative,
            commands::ptz::ptz_move_absolute,
            commands::ptz::ptz_zoom,
            commands::ptz::ptz_recall_preset,
            commands::ptz::ptz_store_preset,
            commands::ptz::ptz_get_position,
            commands::ptz::ptz_home,
            commands::ptz::ptz_continuous_move,
            commands::ptz::ptz_stop,
            commands::ptz::ptz_focus,
            commands::ptz::ptz_focus_stop,
            commands::ptz::ptz_set_autofocus,
            commands::ptz::ptz_autofocus_trigger,
            commands::ptz::ptz_get_capabilities,
            commands::presets::get_all_presets,
            commands::presets::create_preset,
            commands::presets::update_preset,
            commands::presets::delete_preset,
            commands::presets::get_profiles,
            commands::presets::get_active_profile_id,
            commands::presets::save_profile,
            commands::presets::load_profile,
            commands::presets::delete_profile,
            commands::endpoints::get_endpoints,
            commands::endpoints::create_endpoint,
            commands::endpoints::update_endpoint,
            commands::endpoints::delete_endpoint,
            commands::endpoints::set_active_endpoint,
            commands::endpoints::clear_active_endpoint,
            commands::endpoints::test_endpoint_connection,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
