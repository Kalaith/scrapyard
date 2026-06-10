// settings.rs - Game settings with save/load to config.json

use serde::{Deserialize, Serialize};
use std::io;

const CONFIG_PATH: &str = "config.json";
#[cfg(target_arch = "wasm32")]
const GAME_NAME: &str = "scrapyard";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub master_volume: f32, // 0.0 - 1.0
    pub sfx_volume: f32,    // 0.0 - 1.0
    pub music_volume: f32,  // 0.0 - 1.0
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
        match load_settings() {
            Ok(settings) => settings,
            Err(_) => Self::default(),
        }
    }

    /// Save settings to config.json
    pub fn save(&self) -> std::io::Result<()> {
        save_settings(self).map_err(|error| io::Error::new(io::ErrorKind::Other, error))
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

#[cfg(not(target_arch = "wasm32"))]
fn load_settings() -> Result<Settings, String> {
    macroquad_toolkit::persistence::load_json(CONFIG_PATH)
}

#[cfg(target_arch = "wasm32")]
fn load_settings() -> Result<Settings, String> {
    macroquad_toolkit::persistence::load_json_key(GAME_NAME, CONFIG_PATH)
}

#[cfg(not(target_arch = "wasm32"))]
fn save_settings(settings: &Settings) -> Result<(), String> {
    macroquad_toolkit::persistence::save_json_atomic(CONFIG_PATH, settings)
}

#[cfg(target_arch = "wasm32")]
fn save_settings(settings: &Settings) -> Result<(), String> {
    macroquad_toolkit::persistence::save_json_key(GAME_NAME, CONFIG_PATH, settings)
}
