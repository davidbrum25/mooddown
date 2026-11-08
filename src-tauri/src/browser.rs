use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DetectedBrowser {
    pub name: String,
    pub path: String,
    pub is_available: bool,
}

pub struct BrowserManager {
    child_process: Arc<Mutex<Option<Child>>>,
    active_pid: Arc<Mutex<Option<u32>>>,
}

impl BrowserManager {
    pub fn new() -> Self {
        Self {
            child_process: Arc::new(Mutex::new(None)),
            active_pid: Arc::new(Mutex::new(None)),
        }
    }

    pub fn profile_dir() -> PathBuf {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = base.join("mooddown").join("profile");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    pub fn detect_browsers() -> Vec<DetectedBrowser> {
        let candidates = vec![
            ("Google Chrome", vec!["google-chrome-stable", "google-chrome", "chrome"]),
            ("Brave Browser", vec!["brave-browser", "brave"]),
            ("Chromium", vec!["chromium", "chromium-browser"]),
            ("Microsoft Edge", vec!["microsoft-edge-stable", "microsoft-edge", "msedge"]),
        ];

        let mut results = Vec::new();

        for (name, binaries) in candidates {
            let mut found_path = None;
            for bin in binaries {
                if let Ok(output) = Command::new("which").arg(bin).output() {
                    if output.status.success() {
                        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !path_str.is_empty() {
                            found_path = Some(path_str);
                            break;
                        }
                    }
                }
            }

            results.push(DetectedBrowser {
                name: name.to_string(),
                path: found_path.clone().unwrap_or_default(),
                is_available: found_path.is_some(),
            });
        }

        results
    }

    pub fn resolve_browser_binary(configured: &str) -> Option<String> {
        if configured != "auto" && !configured.is_empty() {
            if Path::new(configured).exists() {
                return Some(configured.to_string());
            }
            if let Ok(output) = Command::new("which").arg(configured).output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        return Some(path_str);
                    }
                }
            }
        }

        let detected = Self::detect_browsers();
        for b in detected {
            if b.is_available {
                return Some(b.path);
            }
        }
        None
    }

    pub fn is_running(&self) -> bool {
        let mut guard = self.child_process.lock().unwrap();
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    *guard = None;
                    let mut pid_guard = self.active_pid.lock().unwrap();
                    *pid_guard = None;
                }
                Ok(None) => return true,
                Err(_) => {}
            }
        }
        drop(guard);

        // Also check via pgrep if a browser process with our profile is active
        let profile_str = Self::profile_dir().to_string_lossy().to_string();
        if let Ok(output) = Command::new("pgrep").args(["-f", &profile_str]).output() {
            if output.status.success() {
                return true;
            }
        }
        false
    }

    pub fn get_active_pid(&self) -> Option<u32> {
        *self.active_pid.lock().unwrap()
    }

    pub fn launch_or_navigate(&self, browser_binary: &str, target_url: &str) -> Result<u32, String> {
        let bin = Self::resolve_browser_binary(browser_binary)
            .ok_or_else(|| "No compatible Chromium browser found (Chrome, Brave, Chromium, Edge). Please install one or configure its path.".to_string())?;

        let profile_dir = Self::profile_dir();
        let profile_str = profile_dir.to_string_lossy().to_string();

        let mut cmd = Command::new(&bin);
        cmd.arg(format!("--app={}", target_url))
            .arg(format!("--user-data-dir={}", profile_str))
            .arg("--class=mooddown-app")
            .arg("--name=mooddown-app")
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--disable-features=Vulkan,Translate")
            .arg("--window-size=1200,750");

        let child = cmd.spawn().map_err(|e| format!("Failed to launch browser ({}): {}", bin, e))?;
        let pid = child.id();

        let mut guard = self.child_process.lock().unwrap();
        *guard = Some(child);

        let mut pid_guard = self.active_pid.lock().unwrap();
        *pid_guard = Some(pid);

        Ok(pid)
    }

    pub fn kill_existing(&self) {
        let mut guard = self.child_process.lock().unwrap();
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        drop(guard);

        let profile_str = Self::profile_dir().to_string_lossy().to_string();
        let _ = Command::new("pkill").args(["-f", &profile_str]).output();
        std::thread::sleep(std::time::Duration::from_millis(150));
    }

    pub fn restart(&self, browser_binary: &str, target_url: &str) -> Result<u32, String> {
        self.kill_existing();
        self.launch_or_navigate(browser_binary, target_url)
    }
}
