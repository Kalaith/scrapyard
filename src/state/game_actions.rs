//! Game action handlers
//!
//! Contains methods for player-initiated actions: repairs, upgrades, module toggling.

use crate::ship::interior::{Room, RoomType};
use crate::ship::ship::{ModuleState, ModuleType};
use crate::simulation::constants::*;
use crate::simulation::events::{EventBus, GameEvent};
use crate::simulation::gameplay::ModuleRegistry;
use crate::state::game_state::{GameState, PayoutBreakdown};

impl GameState {
    pub fn attempt_repair(&mut self, x: usize, y: usize, events: &mut EventBus) -> bool {
        let repair_cost = if let Some(module) = &self.ship.grid[x][y] {
            if module.state != ModuleState::Destroyed {
                return false;
            }
            self.module_registry.get(module.module_type).base_cost
        } else {
            return false;
        };

        if self.resources.can_afford(repair_cost) {
            self.resources.deduct(repair_cost);
            if let Some(module) = &mut self.ship.grid[x][y] {
                module.state = ModuleState::Active;
                events.push_game(GameEvent::ModuleRepaired {
                    x,
                    y,
                    cost: repair_cost,
                });
                return true;
            }
        }
        false
    }

    /// Apply combat damage to the module at grid `(gx, gy)`.
    ///
    /// The core has no repair-point staging — it *is* the hull, so its damage bleeds
    /// straight into `ship_integrity`. Every other module soaks damage until it crosses
    /// zero, at which point one repair point is knocked out: the module drops offline and
    /// stays down until re-repaired, and if that was its last point it is Destroyed.
    /// This is what makes defense spatial — where an enemy attacks now matters.
    pub fn apply_module_damage(
        &mut self,
        gx: usize,
        gy: usize,
        damage: f32,
        events: &mut EventBus,
    ) {
        let module_type = match self.ship.grid.get(gx).and_then(|row| row.get(gy)) {
            Some(Some(m)) => m.module_type,
            _ => {
                self.ship_integrity -= damage;
                return;
            }
        };

        if module_type == ModuleType::Core {
            self.ship_integrity -= damage;
            return;
        }

        let Some(room_idx) = self
            .interior
            .rooms
            .iter()
            .position(|r| r.module_index == Some((gx, gy)))
        else {
            self.ship_integrity -= damage;
            return;
        };

        {
            let Some(module) = self.ship.grid[gx][gy].as_mut() else {
                return;
            };
            if module.state == ModuleState::Destroyed {
                // Already gutted — a siege parked on a dead module still gnaws the hull.
                self.ship_integrity -= damage * 0.5;
                return;
            }
            module.health -= damage;
            if module.health > 0.0 {
                return;
            }
        }

        // Health crossed zero: knock out one repair point.
        if let Some(point) = self.interior.rooms[room_idx]
            .repair_points
            .iter_mut()
            .rev()
            .find(|p| p.repaired)
        {
            point.repaired = false;
        }

        if self.interior.rooms[room_idx].repaired_count() == 0 {
            self.interior.rooms[room_idx].powered = false;
            if let Some(module) = self.ship.grid[gx][gy].as_mut() {
                module.state = ModuleState::Destroyed;
                module.health = 0.0;
            }
            events.push_game(GameEvent::ModuleDestroyed { x: gx, y: gy });
        } else {
            if let Some(module) = self.ship.grid[gx][gy].as_mut() {
                module.health = MODULE_POINT_HEALTH;
            }
            events.push_game(GameEvent::ModuleDamaged {
                x: gx,
                y: gy,
                damage: MODULE_POINT_HEALTH,
            });
        }
        self.update_power();
    }

    /// Threat tier the current signature sits in (0 = grace, 4 = maximum).
    pub fn signal_tier(&self) -> u8 {
        let s = self.threat_signature;
        if s >= WAVE_T3_POWER {
            4
        } else if s >= WAVE_T2_POWER {
            3
        } else if s >= WAVE_T1_POWER {
            2
        } else if s >= WAVE_GRACE_POWER {
            1
        } else {
            0
        }
    }

