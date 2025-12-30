use macroquad::prelude::*;
use crate::state::{GameState, EngineState};
use crate::enemy::entities::{Enemy, EnemyType, ScrapPile};
use crate::simulation::constants::*;
use crate::ship::ship::{ModuleType, ModuleState, Ship};
use crate::simulation::events::{EventBus, GameEvent};

use crate::enemy::wave::WaveState;

pub fn update_wave_logic(
    total_power: i32,
    engine_state: &EngineState,
    enemies: &mut Vec<Enemy>,
    upgrades: &crate::economy::upgrades::GameUpgrades,
    wave_state: &mut WaveState,
    frame_count: u64,
    dt: f32,
    events: &mut EventBus
) {
    let power_level = total_power;
    
    // Boss mode: Stop normal spawn when engine is charging or power >= 16
    if *engine_state == EngineState::Charging {
        // In boss mode, only spawn boss if not already present
        let has_boss = enemies.iter().any(|e| e.enemy_type == EnemyType::Boss);
        if !has_boss {
            spawn_boss(enemies, events, frame_count);
        }
        return;
    }

    // Normal wave logic based on power level per GDD
    wave_state.update(dt);

    // No enemies spawn until player has enough power (give grace period)
    if power_level < WAVE_GRACE_POWER {
        return;
    }

    let targeting_tier = upgrades.get_level("targeting_tier");
    let diff_mult = 1.0 + (targeting_tier as f32 * 0.5);

    let (drone_interval, guard_interval) = if power_level >= WAVE_T3_POWER {
        (SPAWN_INTERVAL_DRONE_T3 / diff_mult, SPAWN_INTERVAL_GUARD_T3 / diff_mult)
    } else if power_level >= WAVE_T2_POWER {
        (SPAWN_INTERVAL_DRONE_T2 / diff_mult, SPAWN_INTERVAL_GUARD_T2 / diff_mult)
    } else if power_level >= WAVE_T1_POWER {
        (SPAWN_INTERVAL_DRONE_T1 / diff_mult, f32::MAX)
    } else {
        (SPAWN_INTERVAL_DRONE_T0 / diff_mult, f32::MAX)
    };

    if wave_state.spawn_timer >= drone_interval {
        spawn_drone(enemies, frame_count);
        wave_state.reset_spawn_timer();
    }

    if power_level >= 6 && wave_state.guard_timer >= guard_interval {
        spawn_guard(enemies, frame_count);
        wave_state.reset_guard_timer();
    }
}

fn spawn_drone(enemies: &mut Vec<Enemy>, frame_count: u64) {
    let pos = random_spawn_position();
    let id = generate_enemy_id(enemies.len(), frame_count);
    enemies.push(Enemy::new(id, EnemyType::Nanodrone, pos));
}

fn spawn_guard(enemies: &mut Vec<Enemy>, frame_count: u64) {
    let pos = random_spawn_position();
    let id = generate_enemy_id(enemies.len(), frame_count);
    enemies.push(Enemy::new(id, EnemyType::Nanoguard, pos));
}

fn spawn_boss(enemies: &mut Vec<Enemy>, events: &mut EventBus, frame_count: u64) {
    // Spawn boss at top center
    let pos = vec2(SCREEN_WIDTH / 2.0, -100.0);
    let id = generate_enemy_id(enemies.len(), frame_count);
    enemies.push(Enemy::new(id, EnemyType::Boss, pos));
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

fn generate_enemy_id(enemy_count: usize, frame_count: u64) -> u64 {
    enemy_count as u64 + frame_count
}

pub fn spawn_scrap_piles(state: &mut GameState) {
    let interior = &state.interior;
    
    // Initial scrap piles
    let count = rand::gen_range(MIN_SCRAP_PILES, MAX_SCRAP_PILES + 1);
    for _ in 0..count {
        let pos = vec2(
            rand::gen_range(SCRAP_SPAWN_PADDING, interior.width - SCRAP_SPAWN_PADDING),
            rand::gen_range(SCRAP_SPAWN_PADDING, interior.height - SCRAP_SPAWN_PADDING)
        );
        let amount = rand::gen_range(SCRAP_PILE_MIN_AMOUNT, SCRAP_PILE_MAX_AMOUNT + 1);
        state.scrap_piles.push(ScrapPile::new(pos, amount));
    }
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
fn find_priority_target(ship: &Ship) -> Option<(usize, usize)> {
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
