use crate::enemy::entities::{Enemy, EnemyType};
use crate::ship::ship::{ModuleState, ModuleType, Ship};
use crate::simulation::constants::*;
use crate::simulation::events::{EventBus, GameEvent};
use crate::state::{EngineState, GameState};
use macroquad::prelude::*;
use macroquad_toolkit::rng;

pub fn update_wave_logic(state: &mut GameState, dt: f32, events: &mut EventBus) {
    let signature = state.threat_signature;

    if state.engine_state == EngineState::Charging {
        let has_boss = state
            .enemies
            .iter()
            .any(|e| e.enemy_type == EnemyType::Boss);
        if !has_boss {
            spawn_boss(&mut state.enemies, events, state.frame_count);
        }
    }

    state.wave_state.update(dt);
    if signature < WAVE_GRACE_POWER {
        return;
    }

    let targeting_tier = state.upgrades.get_level("targeting_tier");
    let diff_mult = 1.0 + (targeting_tier as f32 * 0.5);

    let (drone_interval, guard_interval) = if signature >= WAVE_T3_POWER {
        (
            SPAWN_INTERVAL_DRONE_T3 / diff_mult,
            SPAWN_INTERVAL_GUARD_T3 / diff_mult,
        )
    } else if signature >= WAVE_T2_POWER {
        (
            SPAWN_INTERVAL_DRONE_T2 / diff_mult,
            SPAWN_INTERVAL_GUARD_T2 / diff_mult,
        )
    } else if signature >= WAVE_T1_POWER {
        (
            SPAWN_INTERVAL_DRONE_T1 / diff_mult,
            SPAWN_INTERVAL_GUARD_T2 * 1.4,
        )
    } else {
        (SPAWN_INTERVAL_DRONE_T0 / diff_mult, f32::MAX)
    };

    if state.wave_state.spawn_timer >= drone_interval {
        spawn_drone(&mut state.enemies, state.frame_count);
        state.wave_state.reset_spawn_timer();
    }

    if (state.weapon_powered() || state.defense_powered() || signature >= WAVE_T2_POWER)
        && state.wave_state.guard_timer >= guard_interval / diff_mult
    {
        spawn_guard(&mut state.enemies, state.frame_count);
        state.wave_state.reset_guard_timer();
    }

    let support_is_hot =
        state.recycler_online() || state.medbay_online() || state.life_support_online();
    if support_is_hot
        && signature >= WAVE_T1_POWER
        && state.wave_state.leech_timer >= SPAWN_INTERVAL_LEECH / diff_mult
    {
        spawn_leech(&mut state.enemies, state.frame_count);
        state.wave_state.reset_leech_timer();
    }

    if (state.engine_powered() || signature >= WAVE_T3_POWER)
        && state.wave_state.siege_timer >= SPAWN_INTERVAL_SIEGE / diff_mult
    {
        spawn_siege(&mut state.enemies, state.frame_count);
        state.wave_state.reset_siege_timer();
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

fn spawn_leech(enemies: &mut Vec<Enemy>, frame_count: u64) {
    let pos = random_spawn_position();
    let id = generate_enemy_id(enemies.len(), frame_count);
    enemies.push(Enemy::new(id, EnemyType::Leech, pos));
}

fn spawn_siege(enemies: &mut Vec<Enemy>, frame_count: u64) {
    let pos = random_spawn_position();
    let id = generate_enemy_id(enemies.len(), frame_count);
    enemies.push(Enemy::new(id, EnemyType::SiegeConstruct, pos));
}

pub fn spawn_boss(enemies: &mut Vec<Enemy>, events: &mut EventBus, frame_count: u64) {
    // Spawn boss at top center
    let pos = vec2(SCREEN_WIDTH / 2.0, -100.0);
    let id = generate_enemy_id(enemies.len(), frame_count);
    enemies.push(Enemy::new(id, EnemyType::Boss, pos));
    events.push_game(GameEvent::EngineActivated); // Reuse for boss spawn notification
}

fn random_spawn_position() -> Vec2 {
    let side = rng::gen_range(0, 4);
    match side {
        0 => vec2(rng::gen_range(0.0, SCREEN_WIDTH), -50.0), // Top
        1 => vec2(SCREEN_WIDTH + 50.0, rng::gen_range(0.0, SCREEN_HEIGHT)), // Right
        2 => vec2(rng::gen_range(0.0, SCREEN_WIDTH), SCREEN_HEIGHT + 50.0), // Bottom
        _ => vec2(-50.0, rng::gen_range(0.0, SCREEN_HEIGHT)), // Left
    }
}

fn generate_enemy_id(enemy_count: usize, frame_count: u64) -> u64 {
    enemy_count as u64 + frame_count
}

// Note: spawn_scrap_piles was moved to GameState::spawn_scrap_piles() for better room-aware placement

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

                // Debug if stuck
                // if state.frame_count % 60 == 0 {
                //      println!("Drone {} at {}, speed {}, dt {}, dir {}, core {}",
                //      enemy.id, enemy.position, enemy.speed, dt, dir, core_pos);
                // }
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
                // Leech: Find utility module or core, attach when close, drain power
                if enemy.attached_to.is_some() {
                    // Already attached - stay in place (damage handled in combat.rs)
                } else {
                    // Try to find a utility module first
                    let target = find_utility_module(&state.ship).or(state.ship.find_core());
                    if let Some(t) = target {
                        let target_pos = grid_to_screen(t.0, t.1);
                        let dist = enemy.position.distance(target_pos);
                        if dist < ENEMY_ATTACK_RANGE {
                            // Attach to the module
                            enemy.attached_to = Some(t);
                            enemy.target_module = Some(t);
                        } else {
                            let dir = (target_pos - enemy.position).normalize_or_zero();
                            enemy.position += dir * enemy.speed * dt;
                            enemy.target_module = Some(t);
                        }
                    }
                }
            }
            EnemyType::SiegeConstruct => {
                // Siege: Very slow, high damage, targets hull/core directly
                // Moves to center of screen (where ship is) and attacks
                let dir = (core_pos - enemy.position).normalize_or_zero();
                enemy.position += dir * enemy.speed * dt;
                enemy.target_module = state.ship.find_core();
            }
            EnemyType::Boss => {
                // Boss: Slow approach, cycles through special abilities
                let center = vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);
                let _dist_to_center = enemy.position.distance(center);

                // Boss moves to Core/Center to attack
                // Removing 150.0 distance stop so it actually attacks
                let dir = (center - enemy.position).normalize_or_zero();
                enemy.position += dir * enemy.speed * dt;

                // Update ability timer
                enemy.ability_timer += dt;

                // Boss targets weapons preferentially, then core
                if let Some(target) = find_priority_target(&state.ship) {
                    enemy.target_module = Some(target);
                } else {
                    enemy.target_module = state.ship.find_core();
                }
            }
        }
    }
}

/// Find active utility modules for Leech targeting
fn find_utility_module(ship: &Ship) -> Option<(usize, usize)> {
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            if let Some(module) = &ship.grid[x][y] {
                if module.state == ModuleState::Active && module.module_type == ModuleType::Utility
                {
                    return Some((x, y));
                }
            }
        }
    }
    None
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
                        ModuleType::Weapon | ModuleType::Defense
                            // Simple: return first found. Could improve with distance check.
                            if best.is_none() => {
                                best = Some((x, y));
                            }
                        _ => {}
                    }
                }
            }
        }
    }

    best
}
