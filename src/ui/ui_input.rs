use macroquad::prelude::*;
use crate::state::GameState;
use crate::simulation::events::{EventBus, UIEvent};
use crate::ui::input_manager::{InputManager, InputState};
use crate::ui::renderer::Renderer;

impl InputManager {
    pub fn handle_menu_input(&self, input: &InputState, events: &mut EventBus) {
        if input.enter_pressed || input.space_pressed {
            events.push_ui(UIEvent::StartGame);
            return;
        }

        if input.left_click {
            // Use Renderer's button bounds for consistency
            let renderer = Renderer::new();
            let (continue_bounds, new_game_bounds) = renderer.get_menu_button_bounds();
            
            // Check Continue button click (if save exists)
            if let Some((btn_x, btn_y, btn_w, btn_h)) = continue_bounds {
                if input.mouse_pos.x >= btn_x && input.mouse_pos.x <= btn_x + btn_w &&
                   input.mouse_pos.y >= btn_y && input.mouse_pos.y <= btn_y + btn_h {
                    events.push_ui(UIEvent::LoadGame(0));
                    return;
                }
            }

            // Check New Game button click
            let (btn_x, btn_y, btn_w, btn_h) = new_game_bounds;
            if input.mouse_pos.x >= btn_x && input.mouse_pos.x <= btn_x + btn_w &&
               input.mouse_pos.y >= btn_y && input.mouse_pos.y <= btn_y + btn_h {
                events.push_ui(UIEvent::StartGame);
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

    pub fn handle_upgrade_input(&self, input: &InputState, state: &GameState, events: &mut EventBus) {
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
            KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5,
            KeyCode::Key6, KeyCode::Key7, KeyCode::Key8, KeyCode::Key9,
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
