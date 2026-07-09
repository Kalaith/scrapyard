//! Per-run contract modifiers — a small twist drawn each run so run 10 differs from run 2.

use macroquad_toolkit::rng;
use serde::Deserialize;

fn one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractModifier {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Flat threat added all run (the swarm is already interested).
    #[serde(default)]
    pub signature_delta: i32,
    /// Scales scrap pile count and richness.
    #[serde(default = "one")]
    pub scrap_multiplier: f32,
    /// Multiplies the final escape payout.
    #[serde(default = "one")]
    pub payout_multiplier: f32,
    /// Bonus scrap in the hold at run start.
    #[serde(default)]
    pub start_scrap: i32,
}

impl ContractModifier {
    /// Load all contracts from the embedded JSON (falls back to a single neutral contract).
    pub fn load_all() -> Vec<ContractModifier> {
        serde_json::from_str(include_str!("../../assets/contracts.json")).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load contracts.json: {e}. Using neutral contract.");
            vec![ContractModifier {
                id: "clear_skies".to_string(),
                name: "Clear Skies".to_string(),
                description: "A quiet sector.".to_string(),
                signature_delta: 0,
                scrap_multiplier: 1.0,
                payout_multiplier: 1.0,
                start_scrap: 0,
            }]
        })
    }

    /// Pick a contract at random from the list (returns a clone).
    pub fn pick(contracts: &[ContractModifier]) -> Option<ContractModifier> {
        rng::choose(contracts).cloned()
    }
}
