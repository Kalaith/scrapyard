//! Player profile for meta-progression across runs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

use crate::economy::upgrades::UpgradeTemplate;

const PROFILE_PATH: &str = "player_profile.json";
#[cfg(target_arch = "wasm32")]
const GAME_NAME: &str = "scrapyard";

/// Persistent player profile that survives across game runs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    /// Highest single-escape payout banked
    #[serde(default)]
    pub best_payout: Option<i32>,
    /// Running total of escape value forfeited by dying instead of launching
    #[serde(default)]
    pub credits_left_behind: i32,
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
        save_profile(self).map_err(io::Error::other)
    }

    /// Record a successful escape
    pub fn record_victory(&mut self, credits_earned: i32, escape_time: f32) {
        self.lifetime_credits += credits_earned;
        self.banked_credits += credits_earned;
        self.runs_completed += 1;

        if self.best_time.is_none() || escape_time < self.best_time.unwrap() {
            self.best_time = Some(escape_time);
        }
        if credits_earned > self.best_payout.unwrap_or(i32::MIN) {
            self.best_payout = Some(credits_earned);
        }
    }

    /// Record a death: track the escape value the player forfeited (regret counter).
    pub fn record_death(&mut self, value_left_behind: i32) {
        self.credits_left_behind += value_left_behind.max(0);
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

    pub fn upgrade_level(&self, upgrade_id: &str) -> u32 {
        *self.permanent_upgrades.get(upgrade_id).unwrap_or(&0)
    }

    pub fn upgrade_cost(&self, template: &UpgradeTemplate) -> i32 {
        let level = self.upgrade_level(&template.id);
        (template.base_cost as f32 * template.cost_multiplier.powi(level as i32)) as i32
    }

    pub fn purchase_upgrade(&mut self, template: &UpgradeTemplate) -> bool {
        let current_level = self.upgrade_level(&template.id);
        if current_level >= template.max_level {
            return false;
        }
        let cost = self.upgrade_cost(template);
        if !self.spend_credits(cost) {
            return false;
        }
        self.permanent_upgrades
            .insert(template.id.clone(), current_level + 1);
        true
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
