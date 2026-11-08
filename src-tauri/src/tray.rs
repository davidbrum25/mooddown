use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};
use crate::browser::BrowserManager;
use crate::config::MooddownConfig;
use crate::kwin::KWinController;
use crate::settings_server;
use std::sync::Arc;

pub fn open_settings_window(_app: &AppHandle) {
    settings_server::open_settings_browser();
}

pub fn setup_tray(app: &AppHandle, browser_mgr: Arc<BrowserManager>) -> Result<(), Box<dyn std::error::Error>> {
    let toggle_i = MenuItem::with_id(app, "toggle", "Toggle AI Dropdown", true, None::<&str>)?;
    let gemini_i = MenuItem::with_id(app, "provider_gemini", "Google Gemini", true, None::<&str>)?;
    let claude_i = MenuItem::with_id(app, "provider_claude", "Anthropic Claude", true, None::<&str>)?;
    let grok_i = MenuItem::with_id(app, "provider_grok", "xAI Grok", true, None::<&str>)?;
    let chatgpt_i = MenuItem::with_id(app, "provider_chatgpt", "ChatGPT", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit Mooddown", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &toggle_i,
            &PredefinedMenuItem::separator(app)?,
            &gemini_i,
            &claude_i,
            &grok_i,
            &chatgpt_i,
            &PredefinedMenuItem::separator(app)?,
            &settings_i,
            &PredefinedMenuItem::separator(app)?,
            &quit_i,
        ],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Mooddown AI Dropdown")
        .on_menu_event({
            let app_handle = app.clone();
            let b_mgr = browser_mgr.clone();
            move |_tray, event| {
                let id = event.id.as_ref();
                match id {
                    "toggle" => {
                        let _ = KWinController::invoke_toggle();
                    }
                    "provider_gemini" | "provider_claude" | "provider_grok" | "provider_chatgpt" => {
                        let mut config = MooddownConfig::load();
                        let new_prov = match id {
                            "provider_gemini" => "gemini",
                            "provider_claude" => "claude",
                            "provider_grok" => "grok",
                            "provider_chatgpt" => "chatgpt",
                            _ => "gemini",
                        };
                        config.general.provider = new_prov.to_string();
                        let _ = config.save();
                        let url = config.get_active_url();
                        let _ = b_mgr.restart(&config.general.browser, &url);
                    }
                    "settings" => {
                        open_settings_window(&app_handle);
                    }
                    "quit" => {
                        settings_server::close_settings_browser();
                        b_mgr.kill_existing();
                        let _ = KWinController::unload();
                        app_handle.exit(0);
                    }
                    _ => {}
                }
            }
        })
        .on_tray_icon_event({
            let _app_handle = app.clone();
            move |_tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    let _ = KWinController::invoke_toggle();
                }
            }
        })
        .build(app)?;

    Ok(())
}
