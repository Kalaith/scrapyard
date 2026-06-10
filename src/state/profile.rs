//! Player profile for meta-progression across runs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

const PROFILE_PATH: &str = "player_profile.json";
#[cfg(target_arch = "wasm32")]
const GAME_NAME: &str = "scrapyard";

/// Persistent player profile that survives across game runs
#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct PlayerProfile {
    /// Total credits earned across all runs
    pub lifetime_credits: i32,
    /// Current banked credits for next run
    pub banked_credits: i32,
    /// Permanent upgrade levels (persist across runs)
    pub permanent_upgrades: HashMap<String, u32>,
    /// Number of successful escapes
    pub runs_completed: u32,
    /// Best escape time in seconds
    pub best_time: Option<f32>,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            lifetime_credits: 0,
            banked_credits: 0,
            permanent_upgrades: HashMap::new(),
            runs_completed: 0,
            best_time: None,
        }
    }
}

impl PlayerProfile {
    /// Load profile from disk, or create default if not found
    pub fn load() -> Self {
        match load_profile() {
            Ok(profile) => profile,
            Err(error) => {
                eprintln!("Warning: Failed to load profile: {error}. Using default.");
                Self::default()
            }
        }
    }

    /// Save profile to disk
    pub fn save(&self) -> std::io::Result<()> {
        save_profile(self).map_err(|error| io::Error::new(io::ErrorKind::Other, error))
    }

    /// Record a successful escape
    pub fn record_victory(&mut self, credits_earned: i32, escape_time: f32) {
        self.lifetime_credits += credits_earned;
        self.banked_credits += credits_earned;
        self.runs_completed += 1;

        if self.best_time.is_none() || escape_time < self.best_time.unwrap() {
            self.best_time = Some(escape_time);
        }
    }

    /// Spend banked credits (returns true if affordable)
    pub fn spend_credits(&mut self, amount: i32) -> bool {
        if self.banked_credits >= amount {
            self.banked_credits -= amount;
            true
        } else {
            false
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_profile() -> Result<PlayerProfile, String> {
    macroquad_toolkit::persistence::load_json(PROFILE_PATH)
}

#[cfg(target_arch = "wasm32")]
fn load_profile() -> Result<PlayerProfile, String> {
    macroquad_toolkit::persistence::load_json_key(GAME_NAME, PROFILE_PATH)
}

#[cfg(not(target_arch = "wasm32"))]
fn save_profile(profile: &PlayerProfile) -> Result<(), String> {
    macroquad_toolkit::persistence::save_json_atomic(PROFILE_PATH, profile)
}

#[cfg(target_arch = "wasm32")]
fn save_profile(profile: &PlayerProfile) -> Result<(), String> {
    macroquad_toolkit::persistence::save_json_key(GAME_NAME, PROFILE_PATH, profile)
}
