use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    #[serde(default)]
    pub api_version: String,
    #[serde(default)]
    pub tts_api_key: String,
    #[serde(default)]
    pub tts_base_url: String,
    #[serde(default)]
    pub tts_api_version: String,
    #[serde(default = "default_tts_model")]
    pub tts_model: String,
    #[serde(default = "default_tts_voice")]
    pub tts_voice: String,
}

fn default_tts_model() -> String {
    "tts".to_string()
}

fn default_tts_voice() -> String {
    "alloy".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o".to_string(),
            api_version: String::new(),
            tts_api_key: String::new(),
            tts_base_url: String::new(),
            tts_api_version: String::new(),
            tts_model: default_tts_model(),
            tts_voice: default_tts_voice(),
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(dir.join("settings.json"))
}

pub fn load(app: &AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let data = fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse settings: {}", e))
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let data = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, data).map_err(|e| format!("Failed to write settings: {}", e))
}
