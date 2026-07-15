use crate::simulation::constants::*;
use crate::state::{GamePhase, GameState};
use macroquad::prelude::*;
use macroquad_toolkit::fx::ScreenShake;

pub struct Renderer {
    pub shake: ScreenShake,
}

impl Renderer {
    pub fn new() -> Self {
        let mut shake = ScreenShake::new(SHAKE_INTENSITY);
        shake.decay_rate = TRAUMA_DECAY_RATE;
        Self { shake }
    }

    /// Add trauma for screen shake (clamped to 1.0)
    pub fn add_trauma(&mut self, amount: f32) {
        self.shake.add_trauma(amount);
    }

    /// Update trauma decay
    pub fn update(&mut self, dt: f32) {
        self.shake.update(dt);
    }

    /// Get current shake offset
    pub fn get_shake_offset(&self) -> Vec2 {
        self.shake.offset()
    }

    pub fn draw(&self, state: &GameState) {
        match state.phase {
            GamePhase::Menu => {
                self.draw_menu(state);
                if state.settings_open {
                    self.draw_settings_panel(state);
                }
            }
            GamePhase::Playing => {
                self.draw_gameplay(state);
                // Draw pause menu overlay if paused
                if state.paused {
                    if state.settings_open {
                        self.draw_settings_panel(state);
                    } else {
                        self.draw_pause_menu(state, state.pause_menu_cursor.index());
                    }
                }
            }
            GamePhase::GameOver => self.draw_game_over(state),
            GamePhase::Victory => self.draw_victory(state),
            GamePhase::InterRound => self.draw_upgrade_screen(state),
        }
    }
}
