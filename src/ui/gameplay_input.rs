use macroquad::prelude::*;
use crate::state::{GameState, ViewMode};
use crate::simulation::events::{EventBus, UIEvent};
use crate::simulation::constants::*;
use crate::ship::interior::Room;
use crate::ship::ship::{ModuleState, ModuleType};
use crate::ui::input_manager::{InputManager, InputState};
use crate::ui::pause_menu::PauseMenuOption;

impl InputManager {
    pub fn handle_gameplay_input(&mut self, input: &InputState, state: &mut GameState, events: &mut EventBus) {
        // If paused, handle pause menu input instead
        if state.paused {
            self.handle_pause_menu_input(input, state, events);
            return;
        }

        // Tab toggles view mode
        if input.tab_pressed {
            state.view_mode = match state.view_mode {
                ViewMode::Interior => ViewMode::Exterior,
                ViewMode::Exterior => ViewMode::Interior,
            };
        }

        // Escape opens pause menu
        if input.escape_pressed {
            events.push_ui(UIEvent::Pause);
            return;
        }

        // P also pauses
        if input.pause_pressed {
            events.push_ui(UIEvent::Pause);
            return;
        }

        // View-specific input
        if state.view_mode == ViewMode::Interior {
            self.handle_interior_input(input, state, events);
        }
    }

    fn handle_pause_menu_input(&mut self, input: &InputState, state: &mut GameState, events: &mut EventBus) {
        let menu_options = PauseMenuOption::all();
        let option_count = menu_options.len();

        // ESC closes pause menu
        if input.escape_pressed {
            events.push_ui(UIEvent::Resume);
            return;
        }

        // Calculate button bounds (must match pause_menu.rs layout)
        let box_w = 300.0;
        let box_h = 320.0;
        let box_x = (screen_width() - box_w) / 2.0;
        let box_y = (screen_height() - box_h) / 2.0;
        let btn_w = 200.0;
        let btn_h = 40.0;
        let btn_x = box_x + (box_w - btn_w) / 2.0;
        let start_y = box_y + 70.0;
        let spacing = 50.0;

        // If settings panel is open, handle settings input instead
        if state.settings_open {
            self.handle_settings_input(input, state, events);
            return;
        }

        // Mouse hover updates selection
        let (mx, my) = (input.mouse_pos.x, input.mouse_pos.y);
        for i in 0..option_count {
            let y = start_y + i as f32 * spacing;
            if mx >= btn_x && mx <= btn_x + btn_w && my >= y && my <= y + btn_h {
                state.pause_menu_selection = i;
                
                // Mouse click selects
                if input.left_click {
                    let selected = menu_options[i];
                    match selected {
                        PauseMenuOption::Resume => events.push_ui(UIEvent::Resume),
                        PauseMenuOption::Settings => {
                            state.settings_open = true;
                            state.settings_selection = 0;
                        }
                        PauseMenuOption::SaveGame => events.push_ui(UIEvent::SaveGame(0)),
                        PauseMenuOption::LoadGame => events.push_ui(UIEvent::LoadGame(0)),
                        PauseMenuOption::ReturnToMenu => events.push_ui(UIEvent::ReturnToMenu),
                        PauseMenuOption::ExitGame => events.push_ui(UIEvent::ExitGame),
                    }
                    return;
                }
            }
        }

        // Arrow up
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            state.pause_menu_selection = if state.pause_menu_selection == 0 {
                option_count - 1
            } else {
                state.pause_menu_selection - 1
            };
        }

        // Arrow down
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            state.pause_menu_selection = (state.pause_menu_selection + 1) % option_count;
        }

        // Enter/Space selects
        if input.enter_pressed || input.space_pressed {
            let selected = menu_options[state.pause_menu_selection];
            match selected {
                PauseMenuOption::Resume => events.push_ui(UIEvent::Resume),
                PauseMenuOption::Settings => {
                    state.settings_open = true;
                    state.settings_selection = 0;
                }
                PauseMenuOption::SaveGame => events.push_ui(UIEvent::SaveGame(0)),
                PauseMenuOption::LoadGame => events.push_ui(UIEvent::LoadGame(0)),
                PauseMenuOption::ReturnToMenu => events.push_ui(UIEvent::ReturnToMenu),
                PauseMenuOption::ExitGame => events.push_ui(UIEvent::ExitGame),
            }
        }
    }

    fn handle_settings_input(&mut self, input: &InputState, state: &mut GameState, events: &mut EventBus) {
        const SETTING_COUNT: usize = 6; // 5 settings + Back
        
        // Up/Down navigation
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            state.settings_selection = if state.settings_selection == 0 {
                SETTING_COUNT - 1
            } else {
                state.settings_selection - 1
            };
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            state.settings_selection = (state.settings_selection + 1) % SETTING_COUNT;
        }

        // Left/Right adjusts value
        let left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
        let right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);
        let delta = if right { 0.1 } else if left { -0.1 } else { 0.0 };

        if delta != 0.0 {
            match state.settings_selection {
                0 => state.settings.master_volume = (state.settings.master_volume + delta).clamp(0.0, 1.0),
                1 => state.settings.sfx_volume = (state.settings.sfx_volume + delta).clamp(0.0, 1.0),
                2 => state.settings.music_volume = (state.settings.music_volume + delta).clamp(0.0, 1.0),
                _ => {}
            }
        }

        // Enter toggles booleans or selects Back
        if input.enter_pressed || input.space_pressed {
            match state.settings_selection {
                3 => {
                    state.settings.fullscreen = !state.settings.fullscreen;
                    // Apply fullscreen immediately
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        macroquad::window::set_fullscreen(state.settings.fullscreen);
                    }
                }
                4 => state.settings.screen_shake = !state.settings.screen_shake,
                5 => {
                    // Back - save and close
                    let _ = state.settings.save();
                    state.settings_open = false;
                }
                _ => {}
            }
        }

        // Escape also closes settings
        if is_key_pressed(KeyCode::Escape) {
            let _ = state.settings.save();
            state.settings_open = false;
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

    fn handle_interact(&self, state: &mut GameState, events: &mut EventBus) {
        // Advance from welcome step on first E press
        if state.tutorial_state.is_welcome() {
            state.tutorial_state.advance(&state.tutorial_config);
            return;
        }
        
        // Allow dismissing the final "complete" step with E
        if let Some(step) = state.tutorial_state.current_step(&state.tutorial_config) {
            if step.id == "complete" {
                state.tutorial_state.advance(&state.tutorial_config);
                return;
            }
        }

        // Find room player is in
        let Some(room_idx) = state.interior.rooms.iter()
            .position(|r: &Room| r.contains(state.player.position)) else { return };
        
        let room = &state.interior.rooms[room_idx];
        
        // Find repair point at player position
        let Some(point_idx) = room.repair_point_at(state.player.position) else { return };
        
        // Attempt repair
        if !state.attempt_interior_repair(room_idx, point_idx, events) { return };
        
        // Advance tutorial when player repairs ANY point in the target room
        // This gives immediate positive feedback instead of requiring full room completion
        let Some(target) = state.tutorial_state.target_room(&state.tutorial_config) else { return };
        let room = &state.interior.rooms[room_idx];
        if room.id == target {
            state.tutorial_state.advance(&state.tutorial_config);
        }
    }

}
