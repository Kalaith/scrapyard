use macroquad::prelude::*;
use crate::state::GameState;
use crate::enemy::entities::{Enemy, Projectile, EnemyType};
use crate::ship::ship::ModuleType;
use crate::simulation::constants::*;
use crate::simulation::events::{EventBus, GameEvent};
use crate::ship::layout::Layout;
use crate::ship::interior::{RoomType, Room};

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
        if room.room_type != RoomType::Module(ModuleType::Weapon) {
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
        
        // Retrieve base stats from registry
        let stats = state.module_registry.get(ModuleType::Weapon);
        let base_fire_rate = stats.fire_rate;
        let base_damage = stats.damage;
        let base_range = stats.range;
        
        // Scale with repair percentage
        let effective_fire_rate = base_fire_rate * repair_pct;
        let effective_damage = base_damage * repair_pct;
        let effective_range = base_range * (0.5 + 0.5 * repair_pct); // 50% base range + 50% from repairs
        
        // Access Module to update cooldown
        // Note: Using disjoint borrow of state should work (interior is borrowed, ship is separate)
        if let Some(cell) = state.ship.grid.get_mut(gx).and_then(|row| row.get_mut(gy)) {
             if let Some(module) = cell {
                 // Decrease cooldown
                 module.cooldown -= dt;
                 
                 // Debug prints every 60 frames (approx 1 sec) to reduce spam?
                 // Or just print if cooldown <= 0?
                 if module.cooldown <= 0.0 && state.frame_count % 60 == 0 {
                    // println!("Weapon Ready: Repaired {}/{} (Pct {:.2}), Rate {:.2}, EffRate {:.2}, Rng {:.0}", 
                    //    room.repaired_count(), room.repair_points.len(), repair_pct, base_fire_rate, effective_fire_rate, effective_range);
                 }
                 
                 // Ready to fire?
                 if module.cooldown <= 0.0 {
                     let tower_pos = Layout::grid_to_screen_center(gx, gy);
                     
                     if let Some(target) = find_nearest_enemy(&state.enemies, tower_pos, effective_range) {
                         new_projectiles.push(Projectile::new(tower_pos, target, 400.0, effective_damage));
                         events.push_game(GameEvent::WeaponFired { x: tower_pos.x, y: tower_pos.y });
                         
                         // Reset cooldown
                         if effective_fire_rate > 0.001 {
                             module.cooldown = 1.0 / effective_fire_rate;
                         } else {
                             module.cooldown = 10.0;
                         }
                     }
                 }
             }
        }
    }
    
    state.projectiles.append(&mut new_projectiles);
}

