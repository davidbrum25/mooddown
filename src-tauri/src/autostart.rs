use std::fs;
use std::path::PathBuf;

const APP_ICON_PNG: &[u8] = include_bytes!("../icons/icon.png");
const APP_ICON_SVG: &[u8] = include_bytes!("../icons/icon.svg");

pub struct AutostartManager;

impl AutostartManager {
    #[cfg(target_os = "linux")]
    pub fn desktop_file_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("autostart").join("mooddown.desktop")
    }

    #[cfg(target_os = "linux")]
    pub fn app_menu_desktop_path() -> PathBuf {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("applications").join("mooddown.desktop")
    }

    #[cfg(target_os = "linux")]
    pub fn icon_file_path() -> PathBuf {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("icons")
            .join("hicolor")
            .join("scalable")
            .join("apps")
            .join("mooddown.svg")
    }

    #[cfg(target_os = "linux")]
    pub fn icon_png_path() -> PathBuf {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("icons")
            .join("hicolor")
            .join("256x256")
            .join("apps")
            .join("mooddown.png")
    }

    #[cfg(target_os = "linux")]
    pub fn install_app_menu_launcher() -> Result<(), String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to resolve current executable path: {}", e))?;
        let exe_str = exe_path.to_string_lossy().to_string();

        // Install SVG & PNG icons in ~/.local/share/icons/hicolor/
        let icon_svg_path = Self::icon_file_path();
        if let Some(parent) = icon_svg_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&icon_svg_path, APP_ICON_SVG);

        let icon_png = Self::icon_png_path();
        if let Some(parent) = icon_png.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&icon_png, APP_ICON_PNG);

        let icon_str = icon_svg_path.to_string_lossy().to_string();

        // Install KDE / FreeDesktop Application Menu entry in ~/.local/share/applications/mooddown.desktop
        let app_desktop_path = Self::app_menu_desktop_path();
        if let Some(parent) = app_desktop_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Version=1.0\n\
             Name=Mooddown\n\
             GenericName=AI Dropdown Companion\n\
             Comment=Quake-style Dropdown AI Chat Companion (Gemini, Claude, Grok, ChatGPT)\n\
             Exec=\"{exe}\" --toggle\n\
             Icon={icon}\n\
             Terminal=false\n\
             Categories=Utility;Network;Chat;\n\
             Keywords=AI;Gemini;Claude;Grok;ChatGPT;Dropdown;Assistant;Mooddown;\n\
             StartupNotify=false\n\
             Actions=Toggle;Settings;Quit;\n\
             \n\
             [Desktop Action Toggle]\n\
             Name=Toggle AI Dropdown\n\
             Name[es]=Abrir / Ocultar Dropdown IA\n\
             Exec=\"{exe}\" --toggle\n\
             \n\
             [Desktop Action Settings]\n\
             Name=Settings...\n\
             Name[es]=Configuración...\n\
             Exec=\"{exe}\" --settings\n\
             \n\
             [Desktop Action Quit]\n\
             Name=Quit Mooddown\n\
             Name[es]=Salir de Mooddown\n\
             Exec=\"{exe}\" --quit\n",
            exe = exe_str,
            icon = icon_str
        );

        fs::write(&app_desktop_path, content)
            .map_err(|e| format!("Failed to write KDE application menu file: {}", e))?;

        // Refresh KDE sycoca cache asynchronously so the launcher appears immediately
        let _ = std::process::Command::new("kbuildsycoca6").spawn();
        let _ = std::process::Command::new("update-desktop-database")
            .arg(app_desktop_path.parent().unwrap())
            .spawn();

        Ok(())
    }

    pub fn sync(enabled: bool) -> Result<(), String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to resolve current executable path: {}", e))?;
        let exe_str = exe_path.to_string_lossy().to_string();

        #[cfg(target_os = "linux")]
        {
            let _ = Self::install_app_menu_launcher();
            let icon_str = Self::icon_file_path().to_string_lossy().to_string();

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
                     Exec=\"{}\" --autostart\n\
                     Icon={}\n\
                     Terminal=false\n\
                     Categories=Utility;Network;Chat;\n\
                     StartupNotify=false\n\
                     X-KDE-autostart-phase=2\n\
                     X-GNOME-Autostart-enabled=true\n",
                    exe_str, icon_str
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
