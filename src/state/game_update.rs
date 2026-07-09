//! Game state update logic
//!
//! Contains the main update loop and sub-system updates for power, resources, and engine.

use crate::ship::interior::RoomType;
use crate::ship::ship::{ModuleState, ModuleType};
use crate::simulation::constants::*;
use crate::simulation::events::{EventBus, GameEvent};
use crate::state::game_state::{EngineState, GamePhase, GameState, ViewMode};

impl GameState {
    pub fn update(&mut self, dt: f32, events: &mut EventBus) {
        if self.phase == GamePhase::Playing && !self.paused {
            if self.view_mode == ViewMode::Interior {
                self.player.update(dt, &self.interior);
                self.player.update_nearby_module(&self.interior);
            }
            self.update_power();
            self.update_resources();
            self.update_engine(dt, events);
            crate::enemy::ai::update_wave_logic(self, dt, events);
            crate::enemy::ai::update_enemies(self, dt);
            crate::enemy::combat::update_combat(self, dt, events);
            self.update_particles(dt);
            self.frame_count += 1;
            self.time_survived += dt;

            self.update_support_systems(dt);
            self.emit_threat_escalation(events);
            self.check_game_over(events);
        }
    }

    /// Advance and cull the visual particle pool (kill bursts, hit sparks, repair flashes).
    fn update_particles(&mut self, dt: f32) {
        let drag = (3.0 * dt).min(1.0);
        for p in &mut self.particles {
            p.position += p.velocity * dt;
            p.velocity -= p.velocity * drag;
            p.lifetime -= dt;
            if p.lifetime <= 0.0 {
                p.active = false;
            }
        }
        self.particles.retain(|p| p.active && p.lifetime > 0.0);
    }

    /// Fire a one-shot event whenever the signal climbs into a higher threat tier, so the
    /// HUD/audio can punctuate the moment the player just made the ship louder.
    fn emit_threat_escalation(&mut self, events: &mut EventBus) {
        let tier = self.signal_tier();
        if tier > self.last_signal_tier {
            events.push_game(GameEvent::ThreatEscalated { tier });
        }
        self.last_signal_tier = tier;
    }

    pub(crate) fn update_power(&mut self) {
        self.total_power = 0;
        self.used_power = 0;
        for room in &self.interior.rooms {
            if room.repair_points.is_empty() {
                continue;
            }
            let repaired = room.repaired_count() as i32;
            if repaired == 0 {
                continue;
            }
            match room.room_type {
                RoomType::Module(ModuleType::Core) => {
                    self.total_power += repaired * POWER_PER_CORE_POINT
                }
                _ => self.used_power += Self::active_room_power_draw(room),
            }
        }

        if self.used_power > self.total_power {
            self.shed_power_to_capacity();
            self.used_power = 0;
            for room in &self.interior.rooms {
                if room.repair_points.is_empty()
                    || room.repaired_count() == 0
                    || matches!(room.room_type, RoomType::Module(ModuleType::Core))
                {
                    continue;
                }
                self.used_power += Self::active_room_power_draw(room);
            }
        }

        self.threat_signature = self.calculate_threat_signature();
        self.sync_module_states();
    }

    fn shed_power_to_capacity(&mut self) {
        let mut indices = self.routeable_room_indices();
        indices.sort_by_key(|idx| shutdown_priority(self.interior.rooms[*idx].room_type));

        for idx in indices {
            if self.used_power <= self.total_power {
                break;
            }
            if self.interior.rooms[idx].powered {
                let draw = Self::room_power_need(&self.interior.rooms[idx]);
                self.interior.rooms[idx].powered = false;
                self.used_power = (self.used_power - draw).max(0);
            }
        }
    }

    fn calculate_threat_signature(&self) -> i32 {
        let mut signature = self.used_power;
        if self.recycler_online() {
            signature += RECYCLER_SIGNATURE_BONUS;
        }
        if self.engine_state == EngineState::Charging {
            signature += ENGINE_CHARGE_SIGNATURE_BONUS;
        }
        if !self.life_support_online() && self.used_power > 0 {
            signature += LIFE_SUPPORT_OFFLINE_SIGNATURE;
        }
        let repaired_value = self
            .interior
            .rooms
            .iter()
            .filter(|room| !room.name().is_empty() && room.is_fully_repaired())
            .count() as i32
            * PAYOUT_FULL_REPAIR_BONUS;
        let reserve_value = self.resources.scrap / PAYOUT_SCRAP_DIVISOR;
        let combat_value = self.enemies_destroyed * PAYOUT_ENEMY_DESTROYED_BONUS;
        signature +=
            ((repaired_value + reserve_value + combat_value) / HIGH_VALUE_SIGNATURE_DIVISOR).max(0);
        signature
    }

