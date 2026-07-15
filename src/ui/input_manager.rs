use crate::simulation::events::EventBus;
use crate::state::{GamePhase, GameState};
use macroquad::prelude::*;
use macroquad_toolkit::input::InputState as BaseInputState;

/// Captures current input state for the frame: the toolkit's shared snapshot
/// (mouse buttons/position, Escape/Enter/Space) plus scrapyard-specific keys.
#[derive(Debug, Clone)]
pub struct InputState {
    pub base: BaseInputState,
    pub pause_pressed: bool,
    pub tab_pressed: bool,
    pub interact_pressed: bool,
}

impl InputState {
    pub fn capture() -> Self {
        Self {
            base: BaseInputState::capture(),
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
            GamePhase::Menu => self.handle_menu_input(&input, state, events),
            GamePhase::Playing => self.handle_gameplay_input(&input, state, events),
            GamePhase::GameOver => self.handle_game_over_input(&input, events),
            GamePhase::Victory => self.handle_victory_input(&input, events),
            GamePhase::InterRound => self.handle_upgrade_input(&input, state, events),
        }
    }
}
