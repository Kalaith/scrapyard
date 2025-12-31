//! Game action handlers
//! 
//! Contains methods for player-initiated actions: repairs, upgrades, module toggling.

use crate::state::game_state::GameState;
use crate::ship::ship::{ModuleType, ModuleState};
use crate::ship::interior::RoomType;
use crate::simulation::events::{EventBus, GameEvent};
use crate::simulation::constants::*;

impl GameState {
    pub fn attempt_repair(&mut self, x: usize, y: usize, events: &mut EventBus) -> bool {
        let repair_cost = if let Some(module) = &self.ship.grid[x][y] {
            if module.state != ModuleState::Destroyed { return false; }
            self.module_registry.get(module.module_type).base_cost
        } else { return false; };

        if self.resources.can_afford(repair_cost) {
            self.resources.deduct(repair_cost);
            if let Some(module) = &mut self.ship.grid[x][y] {
                module.state = ModuleState::Active;
                events.push_game(GameEvent::ModuleRepaired { x, y, cost: repair_cost });
                return true;
            }
        }
        false
    }

    pub fn attempt_upgrade(&mut self, x: usize, y: usize, events: &mut EventBus) -> bool {
        let upgrade_cost = if let Some(module) = &self.ship.grid[x][y] {
            if module.state == ModuleState::Destroyed || module.level >= MODULE_MAX_LEVEL { return false; }
            let base_cost = self.module_registry.get(module.module_type).base_cost;
            (base_cost as f32 * (module.level as f32 * 0.5 + 1.0)) as i32
        } else { return false; };

        if self.resources.can_afford(upgrade_cost) {
            self.resources.deduct(upgrade_cost);
            if let Some(module) = &mut self.ship.grid[x][y] {
                module.level += 1;
                module.max_health *= MODULE_UPGRADE_HP_MULTIPLIER;
                module.health = module.max_health;
                events.push_game(GameEvent::ModuleUpgraded { x, y, new_level: module.level });
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

    pub fn get_repair_cost(&self, room_idx: usize, _point_idx: usize) -> Option<(i32, i32)> {
        if room_idx >= self.interior.rooms.len() { return None; }
        let room = &self.interior.rooms[room_idx];
        let scrap_cost = REPAIR_SCRAP_COST;
        let power_cost = match room.room_type {
            RoomType::Module(ModuleType::Core) => 0,
            RoomType::Module(ModuleType::Weapon) => POWER_COST_WEAPON,
            RoomType::Module(ModuleType::Defense) => POWER_COST_DEFENSE,
            RoomType::Module(ModuleType::Utility) => POWER_COST_UTILITY,
            RoomType::Module(ModuleType::Engine) => POWER_COST_ENGINE,
            RoomType::Cockpit => POWER_COST_COCKPIT,
            RoomType::Medbay => POWER_COST_MEDBAY,
            _ => 0,
        };
        Some((scrap_cost, power_cost))
    }

    pub fn attempt_interior_repair(&mut self, room_idx: usize, point_idx: usize) -> bool {
         if room_idx >= self.interior.rooms.len() { return false; }
         let (scrap_cost, power_cost) = match self.get_repair_cost(room_idx, point_idx) {
             Some(c) => c,
             None => return false,
         };
         if self.interior.rooms[room_idx].repair_points.len() <= point_idx || 
            self.interior.rooms[room_idx].repair_points[point_idx].repaired {
             return false;
         }
         let is_reactor = matches!(self.interior.rooms[room_idx].room_type, RoomType::Module(ModuleType::Core));
         if self.resources.scrap < scrap_cost { return false; }
         if !is_reactor && (self.used_power + power_cost > self.total_power) { return false; }
         self.resources.deduct(scrap_cost);
         self.interior.rooms[room_idx].repair_points[point_idx].repaired = true;
         if self.interior.rooms[room_idx].is_fully_repaired() {
            if let Some((gx, gy)) = self.interior.rooms[room_idx].module_index {
                if let Some(module) = &mut self.ship.grid[gx][gy] {
                    module.state = ModuleState::Active;
                    module.health = module.max_health;
                }
            }
         }
         true
    }

    pub fn purchase_upgrade(&mut self, upgrade_id: &str) -> bool {
        let template = self.upgrade_templates.iter().find(|t| t.id == upgrade_id).cloned();
        if let Some(template) = template {
            let current_level = self.upgrades.get_level(upgrade_id);
            if current_level < template.max_level {
                let cost = self.upgrades.get_cost(&template);
                if self.resources.deduct_credits(cost) {
                    self.upgrades.levels.insert(upgrade_id.to_string(), current_level + 1);
                    if upgrade_id == "hull_reinforcement" {
                        self.ship_max_integrity += HULL_UPGRADE_BONUS;
                        self.ship_integrity += HULL_UPGRADE_BONUS;
                    }
                    return true;
                }
            }
        }
        false
    }
}
