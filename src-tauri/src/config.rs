use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameProfile {
    pub id: String,
    pub name: String,
    pub exe_name: String,
    pub enabled: bool,
    pub high_priority: bool,
    /// `None` means use all logical cores; `Some(cores)` restricts to those 0-based core indices.
    pub affinity_cores: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundApp {
    pub exe_name: String,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub language: String,
    pub auto_detect: bool,
    pub poll_interval_ms: u64,
    pub power_plan_boost: bool,
    pub high_res_timer: bool,
    pub eco_qos_enabled: bool,
    pub game_profiles: Vec<GameProfile>,
    pub background_apps: Vec<BackgroundApp>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "tr".to_string(),
            auto_detect: true,
            poll_interval_ms: 2500,
            power_plan_boost: true,
            high_res_timer: true,
            eco_qos_enabled: true,
            game_profiles: vec![
                GameProfile {
                    id: "lol".into(),
                    name: "League of Legends".into(),
                    exe_name: "League of Legends.exe".into(),
                    enabled: true,
                    high_priority: true,
                    affinity_cores: None,
                },
                GameProfile {
                    id: "valorant".into(),
                    name: "VALORANT".into(),
                    exe_name: "VALORANT-Win64-Shipping.exe".into(),
                    enabled: true,
                    high_priority: true,
                    affinity_cores: None,
                },
                GameProfile {
                    id: "cs2".into(),
                    name: "Counter-Strike 2".into(),
                    exe_name: "cs2.exe".into(),
                    enabled: true,
                    high_priority: true,
                    affinity_cores: None,
                },
                GameProfile {
                    id: "apex".into(),
                    name: "Apex Legends".into(),
                    exe_name: "r5apex.exe".into(),
                    enabled: true,
                    high_priority: true,
                    affinity_cores: None,
                },
                GameProfile {
                    id: "cyberpunk".into(),
                    name: "Cyberpunk 2077".into(),
                    exe_name: "Cyberpunk2077.exe".into(),
                    enabled: true,
                    high_priority: true,
                    affinity_cores: None,
                },
            ],
            background_apps: vec![
                BackgroundApp {
                    exe_name: "chrome.exe".into(),
                    label: "Google Chrome".into(),
                    enabled: true,
                },
                BackgroundApp {
                    exe_name: "msedge.exe".into(),
                    label: "Microsoft Edge".into(),
                    enabled: true,
                },
                BackgroundApp {
                    exe_name: "firefox.exe".into(),
                    label: "Mozilla Firefox".into(),
                    enabled: true,
                },
                BackgroundApp {
                    exe_name: "brave.exe".into(),
                    label: "Brave Browser".into(),
                    enabled: true,
                },
                BackgroundApp {
                    exe_name: "opera.exe".into(),
                    label: "Opera / Opera GX".into(),
                    enabled: true,
                },
                BackgroundApp {
                    exe_name: "Discord.exe".into(),
                    label: "Discord".into(),
                    enabled: true,
                },
                BackgroundApp {
                    exe_name: "Spotify.exe".into(),
                    label: "Spotify".into(),
                    enabled: true,
                },
                BackgroundApp {
                    exe_name: "steamwebhelper.exe".into(),
                    label: "Steam WebHelper".into(),
                    enabled: true,
                },
                BackgroundApp {
                    exe_name: "EpicGamesLauncher.exe".into(),
                    label: "Epic Games Launcher".into(),
                    enabled: true,
                },
                BackgroundApp {
                    exe_name: "RiotClientUx.exe".into(),
                    label: "Riot Client".into(),
                    enabled: true,
                },
            ],
        }
    }
}

pub fn config_file_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("StreamBoost").join("config.json")
}

pub fn load_config() -> AppConfig {
    let path = config_file_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
            return cfg;
        }
    }
    let default_cfg = AppConfig::default();
    let _ = save_config(&default_cfg);
    default_cfg
}

pub fn save_config(cfg: &AppConfig) -> Result<(), String> {
    let path = config_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {e}"))?;
    }
    let json =
        serde_json::to_string_pretty(cfg).map_err(|e| format!("Failed to serialize config: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write config: {e}"))
}
