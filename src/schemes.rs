use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Deserialize, Serialize, Clone)]
pub struct AudioScheme {
    pub name: String,
    pub display_name: String,
    pub files: Vec<String>,
    pub non_unique_count: u8,
    pub key_audio_map: BTreeMap<u8, u8>,
}

static CACHED_SCHEMES: Mutex<Option<Vec<AudioScheme>>> = Mutex::new(None);

pub fn load_schemes() -> Vec<AudioScheme> {
    let schemes_path = crate::audio::get_resource_path("schemes.json");
    if !schemes_path.exists() {
        log::warn!("schemes.json not found at {:?}", schemes_path);
        return Vec::new();
    }

    match std::fs::read_to_string(schemes_path) {
        Ok(content) => {
            match serde_json::from_str::<Vec<AudioScheme>>(&content) {
                Ok(schemes) => {
                    log::info!("Loaded {} audio schemes", schemes.len());
                    *CACHED_SCHEMES.lock().unwrap() = Some(schemes.clone());
                    schemes
                }
                Err(e) => {
                    log::error!("Failed to parse schemes.json: {}", e);
                    Vec::new()
                }
            }
        }
        Err(e) => {
            log::error!("Failed to read schemes.json: {}", e);
            Vec::new()
        }
    }
}

pub fn find_scheme(name: &str) -> Option<AudioScheme> {
    let guard = CACHED_SCHEMES.lock().unwrap();
    guard.as_ref()?.iter().find(|s| s.name == name).cloned()
}

pub fn first_scheme_name() -> Option<String> {
    let guard = CACHED_SCHEMES.lock().unwrap();
    guard.as_ref()?.first().map(|s| s.name.clone())
}
