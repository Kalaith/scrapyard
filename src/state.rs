use macroquad::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

use crate::ship::{Ship, ModuleState, ModuleType};
use crate::resources::Resources;
use crate::constants::*;
use crate::gameplay::ModuleRegistry;
use crate::entities::{Enemy, Projectile, Particle, EnemyType};
use crate::events::{EventBus, UIEvent, GameEvent};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum GamePhase {
    Menu,
    Playing,
    GameOver,
    Victory,
}

/// Engine activation state
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum EngineState {
    Idle,
    Charging,
    Escaped,
}

pub struct GameState {
    pub ship: Ship,
    pub resources: Resources,
    pub phase: GamePhase,
    pub module_registry: ModuleRegistry,
    
    // Game state flags
    pub paused: bool,
    pub engine_state: EngineState,
    pub escape_timer: f32, // Countdown in seconds (180s = 3 minutes)
    
    // Entities
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub particles: Vec<Particle>,
    pub frame_count: u64,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            ship: Ship::new(GRID_WIDTH, GRID_HEIGHT),
            resources: Resources::new(),
            phase: GamePhase::Menu,
            module_registry: ModuleRegistry::new(),
            paused: false,
            engine_state: EngineState::Idle,
            escape_timer: 180.0,
            enemies: Vec::new(),
            projectiles: Vec::new(),
            particles: Vec::new(),
            frame_count: 0,
        }
    }

    /// Start a new game, resetting all state to fresh values.
    pub fn start_new_game(&mut self) {
        self.ship = Ship::new(GRID_WIDTH, GRID_HEIGHT);
        self.resources = Resources::new();
        self.enemies.clear();
        self.projectiles.clear();
        self.particles.clear();
        self.frame_count = 0;
        self.paused = false;
        self.engine_state = EngineState::Idle;
        self.escape_timer = 180.0;
        self.phase = GamePhase::Playing;
    }

    pub fn update(&mut self, dt: f32, events: &mut EventBus) {
        match self.phase {
            GamePhase::Playing => {
                if !self.paused {
                    self.update_resources();
                    self.update_engine(dt, events);
                    crate::ai::update_wave_logic(self, dt, events);
                    crate::ai::update_enemies(self, dt);
                    crate::combat::update_combat(self, dt, events);
                    self.frame_count += 1;
                    
                    // Check for game over (core destroyed)
                    self.check_game_over(events);
                }
            }
            _ => {}
        }
    }

    fn check_game_over(&mut self, events: &mut EventBus) {
        // Check if core is destroyed
        if let Some(core_pos) = self.ship.find_core() {
            if let Some(core) = &self.ship.grid[core_pos.0][core_pos.1] {
                if core.state == ModuleState::Destroyed || core.health <= 0.0 {
                    self.phase = GamePhase::GameOver;
                    events.push_game(GameEvent::CoreDestroyed);
                }
            }
        } else {
            // No core found at all
            self.phase = GamePhase::GameOver;
        }
    }

    fn update_resources(&mut self) {
        let mut total_power = 0;
        
        for row in &self.ship.grid {
            for cell in row {
                if let Some(module) = cell {
                    if module.state == ModuleState::Active {
                        let stats = self.module_registry.get(module.module_type);
                        total_power += stats.power_consumption;
                    }
                }
            }
        }
        
        self.resources.power = total_power;
    }

    fn update_engine(&mut self, dt: f32, events: &mut EventBus) {
        if self.engine_state == EngineState::Charging {
            self.escape_timer -= dt;
            if self.escape_timer <= 0.0 {
                self.engine_state = EngineState::Escaped;
                self.phase = GamePhase::Victory;
                events.push_game(GameEvent::EscapeSuccess);
            }
        }
    }

    /// Attempt to repair a module at the given coordinates
    pub fn attempt_repair(&mut self, x: usize, y: usize, events: &mut EventBus) -> bool {
        let repair_cost = {
            if let Some(module) = &self.ship.grid[x][y] {
                if module.state != ModuleState::Destroyed {
                    return false;
                }
                self.module_registry.get(module.module_type).base_cost
            } else {
                return false;
            }
        };

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

    /// Attempt to upgrade a module at the given coordinates
    pub fn attempt_upgrade(&mut self, x: usize, y: usize, events: &mut EventBus) -> bool {
        const MAX_LEVEL: u8 = 5;
        const UPGRADE_MULTIPLIER: f32 = 1.5;

        let upgrade_cost = {
            if let Some(module) = &self.ship.grid[x][y] {
                if module.state == ModuleState::Destroyed || module.level >= MAX_LEVEL {
                    return false;
                }
                let base_cost = self.module_registry.get(module.module_type).base_cost;
                (base_cost as f32 * (module.level as f32 * 0.5 + 1.0)) as i32
            } else {
                return false;
            }
        };

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

    /// Toggle a module online/offline
    pub fn toggle_module(&mut self, x: usize, y: usize) {
        if let Some(module) = &mut self.ship.grid[x][y] {
            match module.state {
                ModuleState::Active => module.state = ModuleState::Offline,
                ModuleState::Offline => module.state = ModuleState::Active,
                ModuleState::Destroyed => {} // Can't toggle destroyed modules
            }
        }
    }

    /// Activate the engine to start escape sequence
    pub fn activate_engine(&mut self, events: &mut EventBus) {
        if self.engine_state == EngineState::Idle {
            // Check if engine module exists and is active
            for row in &self.ship.grid {
                for cell in row {
                    if let Some(module) = cell {
                        if module.module_type == ModuleType::Engine && module.state == ModuleState::Active {
                            self.engine_state = EngineState::Charging;
                            self.escape_timer = 180.0;
                            events.push_game(GameEvent::EngineActivated);
                            return;
                        }
                    }
                }
            }
        }
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
        
        Ok(state)
    }
}

/// Process UI events from the event bus
pub fn process_ui_events(state: &mut GameState, events: &mut EventBus) {
    for event in events.drain_ui() {
        match event {
            UIEvent::StartGame => {
                state.start_new_game();
            }
            UIEvent::ReturnToMenu => {
                state.phase = GamePhase::Menu;
            }
            UIEvent::Pause => {
                state.paused = true;
            }
            UIEvent::Resume => {
                state.paused = false;
            }
            UIEvent::Repair(x, y) => {
                state.attempt_repair(x, y, events);
            }
            UIEvent::Upgrade(x, y) => {
                state.attempt_upgrade(x, y, events);
            }
            UIEvent::Toggle(x, y) => {
                state.toggle_module(x, y);
            }
            UIEvent::ActivateEngine => {
                state.activate_engine(events);
            }
        }
    }
}

// Serialization Helpers

#[derive(Serialize, Deserialize)]
struct SavedEnemy {
    id: u64,
    enemy_type: EnemyType,
    pos: (f32, f32),
    hp: f32,
    max_hp: f32,
    speed: f32,
    damage: f32,
    target: Option<(usize, usize)>,
}

#[derive(Serialize, Deserialize)]
struct SavedProjectile {
    pos: (f32, f32),
    vel: (f32, f32),
    damage: f32,
    active: bool,
}

#[derive(Serialize, Deserialize)]
struct SavedParticle {
    pos: (f32, f32),
    vel: (f32, f32),
    life: f32,
    max_life: f32,
    color: (f32, f32, f32, f32),
    active: bool,
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
    pub frame_count: u64,
}