fn find_nearest_enemy(enemies: &[Enemy], pos: Vec2, range: f32) -> Option<Vec2> {
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
    // 1. Move projectiles first
    for proj in &mut state.projectiles {
        proj.position += proj.velocity * dt;
        
        // Bounds check
        if proj.position.x < -100.0 || proj.position.x > screen_width() + 100.0 || 
           proj.position.y < -100.0 || proj.position.y > screen_height() + 100.0 {
            proj.active = false;
        }
    }
    
    // 2. Spatial Partitioning for Optimized Collision
    // Simple grid buckets: Screen width/height divided into 100px chunks
    // Key = (x/100, y/100) -> Vec of Enemy indices
    let bucket_size = 100.0;
    use std::collections::HashMap;
    let mut buckets: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    
    // Populate buckets with enemies
    for (i, enemy) in state.enemies.iter().enumerate() {
        if enemy.health <= 0.0 { continue; }
        
        let bx = (enemy.position.x / bucket_size).floor() as i32;
        let by = (enemy.position.y / bucket_size).floor() as i32;
        
        // Add to main bucket and neighbors if near edge (simplified: just add to one for now, 
        // effectively 100px cells. For perfect accuracy with large enemies, we'd check bounds, 
        // but center point is okay for this optimization level given small enemy counts)
        buckets.entry((bx, by)).or_default().push(i);
        
        // Overlap checks for edges (if enemy radius > bucket edge distance)
        // For simplicity in this review pass, we'll assume strict bucket ownership by center point
    }
    
    // Run Collisions
    for proj in state.projectiles.iter_mut() {
        if !proj.active { continue; }
        
        let bx = (proj.position.x / bucket_size).floor() as i32;
        let by = (proj.position.y / bucket_size).floor() as i32;
        
        // Check local bucket and neighbors (3x3 grid) to ensure no edge cases missed
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(enemy_indices) = buckets.get(&(bx + dx, by + dy)) {
                    for &idx in enemy_indices {
                         // Double check index validity just in case
                        if idx >= state.enemies.len() { continue; }
                        let enemy = &mut state.enemies[idx];
                        
                        if enemy.health <= 0.0 { continue; }
                        
                        let hit_radius = match enemy.enemy_type {
                            EnemyType::Boss => ENEMY_HIT_RADIUS_BOSS,
                            EnemyType::Nanoguard | EnemyType::SiegeConstruct => ENEMY_HIT_RADIUS_NANOGUARD,
                            _ => ENEMY_HIT_RADIUS_NANODRONE,
                        };
                        
                        if proj.position.distance(enemy.position) < hit_radius {
                            enemy.health -= proj.damage;
                            proj.active = false;
                            
                            if enemy.health <= 0.0 {
                                // Enemy killed
                                let scrap = match enemy.enemy_type {
                                    EnemyType::Nanodrone => 3,
                                    EnemyType::Nanoguard => 10,
                                    EnemyType::Leech => 5,
                                    EnemyType::SiegeConstruct => 25,
                                    EnemyType::Boss => 100,
                                };
                                state.resources.add_scrap(scrap);
                                state.resources.credits += scrap / 2;
                                
                                events.push_game(GameEvent::EnemyKilled { 
                                    x: enemy.position.x, 
                                    y: enemy.position.y, 
                                    scrap_dropped: scrap 
                                });
                            }
                            break; // Proj destroyed
                        }
                    }
                }
                if !proj.active { break; }
            }
            if !proj.active { break; }
        }
    }
    
    // Cleanup
    state.projectiles.retain(|p| p.active);
    state.enemies.retain(|e| e.health > 0.0);
}

fn enemy_attacks(state: &mut GameState, dt: f32, events: &mut EventBus) {
    let attack_range = ENEMY_ATTACK_RANGE;
    
    // Calculate shield reduction from all shield rooms
    let mut shield_reduction: f32 = 0.0;
    for room in &state.interior.rooms {
        if room.room_type == RoomType::Module(ModuleType::Defense) {
            if !room.repair_points.is_empty() {
                let repair_pct = room.repaired_count() as f32 / room.repair_points.len() as f32;
                shield_reduction += repair_pct * 0.5; // Each shield room can block up to 50%
            }
        }
    }
    // Cap at 80% damage reduction max
    shield_reduction = shield_reduction.min(0.8);
    
    for enemy in &mut state.enemies {
        if enemy.health <= 0.0 { continue; }
        
        let mut hit_something = false;
        
        if let Some(grid_pos) = Layout::screen_to_grid(enemy.position) {
            let (gx, gy) = grid_pos;
            
            'outer: for dx in -1i32..=1 {
                for dy in -1i32..=1 {
                    let nx = (gx as i32 + dx) as usize;
                    let ny = (gy as i32 + dy) as usize;
                    
                    if nx < GRID_WIDTH && ny < GRID_HEIGHT {
                        if state.ship.grid[nx][ny].is_some() {
                            let module_pos = Layout::grid_to_screen_center(nx, ny);
                            let dist = enemy.position.distance(module_pos);
                            
                            if dist < attack_range {
                                // Apply shield reduction to damage
                                let base_damage = enemy.damage * dt;
                                let damage = base_damage * (1.0 - shield_reduction);
                                state.ship_integrity -= damage;
                                
                                hit_something = true;
                                
                                // Only play sound (emit event) if not already attacking
                                if !enemy.attacking {
                                    enemy.attacking = true;
                                    events.push_game(GameEvent::ModuleDamaged { 
                                        x: nx, 
                                        y: ny, 
                                        damage 
                                    });
                                }
                                
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
        
        if !hit_something {
            enemy.attacking = false;
        }
    }
}
