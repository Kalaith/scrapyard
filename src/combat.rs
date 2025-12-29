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
    
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            if let Some(module) = &state.ship.grid[x][y].clone() {
                if module.state == ModuleState::Active && module.module_type == ModuleType::Weapon {
                    let stats = state.module_registry.get(module.module_type);
                    
                    // Scale fire rate with module level
                    let effective_fire_rate = stats.fire_rate * (1.0 + (module.level - 1) as f32 * 0.2);
                    let fire_chance = effective_fire_rate * dt;
                    
                    if rand::gen_range(0.0, 1.0) < fire_chance {
                        let tower_pos = grid_to_screen(x, y);
                        
                        // Scale damage with level
                        let effective_damage = stats.damage * (1.0 + (module.level - 1) as f32 * 0.3);
                        let effective_range = stats.range * (1.0 + (module.level - 1) as f32 * 0.1);
                        
                        if let Some(target) = find_nearest_enemy(&state.enemies, tower_pos, effective_range) {
                            new_projectiles.push(Projectile::new(tower_pos, target, 400.0, effective_damage));
                            events.push_game(GameEvent::WeaponFired { x: tower_pos.x, y: tower_pos.y });
                        }
                    }
                }
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
    let attack_range = 30.0; // Distance to start attacking
    
    for enemy in &state.enemies {
        if enemy.health <= 0.0 { continue; }
        
        // Find if enemy is near any module
        if let Some(grid_pos) = screen_to_grid(enemy.position) {
            let (gx, gy) = grid_pos;
            
            // Check the module at this position and adjacent
            for dx in -1i32..=1 {
                for dy in -1i32..=1 {
                    let nx = (gx as i32 + dx) as usize;
                    let ny = (gy as i32 + dy) as usize;
                    
                    if nx < GRID_WIDTH && ny < GRID_HEIGHT {
                        if let Some(module) = &mut state.ship.grid[nx][ny] {
                            if module.state != ModuleState::Destroyed {
                                let module_pos = grid_to_screen(nx, ny);
                                let dist = enemy.position.distance(module_pos);
                                
                                if dist < attack_range {
                                    // Apply damage
                                    let damage = enemy.damage * dt;
                                    module.health -= damage;
                                    
                                    events.push_game(GameEvent::ModuleDamaged { 
                                        x: nx, 
                                        y: ny, 
                                        damage 
                                    });
                                    
                                    // Check for destruction
                                    if module.health <= 0.0 {
                                        module.health = 0.0;
                                        module.state = ModuleState::Destroyed;
                                        
                                        events.push_game(GameEvent::ModuleDestroyed { x: nx, y: ny });
                                        
                                        // Check if core was destroyed
                                        if module.module_type == ModuleType::Core {
                                            events.push_game(GameEvent::CoreDestroyed);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
