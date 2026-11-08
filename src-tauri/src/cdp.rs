use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CdpTarget {
    id: String,
    #[serde(rename = "type")]
    target_type: String,
    url: String,
}

const MOODDOWN_GUARD_JS: &str = r#"
(function() {
    if (window.__mooddownHotkeyGuardInstalled) return;
    window.__mooddownHotkeyGuardInstalled = true;

    function blockEvent(e) {
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        return false;
    }

    window.addEventListener("keydown", function(e) {
        var key = (e.key || "").toLowerCase();

        // Block function keys that trigger Chrome UI (F1 Help, F3 Find, F5 Reload, F7 Caret, F11 Fullscreen, F12 DevTools)
        if (["f1", "f3", "f5", "f7", "f11", "f12"].indexOf(key) >= 0) {
            return blockEvent(e);
        }

        // Block Alt shortcuts that trigger Chrome navigation or menus (Alt+F, Alt+E, Alt+D, Alt+Home, Alt+Left, Alt+Right)
        if (e.altKey && !e.ctrlKey && !e.metaKey) {
            if (["f", "e", "d", "home", "arrowleft", "arrowright"].indexOf(key) >= 0) {
                return blockEvent(e);
            }
        }

        if (e.ctrlKey || e.metaKey) {
            // Block Ctrl+Shift browser shortcuts (Incognito, DevTools, Bookmarks, Clear Data, Reopen Tab, Close Window)
            if (e.shiftKey) {
                if (["n", "t", "w", "i", "j", "c", "o", "b", "m", "q", "r", "delete", "backspace"].indexOf(key) >= 0) {
                    return blockEvent(e);
                }
            }

            // Block Ctrl browser shortcuts:
            // n (New Window), t (New Tab), w (Close Tab), j (Downloads), h (History),
            // p (Print), s (Save Page), o (Open File), u (View Source), d (Bookmark),
            // g (Find Next), f (Find), l (Location bar), r (Reload), m (Mute)
            if (["n", "t", "w", "j", "h", "p", "s", "o", "u", "d", "g", "f", "l", "r", "m", "f4", "f5", "tab", "pageup", "pagedown"].indexOf(key) >= 0) {
                return blockEvent(e);
            }
        }
    }, true);
})();
"#;

fn http_get(port: u16, path: &str) -> Option<String> {
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().ok()?,
        Duration::from_millis(200),
    ).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(250)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));

    let req = format!(
        "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        path, port
    );
    stream.write_all(req.as_bytes()).ok()?;

    let mut resp = String::new();
    stream.read_to_string(&mut resp).ok()?;

    let body_start = resp.find("\r\n\r\n")? + 4;
    Some(resp[body_start..].to_string())
}

fn send_ws_frame(stream: &mut TcpStream, payload: &str) -> Result<(), std::io::Error> {
    let bytes = payload.as_bytes();
    let len = bytes.len();
    let mask: [u8; 4] = [0x37, 0xfa, 0x21, 0x3d];

    let mut frame = Vec::with_capacity(14 + len);
    frame.push(0x81); // FIN + Text opcode

    if len <= 125 {
        frame.push(0x80 | (len as u8));
    } else if len <= 65535 {
        frame.push(0x80 | 126);
        frame.push(((len >> 8) & 0xff) as u8);
        frame.push((len & 0xff) as u8);
    } else {
        frame.push(0x80 | 127);
        for i in (0..8).rev() {
            frame.push(((len >> (i * 8)) & 0xff) as u8);
        }
    }

    frame.extend_from_slice(&mask);
    for (i, &b) in bytes.iter().enumerate() {
        frame.push(b ^ mask[i % 4]);
    }

    stream.write_all(&frame)
}

