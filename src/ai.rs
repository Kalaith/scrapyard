use macroquad::prelude::*;
use crate::state::GameState;
use crate::entities::{Enemy, EnemyType};
use crate::constants::*;

pub fn update_wave_logic(state: &mut GameState, dt: f32) {
    state.frame_count += 1; // Or use dt accumulator for better precision
    
    let power_level = state.resources.power;
    
    // Spawn Logic
    // Simple logic: Base spawn rate + Power modifier
    // E.g. 0 Power = 1 drone per 5s
    // 50 Power = 1 drone per 1s
    
    let base_interval = 5.0; // Seconds
    let spawn_interval = (base_interval - (power_level as f32 * 0.05)).max(0.5); // Cap at 0.5s
    
    // Using a simple timer for now (assuming 60fps approx update)
    // In production, use a dedicated time accumulator in GameState
    let frames_per_spawn = (spawn_interval * 60.0) as u64;
    
    if state.frame_count % frames_per_spawn == 0 {
        spawn_enemy(state);
    }
}

fn spawn_enemy(state: &mut GameState) {
    // Determine Type based on power
    let enemy_type = if state.resources.power > 20 {
        if rand::gen_range(0, 100) < 20 { EnemyType::Nanoguard } else { EnemyType::Nanodrone }
    } else {
        EnemyType::Nanodrone
    };

    // Pick a random edge
    let side = rand::gen_range(0, 4);
    let (x, y) = match side {
        0 => (rand::gen_range(-100.0, SCREEN_WIDTH + 100.0), -50.0), // Top
        1 => (SCREEN_WIDTH + 50.0, rand::gen_range(-100.0, SCREEN_HEIGHT + 100.0)), // Right
        2 => (rand::gen_range(-100.0, SCREEN_WIDTH + 100.0), SCREEN_HEIGHT + 50.0), // Bottom
        _ => (-50.0, rand::gen_range(-100.0, SCREEN_HEIGHT + 100.0)), // Left
    };

    let id = state.enemies.len() as u64 + state.frame_count; // Simple unique ID gen
    let enemy = Enemy::new(id, enemy_type, vec2(x, y));
    state.enemies.push(enemy);
    
    // println!("Spawned enemy at {}, {}", x, y);
}

pub fn update_enemies(state: &mut GameState, dt: f32) {
    // Get target (Core)
    // Ideally find the core position from grid
    let target = vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0); // Center for now

    for enemy in &mut state.enemies {
        let dir = (target - enemy.position).normalize_or_zero();
        enemy.position += dir * enemy.speed * dt;
    }
}