    pub fn attempt_upgrade(&mut self, x: usize, y: usize, events: &mut EventBus) -> bool {
        let upgrade_cost = if let Some(module) = &self.ship.grid[x][y] {
            if module.state == ModuleState::Destroyed || module.level >= MODULE_MAX_LEVEL {
                return false;
            }
            let base_cost = self.module_registry.get(module.module_type).base_cost;
            (base_cost as f32 * (module.level as f32 * 0.5 + 1.0)) as i32
        } else {
            return false;
        };

        if self.resources.can_afford(upgrade_cost) {
            self.resources.deduct(upgrade_cost);
            if let Some(module) = &mut self.ship.grid[x][y] {
                module.level += 1;
                module.max_health *= MODULE_UPGRADE_HP_MULTIPLIER;
                module.health = module.max_health;
                events.push_game(GameEvent::ModuleUpgraded {
                    x,
                    y,
                    new_level: module.level,
                });
                return true;
            }
        }
        false
    }

    pub fn toggle_module(&mut self, x: usize, y: usize) {
        if let Some(module) = &mut self.ship.grid[x][y] {
            match module.state {
                ModuleState::Active => module.state = ModuleState::Offline,
                ModuleState::Offline => module.state = ModuleState::Active,
                ModuleState::Destroyed => {}
            }
        }
    }

