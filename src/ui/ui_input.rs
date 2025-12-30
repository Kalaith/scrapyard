use macroquad::prelude::*;
use crate::state::GameState;
use crate::simulation::events::{EventBus, UIEvent};
use crate::ui::input_manager::{InputManager, InputState};

impl InputManager {
    pub fn handle_menu_input(&self, input: &InputState, events: &mut EventBus) {
        if input.enter_pressed || input.space_pressed {
            events.push_ui(UIEvent::StartGame);
            return;
        }

        if input.left_click {
            let btn_width = 200.0;
            let btn_height = 50.0;
            let btn_x = screen_width() / 2.0 - btn_width / 2.0;
            let btn_y = screen_height() / 2.0 + 50.0;

            if input.mouse_pos.x >= btn_x && input.mouse_pos.x <= btn_x + btn_width &&
               input.mouse_pos.y >= btn_y && input.mouse_pos.y <= btn_y + btn_height {
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
