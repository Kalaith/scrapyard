use macroquad::prelude::*;
use crate::state::{GameState, GamePhase};
use crate::simulation::events::EventBus;

/// Captures current input state for the frame
#[derive(Debug, Clone)]

pub struct InputState {
    pub mouse_pos: Vec2,
    pub mouse_world_pos: Option<(usize, usize)>,
    pub left_click: bool,
    pub right_click: bool,
    pub escape_pressed: bool,
    pub enter_pressed: bool,
    pub space_pressed: bool,
    pub pause_pressed: bool,
    pub tab_pressed: bool,
    pub interact_pressed: bool,
}

impl InputState {
    pub fn capture() -> Self {
        Self {
            mouse_pos: mouse_position().into(),
            mouse_world_pos: None,
            left_click: is_mouse_button_pressed(MouseButton::Left),
            right_click: is_mouse_button_pressed(MouseButton::Right),
            escape_pressed: is_key_pressed(KeyCode::Escape),
            enter_pressed: is_key_pressed(KeyCode::Enter),
            space_pressed: is_key_pressed(KeyCode::Space),
            pause_pressed: is_key_pressed(KeyCode::P),
            tab_pressed: is_key_pressed(KeyCode::Tab),
            interact_pressed: is_key_pressed(KeyCode::E),
        }
    }
}

pub struct InputManager {
    // Current frame state
}

impl InputManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, state: &mut GameState, events: &mut EventBus) {
        let input = InputState::capture();

        match state.phase {
            GamePhase::Menu => self.handle_menu_input(&input, events),
            GamePhase::Playing => self.handle_gameplay_input(&input, state, events),
            GamePhase::GameOver => self.handle_game_over_input(&input, events),
            GamePhase::Victory => self.handle_victory_input(&input, events),
            GamePhase::InterRound => self.handle_upgrade_input(&input, state, events),
        }
    }
}
