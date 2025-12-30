use macroquad::prelude::*;
use crate::state::{GameState, GamePhase};
use crate::simulation::constants::*;

pub struct Renderer {
    pub trauma: f32,
    pub shake_intensity: f32,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            trauma: 0.0,
            shake_intensity: SHAKE_INTENSITY,
        }
    }

    /// Add trauma for screen shake (clamped to 1.0)
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }

    /// Update trauma decay
    pub fn update(&mut self, dt: f32) {
        if self.trauma > 0.0 {
            self.trauma = (self.trauma - dt * TRAUMA_DECAY_RATE).max(0.0);
        }
    }

    /// Get current shake offset
    pub fn get_shake_offset(&self) -> Vec2 {
        if self.trauma <= 0.0 {
            return vec2(0.0, 0.0);
        }

        let shake = self.trauma * self.trauma;
        let offset_x = (rand::gen_range(-1.0, 1.0) * self.shake_intensity * shake);
        let offset_y = (rand::gen_range(-1.0, 1.0) * self.shake_intensity * shake);

        vec2(offset_x, offset_y)
    }

    pub fn draw(&self, state: &GameState) {
        match state.phase {
            GamePhase::Menu => self.draw_menu(),
            GamePhase::Playing => self.draw_gameplay(state),
            GamePhase::GameOver => self.draw_game_over(state),
            GamePhase::Victory => self.draw_victory(state),
            GamePhase::InterRound => self.draw_upgrade_screen(state),
        }
    }
}