fn inject_guard_into_target(port: u16, target_id: &str) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    let Ok(sock_addr) = addr.parse() else { return false };
    let Ok(mut stream) = TcpStream::connect_timeout(&sock_addr, Duration::from_millis(200)) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(200)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));

    let handshake = format!(
        "GET /devtools/page/{} HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         Sec-WebSocket-Version: 13\r\n\r\n",
        target_id, port
    );

    if stream.write_all(handshake.as_bytes()).is_err() {
        return false;
    }

    let mut buf = [0u8; 1024];
    let Ok(n) = stream.read(&mut buf) else { return false };
    let resp = String::from_utf8_lossy(&buf[..n]);
    if !resp.contains("101 Switching Protocols") {
        return false;
    }

    let cmd1 = serde_json::json!({
        "id": 1,
        "method": "Page.enable"
    }).to_string();

    let cmd2 = serde_json::json!({
        "id": 2,
        "method": "Page.addScriptToEvaluateOnNewDocument",
        "params": {
            "source": MOODDOWN_GUARD_JS
        }
    }).to_string();

    let cmd3 = serde_json::json!({
        "id": 3,
        "method": "Runtime.evaluate",
        "params": {
            "expression": MOODDOWN_GUARD_JS
        }
    }).to_string();

    let _ = send_ws_frame(&mut stream, &cmd1);
    let _ = send_ws_frame(&mut stream, &cmd2);
    let _ = send_ws_frame(&mut stream, &cmd3);

    let _ = stream.read(&mut buf);
    true
}

fn is_forbidden_url(url: &str) -> bool {
    let u = url.to_lowercase();
    u.starts_with("chrome://")
        || u.starts_with("chrome-untrusted://")
        || u.starts_with("view-source:")
        || u.starts_with("edge://")
        || u.starts_with("brave://")
        || u == "about:blank"
}

pub fn start_cdp_guardian(profile_dir: PathBuf) {
    std::thread::spawn(move || {
        let port_file = profile_dir.join("DevToolsActivePort");
        let mut main_target_id: Option<String> = None;
        let mut injected_urls: HashMap<String, String> = HashMap::new();
        let mut last_inject = Instant::now();

        loop {
            std::thread::sleep(Duration::from_millis(250));

            let Ok(content) = fs::read_to_string(&port_file) else {
                main_target_id = None;
                injected_urls.clear();
                continue;
            };

            let Some(first_line) = content.lines().next() else { continue };
            let Ok(port) = first_line.trim().parse::<u16>() else { continue };

            let Some(json_body) = http_get(port, "/json/list") else {
                continue;
            };

            let Ok(targets) = serde_json::from_str::<Vec<CdpTarget>>(&json_body) else {
                continue;
            };

            let page_targets: Vec<&CdpTarget> = targets
                .iter()
                .filter(|t| t.target_type == "page")
                .collect();

            if let Some(ref main_id) = main_target_id {
                if !page_targets.iter().any(|t| &t.id == main_id) {
                    main_target_id = None;
                }
            }

            for t in page_targets {
                // Immediately close any internal Chrome page (Downloads, New Tab, History, Settings, etc.)
                if is_forbidden_url(&t.url) {
                    let _ = http_get(port, &format!("/json/close/{}", t.id));
                    continue;
                }

                if main_target_id.is_none() {
                    main_target_id = Some(t.id.clone());
                }

                // Close any secondary window/tab (e.g. Ctrl+N, Ctrl+T, popup window)
                if main_target_id.as_deref() != Some(t.id.as_str()) {
                    let _ = http_get(port, &format!("/json/close/{}", t.id));
                    continue;
                }

                // Inject hotkey blocker when URL changes or periodically every 2s
                let url_changed = injected_urls.get(&t.id) != Some(&t.url);
                if url_changed || last_inject.elapsed() >= Duration::from_secs(2) {
                    if inject_guard_into_target(port, &t.id) {
                        injected_urls.insert(t.id.clone(), t.url.clone());
                        last_inject = Instant::now();
                    }
                }
            }
        }
    });
}
