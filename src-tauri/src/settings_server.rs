use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use crate::autostart::AutostartManager;
use crate::browser::BrowserManager;
use crate::config::MooddownConfig;
use crate::kwin::KWinController;

const INDEX_HTML: &str = include_str!("../../dist/index.html");
const APP_CSS: &str = include_str!("../../dist/assets/app.css");
const APP_JS: &str = include_str!("../../dist/assets/app.js");
const ICON_SVG: &str = include_str!("../icons/icon.svg");

static SETTINGS_PORT: AtomicU16 = AtomicU16::new(0);

pub fn settings_profile_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("mooddown").join("settings-profile");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn close_settings_browser() {
    let profile_str = settings_profile_dir().to_string_lossy().to_string();
    let _ = Command::new("pkill").args(["-f", &profile_str]).output();
}

pub fn open_settings_browser() {
    let port = SETTINGS_PORT.load(Ordering::SeqCst);
    if port == 0 {
        return;
    }

    let config = MooddownConfig::load();
    let Some(bin) = BrowserManager::resolve_browser_binary(&config.general.browser) else {
        return;
    };

    // Close any existing settings window so the new one maps and focuses cleanly on Wayland
    close_settings_browser();

    let profile_str = settings_profile_dir().to_string_lossy().to_string();
    let url = format!("http://127.0.0.1:{}/", port);

    let _ = Command::new(&bin)
        .arg(format!("--app={}", url))
        .arg(format!("--user-data-dir={}", profile_str))
        .arg("--class=mooddown-settings")
        .arg("--name=mooddown-settings")
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--disable-extensions")
        .arg("--disable-default-apps")
        .arg("--disable-component-extensions-with-background-pages")
        .arg("--disable-features=Vulkan,Translate,GlobalMediaControls,MediaRouter,DialMediaRouteProvider")
        .arg("--window-size=640,860")
        .spawn();
}

fn send_response(mut stream: TcpStream, status: &str, content_type: &str, body: &str) {
    let bytes = body.as_bytes();
    let header = format!(
        "HTTP/1.1 {}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\r\n",
        status,
        content_type,
        bytes.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(bytes);
    let _ = stream.flush();
}

fn handle_client(mut stream: TcpStream, browser_manager: Arc<BrowserManager>) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
    let mut buf = [0u8; 16384];
    let Ok(mut total_read) = stream.read(&mut buf) else { return };
    if total_read == 0 {
        return;
    }

    let req_str = String::from_utf8_lossy(&buf[..total_read]).to_string();
    let Some(first_line) = req_str.lines().next() else { return };
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path = parts[1];

    // If POST with Content-Length, ensure full body is read
    let mut body = String::new();
    if let Some(header_end) = req_str.find("\r\n\r\n") {
        let headers_part = &req_str[..header_end];
        let mut content_len = 0usize;
        for line in headers_part.lines() {
            let lower = line.to_lowercase();
            if lower.starts_with("content-length:") {
                if let Ok(len) = line[15..].trim().parse::<usize>() {
                    content_len = len;
                }
            }
        }
        let body_start_bytes = header_end + 4;
        while total_read < body_start_bytes + content_len && total_read < buf.len() {
            match stream.read(&mut buf[total_read..]) {
                Ok(0) | Err(_) => break,
                Ok(n) => total_read += n,
            }
        }
        let full_str = String::from_utf8_lossy(&buf[..total_read]);
        if full_str.len() >= body_start_bytes {
            body = full_str[body_start_bytes..].to_string();
        }
    }

    match (method, path) {
        ("GET", "/") | ("GET", "/index.html") => {
            send_response(stream, "200 OK", "text/html; charset=utf-8", INDEX_HTML);
        }
        ("GET", "/assets/app.css") => {
            send_response(stream, "200 OK", "text/css; charset=utf-8", APP_CSS);
        }
        ("GET", "/assets/app.js") => {
            send_response(stream, "200 OK", "application/javascript; charset=utf-8", APP_JS);
        }
        ("GET", "/icon.svg") | ("GET", "/favicon.ico") => {
            send_response(stream, "200 OK", "image/svg+xml; charset=utf-8", ICON_SVG);
        }
        ("GET", "/api/config") => {
            let cfg = MooddownConfig::load();
            let json = serde_json::to_string(&cfg).unwrap_or_else(|_| "{}".to_string());
            send_response(stream, "200 OK", "application/json", &json);
        }
        ("GET", "/api/browsers") => {
            let list = BrowserManager::detect_browsers();
            let json = serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string());
            send_response(stream, "200 OK", "application/json", &json);
        }
        ("POST", "/api/save") => {
            match serde_json::from_str::<MooddownConfig>(&body) {
                Ok(new_cfg) => {
                    let old_cfg = MooddownConfig::load();
                    if let Err(e) = new_cfg.save() {
                        send_response(stream, "500 Internal Server Error", "text/plain", &e);
                        return;
                    }
                    let _ = AutostartManager::sync(new_cfg.general.start_at_boot);
                    let _ = KWinController::install_and_reload(&new_cfg);

                    if old_cfg.general.provider != new_cfg.general.provider
                        || old_cfg.general.browser != new_cfg.general.browser
                        || old_cfg.get_active_url() != new_cfg.get_active_url()
                    {
                        let url = new_cfg.get_active_url();
                        let _ = browser_manager.restart(&new_cfg.general.browser, &url);
                    }
                    send_response(stream, "200 OK", "application/json", r#"{"ok":true}"#);
                }
                Err(e) => {
                    send_response(stream, "400 Bad Request", "text/plain", &e.to_string());
                }
            }
        }
        ("POST", "/api/toggle") => {
            let _ = KWinController::invoke_toggle();
            send_response(stream, "200 OK", "application/json", r#"{"ok":true}"#);
        }
        ("POST", "/api/open-profile") => {
            let path = BrowserManager::profile_dir();
            let _ = Command::new("xdg-open")
                .arg(path.to_string_lossy().to_string())
                .spawn();
            send_response(stream, "200 OK", "application/json", r#"{"ok":true}"#);
        }
        ("POST", "/api/close") => {
            send_response(stream, "200 OK", "application/json", r#"{"ok":true}"#);
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(80));
                close_settings_browser();
            });
        }
        _ => {
            send_response(stream, "404 Not Found", "text/plain", "Not Found");
        }
    }
}

pub fn start_settings_server(browser_manager: Arc<BrowserManager>) {
    std::thread::spawn(move || {
        let Ok(listener) = TcpListener::bind("127.0.0.1:0") else {
            return;
        };
        if let Ok(addr) = listener.local_addr() {
            SETTINGS_PORT.store(addr.port(), Ordering::SeqCst);
        }

        for stream in listener.incoming().flatten() {
            let b_mgr = browser_manager.clone();
            std::thread::spawn(move || {
                handle_client(stream, b_mgr);
            });
        }
    });
}
