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
use crate::player::Player;
use crate::interior::ShipInterior;

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

/// Current view mode for dual-view gameplay
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    Exterior,  // Ship overview, enemies, combat
    Interior,  // Player walking inside ship
}

/// Tutorial step for guiding player through repairs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialStep {
    Welcome,      // Intro message
    RepairReactor,  // Room 12 - Core
    RepairShields,  // Room 5 - Shields
    RepairWeapon,   // Room 1 - Weapon
    RepairEngine,   // Room 20 - Engine
    Complete,       // Tutorial done
}

impl TutorialStep {
    pub fn target_room(&self) -> Option<usize> {
        match self {
            TutorialStep::RepairReactor => Some(12),
            TutorialStep::RepairShields => Some(5),
            TutorialStep::RepairWeapon => Some(1),
            TutorialStep::RepairEngine => Some(20),
            _ => None,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            TutorialStep::Welcome => "Welcome aboard! Your ship is damaged.\nUse WASD to move. Press E near orange repair points.",
            TutorialStep::RepairReactor => "First, repair the REACTOR to restore power.\nFollow the highlighted path to the central room.",
            TutorialStep::RepairShields => "Good! Now repair the SHIELDS for defense.\nHead to the shield room above.",
            TutorialStep::RepairWeapon => "Shields online! Repair a WEAPON to fight back.\nGo to the left weapon bay.",
            TutorialStep::RepairEngine => "Weapons ready! Finally, repair the ENGINE.\nHead to the engine room below.",
            TutorialStep::Complete => "All systems operational! Enemies will now attack.\nPress Tab to view exterior. Good luck!",
        }
    }

    pub fn next(&self) -> TutorialStep {
        match self {
            TutorialStep::Welcome => TutorialStep::RepairReactor,
            TutorialStep::RepairReactor => TutorialStep::RepairShields,
            TutorialStep::RepairShields => TutorialStep::RepairWeapon,
            TutorialStep::RepairWeapon => TutorialStep::RepairEngine,
            TutorialStep::RepairEngine => TutorialStep::Complete,
            TutorialStep::Complete => TutorialStep::Complete,
        }
    }
}

pub struct GameState {
    pub ship: Ship,
    pub interior: ShipInterior,
    pub resources: Resources,
    pub phase: GamePhase,
    pub module_registry: ModuleRegistry,
    
    // Dual-view system
    pub view_mode: ViewMode,
    pub player: Player,
    pub total_power: i32,
    pub used_power: i32,
    pub required_power: i32,
    
    // Ship integrity (game over when 0)
    pub ship_integrity: f32,
    pub ship_max_integrity: f32,
    
    // Tutorial
    pub tutorial_step: TutorialStep,
    pub tutorial_timer: f32,
    
    // Game state flags
    pub paused: bool,
    pub engine_state: EngineState,
    pub escape_timer: f32,
    
    // Scrap Gathering
    pub scrap_piles: Vec<crate::entities::ScrapPile>,
    pub gathering_target: Option<usize>,
    pub gathering_timer: f32,

    // Entities
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub particles: Vec<Particle>,
    pub frame_count: u64,
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
            tutorial_step: TutorialStep::Welcome,
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
            
