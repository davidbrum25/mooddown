use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub provider: String,
    pub custom_url: String,
    pub browser: String,
    pub hotkey: String,
    pub autohide_on_blur: bool,
    pub pinned: bool,
    pub start_at_boot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUrls {
    pub gemini: String,
    pub claude: String,
    pub grok: String,
    pub chatgpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width_percent: u32,
    pub height_percent: u32,
    pub animation_duration_ms: u32,
    pub target_screen: String, // "cursor" or "0", "1"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MooddownConfig {
    pub general: GeneralConfig,
    pub providers: ProviderUrls,
    pub window: WindowConfig,
}

impl Default for MooddownConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                provider: "gemini".to_string(),
                custom_url: "https://aistudio.google.com".to_string(),
                browser: "auto".to_string(),
                hotkey: "F12".to_string(),
                autohide_on_blur: true,
                pinned: false,
                start_at_boot: true,
            },
            providers: ProviderUrls {
                gemini: "https://gemini.google.com".to_string(),
                claude: "https://claude.ai".to_string(),
                grok: "https://grok.com".to_string(),
                chatgpt: "https://chatgpt.com".to_string(),
            },
            window: WindowConfig {
                width_percent: 65,
                height_percent: 55,
                animation_duration_ms: 180,
                target_screen: "cursor".to_string(),
            },
        }
    }
}

impl MooddownConfig {
    pub fn config_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("mooddown").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<MooddownConfig>(&content) {
                    return config;
                }
            }
        }
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let serialized = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, serialized).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_active_url(&self) -> String {
        match self.general.provider.as_str() {
            "gemini" => self.providers.gemini.clone(),
            "claude" => self.providers.claude.clone(),
            "grok" => self.providers.grok.clone(),
            "chatgpt" => self.providers.chatgpt.clone(),
            "custom" => self.general.custom_url.clone(),
            _ => self.providers.gemini.clone(),
        }
    }
}