    fn sync_module_states(&mut self) {
        for room in &self.interior.rooms {
            let Some((gx, gy)) = room.module_index else {
                continue;
            };
            let Some(cell) = self.ship.grid.get_mut(gx).and_then(|row| row.get_mut(gy)) else {
                continue;
            };
            let Some(module) = cell else {
                continue;
            };

            if let RoomType::Module(module_type) = room.room_type {
                if module.module_type == ModuleType::Empty {
                    module.module_type = module_type;
                }
                let repaired = room.repaired_count();
                module.state = if repaired == 0 {
                    ModuleState::Destroyed
                } else if room.powered || module_type == ModuleType::Core {
                    ModuleState::Active
                } else {
                    ModuleState::Offline
                };
                if repaired > 0 && module.health <= 0.0 {
                    module.health = module.max_health;
                }
            }
        }
    }

    fn check_game_over(&mut self, events: &mut EventBus) {
        if self.ship_integrity <= 0.0 {
            self.ship_integrity = 0.0;
            self.last_payout = Some(self.calculate_failure_payout());
            self.phase = GamePhase::GameOver;
            events.push_game(GameEvent::CoreDestroyed);
        }
    }

    fn update_resources(&mut self) {
        // Power calculation is handled by update_power() - interior-based system only
    }

    fn update_support_systems(&mut self, dt: f32) {
        if self.life_support_online() || self.used_power == 0 {
            self.life_support_timer = 0.0;
        } else {
            self.life_support_timer += dt;
            if self.life_support_timer > LIFE_SUPPORT_GRACE_SECONDS {
                self.ship_integrity -= LIFE_SUPPORT_OFFLINE_DAMAGE_PER_SEC * dt;
                self.engine_stress += 0.25 * dt;
            }
        }

        if self.medbay_online() {
            self.ship_integrity = (self.ship_integrity + MEDBAY_HULL_REPAIR_PER_SEC * dt)
                .min(self.ship_max_integrity);
            self.engine_stress = (self.engine_stress - MEDBAY_STRESS_RELIEF_PER_SEC * dt).max(0.0);
        }
    }

    fn update_engine(&mut self, dt: f32, events: &mut EventBus) {
        let mut engine_repair_pct = 0.0;
        for room in &self.interior.rooms {
            if let RoomType::Module(ModuleType::Engine) = room.room_type {
                if !room.repair_points.is_empty() {
                    engine_repair_pct =
                        room.repaired_count() as f32 / room.repair_points.len() as f32;
                }
            }
        }
        // Record the first moment escape became available — death forensics use this to
        // prove that leaving was a choice the player declined.
        if engine_repair_pct >= ENGINE_MIN_REPAIR_PERCENT
            && self.engine_powered()
            && self.engine_ready_at.is_none()
        {
            self.engine_ready_at = Some(self.time_survived);
        }

        // Engine Charging Logic with Hysteresis (Safety Shutdown)
        if engine_repair_pct >= ENGINE_MIN_REPAIR_PERCENT && self.engine_powered() {
            match self.engine_state {
                EngineState::Idle => {
                    // Only start charging if we have cooled down sufficently (Hysteresis)
                    // If we just hit Critical, we wait until Unstable (31) to restart.
                    // (If we were never critical, we start immediately since stress starts at 0)
                    if self.engine_stress <= STRESS_THRESHOLD_UNSTABLE {
                        self.engine_state = EngineState::Charging;
                    }
                }
                EngineState::Charging
                    // Check for Overheat (Critical Stress)
                    // If critical, trigger Emergency Shutdown (Force Idle)
                    if self.engine_stress >= STRESS_THRESHOLD_CRITICAL => {
                        self.engine_state = EngineState::Idle;
                        // Note: Cascade damage logic below will still tick for this frame,
                        // but next frame we are Idle and decaying.
                    }
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
                let gain = STRESS_CHARGE_GAIN_BASE * (self.nanite_alert / NANITE_ALERT_BASE);
                self.engine_stress += gain * dt;

                // Original Charging Logic within Charging State
                self.escape_timer -= dt * engine_repair_pct;
                if self.escape_timer <= 0.0 {
                    self.engine_state = EngineState::Escaped;
                    self.phase = GamePhase::Victory;
                    let payout = self.calculate_payout();
                    self.profile
                        .record_victory(payout.total, self.time_survived);
                    if let Err(error) = self.profile.save() {
                        eprintln!("Failed to save profile after victory: {error}");
                    }
                    self.resources.credits = self.profile.banked_credits;
                    self.last_payout = Some(payout);
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
            let has_boss = self
                .enemies
                .iter()
                .any(|e| e.enemy_type == crate::enemy::entities::EnemyType::Boss);
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

fn shutdown_priority(room_type: RoomType) -> i32 {
    match room_type {
        RoomType::Module(ModuleType::Utility) => 0,
        RoomType::Medbay => 1,
        RoomType::Cockpit => 2,
        RoomType::Module(ModuleType::Defense) => 3,
        RoomType::Module(ModuleType::Weapon) => 4,
        RoomType::Module(ModuleType::Engine) => 5,
        _ => 6,
    }
}
