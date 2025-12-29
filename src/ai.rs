use macroquad::prelude::*;
use crate::state::{GameState, EngineState};
use crate::entities::{Enemy, EnemyType};
use crate::constants::*;
use crate::ship::{ModuleType, ModuleState};
use crate::events::{EventBus, GameEvent};

/// Accumulated time for spawn timing
static mut SPAWN_TIMER: f32 = 0.0;
static mut GUARD_TIMER: f32 = 0.0;

pub fn update_wave_logic(state: &mut GameState, dt: f32, events: &mut EventBus) {
    let power_level = state.resources.power;
    
    // Boss mode: Stop normal spawn when engine is charging or power >= 16
    if state.engine_state == EngineState::Charging {
        // In boss mode, only spawn boss if not already present
        let has_boss = state.enemies.iter().any(|e| e.enemy_type == EnemyType::Boss);
        if !has_boss {
            spawn_boss(state, events);
        }
        return;
    }

    // Normal wave logic based on power level per GDD
    unsafe {
        SPAWN_TIMER += dt;
        GUARD_TIMER += dt;
    }

    // Power 0-5: Drone every 3s
    // Power 6-10: Drone every 1s + Guard every 10s
    // Power 11-15: Drone every 0.5s + Guard every 5s
    // Power 16+: Boss trigger (handled above when engine activates)

    let (drone_interval, guard_interval) = if power_level >= 11 {
        (0.5, 5.0)
    } else if power_level >= 6 {
        (1.0, 10.0)
    } else {
        (3.0, f32::MAX) // No guards at low power
    };

    unsafe {
        if SPAWN_TIMER >= drone_interval {
            spawn_drone(state, events);
            SPAWN_TIMER = 0.0;
        }

        if power_level >= 6 && GUARD_TIMER >= guard_interval {
            spawn_guard(state, events);
            GUARD_TIMER = 0.0;
        }
    }
}

fn spawn_drone(state: &mut GameState, _events: &mut EventBus) {
    let pos = random_spawn_position();
    let id = generate_enemy_id(state);
    state.enemies.push(Enemy::new(id, EnemyType::Nanodrone, pos));
}

fn spawn_guard(state: &mut GameState, _events: &mut EventBus) {
    let pos = random_spawn_position();
    let id = generate_enemy_id(state);
    state.enemies.push(Enemy::new(id, EnemyType::Nanoguard, pos));
}

fn spawn_boss(state: &mut GameState, events: &mut EventBus) {
    // Spawn boss at top center
    let pos = vec2(SCREEN_WIDTH / 2.0, -100.0);
    let id = generate_enemy_id(state);
    state.enemies.push(Enemy::new(id, EnemyType::Boss, pos));
    events.push_game(GameEvent::EngineActivated); // Reuse for boss spawn notification
}

fn random_spawn_position() -> Vec2 {
    let side = rand::gen_range(0, 4);
    match side {
        0 => vec2(rand::gen_range(0.0, SCREEN_WIDTH), -50.0), // Top
        1 => vec2(SCREEN_WIDTH + 50.0, rand::gen_range(0.0, SCREEN_HEIGHT)), // Right
        2 => vec2(rand::gen_range(0.0, SCREEN_WIDTH), SCREEN_HEIGHT + 50.0), // Bottom
        _ => vec2(-50.0, rand::gen_range(0.0, SCREEN_HEIGHT)), // Left
    }
}

fn generate_enemy_id(state: &GameState) -> u64 {
    state.enemies.len() as u64 + state.frame_count
}

pub fn update_enemies(state: &mut GameState, dt: f32) {
    // Calculate core position from grid
    let core_pos = get_core_screen_position(state);
    
    for enemy in &mut state.enemies {
        match enemy.enemy_type {
            EnemyType::Nanodrone => {
                // Rusher: Move directly to core
                let dir = (core_pos - enemy.position).normalize_or_zero();
                enemy.position += dir * enemy.speed * dt;
                enemy.target_module = state.ship.find_core();
            }
            EnemyType::Nanoguard => {
                // Tank: Try to find nearest weapon/shield first, then core
                if let Some(target) = find_priority_target(&state.ship) {
                    let target_pos = grid_to_screen(target.0, target.1);
                    let dir = (target_pos - enemy.position).normalize_or_zero();
                    enemy.position += dir * enemy.speed * dt;
                    enemy.target_module = Some(target);
                } else {
                    // No priority target, go for core
                    let dir = (core_pos - enemy.position).normalize_or_zero();
                    enemy.position += dir * enemy.speed * dt;
                    enemy.target_module = state.ship.find_core();
                }
            }
            EnemyType::Leech => {
                // Leech: Go for utility/core and attach
                let dir = (core_pos - enemy.position).normalize_or_zero();
                enemy.position += dir * enemy.speed * dt;
                enemy.target_module = state.ship.find_core();
            }
            EnemyType::Boss => {
                // Boss: Slow approach to center
                let center = vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);
                let dir = (center - enemy.position).normalize_or_zero();
                enemy.position += dir * enemy.speed * dt;
                enemy.target_module = state.ship.find_core();
            }
        }
    }
}

fn get_core_screen_position(state: &GameState) -> Vec2 {
    if let Some((x, y)) = state.ship.find_core() {
        grid_to_screen(x, y)
    } else {
        vec2(screen_width() / 2.0, screen_height() / 2.0)
    }
}

fn grid_to_screen(x: usize, y: usize) -> Vec2 {
    let start_x = (screen_width() - GRID_WIDTH as f32 * CELL_SIZE) / 2.0;
    let start_y = (screen_height() - GRID_HEIGHT as f32 * CELL_SIZE) / 2.0;
    vec2(
        start_x + x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        start_y + y as f32 * CELL_SIZE + CELL_SIZE / 2.0,
    )
}

/// Find nearest active weapon or defense module for Nanoguard targeting
fn find_priority_target(ship: &crate::ship::Ship) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            if let Some(module) = &ship.grid[x][y] {
                if module.state == ModuleState::Active {
                    match module.module_type {
                        ModuleType::Weapon | ModuleType::Defense => {
                            // Simple: return first found. Could improve with distance check.
                            if best.is_none() {
                                best = Some((x, y));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    best
}
