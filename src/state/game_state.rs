use macroquad::prelude::*;
use macroquad_toolkit::rng;
use serde::{Deserialize, Serialize};

use super::tutorial::{TutorialConfig, TutorialState};
use crate::data::settings::Settings;
use crate::economy::resources::Resources;
use crate::economy::upgrades::{GameUpgrades, UpgradeTemplate};
use crate::enemy::entities::{Enemy, Particle, Projectile, ScrapPile};
use crate::enemy::wave::WaveState;
use crate::ship::interior::{RoomType, ShipInterior};
use crate::ship::player::Player;
use crate::ship::ship::Ship;
use crate::simulation::constants::*;
use crate::simulation::events::GameEvent;
use crate::simulation::gameplay::ModuleRegistry;
use crate::state::profile::PlayerProfile;
use crate::ui::assets::AssetManager;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum GamePhase {
    Menu,
    Playing,
    GameOver,
    Victory,
    InterRound,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum EngineState {
    Idle,
    Charging,
    Escaped,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ViewMode {
    Exterior,
    Interior,
}

#[derive(Debug, Clone, Copy)]
pub struct PayoutBreakdown {
    pub base: i32,
    pub repaired_bonus: i32,
    pub powered_bonus: i32,
    pub hull_bonus: i32,
    pub scrap_bonus: i32,
    pub combat_bonus: i32,
    pub risk_bonus: i32,
    pub penalties: i32,
    pub total: i32,
}

pub struct GameState {
    pub ship: Ship,
    pub interior: ShipInterior,
    pub resources: Resources,
    pub phase: GamePhase,
    pub module_registry: ModuleRegistry,
    pub assets: crate::ui::assets::AssetManager,
    pub view_mode: ViewMode,
    pub player: Player,
    pub total_power: i32,
    pub used_power: i32,
    pub required_power: i32,
    pub threat_signature: i32,
    pub ship_integrity: f32,
    pub ship_max_integrity: f32,
    pub tutorial_config: TutorialConfig,
    pub tutorial_state: TutorialState,
    pub tutorial_timer: f32,
    pub paused: bool,
    pub engine_state: EngineState,
    pub escape_timer: f32,
    pub scrap_piles: Vec<ScrapPile>,
    pub gathering_target: Option<usize>,
    pub gathering_timer: f32,
    pub upgrades: GameUpgrades,
    pub upgrade_templates: Vec<UpgradeTemplate>,
    pub profile: PlayerProfile,
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub particles: Vec<Particle>,
    pub frame_count: u64,
    pub time_survived: f32,
    pub wave_state: WaveState,
    pub repair_timer: f32,
    pub pause_menu_selection: usize,
    pub settings_open: bool,
    pub settings_selection: usize,
    pub settings: Settings,
    pub engine_stress: f32,
    pub nanite_alert: f32,
    pub life_support_timer: f32,
    pub enemies_destroyed: i32,
    pub last_payout: Option<PayoutBreakdown>,
    pub recent_events: Vec<String>,
    /// Highest threat tier crossed so far this frame-window, for escalation stings.
    pub last_signal_tier: u8,
    /// time_survived at which the engine first became escape-ready (repaired + powered).
    /// Drives death-screen forensics: proves escape was a choice the player declined.
    pub engine_ready_at: Option<f32>,
}

impl GameState {
    pub fn new() -> Self {
        let interior = ShipInterior::starter_ship();
        let player = Player::new_at(interior.player_start_position());

        let mut state = Self {
            ship: Ship::new(GRID_WIDTH, GRID_HEIGHT),
            interior,
            resources: Resources::new(),
            phase: GamePhase::Menu,
            module_registry: ModuleRegistry::new(),
            assets: {
                let am = AssetManager::new();
                // Note: We can't await here easily in new(), so we usually load assets in main
                // and pass them in, or use a lazy loader.
                // For simplicity in this codebase, we'll initialize empty and load in main.
                am
            },
            view_mode: ViewMode::Interior,
            player,
            total_power: 0,
            used_power: 0,
            required_power: 100,
            threat_signature: 0,
            ship_integrity: SHIP_BASE_INTEGRITY,
            ship_max_integrity: SHIP_BASE_INTEGRITY,
            tutorial_config: TutorialConfig::load(),
            tutorial_state: TutorialState::new(),
            tutorial_timer: 0.0,
            paused: false,
            engine_state: EngineState::Idle,
            escape_timer: 60.0,
            enemies: Vec::new(),
            projectiles: Vec::new(),
            particles: Vec::new(),
            scrap_piles: Vec::new(),
            gathering_target: None,
            gathering_timer: 0.0,
            upgrades: GameUpgrades::new(),
            upgrade_templates: serde_json::from_str(include_str!("../../assets/upgrades.json"))
                .unwrap_or_else(|e| {
                    eprintln!(
                        "Warning: Failed to load upgrades.json: {}. Using empty list.",
                        e
                    );
                    Vec::new()
                }),
            profile: PlayerProfile::load(),
            frame_count: 0,
            time_survived: 0.0,
            wave_state: WaveState::new(),
            repair_timer: 0.0,
            pause_menu_selection: 0,
            settings_open: false,
            settings_selection: 0,
            settings: Settings::load(),
            engine_stress: 0.0,
            nanite_alert: NANITE_ALERT_BASE, // Initial alert level
            life_support_timer: 0.0,
            enemies_destroyed: 0,
            last_payout: None,
            recent_events: vec!["Systems waiting for repair".to_string()],
            last_signal_tier: 0,
            engine_ready_at: None,
        };

        state.sync_upgrades_from_profile();
        state.spawn_scrap_piles();
        state
    }

    pub fn start_new_game(&mut self) {
        self.ship = Ship::new(GRID_WIDTH, GRID_HEIGHT);
        self.interior = ShipInterior::starter_ship();
        self.resources = Resources::new();
        self.resources.scrap = 50;
        self.resources.credits = self.profile.banked_credits;
        self.enemies.clear();
        self.projectiles.clear();
        self.particles.clear();
        self.frame_count = 0;
        self.time_survived = 0.0;
        self.paused = false;
        self.engine_state = EngineState::Idle;
        self.escape_timer = 60.0;
        self.view_mode = ViewMode::Interior;
        self.player = Player::new_at(self.interior.player_start_position());
        self.engine_stress = 0.0;
        self.nanite_alert = NANITE_ALERT_BASE;

        self.total_power = 0;
        self.used_power = 0;
        self.threat_signature = 0;
        self.ship_integrity = SHIP_BASE_INTEGRITY;
        self.ship_max_integrity = SHIP_BASE_INTEGRITY;
        self.tutorial_state = TutorialState::new();
        self.tutorial_timer = 0.0;
        self.phase = GamePhase::Playing;
        self.scrap_piles.clear();
        self.gathering_target = None;
        self.gathering_timer = 0.0;

        self.wave_state = WaveState::new();
        self.repair_timer = 0.0;
        self.life_support_timer = 0.0;
        self.enemies_destroyed = 0;
        self.last_payout = None;
        self.last_signal_tier = 0;
        self.engine_ready_at = None;
        self.pause_menu_selection = 0;
        self.recent_events.clear();
        self.recent_events
            .push("New salvage run started".to_string());

        self.apply_meta_progression_to_run();
        self.spawn_scrap_piles();
    }

    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "menu" => {
                self.phase = GamePhase::Menu;
            }
            "pause" => {
                self.start_new_game();
                self.paused = true;
            }
            _ => {
                // Default: jump straight into gameplay.
                self.start_new_game();
            }
        }
    }

    pub fn spawn_scrap_piles(&mut self) {
        let count = rng::gen_range(MIN_SCRAP_PILES, MAX_SCRAP_PILES + 1);
        for _ in 0..count {
            if let Some(room) = rng::choose(&self.interior.rooms) {
                if room.room_type == RoomType::Empty {
                    continue;
                }
                let w = room.width - SCRAP_SPAWN_PADDING * 2.0;
                let h = room.height - SCRAP_SPAWN_PADDING * 2.0;
                let x = room.x + SCRAP_SPAWN_PADDING + rng::gen_range(0.0, w);
                let y = room.y + SCRAP_SPAWN_PADDING + rng::gen_range(0.0, h);
                let amount = rng::gen_range(SCRAP_PILE_MIN_AMOUNT, SCRAP_PILE_MAX_AMOUNT + 1);
                self.scrap_piles.push(ScrapPile::new(vec2(x, y), amount));
            }
        }
    }

    /// Emit a radial burst of particles at a screen position (deterministic angles).
    pub fn spawn_burst(&mut self, pos: Vec2, color: Color, count: usize, speed: f32, life: f32) {
        if count == 0 {
            return;
        }
        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            let vel = vec2(angle.cos(), angle.sin()) * speed;
            self.particles.push(Particle::new(pos, vel, life, color));
        }
        // Bound the pool so long fights can't grow it without limit.
        if self.particles.len() > 512 {
            let overflow = self.particles.len() - 512;
            self.particles.drain(0..overflow);
        }
    }

    pub fn record_event(&mut self, event: &GameEvent) {
        let message = match event {
            GameEvent::ModuleRepaired { cost, .. } => format!("Module repaired (-{} scrap)", cost),
            GameEvent::ModuleUpgraded { new_level, .. } => {
                format!("Module upgraded to level {}", new_level)
            }
            GameEvent::ModuleDestroyed { .. } => "Module destroyed".to_string(),
            GameEvent::EnemyKilled { scrap_dropped, .. } => {
                format!("Nanite killed (+{} scrap)", scrap_dropped)
            }
            GameEvent::ModuleDamaged { damage, .. } => {
                format!("Module damaged ({:.0})", damage)
            }
            GameEvent::CoreDamaged {
                damage,
                remaining_hp,
            } => format!("Core hit {:.0}, {:.0} hull left", damage, remaining_hp),
            GameEvent::EngineActivated => "Engine event detected".to_string(),
            GameEvent::PowerRouted { system, powered } => {
                let state = if *powered { "online" } else { "offline" };
                format!("{system} routed {state}")
            }
            GameEvent::EscapeSuccess => "Escape successful".to_string(),
            GameEvent::CoreDestroyed => "Core destroyed".to_string(),
            GameEvent::ThreatEscalated { tier } => format!("Threat escalated to tier {}", tier),
            GameEvent::EmpPulse => "EMP pulse! Systems knocked offline".to_string(),
            GameEvent::WeaponFired { .. } => return,
        };

        self.recent_events.push(message);
        if self.recent_events.len() > 12 {
            self.recent_events.remove(0);
        }
    }
}
