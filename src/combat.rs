use macroquad::prelude::*;
use crate::state::GameState;
use crate::entities::Projectile;
use crate::ship::{ModuleType, ModuleState};
use crate::constants::*;

pub fn update_combat(state: &mut GameState, dt: f32) {
    // 1. Modules Fire (Towers)
    fire_towers(state, dt);
    
    // 2. Projectiles Move & Collide
    update_projectiles(state, dt);
    
    // 3. Enemies Attack (Simplified for now: If close to core/module)
    // Needs proper spatial query or simple loop
}

fn fire_towers(state: &mut GameState, dt: f32) {
    // Iterate grid to find active weapons
    // NOTE: In a real ECS or optimized engine, we'd have a list of ActiveWeapons to avoid iterating the whole grid.
    
    // We need to collect projectiles to add to avoid mutable borrow conflict if we tried to push to state.projectiles inside the loop
    let mut new_projectiles = Vec::new(); // (pos, target, damage, speed)
    
    // Read-only pass for potential targets
     // Optimization: Find nearest enemy once or per tower
     // For prototype: Exhaustive search is fine for < 100 enemies
    
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
             if let Some(module) = &match &state.ship.grid[x][y] { Some(m) => Some(m.clone()), None => None } {
                 if module.state == ModuleState::Active && module.module_type == ModuleType::Weapon {
                     // Check cooldown? struct Module doesn't have cooldown state yet.
                     // IMPORTANT: We need to add 'cooldown' to Module struct or use a separate map.
                     // For MVP: Random chance to fire per frame based on fire_rate? 
                     // Or better: Let's assume fire_rate 1.0 means 1 per sec.
                     // We need a timer.
                     
                     // Adding a simplified random fire for now until Module struct has dynamic fields for runtime state
                     let stats = state.module_registry.get(module.module_type);
                     let fire_chance = stats.fire_rate * dt; 
                     
                     if rand::gen_range(0.0, 1.0) < fire_chance {
                         // Find target
                         let tower_pos = vec2(
                             (screen_width() - (GRID_WIDTH as f32 * CELL_SIZE)) / 2.0 + x as f32 * CELL_SIZE,
                             (screen_height() - (GRID_HEIGHT as f32 * CELL_SIZE)) / 2.0 + y as f32 * CELL_SIZE
                         );
                         
                         if let Some(target) = find_nearest_enemy(&state.enemies, tower_pos, stats.range) {
                             new_projectiles.push(Projectile::new(tower_pos, target, 300.0, stats.damage));
                             // Play sound event?
                         }
                     }
                 }
             }
        }
    }
    
    state.projectiles.append(&mut new_projectiles);
}

fn find_nearest_enemy(enemies: &[crate::entities::Enemy], pos: Vec2, range: f32) -> Option<Vec2> {
    let mut nearest = None;
    let mut min_dist = range;
    
    for enemy in enemies {
        let d = pos.distance(enemy.position);
        if d < min_dist {
            min_dist = d;
            nearest = Some(enemy.position);
        }
    }
    
    nearest
}

fn update_projectiles(state: &mut GameState, dt: f32) {
    let mut _dead_projectiles = Vec::<usize>::new();
    let mut _dead_enemies = Vec::<usize>::new(); // Indicies
    
    // Move
    for proj in &mut state.projectiles {
        proj.position += proj.velocity * dt;
        
        // Bounds check
        if proj.position.x < -100.0 || proj.position.x > SCREEN_WIDTH + 100.0 || 
           proj.position.y < -100.0 || proj.position.y > SCREEN_HEIGHT + 100.0 {
            proj.active = false;
        }
    }
    
    // Collision (N*M, careful)
    for (_p_idx, proj) in state.projectiles.iter_mut().enumerate() {
        if !proj.active { continue; }
        
        for (_e_idx, enemy) in state.enemies.iter_mut().enumerate() {
             if enemy.health <= 0.0 { continue; }
             
             if proj.position.distance(enemy.position) < 20.0 { // Hit radius
                 enemy.health -= proj.damage;
                 proj.active = false; // Destroy bullet
                 
                 // dead_projectiles.push(p_idx); // Handled by active flag
                 
                 if enemy.health <= 0.0 {
                     // Enemy Dead
                     state.resources.add_scrap(5); // Loot
                 }
                 break; // Bullet hits one target
             }
        }
    }
    
    // Cleanup
    state.projectiles.retain(|p| p.active);
    state.enemies.retain(|e| e.health > 0.0);
}
