use crate::enemy::entities::{Enemy, EnemyType, Projectile};
use crate::ship::interior::RoomType;
use crate::ship::layout::Layout;
use crate::ship::ship::ModuleType;
use crate::simulation::constants::*;
use crate::simulation::events::{EventBus, GameEvent};
use crate::state::GameState;
use macroquad::prelude::*;

pub fn update_combat(state: &mut GameState, dt: f32, events: &mut EventBus) {
    // 1. Modules Fire (Towers)
    fire_towers(state, dt, events);

    // 2. Projectiles Move & Collide
    update_projectiles(state, dt, events);

    // 3. Enemies Attack Modules
    enemy_attacks(state, dt, events);

    // 4. Boss special abilities (EMP pulse)
    update_boss_abilities(state, dt, events);
}

fn fire_towers(state: &mut GameState, dt: f32, events: &mut EventBus) {
    let mut new_projectiles = Vec::new();

    // Check each weapon room for repair percentage
    for room in &state.interior.rooms {
        // Only process weapon rooms
        if room.room_type != RoomType::Module(ModuleType::Weapon) {
            continue;
        }
        if !room.powered {
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
        if let Some(Some(module)) = state.ship.grid.get_mut(gx).and_then(|row| row.get_mut(gy)) {
            // Decrease cooldown
            module.cooldown -= dt;

            // In-run upgrades boost turret output (damage + fire rate) per level.
            let level_mult =
                1.0 + (module.level.saturating_sub(1) as f32 * MODULE_UPGRADE_EFFECT_PER_LEVEL);
            let effective_damage = effective_damage * level_mult;
            let effective_fire_rate = effective_fire_rate * level_mult;

            // Debug prints every 60 frames (approx 1 sec) to reduce spam?
            // Or just print if cooldown <= 0?
            if module.cooldown <= 0.0 && state.frame_count.is_multiple_of(60) {
                // println!("Weapon Ready: Repaired {}/{} (Pct {:.2}), Rate {:.2}, EffRate {:.2}, Rng {:.0}",
                //    room.repaired_count(), room.repair_points.len(), repair_pct, base_fire_rate, effective_fire_rate, effective_range);
            }

            // Ready to fire?
            if module.cooldown <= 0.0 {
                let tower_pos = Layout::grid_to_screen_center(gx, gy);

                if let Some(target) = find_nearest_enemy(&state.enemies, tower_pos, effective_range)
                {
                    new_projectiles.push(Projectile::new(
                        tower_pos,
                        target,
                        400.0,
                        effective_damage,
                    ));
                    events.push_game(GameEvent::WeaponFired {
                        x: tower_pos.x,
                        y: tower_pos.y,
                    });

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

    state.projectiles.append(&mut new_projectiles);
}

fn find_nearest_enemy(enemies: &[Enemy], pos: Vec2, range: f32) -> Option<Vec2> {
    let mut nearest = None;
    let mut min_dist = range;

    for enemy in enemies {
        if enemy.health <= 0.0 {
            continue;
        }
        let d = pos.distance(enemy.position);
        if d < min_dist {
            min_dist = d;
            nearest = Some(enemy.position);
        }
    }

    nearest
}

fn update_projectiles(state: &mut GameState, dt: f32, events: &mut EventBus) {
    let kill_scrap_multiplier = state.kill_scrap_multiplier();

    // 1. Move projectiles first
    for proj in &mut state.projectiles {
        proj.position += proj.velocity * dt;

        // Bounds check
        if proj.position.x < -100.0
            || proj.position.x > screen_width() + 100.0
            || proj.position.y < -100.0
            || proj.position.y > screen_height() + 100.0
        {
            proj.active = false;
        }
    }

    // 2. Spatial Partitioning for Optimized Collision
    // Simple grid buckets: Screen width/height divided into 100px chunks
    // Key = (x/100, y/100) -> Vec of Enemy indices
    let bucket_size = 100.0;
    use std::collections::HashMap;
    let mut buckets: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    // Positions where a boss died this frame — each spawns fragment drones afterwards.
    let mut boss_deaths: Vec<Vec2> = Vec::new();

    // Populate buckets with enemies
    for (i, enemy) in state.enemies.iter().enumerate() {
        if enemy.health <= 0.0 {
            continue;
        }

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
        if !proj.active {
            continue;
        }

        let bx = (proj.position.x / bucket_size).floor() as i32;
        let by = (proj.position.y / bucket_size).floor() as i32;

        // Check local bucket and neighbors (3x3 grid) to ensure no edge cases missed
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(enemy_indices) = buckets.get(&(bx + dx, by + dy)) {
                    for &idx in enemy_indices {
                        // Double check index validity just in case
                        if idx >= state.enemies.len() {
                            continue;
                        }
                        let enemy = &mut state.enemies[idx];

                        if enemy.health <= 0.0 {
                            continue;
                        }

                        let hit_radius = match enemy.enemy_type {
                            EnemyType::Boss => ENEMY_HIT_RADIUS_BOSS,
                            EnemyType::Nanoguard | EnemyType::SiegeConstruct => {
                                ENEMY_HIT_RADIUS_NANOGUARD
                            }
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
                                    EnemyType::Burrower => 8,
                                    EnemyType::Boss => 100,
                                };
                                let scrap = (scrap as f32 * kill_scrap_multiplier) as i32;
                                state.resources.add_scrap(scrap);
                                state.resources.credits += scrap / 2;
                                state.enemies_destroyed += 1;

                                if enemy.enemy_type == EnemyType::Boss {
                                    boss_deaths.push(enemy.position);
                                }

                                events.push_game(GameEvent::EnemyKilled {
                                    x: enemy.position.x,
                                    y: enemy.position.y,
                                    scrap_dropped: scrap,
                                });
                            }
                            break; // Proj destroyed
                        }
                    }
                }
                if !proj.active {
                    break;
                }
            }
            if !proj.active {
                break;
            }
        }
    }

    // Cleanup
    state.projectiles.retain(|p| p.active);
    state.enemies.retain(|e| e.health > 0.0);

    // Boss split-on-death: each dead boss bursts into fragment drones (designed behaviour
    // that was previously loaded but never used), keeping pressure on at the climax.
    for pos in boss_deaths {
        for i in 0..BOSS_SPLIT_COUNT {
            let angle = (i as f32 / BOSS_SPLIT_COUNT as f32) * std::f32::consts::TAU;
            let offset = vec2(angle.cos(), angle.sin()) * 40.0;
            let id = state.enemies.len() as u64 + state.frame_count + i as u64;
            let mut fragment = Enemy::new(id, EnemyType::Nanodrone, pos + offset);
            fragment.health = BOSS_FRAGMENT_HP;
            fragment.max_health = BOSS_FRAGMENT_HP;
            state.enemies.push(fragment);
        }
    }
}

fn enemy_attacks(state: &mut GameState, dt: f32, events: &mut EventBus) {
    let attack_range = ENEMY_ATTACK_RANGE;

    // Calculate shield reduction from all shield rooms
    let mut shield_reduction: f32 = 0.0;
    for room in &state.interior.rooms {
        if room.room_type == RoomType::Module(ModuleType::Defense)
            && room.powered
            && !room.repair_points.is_empty()
        {
            let repair_pct = room.repaired_count() as f32 / room.repair_points.len() as f32;
            // In-run upgrades widen shield coverage per level.
            let level = room
                .module_index
                .and_then(|(gx, gy)| state.ship.grid[gx][gy].as_ref())
                .map(|m| m.level)
                .unwrap_or(1);
            let level_mult =
                1.0 + (level.saturating_sub(1) as f32 * MODULE_UPGRADE_EFFECT_PER_LEVEL);
            shield_reduction += repair_pct * 0.5 * level_mult; // Each shield room blocks up to 50%
        }
    }
    // Cap at 80% damage reduction max
    shield_reduction = shield_reduction.min(0.8);

    update_leech_drains(state, dt, events);

    // First pass: resolve each enemy's target module without mutating the ship, so we can
    // apply staged per-module damage afterwards (avoids aliasing state.enemies + state.ship).
    let mut hits: Vec<(usize, usize, f32)> = Vec::new();
    for enemy in &mut state.enemies {
        if enemy.health <= 0.0 {
            continue;
        }

        let mut target: Option<(usize, usize)> = None;
        if let Some((gx, gy)) = Layout::screen_to_grid(enemy.position) {
            'outer: for dx in -1i32..=1 {
                for dy in -1i32..=1 {
                    let nx = (gx as i32 + dx) as usize;
                    let ny = (gy as i32 + dy) as usize;
                    if nx < GRID_WIDTH && ny < GRID_HEIGHT && state.ship.grid[nx][ny].is_some() {
                        let module_pos = Layout::grid_to_screen_center(nx, ny);
                        if enemy.position.distance(module_pos) < attack_range {
                            target = Some((nx, ny));
                            break 'outer;
                        }
                    }
                }
            }
        }

        if let Some((nx, ny)) = target {
            let damage = enemy.damage * dt * (1.0 - shield_reduction);
            hits.push((nx, ny, damage));
            enemy.attacking = true;
        } else {
            enemy.attacking = false;
        }
    }

    // Second pass: chew the targeted modules (destroyed modules go offline / lose points).
    for (gx, gy, damage) in hits {
        state.apply_module_damage(gx, gy, damage, events);
    }
}

