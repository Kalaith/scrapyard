//! Pause menu overlay UI

use macroquad::prelude::*;
use crate::state::GameState;
use crate::ui::renderer::Renderer;

/// Pause menu state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuOption {
    Resume,
    SaveGame,
    LoadGame,
    ReturnToMenu,
    ExitGame,
}

impl PauseMenuOption {
    pub fn all() -> [PauseMenuOption; 5] {
        [
            PauseMenuOption::Resume,
            PauseMenuOption::SaveGame,
            PauseMenuOption::LoadGame,
            PauseMenuOption::ReturnToMenu,
            PauseMenuOption::ExitGame,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            PauseMenuOption::Resume => "Resume",
            PauseMenuOption::SaveGame => "Save Game",
            PauseMenuOption::LoadGame => "Load Game",
            PauseMenuOption::ReturnToMenu => "Return to Menu",
            PauseMenuOption::ExitGame => "Exit Game",
        }
    }
}

impl Renderer {
    pub fn draw_pause_menu(&self, _state: &GameState, selected: usize) {
        // Dim background
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(0, 0, 0, 180));

        // Menu box
        let box_w = 300.0;
        let box_h = 320.0;
        let box_x = (screen_width() - box_w) / 2.0;
        let box_y = (screen_height() - box_h) / 2.0;

        draw_rectangle(box_x, box_y, box_w, box_h, color_u8!(30, 30, 40, 255));
        draw_rectangle_lines(box_x, box_y, box_w, box_h, 3.0, color_u8!(100, 100, 140, 255));

        // Title
        let title = "PAUSED";
        let title_w = measure_text(title, None, 32, 1.0).width;
        draw_text(title, box_x + (box_w - title_w) / 2.0, box_y + 40.0, 32.0, WHITE);

        // Menu options
        let options = PauseMenuOption::all();
        let btn_w = 200.0;
        let btn_h = 40.0;
        let btn_x = box_x + (box_w - btn_w) / 2.0;
        let start_y = box_y + 70.0;
        let spacing = 50.0;

        for (i, option) in options.iter().enumerate() {
            let y = start_y + i as f32 * spacing;
            let is_selected = i == selected;

            let bg_color = if is_selected {
                color_u8!(70, 70, 100, 255)
            } else {
                color_u8!(50, 50, 60, 255)
            };
            let border_color = if is_selected { YELLOW } else { GRAY };

            draw_rectangle(btn_x, y, btn_w, btn_h, bg_color);
            draw_rectangle_lines(btn_x, y, btn_w, btn_h, 2.0, border_color);

            let label = option.label();
            let text_w = measure_text(label, None, 20, 1.0).width;
            let text_color = if is_selected { WHITE } else { LIGHTGRAY };
            draw_text(label, btn_x + (btn_w - text_w) / 2.0, y + 26.0, 20.0, text_color);
        }

        // Controls hint
        let hint = "Arrow Keys / Enter to select";
        let hint_w = measure_text(hint, None, 14, 1.0).width;
        draw_text(hint, box_x + (box_w - hint_w) / 2.0, box_y + box_h - 15.0, 14.0, GRAY);
    }
}
