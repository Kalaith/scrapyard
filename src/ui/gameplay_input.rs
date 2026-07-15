use crate::data::settings;
use crate::ship::interior::Room;
use crate::simulation::constants::*;
use crate::simulation::events::{EventBus, UIEvent};
use crate::state::{GameState, ViewMode};
use crate::ui::input_manager::{InputManager, InputState};
use crate::ui::pause_menu::PauseMenuOption;
use macroquad::prelude::*;
use macroquad_toolkit::input::{menu_nav_horizontal, menu_nav_vertical};

impl InputManager {
    pub fn handle_gameplay_input(
        &mut self,
        input: &InputState,
        state: &mut GameState,
        events: &mut EventBus,
    ) {
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
        if input.base.escape_pressed {
            events.push_ui(UIEvent::Pause);
            return;
        }

        // P also pauses
        if input.pause_pressed {
            events.push_ui(UIEvent::Pause);
            return;
        }

        self.handle_power_routing_input(input, state, events);

        // View-specific input
        if state.view_mode == ViewMode::Interior {
            self.handle_interior_input(input, state, events);
        }
    }

    fn handle_pause_menu_input(
        &mut self,
        input: &InputState,
        state: &mut GameState,
        events: &mut EventBus,
    ) {
        let menu_options = PauseMenuOption::all();

        // ESC closes pause menu
        if input.base.escape_pressed {
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
        for (i, selected) in menu_options.iter().enumerate() {
            let y = start_y + i as f32 * spacing;
            if input.base.hovered_rect(Rect::new(btn_x, y, btn_w, btn_h)) {
                state.pause_menu_cursor.set_index(i);

                // Mouse click selects
                if input.base.left_click {
                    match selected {
                        PauseMenuOption::Resume => events.push_ui(UIEvent::Resume),
                        PauseMenuOption::Settings => {
                            state.settings_open = true;
                            state.settings_cursor.set_index(0);
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

        // Arrow keys / WASD move the selection with wrap-around
        state.pause_menu_cursor.navigate(menu_nav_vertical());

        // Enter/Space selects
        if input.base.enter_pressed || input.base.space_pressed {
            let selected = menu_options[state.pause_menu_cursor.index()];
            match selected {
                PauseMenuOption::Resume => events.push_ui(UIEvent::Resume),
                PauseMenuOption::Settings => {
                    state.settings_open = true;
                    state.settings_cursor.set_index(0);
                }
                PauseMenuOption::SaveGame => events.push_ui(UIEvent::SaveGame(0)),
                PauseMenuOption::LoadGame => events.push_ui(UIEvent::LoadGame(0)),
                PauseMenuOption::ReturnToMenu => events.push_ui(UIEvent::ReturnToMenu),
                PauseMenuOption::ExitGame => events.push_ui(UIEvent::ExitGame),
            }
        }
    }

    pub(crate) fn handle_settings_input(
        &mut self,
        input: &InputState,
        state: &mut GameState,
        _events: &mut EventBus,
    ) {
        // Up/Down navigation with wrap-around
        state.settings_cursor.navigate(menu_nav_vertical());

        // Left/Right adjusts value
        let delta = menu_nav_horizontal() as f32 * 0.1;

        if delta != 0.0 {
            match state.settings_cursor.index() {
                0 => {
                    state.settings.master_volume =
                        (state.settings.master_volume + delta).clamp(0.0, 1.0)
                }
                1 => {
                    state.settings.sfx_volume = (state.settings.sfx_volume + delta).clamp(0.0, 1.0)
                }
                2 => {
                    state.settings.music_volume =
                        (state.settings.music_volume + delta).clamp(0.0, 1.0)
                }
                _ => {}
            }
        }

        // Enter toggles booleans or selects Back
        if input.base.enter_pressed || input.base.space_pressed {
            match state.settings_cursor.index() {
                3 => state.settings.toggle_fullscreen(),
                4 => state.settings.screen_shake = !state.settings.screen_shake,
                5 => {
                    // Back - save and close
                    let _ = settings::save(&state.settings);
                    state.settings_open = false;
                }
                _ => {}
            }
        }

        // Escape also closes settings
        if is_key_pressed(KeyCode::Escape) {
            let _ = settings::save(&state.settings);
            state.settings_open = false;
        }
    }

    fn handle_interior_input(
        &mut self,
        input: &InputState,
        state: &mut GameState,
        events: &mut EventBus,
    ) {
        self.handle_scrap_gathering(state, events);

        if input.interact_pressed {
            self.handle_interact(state, events);
        }
    }

    fn handle_power_routing_input(
        &self,
        input: &InputState,
        state: &mut GameState,
        events: &mut EventBus,
    ) {
        let keys = [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
        ];

        for (slot, key) in keys.iter().enumerate() {
            if is_key_pressed(*key) && state.toggle_route_slot(slot, events) {
                state.advance_tutorial_after_power_route();
                return;
            }
        }

        if input.base.left_click {
            if let Some(slot) = route_slot_at(input.base.mouse_pos) {
                if state.toggle_route_slot(slot, events) {
                    state.advance_tutorial_after_power_route();
                }
            }
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
        let Some(target_idx) = state.gathering_target else {
            return;
        };
        if target_idx >= state.scrap_piles.len() {
            return;
        };

        state.gathering_timer += get_frame_time();
        if state.gathering_timer < GATHERING_TIME_SECONDS {
            return;
        };

        // Complete gathering
        let mut amount = state.scrap_piles[target_idx].amount;
        let bonus_pct =
            state.upgrades.get_level("scrap_efficiency") as f32 * SCRAP_EFFICIENCY_BONUS;
        amount = (amount as f32 * (1.0 + bonus_pct)) as i32;

        state.resources.add_scrap(amount);
        state.scrap_piles[target_idx].active = false;
        // Scavenging spends the startup calm — the more you take, the sooner they come.
        state.startup_grace = (state.startup_grace - STARTUP_GRACE_REDUCE_PER_SCRAP).max(0.0);
        events.push_ui(UIEvent::Toggle(0, 0));
        state.gathering_target = None;
        state.gathering_timer = 0.0;
    }

    fn find_nearest_scrap_pile(&self, state: &GameState) -> Option<usize> {
        let mut nearest = None;
        let mut min_dist = INTERACTION_RANGE;

        for (i, pile) in state.scrap_piles.iter().enumerate() {
            if !pile.active {
                continue;
            }
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
        let Some(room_idx) = state
            .interior
            .rooms
            .iter()
            .position(|r: &Room| r.contains(state.player.position))
        else {
            return;
        };

        let room = &state.interior.rooms[room_idx];

        // If standing on an unrepaired point, repair it.
        if let Some(point_idx) = room.repair_point_at(state.player.position) {
            if !room.repair_points[point_idx].repaired {
                if state.attempt_interior_repair(room_idx, point_idx, events) {
                    state.advance_tutorial_after_repair(room_idx);
                }
                return;
            }
        }

        // Otherwise, a finished weapon/shield room can be upgraded in-run for scrap.
        state.attempt_interior_upgrade(room_idx, events);
    }
}

fn route_slot_at(mouse_pos: Vec2) -> Option<usize> {
    let x = 12.0;
    let y = 92.0;
    let w = 286.0;
    let row_h = 28.0;
    let first_row_y = y + 46.0;
    if mouse_pos.x < x || mouse_pos.x > x + w || mouse_pos.y < first_row_y {
        return None;
    }
    let slot = ((mouse_pos.y - first_row_y) / row_h).floor() as usize;
    if slot < 8 {
        Some(slot)
    } else {
        None
    }
}
