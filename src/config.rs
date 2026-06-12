use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub scheme: String,
    pub volume: f32,
    pub pitch: f32,
    pub max_sources: usize,
    pub filter_list: Vec<String>,
    pub filter_mode: FilterMode,
    pub auto_start: bool,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub enum FilterMode {
    BlackList,
    WhiteList,
}

impl Config {
    pub fn load() -> Config {
        Config::default()
    }

    pub fn save(&self) {}
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scheme: String::new(),
            volume: 0.5,
            pitch: 1.0,
            max_sources: 2,
            filter_list: Vec::new(),
            filter_mode: FilterMode::BlackList,
            auto_start: false,
        }
    }
}
