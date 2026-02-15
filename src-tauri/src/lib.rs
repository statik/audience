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

use tauri::Manager;

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
            app.manage(state);
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
            commands::presets::get_all_presets,
            commands::presets::create_preset,
            commands::presets::update_preset,
            commands::presets::delete_preset,
            commands::presets::get_profiles,
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
