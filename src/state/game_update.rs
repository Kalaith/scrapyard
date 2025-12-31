//! Game state update logic
//! 
//! Contains the main update loop and sub-system updates for power, resources, and engine.

use crate::state::game_state::{GameState, GamePhase, EngineState, ViewMode};
use crate::ship::ship::{ModuleType, ModuleState};
use crate::ship::interior::RoomType;
use crate::simulation::events::{EventBus, GameEvent};
use crate::simulation::constants::*;

impl GameState {
    pub fn update(&mut self, dt: f32, events: &mut EventBus) {
        match self.phase {
            GamePhase::Playing => {
                if !self.paused {
                    if self.view_mode == ViewMode::Interior {
                        self.player.update(dt, &self.interior);
                        self.player.update_nearby_module(&self.interior);
                    }
                    self.update_power();
                    self.update_resources();
                    self.update_engine(dt, events);
                    crate::enemy::ai::update_wave_logic(
                        self.total_power,
                        &self.engine_state,
                        &mut self.enemies,
                        &self.upgrades,
                        &mut self.wave_state,
                        self.frame_count,
                        dt,
                        events
                    );
                    crate::enemy::ai::update_enemies(self, dt);
                    crate::enemy::combat::update_combat(self, dt, events);
                    self.frame_count += 1;
                    self.time_survived += dt;

                    self.update_auto_repair(dt);
                    self.check_game_over(events);
                }
            }
            _ => {}
        }
    }

    fn update_auto_repair(&mut self, dt: f32) {
        let robotics_level = self.upgrades.get_level("auto_repairs");
        self.repair_timer += dt;
        if self.repair_timer >= NANO_REPAIR_INTERVAL_SECONDS {
            self.repair_timer = 0.0;
            let repair_amount = robotics_level as f32 * NANO_REPAIR_RATE_PER_LEVEL;
            for x in 0..GRID_WIDTH {
                for y in 0..GRID_HEIGHT {
                    if let Some(module) = &mut self.ship.grid[x][y] {
                        if module.state != ModuleState::Destroyed && module.health < module.max_health {
                            module.health = (module.health + repair_amount).min(module.max_health);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn update_power(&mut self) {
        self.total_power = 0;
        self.used_power = 0;
        for room in &self.interior.rooms {
            if room.repair_points.is_empty() { continue; }
            let repaired = room.repaired_count() as i32;
            if repaired > 0 {
                match room.room_type {
                    RoomType::Module(ModuleType::Core) => self.total_power += repaired * POWER_PER_CORE_POINT,
                    RoomType::Module(ModuleType::Weapon) => self.used_power += repaired * POWER_COST_WEAPON,
                    RoomType::Module(ModuleType::Defense) => self.used_power += repaired * POWER_COST_DEFENSE,
                    RoomType::Module(ModuleType::Utility) => self.used_power += repaired * POWER_COST_UTILITY,
                    RoomType::Module(ModuleType::Engine) => self.used_power += repaired * POWER_COST_ENGINE,
                    RoomType::Cockpit => self.used_power += repaired * POWER_COST_COCKPIT,
                    RoomType::Medbay => self.used_power += repaired * POWER_COST_MEDBAY,
                    _ => {}
                }
            }
        }
    }

    fn check_game_over(&mut self, events: &mut EventBus) {
        if self.ship_integrity <= 0.0 {
            self.ship_integrity = 0.0;
            self.phase = GamePhase::GameOver;
            events.push_game(GameEvent::CoreDestroyed);
        }
    }

    fn update_resources(&mut self) {
        // Power calculation is handled by update_power() - interior-based system only
    }

    fn update_engine(&mut self, dt: f32, events: &mut EventBus) {
        let mut engine_repair_pct = 0.0;
        for room in &self.interior.rooms {
            if let RoomType::Module(ModuleType::Engine) = room.room_type {
                 if !room.repair_points.is_empty() {
                    engine_repair_pct = room.repaired_count() as f32 / room.repair_points.len() as f32;
                 }
            }
        }
        // Engine Charging Logic with Hysteresis (Safety Shutdown)
        if engine_repair_pct >= ENGINE_MIN_REPAIR_PERCENT {
             match self.engine_state {
                 EngineState::Idle => {
                     // Only start charging if we have cooled down sufficently (Hysteresis)
                     // If we just hit Critical, we wait until Unstable (31) to restart.
                     // (If we were never critical, we start immediately since stress starts at 0)
                     if self.engine_stress <= STRESS_THRESHOLD_UNSTABLE {
                         self.engine_state = EngineState::Charging;
                     }
                 },
                 EngineState::Charging => {
                     // Check for Overheat (Critical Stress)
                     // If critical, trigger Emergency Shutdown (Force Idle)
                     if self.engine_stress >= STRESS_THRESHOLD_CRITICAL {
                         self.engine_state = EngineState::Idle;
                         // Note: Cascade damage logic below will still tick for this frame, 
                         // but next frame we are Idle and decaying.
                     }
                 },
                 _ => {}
             }
        } else {
             self.engine_state = EngineState::Idle;
        }

        // --- NANITE ALERT ---
        self.nanite_alert += dt * 0.1; // Base growth over time

        // --- ENGINE STRESS ---
        match self.engine_state {
            EngineState::Idle => {
                if self.engine_stress > 0.0 {
                    self.engine_stress = (self.engine_stress - STRESS_DECAY_IDLE * dt).max(0.0);
                }
            }
            EngineState::Charging => {
                let gain = 1.0 * (self.nanite_alert / NANITE_ALERT_BASE);
                self.engine_stress += gain * dt;
                
                // Original Charging Logic within Charging State
                self.escape_timer -= dt * engine_repair_pct;
                if self.escape_timer <= 0.0 {
                    self.engine_state = EngineState::Escaped;
                    self.phase = GamePhase::Victory;
                    let bonus_mult = 1.0 + (self.upgrades.get_level("credit_bonus") as f32 * CREDIT_BONUS_PER_LEVEL);
                    let total_credits = (BASE_ESCAPE_CREDITS as f32 * bonus_mult) as i32;
                    self.resources.add_credits(total_credits);
                    events.push_game(GameEvent::EscapeSuccess);
                }
            }
            _ => {}
        }
        
        // --- CASCADE FAILURE ---
        if self.engine_stress >= STRESS_THRESHOLD_CRITICAL {
             // 1. Rapid Internal Damage
             self.ship_integrity -= CASCADE_DAMAGE_PER_SEC * dt;
             
             // 2. Spawn Boss + Alert Spike
             let has_boss = self.enemies.iter().any(|e| e.enemy_type == crate::enemy::entities::EnemyType::Boss);
             if !has_boss {
                 crate::enemy::ai::spawn_boss(&mut self.enemies, events, self.frame_count);
                 self.nanite_alert += 8.0; 
             }
             
             // 3. Charge Reversal (Engine fighting itself)
             if self.engine_state == EngineState::Charging {
                self.escape_timer += dt * 5.0; // Reverse progress significantly
             }
        }
    }
}
