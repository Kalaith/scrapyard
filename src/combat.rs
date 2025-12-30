use macroquad::prelude::*;
use crate::state::GameState;
use crate::entities::Projectile;
use crate::ship::{ModuleType, ModuleState};
use crate::constants::*;
use crate::events::{EventBus, GameEvent};

pub fn update_combat(state: &mut GameState, dt: f32, events: &mut EventBus) {
    // 1. Modules Fire (Towers)
    fire_towers(state, dt, events);
    
    // 2. Projectiles Move & Collide
    update_projectiles(state, dt, events);
    
    // 3. Enemies Attack Modules
    enemy_attacks(state, dt, events);
}

fn fire_towers(state: &mut GameState, dt: f32, events: &mut EventBus) {
    let mut new_projectiles = Vec::new();
    
    // Check each weapon room for repair percentage
    for room in &state.interior.rooms {
        // Only process weapon rooms
        if room.room_type != crate::interior::RoomType::Module(ModuleType::Weapon) {
            continue;
        }
        
        // Skip if no repair points
        if room.repair_points.is_empty() {
            continue;
        }
        
        // Calculate repair percentage (0.0 to 1.0)
        let repaired = room.repaired_count();
        if repaired == 0 {
            continue; // Not operational at all
        }
        
        let repair_pct = repaired as f32 / room.repair_points.len() as f32;
        
        // Get the linked module position for screen coordinates
        let (gx, gy) = match room.module_index {
            Some(pos) => pos,
            None => continue,
        };
        
        // Base stats
        let base_fire_rate = 0.8; // Shots per second at full power
        let base_damage = 25.0;
        let base_range = 300.0;
        
        // Scale with repair percentage
        let effective_fire_rate = base_fire_rate * repair_pct;
        let effective_damage = base_damage * repair_pct;
        let effective_range = base_range; // Range stays constant
        
        let fire_chance = effective_fire_rate * dt;
        
        if rand::gen_range(0.0, 1.0) < fire_chance {
            let tower_pos = grid_to_screen(gx, gy);
            
            if let Some(target) = find_nearest_enemy(&state.enemies, tower_pos, effective_range) {
                new_projectiles.push(Projectile::new(tower_pos, target, 400.0, effective_damage));
                events.push_game(GameEvent::WeaponFired { x: tower_pos.x, y: tower_pos.y });
            }
        }
    }
    
    state.projectiles.append(&mut new_projectiles);
}

fn grid_to_screen(x: usize, y: usize) -> Vec2 {
    let start_x = (screen_width() - GRID_WIDTH as f32 * CELL_SIZE) / 2.0;
    let start_y = (screen_height() - GRID_HEIGHT as f32 * CELL_SIZE) / 2.0;
    vec2(
        start_x + x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        start_y + y as f32 * CELL_SIZE + CELL_SIZE / 2.0,
    )
}

fn screen_to_grid(pos: Vec2) -> Option<(usize, usize)> {
    let start_x = (screen_width() - GRID_WIDTH as f32 * CELL_SIZE) / 2.0;
    let start_y = (screen_height() - GRID_HEIGHT as f32 * CELL_SIZE) / 2.0;
    
    if pos.x < start_x || pos.y < start_y {
        return None;
    }
    
    let x = ((pos.x - start_x) / CELL_SIZE) as usize;
    let y = ((pos.y - start_y) / CELL_SIZE) as usize;
    
    if x < GRID_WIDTH && y < GRID_HEIGHT {
        Some((x, y))
    } else {
        None
    }
}

fn find_nearest_enemy(enemies: &[crate::entities::Enemy], pos: Vec2, range: f32) -> Option<Vec2> {
    let mut nearest = None;
    let mut min_dist = range;
    
    for enemy in enemies {
        if enemy.health <= 0.0 { continue; }
        let d = pos.distance(enemy.position);
        if d < min_dist {
            min_dist = d;
            nearest = Some(enemy.position);
        }
    }
    
    nearest
}

fn update_projectiles(state: &mut GameState, dt: f32, events: &mut EventBus) {
    // Move projectiles
    for proj in &mut state.projectiles {
        proj.position += proj.velocity * dt;
        
        // Bounds check
        if proj.position.x < -100.0 || proj.position.x > screen_width() + 100.0 || 
           proj.position.y < -100.0 || proj.position.y > screen_height() + 100.0 {
            proj.active = false;
        }
    }
    
    // Collision detection
    for proj in state.projectiles.iter_mut() {
        if !proj.active { continue; }
        
        for enemy in state.enemies.iter_mut() {
            if enemy.health <= 0.0 { continue; }
            
            let hit_radius = match enemy.enemy_type {
                crate::entities::EnemyType::Boss => 40.0,
                crate::entities::EnemyType::Nanoguard => 15.0,
                _ => 10.0,
            };
            
            if proj.position.distance(enemy.position) < hit_radius {
                enemy.health -= proj.damage;
                proj.active = false;
                
                if enemy.health <= 0.0 {
                    // Enemy killed - drop scrap based on type
                    let scrap = match enemy.enemy_type {
                        crate::entities::EnemyType::Nanodrone => 3,
                        crate::entities::EnemyType::Nanoguard => 10,
                        crate::entities::EnemyType::Leech => 5,
                        crate::entities::EnemyType::Boss => 100,
                    };
                    state.resources.add_scrap(scrap);
                    state.resources.credits += scrap / 2;
                    
                    events.push_game(GameEvent::EnemyKilled { 
                        x: enemy.position.x, 
                        y: enemy.position.y, 
                        scrap_dropped: scrap 
                    });
                }
                break;
            }
        }
    }
    
    // Cleanup
    state.projectiles.retain(|p| p.active);
    state.enemies.retain(|e| e.health > 0.0);
}

fn enemy_attacks(state: &mut GameState, dt: f32, events: &mut EventBus) {
    let attack_range = 30.0;
    
    // Calculate shield reduction from all shield rooms
    let mut shield_reduction = 0.0;
    for room in &state.interior.rooms {
        if room.room_type == crate::interior::RoomType::Module(ModuleType::Defense) {
            if !room.repair_points.is_empty() {
                let repair_pct = room.repaired_count() as f32 / room.repair_points.len() as f32;
                shield_reduction += repair_pct * 0.5; // Each shield room can block up to 50%
            }
        }
    }
    // Cap at 80% damage reduction max
    shield_reduction = shield_reduction.min(0.8);
    
    for enemy in &state.enemies {
        if enemy.health <= 0.0 { continue; }
        
        if let Some(grid_pos) = screen_to_grid(enemy.position) {
            let (gx, gy) = grid_pos;
            
            for dx in -1i32..=1 {
                for dy in -1i32..=1 {
                    let nx = (gx as i32 + dx) as usize;
                    let ny = (gy as i32 + dy) as usize;
                    
                    if nx < GRID_WIDTH && ny < GRID_HEIGHT {
                        if state.ship.grid[nx][ny].is_some() {
                            let module_pos = grid_to_screen(nx, ny);
                            let dist = enemy.position.distance(module_pos);
                            
                            if dist < attack_range {
                                // Apply shield reduction to damage
                                let base_damage = enemy.damage * dt;
                                let damage = base_damage * (1.0 - shield_reduction);
                                state.ship_integrity -= damage;
                                
                                events.push_game(GameEvent::ModuleDamaged { 
                                    x: nx, 
                                    y: ny, 
                                    damage 
                                });
                                
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