            frame_count: 0,
        };
        
        state.spawn_scrap_piles();
        state
    }

    /// Start a new game, resetting all state to fresh values.
    pub fn start_new_game(&mut self) {
        self.ship = Ship::new(GRID_WIDTH, GRID_HEIGHT);
        self.interior = ShipInterior::starter_ship();
        self.resources = Resources::new();
        self.resources.scrap = 50; // Low starting scrap, forcing gathering
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
        self.tutorial_step = TutorialStep::Welcome;
        self.tutorial_timer = 0.0;
        self.phase = GamePhase::Playing;
        
        self.scrap_piles.clear();
        self.gathering_target = None;
        self.gathering_timer = 0.0;
        self.spawn_scrap_piles();
    }

    pub fn spawn_scrap_piles(&mut self) {
        use macroquad::rand::ChooseRandom;
        use crate::entities::ScrapPile;
        
        // Spawn 8-12 random scrap piles in random walkable rooms
        let count = ::rand::random::<usize>() % 5 + 8;
        
        for _ in 0..count {
            // Pick a random room
            if let Some(room) = self.interior.rooms.choose() {
                if room.room_type == crate::interior::RoomType::Empty { continue; }
                
                // Random pos within room
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
                    // Update player (movement in interior view)
                    if self.view_mode == ViewMode::Interior {
                        self.player.update(dt, &self.interior);
                        self.player.update_nearby_module(&self.interior);
                    }
                    
                    self.update_resources();
                    self.update_power();
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

    /// Calculate total power (generation) and used power (consumption)
    fn update_power(&mut self) {
        self.total_power = 0;
        self.used_power = 0;
        
        for room in &self.interior.rooms {
            if room.repair_points.is_empty() {
                continue;
            }
            
            let repaired = room.repaired_count() as i32;
            
            if repaired > 0 {
                match room.room_type {
                    crate::interior::RoomType::Module(ModuleType::Core) => {
                         // Reactor GENERATES power (1 per point)
                         self.total_power += repaired;
                    },
                    crate::interior::RoomType::Module(ModuleType::Weapon) => self.used_power += repaired * 1,   // 1 power per point
                    crate::interior::RoomType::Module(ModuleType::Defense) => self.used_power += repaired * 1,  // 1 power per point
                    crate::interior::RoomType::Module(ModuleType::Utility) => self.used_power += repaired * 1,  // 1 power per point
                    crate::interior::RoomType::Module(ModuleType::Engine) => self.used_power += repaired * 1,   // 1 power per point
                    crate::interior::RoomType::Cockpit => self.used_power += repaired * 1,                       // 1 power per point
                    crate::interior::RoomType::Medbay => self.used_power += repaired * 1,                        // 1 power per point
                    _ => {}
                }
            }
        }
    }

    fn check_game_over(&mut self, events: &mut EventBus) {
        // Game over when ship integrity reaches 0
        if self.ship_integrity <= 0.0 {
            self.ship_integrity = 0.0;
            self.phase = GamePhase::GameOver;
            events.push_game(GameEvent::CoreDestroyed);
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
        // Find engine room repair status
        let mut engine_repair_pct = 0.0;
        for room in &self.interior.rooms {
            if let crate::interior::RoomType::Module(ModuleType::Engine) = room.room_type {
                 if !room.repair_points.is_empty() {
                    engine_repair_pct = room.repaired_count() as f32 / room.repair_points.len() as f32;
                 }
            }
        }

        // If engine has > 25% repairs, it starts charging (prevents accidental trigger)
        if engine_repair_pct >= 0.25 {
            if self.engine_state == EngineState::Idle {
                self.engine_state = EngineState::Charging;
            }
            
            // Charge speed scales with repair percentage
            // 60s base time (was 180s). 
            // At 100% repair = 60s survival.
            // At 25% repair = 240s (4 mins) survival.
            if self.engine_state == EngineState::Charging {
                self.escape_timer -= dt * engine_repair_pct;
                
                if self.escape_timer <= 0.0 {
                    self.engine_state = EngineState::Escaped;
                    self.phase = GamePhase::Victory;
                    events.push_game(GameEvent::EscapeSuccess);
                }
            }
        } else {
            // Not repaired enough to charge
             self.engine_state = EngineState::Idle;
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

    /// Get cost to repair a specific point in a room
    pub fn get_repair_cost(&self, room_idx: usize, _point_idx: usize) -> Option<(i32, i32)> {
        if room_idx >= self.interior.rooms.len() {
            return None;
        }
        let room = &self.interior.rooms[room_idx];
        
        // Base cost
        let scrap_cost = 10;
        
        let power_cost = match room.room_type {
            crate::interior::RoomType::Module(crate::ship::ModuleType::Core) => 0,
            crate::interior::RoomType::Module(crate::ship::ModuleType::Weapon) => 2,
            crate::interior::RoomType::Module(crate::ship::ModuleType::Defense) => 2,
            crate::interior::RoomType::Module(crate::ship::ModuleType::Utility) => 1,
            crate::interior::RoomType::Module(crate::ship::ModuleType::Engine) => 3,
            crate::interior::RoomType::Cockpit => 2,
            crate::interior::RoomType::Medbay => 1,
            _ => 0,
        };
        
        Some((scrap_cost, power_cost))
    }

    /// Attempt to repair an interior point
    pub fn attempt_interior_repair(&mut self, room_idx: usize, point_idx: usize) -> bool {
         if room_idx >= self.interior.rooms.len() { return false; }
         
         // Use a scope to borrow room briefly or just access fields directly if possible
         // We need to check costs which are on self, and mutate self.
         
         let (scrap_cost, power_cost) = match self.get_repair_cost(room_idx, point_idx) {
             Some(c) => c,
             None => return false,
         };
         
         // Check if already repaired
         if self.interior.rooms[room_idx].repair_points.len() <= point_idx || 
            self.interior.rooms[room_idx].repair_points[point_idx].repaired {
             return false;
         }

         let is_reactor = power_cost == 0;

         // Check affordability
         if self.resources.scrap < scrap_cost {
             return false;
         }
         
         if !is_reactor && (self.used_power + power_cost > self.total_power) {
             return false;
         }

         // Perform repair
         self.resources.deduct(scrap_cost);
         self.interior.rooms[room_idx].repair_points[point_idx].repaired = true;
         
         // We don't update used_power here manually, because update_power() is called every frame
         // and will detect the new repair status.
         
         // Check if fully repaired to activate module
         if self.interior.rooms[room_idx].is_fully_repaired() {
            if let Some((gx, gy)) = self.interior.rooms[room_idx].module_index {
                // We need to invalidate path cache if a module state changes significantly? 
                // Actually cache invalidation is for STRUCTURE changes (add/remove module), 
                // but if we had blocked paths based on state, we'd need it. 
                // Currently pathfinding ignores state (walkable), so no invalidation needed.
                
                if let Some(module) = &mut self.ship.grid[gx][gy] {
                    module.state = ModuleState::Active;
                    module.health = module.max_health;
                }
            }
         }
         
         true
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
        
        state.scrap_piles = save_data.scrap_piles.into_iter().map(|s| crate::entities::ScrapPile {
            position: vec2(s.pos.0, s.pos.1),
            amount: s.amount,
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
struct SavedScrapPile {
    pos: (f32, f32),
    amount: i32,
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
    pub scrap_piles: Vec<SavedScrapPile>,
    pub frame_count: u64,
}
