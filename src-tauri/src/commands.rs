use tauri::{AppHandle, Manager, State};
use std::sync::Arc;
use crate::browser::{BrowserManager, DetectedBrowser};
use crate::config::MooddownConfig;
use crate::kwin::KWinController;

pub struct AppState {
    pub browser_manager: Arc<BrowserManager>,
}

#[tauri::command]
pub fn get_config() -> Result<MooddownConfig, String> {
    Ok(MooddownConfig::load())
}

#[tauri::command]
pub fn save_config(config: MooddownConfig, state: State<'_, AppState>) -> Result<(), String> {
    let old_config = MooddownConfig::load();
    config.save()?;

    // Reload KWin script with updated geometry/animation/hotkey settings
    let _ = KWinController::install_and_reload(&config);

    // If provider or browser changed, restart browser session with new URL
    if old_config.general.provider != config.general.provider
        || old_config.general.browser != config.general.browser
        || old_config.get_active_url() != config.get_active_url()
    {
        let url = config.get_active_url();
        let _ = state.browser_manager.restart(&config.general.browser, &url);
    }

    Ok(())
}

#[tauri::command]
pub fn toggle_dropdown(_state: State<'_, AppState>) -> Result<(), String> {
    KWinController::invoke_toggle()
}

#[tauri::command]
pub fn get_detected_browsers() -> Vec<DetectedBrowser> {
    BrowserManager::detect_browsers()
}

#[tauri::command]
pub fn open_profile_folder() -> Result<(), String> {
    let path = BrowserManager::profile_dir();
    let path_str = path.to_string_lossy().to_string();
    std::process::Command::new("xdg-open")
        .arg(&path_str)
        .spawn()
        .map_err(|e| format!("Failed to open folder: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn hide_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.close();
    }
    Ok(())
}