    pub fn routeable_room_indices(&self) -> Vec<usize> {
        self.interior
            .rooms
            .iter()
            .enumerate()
            .filter_map(|(idx, room)| {
                if Self::is_routeable_room(room) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn is_routeable_room(room: &Room) -> bool {
        !room.repair_points.is_empty()
            && !matches!(
                room.room_type,
                RoomType::Module(ModuleType::Core) | RoomType::Storage | RoomType::Empty
            )
    }

    pub fn room_power_need(room: &Room, registry: &ModuleRegistry) -> i32 {
        let repaired = room.repaired_count() as i32;
        if repaired == 0 {
            return 0;
        }

        // Weapon/defense/utility/engine draws are data-driven per point (modules.json);
        // cockpit + medbay aren't grid modules so they keep constant costs.
        let cost = match room.room_type {
            RoomType::Module(ModuleType::Core) | RoomType::Module(ModuleType::Empty) => 0,
            RoomType::Module(mt) => registry.power_cost(mt),
            RoomType::Cockpit => POWER_COST_COCKPIT,
            RoomType::Medbay => POWER_COST_MEDBAY,
            _ => 0,
        };
        repaired * cost
    }

    pub fn room_power_need_by_idx(&self, room_idx: usize) -> i32 {
        self.interior
            .rooms
            .get(room_idx)
            .map(|room| Self::room_power_need(room, &self.module_registry))
            .unwrap_or(0)
    }

    pub fn active_room_power_draw(room: &Room, registry: &ModuleRegistry) -> i32 {
        if room.powered {
            Self::room_power_need(room, registry)
        } else {
            0
        }
    }

    pub fn can_power_room(&self, room_idx: usize) -> bool {
        let Some(room) = self.interior.rooms.get(room_idx) else {
            return false;
        };
        if !Self::is_routeable_room(room) || room.powered || room.repaired_count() == 0 {
            return false;
        }
        self.used_power + Self::room_power_need(room, &self.module_registry) <= self.total_power
    }

    pub fn toggle_room_power(&mut self, room_idx: usize, events: &mut EventBus) -> bool {
        if room_idx >= self.interior.rooms.len() {
            return false;
        }
        if !Self::is_routeable_room(&self.interior.rooms[room_idx]) {
            return false;
        }
        if self.interior.rooms[room_idx].repaired_count() == 0 {
            return false;
        }

        if self.interior.rooms[room_idx].powered {
            let system = self.interior.rooms[room_idx].name().to_string();
            self.interior.rooms[room_idx].powered = false;
            self.update_power();
            events.push_game(GameEvent::PowerRouted {
                system,
                powered: false,
            });
            return true;
        }

        self.update_power();
        let needed = self.room_power_need_by_idx(room_idx);
        if self.used_power + needed > self.total_power {
            return false;
        }

        let system = self.interior.rooms[room_idx].name().to_string();
        self.interior.rooms[room_idx].powered = true;
        self.update_power();
        events.push_game(GameEvent::PowerRouted {
            system,
            powered: true,
        });
        true
    }

    pub fn toggle_route_slot(&mut self, route_slot: usize, events: &mut EventBus) -> bool {
        let indices = self.routeable_room_indices();
        let Some(room_idx) = indices.get(route_slot).copied() else {
            return false;
        };
        self.toggle_room_power(room_idx, events)
    }

    pub fn is_room_type_powered(&self, room_type: RoomType) -> bool {
        self.interior
            .rooms
            .iter()
            .any(|room| room.room_type == room_type && room.powered && room.repaired_count() > 0)
    }

    pub fn life_support_online(&self) -> bool {
        self.is_room_type_powered(RoomType::Cockpit)
    }

    pub fn recycler_online(&self) -> bool {
        self.is_room_type_powered(RoomType::Module(ModuleType::Utility))
    }

    pub fn medbay_online(&self) -> bool {
        self.is_room_type_powered(RoomType::Medbay)
    }

    pub fn engine_powered(&self) -> bool {
        self.is_room_type_powered(RoomType::Module(ModuleType::Engine))
    }

    pub fn weapon_powered(&self) -> bool {
        self.is_room_type_powered(RoomType::Module(ModuleType::Weapon))
    }

    pub fn defense_powered(&self) -> bool {
        self.is_room_type_powered(RoomType::Module(ModuleType::Defense))
    }

    pub fn kill_scrap_multiplier(&self) -> f32 {
        if self.recycler_online() {
            1.0 + RECYCLER_KILL_BONUS
        } else {
            1.0
        }
    }

    pub fn sync_upgrades_from_profile(&mut self) {
        self.upgrades.levels = self.profile.permanent_upgrades.clone();
        self.resources.credits = self.profile.banked_credits;
    }

    pub fn apply_meta_progression_to_run(&mut self) {
        self.sync_upgrades_from_profile();

        let scrap_level = self.upgrades.get_level("scrap_efficiency") as i32;
        let storage_level = self.upgrades.get_level("expanded_hold") as i32;
        self.resources.max_scrap += storage_level * META_STORAGE_PER_LEVEL;
        self.resources.scrap = (self.resources.scrap + scrap_level * META_STARTING_SCRAP_PER_LEVEL)
            .min(self.resources.max_scrap);

        let hull_level = self.upgrades.get_level("hull_reinforcement") as f32;
        self.ship_max_integrity = SHIP_BASE_INTEGRITY + hull_level * HULL_UPGRADE_BONUS;
        self.ship_integrity = self.ship_max_integrity;

        let core_points =
            self.upgrades.get_level("reactor_bootstrap") as usize * META_REACTOR_POINTS_PER_LEVEL;
        self.restore_points_by_type(RoomType::Module(ModuleType::Core), core_points);

        let restored_points =
            self.upgrades.get_level("restoration_permit") as usize * META_RESTORED_POINTS_PER_LEVEL;
        self.restore_priority_system_points(restored_points);
        self.update_power();
    }

    fn restore_points_by_type(&mut self, room_type: RoomType, mut points: usize) {
        for room in &mut self.interior.rooms {
            if points == 0 {
                break;
            }
            if room.room_type != room_type {
                continue;
            }
            for point in &mut room.repair_points {
                if points == 0 {
                    break;
                }
                if !point.repaired {
                    point.repaired = true;
                    points -= 1;
                }
            }
            if room.room_type == RoomType::Module(ModuleType::Core) && room.repaired_count() > 0 {
                room.powered = true;
            }
        }
    }

    fn restore_priority_system_points(&mut self, mut points: usize) {
        let priority = [
            RoomType::Cockpit,
            RoomType::Module(ModuleType::Weapon),
            RoomType::Module(ModuleType::Defense),
            RoomType::Module(ModuleType::Utility),
            RoomType::Medbay,
            RoomType::Module(ModuleType::Engine),
        ];

        for room_type in priority {
            if points == 0 {
                break;
            }
            for room in &mut self.interior.rooms {
                if points == 0 {
                    break;
                }
                if room.room_type != room_type {
                    continue;
                }
                for point in &mut room.repair_points {
                    if points == 0 {
                        break;
                    }
                    if !point.repaired {
                        point.repaired = true;
                        points -= 1;
                    }
                }
            }
        }
    }

    pub fn calculate_payout(&self) -> PayoutBreakdown {
        let repaired_systems = self
            .interior
            .rooms
            .iter()
            .filter(|room| !room.name().is_empty() && room.is_fully_repaired())
            .count() as i32;
        let powered_systems = self
            .interior
            .rooms
            .iter()
            .filter(|room| room.powered && Self::is_routeable_room(room))
            .count() as i32;
        let hull_pct = if self.ship_max_integrity > 0.0 {
            (self.ship_integrity / self.ship_max_integrity).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let base = BASE_ESCAPE_CREDITS;
        let repaired_bonus = repaired_systems * PAYOUT_FULL_REPAIR_BONUS;
        let powered_bonus = powered_systems * PAYOUT_POWERED_SYSTEM_BONUS;
        let hull_bonus = (hull_pct * PAYOUT_HULL_MAX_BONUS as f32) as i32;
        let scrap_bonus = self.resources.scrap / PAYOUT_SCRAP_DIVISOR;
        let combat_bonus = self.enemies_destroyed * PAYOUT_ENEMY_DESTROYED_BONUS;
        let contract_bonus = self.upgrades.get_level("targeting_tier") as i32 * 100;
        // Uncapped: if danger scales with unbanked value, the reward must keep pace.
        let risk_bonus = (self.threat_signature.max(0) * PAYOUT_SIGNATURE_BONUS) + contract_bonus;
        let stress_penalty =
            (self.engine_stress as i32).max(0) * PAYOUT_ENGINE_STRESS_PENALTY_PER_POINT;
        let low_hull_penalty = if hull_pct < 0.35 { 120 } else { 0 };
        let penalties = stress_penalty + low_hull_penalty;
        let subtotal = base
            + repaired_bonus
            + powered_bonus
            + hull_bonus
            + scrap_bonus
            + combat_bonus
            + risk_bonus
            - penalties;
        let bonus_mult =
            1.0 + (self.upgrades.get_level("credit_bonus") as f32 * CREDIT_BONUS_PER_LEVEL);
        let contract_mult = self
            .active_contract
            .as_ref()
            .map_or(1.0, |c| c.payout_multiplier);
        let total = ((subtotal.max(0) as f32) * bonus_mult * contract_mult) as i32;

        PayoutBreakdown {
            base,
            repaired_bonus,
            powered_bonus,
            hull_bonus,
            scrap_bonus,
            combat_bonus,
            risk_bonus,
            penalties,
            total,
        }
    }

    pub fn calculate_failure_payout(&self) -> PayoutBreakdown {
        let mut payout = self.calculate_payout();
        payout.total = (payout.total as f32 * 0.2) as i32;
        payout.penalties += payout.base;
        payout
    }

    pub fn get_repair_cost(&self, room_idx: usize, _point_idx: usize) -> Option<(i32, i32)> {
        if room_idx >= self.interior.rooms.len() {
            return None;
        }
        let room = &self.interior.rooms[room_idx];
        let support_discount = if self.life_support_online()
            && !matches!(
                room.room_type,
                RoomType::Cockpit | RoomType::Module(ModuleType::Core)
            ) {
            LIFE_SUPPORT_REPAIR_DISCOUNT
        } else {
            0
        };
        let expertise_discount =
            self.upgrades.get_level("repair_expertise") as i32 * META_REPAIR_DISCOUNT_PER_LEVEL;
        let discount = support_discount + expertise_discount;
        let scrap_cost = (REPAIR_SCRAP_COST - discount).max(1);
        Some((
            scrap_cost,
            Self::room_power_need(room, &self.module_registry),
        ))
    }

    pub fn attempt_interior_repair(
        &mut self,
        room_idx: usize,
        point_idx: usize,
        events: &mut EventBus,
    ) -> bool {
        if room_idx >= self.interior.rooms.len() {
            return false;
        }
        let (scrap_cost, _) = match self.get_repair_cost(room_idx, point_idx) {
            Some(c) => c,
            None => return false,
        };
        if self.interior.rooms[room_idx].repair_points.len() <= point_idx
            || self.interior.rooms[room_idx].repair_points[point_idx].repaired
        {
            return false;
        }
        if self.resources.scrap < scrap_cost {
            return false;
        }
        self.resources.deduct(scrap_cost);
        self.interior.rooms[room_idx].repair_points[point_idx].repaired = true;

        if matches!(
            self.interior.rooms[room_idx].room_type,
            RoomType::Module(ModuleType::Core)
        ) {
            self.interior.rooms[room_idx].powered = true;
        }

        // Engine Stress Logic
        if matches!(
            self.interior.rooms[room_idx].room_type,
            RoomType::Module(ModuleType::Engine)
        ) {
            self.engine_stress += STRESS_GAIN_PER_REPAIR;
        }

        events.push_game(GameEvent::ModuleRepaired {
            x: 0,
            y: 0,
            cost: scrap_cost,
        }); // Coords meaningless for interior points

        if self.interior.rooms[room_idx].is_fully_repaired() {
            if let Some((gx, gy)) = self.interior.rooms[room_idx].module_index {
                if let Some(module) = &mut self.ship.grid[gx][gy] {
                    module.state = if self.interior.rooms[room_idx].powered {
                        ModuleState::Active
                    } else {
                        ModuleState::Offline
                    };
                    module.health = module.max_health;
                }
            }
        }
        self.update_power();
        true
    }

    /// Scrap cost to raise the module in `room_idx` one level, if it's an upgradeable
    /// (weapon/shield), fully-repaired system that isn't already maxed.
    pub fn interior_upgrade_cost(&self, room_idx: usize) -> Option<i32> {
        let room = self.interior.rooms.get(room_idx)?;
        if !room.is_fully_repaired()
            || !matches!(
                room.room_type,
                RoomType::Module(ModuleType::Weapon) | RoomType::Module(ModuleType::Defense)
            )
        {
            return None;
        }
        let (gx, gy) = room.module_index?;
        let module = self.ship.grid[gx][gy].as_ref()?;
        if module.level >= MODULE_MAX_LEVEL {
            return None;
        }
        Some(MODULE_UPGRADE_SCRAP_BASE * module.level as i32)
    }

    /// In-run upgrade: spend scrap to raise a finished weapon/shield a level (+damage/+shield).
    /// This is the late-run scrap sink the economy was missing.
    pub fn attempt_interior_upgrade(&mut self, room_idx: usize, events: &mut EventBus) -> bool {
        let Some(cost) = self.interior_upgrade_cost(room_idx) else {
            return false;
        };
        if self.resources.scrap < cost {
            return false;
        }
        let Some((gx, gy)) = self.interior.rooms[room_idx].module_index else {
            return false;
        };
        self.resources.deduct(cost);
        if let Some(module) = self.ship.grid[gx][gy].as_mut() {
            module.level += 1;
            module.max_health *= MODULE_UPGRADE_HP_MULTIPLIER;
            module.health = module.max_health;
            let new_level = module.level;
            events.push_game(GameEvent::ModuleUpgraded {
                x: gx,
                y: gy,
                new_level,
            });
        }
        true
    }

    pub fn advance_tutorial_after_repair(&mut self, room_idx: usize) {
        let Some(step) = self
            .tutorial_state
            .current_step(&self.tutorial_config)
            .map(|step| step.id.clone())
        else {
            return;
        };
        match step.as_str() {
            "repair_reactor" => {
                if self.interior.rooms[room_idx].room_type == RoomType::Module(ModuleType::Core)
                    && self.interior.rooms[room_idx].repaired_count() > 0
                {
                    self.tutorial_state.advance(&self.tutorial_config);
                }
            }
            "repair_engine"
                if self.interior.rooms[room_idx].room_type
                    == RoomType::Module(ModuleType::Engine)
                    && self.interior.rooms[room_idx].repaired_count() > 0 =>
            {
                self.tutorial_state.advance(&self.tutorial_config);
            }
            _ => {}
        }
    }

    pub fn advance_tutorial_after_power_route(&mut self) {
        let Some(step) = self
            .tutorial_state
            .current_step(&self.tutorial_config)
            .map(|step| step.id.clone())
        else {
            return;
        };
        if step == "choose_power"
            && (self.weapon_powered() || self.defense_powered() || self.recycler_online())
        {
            self.tutorial_state.advance(&self.tutorial_config);
        }
    }

    pub fn purchase_upgrade(&mut self, upgrade_id: &str) -> bool {
        if self.phase == crate::state::game_state::GamePhase::Victory {
            self.phase = crate::state::game_state::GamePhase::InterRound;
            self.sync_upgrades_from_profile();
            return true;
        }

        let template = self
            .upgrade_templates
            .iter()
            .find(|t| t.id == upgrade_id)
            .cloned();
        if let Some(template) = template {
            if self.profile.purchase_upgrade(&template) {
                self.sync_upgrades_from_profile();
                if let Err(error) = self.profile.save() {
                    eprintln!("Failed to save profile after upgrade: {error}");
                }
                return true;
            }
        }
        false
    }
}
