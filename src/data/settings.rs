// settings.rs - Game settings with save/load to config.json

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

const CONFIG_PATH: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub master_volume: f32,    // 0.0 - 1.0
    pub sfx_volume: f32,       // 0.0 - 1.0
    pub music_volume: f32,     // 0.0 - 1.0
    pub fullscreen: bool,
    pub show_fps: bool,
    pub screen_shake: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sfx_volume: 0.8,
            music_volume: 0.6,
            fullscreen: false,
            show_fps: false,
            screen_shake: true,
        }
    }
}


impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load settings from config.json, or return defaults if file doesn't exist
    pub fn load() -> Self {
        match File::open(CONFIG_PATH) {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader).unwrap_or_default()
            }
            Err(_) => Self::default(),
        }
    }

    /// Save settings to config.json
    pub fn save(&self) -> std::io::Result<()> {
        let file = File::create(CONFIG_PATH)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Get effective SFX volume (master * sfx)
    pub fn effective_sfx_volume(&self) -> f32 {
        self.master_volume * self.sfx_volume
    }

    /// Get effective music volume (master * music)
    pub fn effective_music_volume(&self) -> f32 {
        self.master_volume * self.music_volume
    }
}
