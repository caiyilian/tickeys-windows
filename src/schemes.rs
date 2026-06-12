use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Deserialize, Serialize, Clone)]
pub struct AudioScheme {
    pub name: String,
    pub display_name: String,
    pub files: Vec<String>,
    pub non_unique_count: u8,
    pub key_audio_map: BTreeMap<u8, u8>,
}

pub fn load_schemes() -> Vec<AudioScheme> {
    let schemes_path = Path::new("resource/data/schemes.json");
    if !schemes_path.exists() {
        log::warn!("schemes.json not found at {:?}", schemes_path);
        return Vec::new();
    }

    match std::fs::read_to_string(schemes_path) {
        Ok(content) => {
            match serde_json::from_str::<Vec<AudioScheme>>(&content) {
                Ok(schemes) => {
                    log::info!("Loaded {} audio schemes", schemes.len());
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
