use std::fs;
use std::path::PathBuf;

pub struct AutostartManager;

impl AutostartManager {
    #[cfg(target_os = "linux")]
    pub fn desktop_file_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("autostart").join("mooddown.desktop")
    }

    pub fn sync(enabled: bool) -> Result<(), String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to resolve current executable path: {}", e))?;
        let exe_str = exe_path.to_string_lossy().to_string();

        #[cfg(target_os = "linux")]
        {
            let desktop_path = Self::desktop_file_path();
            if enabled {
                if let Some(parent) = desktop_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let content = format!(
                    "[Desktop Entry]\n\
                     Type=Application\n\
                     Name=Mooddown\n\
                     Comment=Dropdown AI Chat Companion (Gemini, Claude, Grok, ChatGPT)\n\
                     Exec=\"{}\"\n\
                     Icon=mooddown\n\
                     Terminal=false\n\
                     Categories=Utility;Network;Chat;\n\
                     StartupNotify=false\n\
                     X-KDE-autostart-phase=2\n\
                     X-GNOME-Autostart-enabled=true\n",
                    exe_str
                );
                fs::write(&desktop_path, content)
                    .map_err(|e| format!("Failed to write autostart file: {}", e))?;
            } else if desktop_path.exists() {
                fs::remove_file(&desktop_path)
                    .map_err(|e| format!("Failed to remove autostart file: {}", e))?;
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            if enabled {
                let _ = Command::new("reg")
                    .args([
                        "add",
                        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                        "/v",
                        "Mooddown",
                        "/t",
                        "REG_SZ",
                        "/d",
                        &format!("\"{}\"", exe_str),
                        "/f",
                    ])
                    .output();
            } else {
                let _ = Command::new("reg")
                    .args([
                        "delete",
                        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                        "/v",
                        "Mooddown",
                        "/f",
                    ])
                    .output();
            }
        }

        Ok(())
    }
}