/// Boss EMP: every `BOSS_ABILITY_COOLDOWN` seconds the boss knocks every routed system
/// offline, forcing the player to re-route under fire at the moment of maximum tension.
fn update_boss_abilities(state: &mut GameState, dt: f32, events: &mut EventBus) {
    let mut emp = false;
    for enemy in &mut state.enemies {
        if enemy.enemy_type != EnemyType::Boss || enemy.health <= 0.0 {
            continue;
        }
        enemy.ability_timer += dt;
        if enemy.ability_timer >= BOSS_ABILITY_COOLDOWN {
            enemy.ability_timer = 0.0;
            emp = true;
        }
    }

    if emp {
        let mut knocked = false;
        for room in &mut state.interior.rooms {
            if room.powered && GameState::is_routeable_room(room) {
                room.powered = false;
                knocked = true;
            }
        }
        if knocked {
            state.update_power();
            events.push_game(GameEvent::EmpPulse);
        }
    }
}

fn update_leech_drains(state: &mut GameState, dt: f32, events: &mut EventBus) {
    let mut drained_targets = Vec::new();
    let mut hull_drain_count = 0;

    for enemy in &mut state.enemies {
        if enemy.enemy_type != EnemyType::Leech || enemy.health <= 0.0 {
            continue;
        }
        let Some(target) = enemy.attached_to else {
            continue;
        };
        enemy.ability_timer += dt;
        if enemy.ability_timer >= 2.0 {
            enemy.ability_timer = 0.0;
            drained_targets.push(target);
        }
    }

    for target in drained_targets {
        if let Some(room_idx) = state
            .interior
            .rooms
            .iter()
            .position(|room| room.module_index == Some(target) && room.powered)
        {
            let system = state.interior.rooms[room_idx].name().to_string();
            state.interior.rooms[room_idx].powered = false;
            state.update_power();
            events.push_game(GameEvent::PowerRouted {
                system,
                powered: false,
            });
        } else {
            hull_drain_count += 1;
        }
    }

    if hull_drain_count > 0 {
        state.ship_integrity -= ENEMY_LEECH_DAMAGE * hull_drain_count as f32 * dt;
    }
}
