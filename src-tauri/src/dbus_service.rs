use std::sync::Arc;
use zbus::{connection, interface};
use crate::browser::BrowserManager;
use crate::config::MooddownConfig;

pub struct MooddownDbusService {
    pub browser_manager: Arc<BrowserManager>,
}

#[interface(name = "com.mooddown.Daemon")]
impl MooddownDbusService {
    fn launch_browser(&self) {
        let config = MooddownConfig::load();
        let url = config.get_active_url();
        let _ = self.browser_manager.launch_or_navigate(&config.general.browser, &url);
    }

    fn toggle(&self) {
        let _ = crate::kwin::KWinController::invoke_toggle();
    }

    fn open_settings(&self) {
        crate::settings_server::open_settings_browser();
    }

    fn quit(&self) {
        crate::settings_server::close_settings_browser();
        self.browser_manager.kill_existing();
        let _ = crate::kwin::KWinController::unload();
        std::process::exit(0);
    }
}

pub fn try_forward_to_running_daemon(action: &str) -> bool {
    let method = match action {
        "settings" => "OpenSettings",
        "quit" => "Quit",
        _ => "Toggle",
    };

    if let Ok(output) = std::process::Command::new("qdbus6")
        .args([
            "com.mooddown.Daemon",
            "/Mooddown",
            &format!("com.mooddown.Daemon.{}", method),
        ])
        .output()
    {
        return output.status.success();
    }
    false
}

pub fn start_dbus_service(browser_manager: Arc<BrowserManager>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for D-Bus");
        rt.block_on(async move {
            let service = MooddownDbusService { browser_manager };
            if let Ok(builder) = connection::Builder::session() {
                if let Ok(builder) = builder.name("com.mooddown.Daemon") {
                    if let Ok(builder) = builder.serve_at("/Mooddown", service) {
                        if let Ok(_conn) = builder.build().await {
                            std::future::pending::<()>().await;
                        }
                    }
                }
            }
        });
    });
}
