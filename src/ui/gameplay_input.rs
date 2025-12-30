use macroquad::prelude::*;
use crate::state::{GameState, ViewMode};
use crate::simulation::events::{EventBus, UIEvent};
use crate::simulation::constants::*;
use crate::ship::interior::Room;
use crate::ship::ship::{ModuleState, ModuleType};
use crate::ui::input_manager::{InputManager, InputState};

impl InputManager {
    pub fn handle_gameplay_input(&mut self, input: &InputState, state: &mut GameState, events: &mut EventBus) {
        // Tab toggles view mode
        if input.tab_pressed {
            state.view_mode = match state.view_mode {
                ViewMode::Interior => ViewMode::Exterior,
                ViewMode::Exterior => ViewMode::Interior,
            };
        }

        // Pause toggle
        if input.pause_pressed {
            let event = if state.paused { UIEvent::Resume } else { UIEvent::Pause };
            events.push_ui(event);
        }

        // Escape to return to menu
        if input.escape_pressed {
            events.push_ui(UIEvent::ReturnToMenu);
        }

        // View-specific input
        if state.view_mode == ViewMode::Interior {
            self.handle_interior_input(input, state, events);
        }
    }

    fn handle_interior_input(&mut self, input: &InputState, state: &mut GameState, events: &mut EventBus) {
        self.handle_scrap_gathering(state, events);
        
        if input.interact_pressed {
            self.handle_interact(state, events);
        }
    }

    fn handle_scrap_gathering(&self, state: &mut GameState, events: &mut EventBus) {
        // Cancel gathering if not holding E or moving
        if !is_key_down(KeyCode::E) || state.player.velocity.length() >= 0.1 {
            state.gathering_target = None;
            state.gathering_timer = 0.0;
            return;
        }

        // Find nearest scrap pile if not already targeting one
        if state.gathering_target.is_none() {
            state.gathering_target = self.find_nearest_scrap_pile(state);
        }

        // Process gathering progress
        let Some(target_idx) = state.gathering_target else { return };
        if target_idx >= state.scrap_piles.len() { return };

        state.gathering_timer += get_frame_time();
        if state.gathering_timer < GATHERING_TIME_SECONDS { return };

        // Complete gathering
        let mut amount = state.scrap_piles[target_idx].amount;
        let bonus_pct = state.upgrades.get_level("scrap_efficiency") as f32 * SCRAP_EFFICIENCY_BONUS;
        amount = (amount as f32 * (1.0 + bonus_pct)) as i32;
        
        state.resources.add_scrap(amount);
        state.scrap_piles[target_idx].active = false;
        events.push_ui(UIEvent::Toggle(0, 0));
        state.gathering_target = None;
        state.gathering_timer = 0.0;
    }

    fn find_nearest_scrap_pile(&self, state: &GameState) -> Option<usize> {
        let mut nearest = None;
        let mut min_dist = INTERACTION_RANGE;
        
        for (i, pile) in state.scrap_piles.iter().enumerate() {
            if !pile.active { continue; }
            let d = pile.position.distance(state.player.position);
            if d < min_dist {
                min_dist = d;
                nearest = Some(i);
            }
        }
        nearest
    }

    fn handle_interact(&self, state: &mut GameState, _events: &mut EventBus) {
        // Advance from welcome step on first E press
        if state.tutorial_state.is_welcome() {
            state.tutorial_state.advance(&state.tutorial_config);
        }

        // Find room player is in
        let Some(room_idx) = state.interior.rooms.iter()
            .position(|r: &Room| r.contains(state.player.position)) else { return };
        
        let room = &state.interior.rooms[room_idx];
        
        // Find repair point at player position
        let Some(point_idx) = room.repair_point_at(state.player.position) else { return };
        
        // Attempt repair
        if !state.attempt_interior_repair(room_idx, point_idx) { return };
        
        // Advance tutorial if this is the target room
        let Some(target) = state.tutorial_state.target_room(&state.tutorial_config) else { return };
        if room_idx == target {
            state.tutorial_state.advance(&state.tutorial_config);
        }
    }

    pub fn handle_grid_click(&self, x: usize, y: usize, state: &GameState, events: &mut EventBus) {
        if let Some(module) = &state.ship.grid[x][y] {
            match module.state {
                ModuleState::Destroyed => {
                    events.push_ui(UIEvent::Repair(x, y));
                }
                ModuleState::Active | ModuleState::Offline => {
                    if module.module_type == ModuleType::Engine 
                       && module.state == ModuleState::Active {
                        events.push_ui(UIEvent::ActivateEngine);
                    } else {
                        events.push_ui(UIEvent::Toggle(x, y));
                    }
                }
            }
        }
    }
}
