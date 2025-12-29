use macroquad::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

use crate::ship::{Ship, ModuleState};
use crate::resources::Resources;
use crate::constants::*;
use crate::gameplay::ModuleRegistry;
use crate::entities::{Enemy, Projectile, Particle, EnemyType};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum GamePhase {
    Menu,
    Playing,
    GameOver,
}

pub struct GameState {
    pub ship: Ship,
    pub resources: Resources,
    pub phase: GamePhase,
    pub module_registry: ModuleRegistry,
    
    // Phase 3: Entities
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub particles: Vec<Particle>,
    pub frame_count: u64, // Used for spawning logic
}

impl GameState {
    pub fn new() -> Self {
        Self {
            ship: Ship::new(GRID_WIDTH, GRID_HEIGHT),
            resources: Resources::new(),
            phase: GamePhase::Menu,
            module_registry: ModuleRegistry::new(),
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
        self.phase = GamePhase::Playing;
    }

    pub fn update(&mut self, dt: f32) {
        match self.phase {
            GamePhase::Playing => {
                self.update_resources();
                crate::ai::update_wave_logic(self, dt);
                crate::ai::update_enemies(self, dt);
                crate::combat::update_combat(self, dt);
            }
            _ => {}
        }
    }

    fn update_resources(&mut self) {
        // Calculate total power
        let mut total_power = 0;
        
        for row in &self.ship.grid {
            for cell in row {
                if let Some(module) = cell {
                    if module.state == ModuleState::Active {
                        let stats = self.module_registry.get(module.module_type);
                        total_power += stats.power_consumption; // Negative for consumption, Positive for gen
                    }
                }
            }
        }
        
        self.resources.power = total_power;
    }

    pub fn draw(&self) {
        match self.phase {
            GamePhase::Playing => {
                self.draw_grid();
                self.draw_hud();
            }
            _ => {}
        }
    }

    fn draw_grid(&self) {
        // Temporary Debug Draw
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let px = x as f32 * CELL_SIZE + 50.0;
                let py = y as f32 * CELL_SIZE + 50.0;
                
                draw_rectangle_lines(px, py, CELL_SIZE, CELL_SIZE, 1.0, COLOR_GRID_LINE);

                if let Some(module) = &self.ship.grid[x][y] {
                    // Simple color coding for debugging
                    let color = match module.module_type {
                        crate::ship::ModuleType::Core => RED,
                        crate::ship::ModuleType::Empty => DARKGRAY,
                        _ => BLUE,
                    };
                    draw_rectangle(px + 2.0, py + 2.0, CELL_SIZE - 4.0, CELL_SIZE - 4.0, color);
                }
            }
        }
    }

    fn draw_hud(&self) {
        draw_text(
            &format!("Scrap: {}/{}", self.resources.scrap, self.resources.max_scrap),
            10.0,
            30.0,
            30.0,
            WHITE
        );
        draw_text(
            &format!("Power: {}", self.resources.power),
            200.0,
            30.0,
            30.0,
            YELLOW
        );
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let save_data = SaveData {
            ship: self.ship.clone(),
            resources: self.resources.clone(),
            phase: self.phase,
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
    pub enemies: Vec<SavedEnemy>,
    pub projectiles: Vec<SavedProjectile>,
    pub particles: Vec<SavedParticle>,
    pub frame_count: u64,
}
