use serde::{Serialize, Deserialize};
use crate::ship::ship::Ship;
use crate::economy::resources::Resources;
use crate::enemy::entities::EnemyType;
use crate::economy::upgrades::GameUpgrades;
use super::game_state::{GamePhase, EngineState, ViewMode};

#[derive(Serialize, Deserialize)]
pub struct SavedEnemy {
    pub id: u64,
    pub enemy_type: EnemyType,
    pub pos: (f32, f32),
    pub hp: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub target: Option<(usize, usize)>,
    pub attached_to: Option<(usize, usize)>, // For Leech attachment
    pub ability_timer: f32,                   // For Boss abilities
}

#[derive(Serialize, Deserialize)]
pub struct SavedProjectile {
    pub pos: (f32, f32),
    pub vel: (f32, f32),
    pub damage: f32,
    pub active: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SavedParticle {
    pub pos: (f32, f32),
    pub vel: (f32, f32),
    pub life: f32,
    pub max_life: f32,
    pub color: (f32, f32, f32, f32),
    pub active: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SavedScrapPile {
    pub pos: (f32, f32),
    pub amount: i32,
    pub active: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub ship: Ship,
    pub resources: Resources,
    pub phase: GamePhase,
    pub engine_state: EngineState,
    pub escape_timer: f32,
    pub enemies: Vec<SavedEnemy>,
    pub projectiles: Vec<SavedProjectile>,
    pub particles: Vec<SavedParticle>,
    pub scrap_piles: Vec<SavedScrapPile>,
    pub upgrades: GameUpgrades,
    pub frame_count: u64,
    // Interior repair states: room_id -> list of repaired repair point indices
    pub room_repair_states: Vec<Vec<bool>>,
    // Player state
    pub player_pos: (f32, f32),
    pub view_mode: ViewMode,
    // Tutorial state
    pub tutorial_index: usize,
    pub tutorial_completed: bool,
}
