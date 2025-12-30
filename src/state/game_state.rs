use macroquad::prelude::*;
use serde::{Deserialize, Serialize}; // Added Serialize as it's used by derives
use std::fs::File;
use std::io::{BufReader, BufWriter};

use crate::ship::ship::{Ship, ModuleState, ModuleType}; // Added ModuleType as it's used
use crate::ship::interior::{ShipInterior, RoomType};
use crate::ship::player::Player;
use crate::economy::resources::Resources;
use crate::economy::upgrades::{GameUpgrades, UpgradeTemplate};
use crate::simulation::constants::*;
use crate::simulation::gameplay::ModuleRegistry;
use crate::simulation::events::{EventBus, GameEvent};
use crate::enemy::entities::{Enemy, Projectile, Particle, ScrapPile};
use crate::enemy::wave::WaveState;
use super::tutorial::{TutorialStep, TutorialConfig, TutorialState};
use super::persistence::{SaveData, SavedEnemy, SavedProjectile, SavedParticle, SavedScrapPile};

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    pub wave_state: WaveState,
    pub repair_timer: f32,
    pub pause_menu_selection: usize,
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
            view_mode: ViewMode::Interior,
            player,
            total_power: 0,
            used_power: 0,
            required_power: 100,
            ship_integrity: 1000.0,
            ship_max_integrity: 1000.0,
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
            wave_state: WaveState::new(),
            repair_timer: 0.0,
            pause_menu_selection: 0,
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
        self.paused = false;
        self.engine_state = EngineState::Idle;
        self.escape_timer = 60.0;
        self.view_mode = ViewMode::Interior;
        self.player = Player::new_at(self.interior.player_start_position());
        self.total_power = 0;
        self.used_power = 0;
        self.ship_integrity = 1000.0;
        self.ship_max_integrity = 1000.0;
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
        let count = ::rand::random::<usize>() % 5 + 8;
        for _ in 0..count {
            if let Some(room) = self.interior.rooms.choose() {
                if room.room_type == RoomType::Empty { continue; }
                let w = room.width - 40.0;
                let h = room.height - 40.0;
                let x = room.x + 20.0 + macroquad::rand::gen_range(0.0, w);
                let y = room.y + 20.0 + macroquad::rand::gen_range(0.0, h);
                let amount = macroquad::rand::gen_range(15, 40);
                self.scrap_piles.push(ScrapPile::new(vec2(x, y), amount));
            }
        }
    }

    pub fn update(&mut self, dt: f32, events: &mut EventBus) {
        match self.phase {
            GamePhase::Playing => {
                if !self.paused {
                    if self.view_mode == ViewMode::Interior {
                        self.player.update(dt, &self.interior);
                        self.player.update_nearby_module(&self.interior);
                    }
                    self.update_power();
                    self.update_resources();
                    self.update_engine(dt, events);
                    crate::enemy::ai::update_wave_logic(
                        self.total_power,
                        &self.engine_state,
                        &mut self.enemies,
                        &self.upgrades,
                        &mut self.wave_state,
                        self.frame_count,
                        dt,
                        events
                    );
                    crate::enemy::ai::update_enemies(self, dt);
                    crate::enemy::combat::update_combat(self, dt, events);
                    self.frame_count += 1;

                    let robotics_level = self.upgrades.get_level("auto_repairs");
                    self.repair_timer += dt;
                    if self.repair_timer >= NANO_REPAIR_INTERVAL_SECONDS {
                        self.repair_timer = 0.0;
                        let repair_amount = robotics_level as f32 * NANO_REPAIR_RATE_PER_LEVEL;
                        for x in 0..GRID_WIDTH {
                            for y in 0..GRID_HEIGHT {
                                if let Some(module) = &mut self.ship.grid[x][y] {
                                    if module.state != ModuleState::Destroyed && module.health < module.max_health {
                                        module.health = (module.health + repair_amount).min(module.max_health);
                                    }
                                }
                            }
                        }
                    }
                    self.check_game_over(events);
                }
            }
            _ => {}
        }
    }

    fn update_power(&mut self) {
        self.total_power = 0;
        self.used_power = 0;
        for room in &self.interior.rooms {
            if room.repair_points.is_empty() { continue; }
            let repaired = room.repaired_count() as i32;
            if repaired > 0 {
                match room.room_type {
                    RoomType::Module(ModuleType::Core) => self.total_power += repaired * POWER_PER_CORE_POINT,
                    RoomType::Module(ModuleType::Weapon) => self.used_power += repaired * POWER_COST_WEAPON,
                    RoomType::Module(ModuleType::Defense) => self.used_power += repaired * POWER_COST_DEFENSE,
                    RoomType::Module(ModuleType::Utility) => self.used_power += repaired * POWER_COST_UTILITY,
                    RoomType::Module(ModuleType::Engine) => self.used_power += repaired * POWER_COST_ENGINE,
                    RoomType::Cockpit => self.used_power += repaired * POWER_COST_COCKPIT,
                    RoomType::Medbay => self.used_power += repaired * POWER_COST_MEDBAY,
                    _ => {}
                }
            }
        }
    }

    fn check_game_over(&mut self, events: &mut EventBus) {
        if self.ship_integrity <= 0.0 {
            self.ship_integrity = 0.0;
            self.phase = GamePhase::GameOver;
            events.push_game(GameEvent::CoreDestroyed);
        }
    }

    fn update_resources(&mut self) {
        // Power calculation moved to update_power() for consistency with interior system
        self.resources.power = self.used_power;
    }

    fn update_engine(&mut self, dt: f32, events: &mut EventBus) {
        let mut engine_repair_pct = 0.0;
        for room in &self.interior.rooms {
            if let RoomType::Module(ModuleType::Engine) = room.room_type {
                 if !room.repair_points.is_empty() {
                    engine_repair_pct = room.repaired_count() as f32 / room.repair_points.len() as f32;
                 }
            }
        }
        if engine_repair_pct >= ENGINE_MIN_REPAIR_PERCENT {
            if self.engine_state == EngineState::Idle {
                self.engine_state = EngineState::Charging;
            }
            if self.engine_state == EngineState::Charging {
                self.escape_timer -= dt * engine_repair_pct;
                if self.escape_timer <= 0.0 {
                    self.engine_state = EngineState::Escaped;
                    self.phase = GamePhase::Victory;
                    let bonus_mult = 1.0 + (self.upgrades.get_level("credit_bonus") as f32 * CREDIT_BONUS_PER_LEVEL);
                    let total_credits = (BASE_ESCAPE_CREDITS as f32 * bonus_mult) as i32;
                    self.resources.add_credits(total_credits);
                    events.push_game(GameEvent::EscapeSuccess);
                }
            }
        } else {
             self.engine_state = EngineState::Idle;
        }
    }

    pub fn attempt_repair(&mut self, x: usize, y: usize, events: &mut EventBus) -> bool {
        let repair_cost = if let Some(module) = &self.ship.grid[x][y] {
            if module.state != ModuleState::Destroyed { return false; }
            self.module_registry.get(module.module_type).base_cost
        } else { return false; };

        if self.resources.can_afford(repair_cost) {
            self.resources.deduct(repair_cost);
            if let Some(module) = &mut self.ship.grid[x][y] {
                module.state = ModuleState::Active;
                events.push_game(GameEvent::ModuleRepaired { x, y, cost: repair_cost });
                return true;
            }
        }
        false
    }

    pub fn attempt_upgrade(&mut self, x: usize, y: usize, events: &mut EventBus) -> bool {
        const MAX_LEVEL: u8 = 5;
        const UPGRADE_MULTIPLIER: f32 = 1.5;
        let upgrade_cost = if let Some(module) = &self.ship.grid[x][y] {
            if module.state == ModuleState::Destroyed || module.level >= MAX_LEVEL { return false; }
            let base_cost = self.module_registry.get(module.module_type).base_cost;
            (base_cost as f32 * (module.level as f32 * 0.5 + 1.0)) as i32
        } else { return false; };

        if self.resources.can_afford(upgrade_cost) {
            self.resources.deduct(upgrade_cost);
            if let Some(module) = &mut self.ship.grid[x][y] {
                module.level += 1;
                module.max_health *= UPGRADE_MULTIPLIER;
                module.health = module.max_health;
                events.push_game(GameEvent::ModuleUpgraded { x, y, new_level: module.level });
                return true;
            }
        }
        false
    }

    pub fn toggle_module(&mut self, x: usize, y: usize) {
        if let Some(module) = &mut self.ship.grid[x][y] {
            match module.state {
                ModuleState::Active => module.state = ModuleState::Offline,
                ModuleState::Offline => module.state = ModuleState::Active,
                ModuleState::Destroyed => {}
            }
        }
    }

    pub fn activate_engine(&mut self, events: &mut EventBus) {
        if self.engine_state == EngineState::Idle {
            for row in &self.ship.grid {
                for cell in row {
                    if let Some(module) = cell {
                        if module.module_type == ModuleType::Engine && module.state == ModuleState::Active {
                            if self.engine_state == EngineState::Idle {
                                self.engine_state = EngineState::Charging;
                                self.escape_timer = ENGINE_CHARGE_BASE_TIME;
                                events.push_game(GameEvent::EngineActivated);
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_repair_cost(&self, room_idx: usize, _point_idx: usize) -> Option<(i32, i32)> {
        if room_idx >= self.interior.rooms.len() { return None; }
        let room = &self.interior.rooms[room_idx];
        let scrap_cost = 10;
        let power_cost = match room.room_type {
            RoomType::Module(ModuleType::Core) => 0,
            RoomType::Module(ModuleType::Weapon) => 2,
            RoomType::Module(ModuleType::Defense) => 2,
            RoomType::Module(ModuleType::Utility) => 1,
            RoomType::Module(ModuleType::Engine) => 3,
            RoomType::Cockpit => 2,
            RoomType::Medbay => 1,
            _ => 0,
        };
        Some((scrap_cost, power_cost))
    }

    pub fn attempt_interior_repair(&mut self, room_idx: usize, point_idx: usize) -> bool {
         if room_idx >= self.interior.rooms.len() { return false; }
         let (scrap_cost, power_cost) = match self.get_repair_cost(room_idx, point_idx) {
             Some(c) => c,
             None => return false,
         };
         if self.interior.rooms[room_idx].repair_points.len() <= point_idx || 
            self.interior.rooms[room_idx].repair_points[point_idx].repaired {
             return false;
         }
         let is_reactor = matches!(self.interior.rooms[room_idx].room_type, RoomType::Module(ModuleType::Core));
         if self.resources.scrap < scrap_cost { return false; }
         if !is_reactor && (self.used_power + power_cost > self.total_power) { return false; }
         self.resources.deduct(scrap_cost);
         self.interior.rooms[room_idx].repair_points[point_idx].repaired = true;
         if self.interior.rooms[room_idx].is_fully_repaired() {
            if let Some((gx, gy)) = self.interior.rooms[room_idx].module_index {
                if let Some(module) = &mut self.ship.grid[gx][gy] {
                    module.state = ModuleState::Active;
                    module.health = module.max_health;
                }
            }
         }
         true
    }

    pub fn purchase_upgrade(&mut self, upgrade_id: &str) -> bool {
        let template = self.upgrade_templates.iter().find(|t| t.id == upgrade_id).cloned();
        if let Some(template) = template {
            let current_level = self.upgrades.get_level(upgrade_id);
            if current_level < template.max_level {
                let cost = self.upgrades.get_cost(&template);
                if self.resources.deduct_credits(cost) {
                    self.upgrades.levels.insert(upgrade_id.to_string(), current_level + 1);
                    if upgrade_id == "hull_reinforcement" {
                        self.ship_max_integrity += 200.0;
                        self.ship_integrity += 200.0;
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let save_data = SaveData {
            ship: self.ship.clone(),
            resources: self.resources.clone(),
            phase: self.phase,
            engine_state: self.engine_state,
            escape_timer: self.escape_timer,
            enemies: self.enemies.iter().map(|e| SavedEnemy {
                id: e.id,
                enemy_type: e.enemy_type.clone(),
                pos: (e.position.x, e.position.y),
                hp: e.health,
                max_hp: e.max_health,
                speed: e.speed,
                damage: e.damage,
                target: e.target_module,
            }).collect(),
            projectiles: self.projectiles.iter().map(|p| SavedProjectile {
                pos: (p.position.x, p.position.y),
                vel: (p.velocity.x, p.velocity.y),
                damage: p.damage,
                active: p.active,
            }).collect(),
            particles: self.particles.iter().map(|p| SavedParticle {
                pos: (p.position.x, p.position.y),
                vel: (p.velocity.x, p.velocity.y),
                life: p.lifetime,
                max_life: p.max_lifetime,
                color: (p.color.r, p.color.g, p.color.b, p.color.a),
                active: p.active,
            }).collect(),
            scrap_piles: self.scrap_piles.iter().map(|p| SavedScrapPile {
                pos: (p.position.x, p.position.y),
                amount: p.amount,
                active: p.active,
            }).collect(),
            upgrades: self.upgrades.clone(),
            frame_count: self.frame_count,
        };
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &save_data)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let save_data: SaveData = serde_json::from_reader(reader)?;
        let mut state = GameState::new();
        state.ship = save_data.ship;
        state.resources = save_data.resources;
        state.phase = save_data.phase;
        state.engine_state = save_data.engine_state;
        state.escape_timer = save_data.escape_timer;
        state.upgrades = save_data.upgrades;
        state.frame_count = save_data.frame_count;
        state.enemies = save_data.enemies.into_iter().map(|s| Enemy {
            id: s.id,
            enemy_type: s.enemy_type,
            position: vec2(s.pos.0, s.pos.1),
            health: s.hp,
            max_health: s.max_hp,
            speed: s.speed,
            damage: s.damage,
            target_module: s.target,
        }).collect();
        state.projectiles = save_data.projectiles.into_iter().map(|s| Projectile {
            position: vec2(s.pos.0, s.pos.1),
            velocity: vec2(s.vel.0, s.vel.1),
            damage: s.damage,
            active: s.active,
        }).collect();
        state.particles = save_data.particles.into_iter().map(|s| Particle {
            position: vec2(s.pos.0, s.pos.1),
            velocity: vec2(s.vel.0, s.vel.1),
            lifetime: s.life,
            max_lifetime: s.max_life,
            color: Color::new(s.color.0, s.color.1, s.color.2, s.color.3),
            active: s.active,
        }).collect();
        state.scrap_piles = save_data.scrap_piles.into_iter().map(|s| ScrapPile {
            position: vec2(s.pos.0, s.pos.1),
            amount: s.amount,
            active: s.active,
        }).collect();
        Ok(state)
    }
}
