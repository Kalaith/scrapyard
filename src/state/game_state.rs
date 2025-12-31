use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ship::ship::Ship;
use crate::ship::interior::{ShipInterior, RoomType};
use crate::ship::player::Player;
use crate::economy::resources::Resources;
use crate::economy::upgrades::{GameUpgrades, UpgradeTemplate};
use crate::simulation::constants::*;
use crate::simulation::gameplay::ModuleRegistry;
use crate::enemy::entities::{Enemy, Projectile, Particle, ScrapPile};
use crate::enemy::wave::WaveState;
use super::tutorial::{TutorialConfig, TutorialState};
use crate::data::settings::Settings;
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
                let mut am = AssetManager::new();
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
                    eprintln!("Warning: Failed to load upgrades.json: {}. Using empty list.", e);
                    Vec::new()
                }),
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
        };
        
        state.spawn_scrap_piles();
        state
    }

    pub fn start_new_game(&mut self) {
        self.ship = Ship::new(GRID_WIDTH, GRID_HEIGHT);
        self.interior = ShipInterior::starter_ship();
        self.resources = Resources::new();
        self.resources.scrap = 50;
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
        self.pause_menu_selection = 0;

        self.spawn_scrap_piles();
    }

    pub fn spawn_scrap_piles(&mut self) {
        use macroquad::rand::ChooseRandom;
        let count = macroquad::rand::gen_range(MIN_SCRAP_PILES, MAX_SCRAP_PILES + 1);
        for _ in 0..count {
            if let Some(room) = self.interior.rooms.choose() {
                if room.room_type == RoomType::Empty { continue; }
                let w = room.width - SCRAP_SPAWN_PADDING * 2.0;
                let h = room.height - SCRAP_SPAWN_PADDING * 2.0;
                let x = room.x + SCRAP_SPAWN_PADDING + macroquad::rand::gen_range(0.0, w);
                let y = room.y + SCRAP_SPAWN_PADDING + macroquad::rand::gen_range(0.0, h);
                let amount = macroquad::rand::gen_range(SCRAP_PILE_MIN_AMOUNT, SCRAP_PILE_MAX_AMOUNT + 1);
                self.scrap_piles.push(ScrapPile::new(vec2(x, y), amount));
            }
        }
    }
}

