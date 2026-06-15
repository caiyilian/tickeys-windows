use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub scheme: String,
    pub volume: f32,
    pub pitch: f32,
    pub max_sources: usize,
    #[serde(default = "default_debounce_ms")]
    pub key_debounce_ms: u32,
    pub filter_list: Vec<String>,
    pub filter_mode: FilterMode,
    pub auto_start: bool,
    pub blocked_keys: Vec<u16>,
    pub settings_x: i32,
    pub settings_y: i32,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub enum FilterMode {
    BlackList,
    WhiteList,
}

fn default_debounce_ms() -> u32 {
    20
}

fn config_dir() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join("Tickeys")
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

impl Config {
    pub fn load() -> Config {
        let path = config_path();
        if !path.exists() {
            log::info!("No config file at {:?}, using defaults", path);
            let cfg = Config::default();
            let _ = cfg.save();
            return cfg;
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<Config>(&content) {
                    Ok(mut cfg) => {
                        cfg.validate();
                        log::info!("Config loaded from {:?}", path);
                        cfg
                    }
                    Err(e) => {
                        log::error!("Failed to parse config: {}, using defaults", e);
                        let cfg = Config::default();
                        let _ = cfg.save();
                        cfg
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read config: {}, using defaults", e);
                Config::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = config_dir();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create config dir {:?}: {}", dir, e))?;

        let path = config_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&path, &content)
            .map_err(|e| format!("Failed to write config {:?}: {}", path, e))?;

        log::info!("Config saved to {:?}", path);
        Ok(())
    }

    fn validate(&mut self) {
        self.volume = self.volume.clamp(0.0, 5.0);
        self.pitch = self.pitch.clamp(0.5, 2.0);
        self.max_sources = self.max_sources.clamp(1, 20);
        self.key_debounce_ms = self.key_debounce_ms.clamp(10, 500);

        self.filter_list.sort();
        self.filter_list.dedup();

        self.blocked_keys.sort();
        self.blocked_keys.dedup();
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scheme: String::new(),
            volume: 0.5,
            pitch: 1.0,
            max_sources: 2,
            key_debounce_ms: 20,
            filter_list: Vec::new(),
            filter_mode: FilterMode::BlackList,
            auto_start: false,
            blocked_keys: vec![
                8,   // VK_BACK
                13,  // VK_RETURN
                32,  // VK_SPACE
                37,  // VK_LEFT
                38,  // VK_UP
                39,  // VK_RIGHT
                40,  // VK_DOWN
                112, // VK_F1
                113, // VK_F2
                114, // VK_F3
                115, // VK_F4
                116, // VK_F5
                117, // VK_F6
                118, // VK_F7
                119, // VK_F8
                120, // VK_F9
                121, // VK_F10
                122, // VK_F11
                123, // VK_F12
            ],
            settings_x: -1,
            settings_y: -1,
        }
    }
}

static GLOBAL_CONFIG: Mutex<Option<Config>> = Mutex::new(None);

pub fn init_config() -> Config {
    let cfg = Config::load();
    *GLOBAL_CONFIG.lock().unwrap() = Some(cfg.clone());
    cfg
}

pub fn get_config() -> Option<Config> {
    GLOBAL_CONFIG.lock().unwrap().clone()
}

pub fn update_config(cfg: &Config) {
    let _ = cfg.save();
    *GLOBAL_CONFIG.lock().unwrap() = Some(cfg.clone());
}
