pub mod browser;
pub mod cdp;
pub mod commands;
pub mod config;
#[cfg(target_os = "linux")]
pub mod dbus_service;
pub mod kwin;
pub mod tray;

use std::sync::Arc;
use tauri::RunEvent;
use browser::BrowserManager;
use commands::AppState;
use config::MooddownConfig;
use kwin::KWinController;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let browser_manager = Arc::new(BrowserManager::new());
    let browser_mgr_clone = browser_manager.clone();

    #[cfg(target_os = "linux")]
    {
        dbus_service::start_dbus_service(browser_manager.clone());
    }

    // Start Chrome DevTools Protocol guardian to block browser hotkeys & internal Chrome windows
    cdp::start_cdp_guardian(BrowserManager::profile_dir());

    let config = MooddownConfig::load();
    let _ = KWinController::install_and_reload(&config);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            browser_manager: browser_manager.clone(),
        })
        .setup(move |app| {
            let handle = app.handle();
            tray::setup_tray(handle, browser_mgr_clone)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::toggle_dropdown,
            commands::get_detected_browsers,
            commands::open_profile_folder,
            commands::hide_settings_window,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Mooddown application");

    app.run(|_app_handle, event| {
        if let RunEvent::ExitRequested { api, code, .. } = event {
            // Keep daemon running in tray when Settings window is closed
            if code.is_none() {
                api.prevent_exit();
            }
        }
    });
}
