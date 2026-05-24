use crate::simulation::events::{EventBus, UIEvent};
use crate::state::GameState;
use crate::ui::input_manager::{InputManager, InputState};
use crate::ui::renderer::Renderer;
use macroquad::prelude::*;

impl InputManager {
    pub fn handle_menu_input(
        &mut self,
        input: &InputState,
        state: &mut GameState,
        events: &mut EventBus,
    ) {
        if state.settings_open {
            self.handle_settings_input(input, state, events);
            return;
        }

        if input.enter_pressed || input.space_pressed {
            events.push_ui(UIEvent::StartGame);
            return;
        }

        if input.left_click {
            // Use Renderer's button bounds for consistency
            let renderer = Renderer::new();

            // Check New Game button click
            if contains_mouse(renderer.get_new_game_button_bounds(), input.mouse_pos) {
                events.push_ui(UIEvent::StartGame);
                return;
            }

            if contains_mouse(renderer.get_settings_button_bounds(), input.mouse_pos) {
                state.settings_open = true;
                state.settings_selection = 0;
                return;
            }

            if contains_mouse(renderer.get_exit_button_bounds(), input.mouse_pos) {
                events.push_ui(UIEvent::ExitGame);
            }
        }
    }

    pub fn handle_game_over_input(&self, input: &InputState, events: &mut EventBus) {
        if input.enter_pressed || input.space_pressed {
            events.push_ui(UIEvent::ReturnToMenu);
        }
    }

    pub fn handle_victory_input(&self, input: &InputState, events: &mut EventBus) {
        if input.enter_pressed || input.space_pressed {
            events.push_ui(UIEvent::PurchaseUpgrade("dummy".to_string()));
        }
    }

    pub fn handle_upgrade_input(
        &self,
        input: &InputState,
        state: &GameState,
        events: &mut EventBus,
    ) {
        if input.enter_pressed {
            events.push_ui(UIEvent::NextRound);
            return;
        }

        if input.escape_pressed {
            events.push_ui(UIEvent::ReturnToMenu);
            return;
        }

        // Number keys 1-5 for purchasing upgrades
        let keys = [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
        ];

        for (i, key) in keys.iter().enumerate() {
            if is_key_pressed(*key) {
                if let Some(template) = state.upgrade_templates.get(i) {
                    events.push_ui(UIEvent::PurchaseUpgrade(template.id.clone()));
                }
            }
        }
    }
}

fn contains_mouse(bounds: (f32, f32, f32, f32), mouse_pos: Vec2) -> bool {
    let (x, y, w, h) = bounds;
    mouse_pos.x >= x && mouse_pos.x <= x + w && mouse_pos.y >= y && mouse_pos.y <= y + h
}
