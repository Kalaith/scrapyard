//! Player profile for meta-progression across runs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[allow(dead_code)]
const PROFILE_PATH: &str = "player_profile.json";

/// Persistent player profile that survives across game runs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
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

#[allow(dead_code)]
impl PlayerProfile {
    /// Load profile from disk, or create default if not found
    pub fn load() -> Self {
        match File::open(PROFILE_PATH) {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader).unwrap_or_else(|e| {
                    eprintln!("Warning: Failed to parse profile: {}. Using default.", e);
                    Self::default()
                })
            }
            Err(_) => Self::default(),
        }
    }

    /// Save profile to disk
    pub fn save(&self) -> std::io::Result<()> {
        let file = File::create(PROFILE_PATH)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
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
