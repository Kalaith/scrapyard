// settings.rs - Game settings, backed by the shared macroquad-toolkit `GameSettings`
// shape, persisted to config.json (native) / keyed browser storage (wasm).

use macroquad_toolkit::settings::GameSettings;
use std::io;

const CONFIG_PATH: &str = "config.json";
#[cfg(target_arch = "wasm32")]
const GAME_NAME: &str = "scrapyard";

/// Scrapyard has no genuinely game-specific settings beyond the shared shape, so this is
/// a plain alias rather than a wrapper struct.
pub type Settings = GameSettings;

/// Load settings from config.json, or return defaults if the file doesn't exist.
pub fn load() -> Settings {
    load_settings().unwrap_or_default()
}

/// Save settings to config.json.
pub fn save(settings: &Settings) -> std::io::Result<()> {
    save_settings(settings).map_err(io::Error::other)
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
