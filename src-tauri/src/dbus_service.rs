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
