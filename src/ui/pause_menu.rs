//! Pause menu overlay UI

use macroquad::prelude::*;
use crate::state::GameState;
use crate::ui::renderer::Renderer;

/// Pause menu state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuOption {
    Resume,
    Settings,
    SaveGame,
    LoadGame,
    ReturnToMenu,
    ExitGame,
}

impl PauseMenuOption {
    pub fn all() -> [PauseMenuOption; 6] {
        [
            PauseMenuOption::Resume,
            PauseMenuOption::Settings,
            PauseMenuOption::SaveGame,
            PauseMenuOption::LoadGame,
            PauseMenuOption::ReturnToMenu,
            PauseMenuOption::ExitGame,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            PauseMenuOption::Resume => "Resume",
            PauseMenuOption::Settings => "Settings",
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
        let box_h = 370.0;  // Increased for 6 options
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

    pub fn draw_settings_panel(&self, state: &GameState) {
        // Dim background
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(0, 0, 0, 200));

        // Settings box
        let box_w = 400.0;
        let box_h = 350.0;
        let box_x = (screen_width() - box_w) / 2.0;
        let box_y = (screen_height() - box_h) / 2.0;

        draw_rectangle(box_x, box_y, box_w, box_h, color_u8!(25, 25, 35, 255));
        draw_rectangle_lines(box_x, box_y, box_w, box_h, 3.0, color_u8!(80, 80, 120, 255));

        // Title
        let title = "SETTINGS";
        let title_w = measure_text(title, None, 32, 1.0).width;
        draw_text(title, box_x + (box_w - title_w) / 2.0, box_y + 40.0, 32.0, WHITE);

        let settings = &state.settings;
        let selected = state.settings_selection;
        let row_height = 50.0;
        let start_y = box_y + 80.0;
        let label_x = box_x + 30.0;
        let slider_x = box_x + 180.0;
        let slider_w = 180.0;

        // Setting rows
        let options = [
            ("Master Volume", settings.master_volume, true),
            ("SFX Volume", settings.sfx_volume, true),
            ("Music Volume", settings.music_volume, true),
            ("Fullscreen", if settings.fullscreen { 1.0 } else { 0.0 }, false),
            ("Screen Shake", if settings.screen_shake { 1.0 } else { 0.0 }, false),
        ];

        for (i, (label, value, is_slider)) in options.iter().enumerate() {
            let y = start_y + i as f32 * row_height;
            let is_selected = i == selected;

            // Highlight selected row
            if is_selected {
                draw_rectangle(box_x + 10.0, y - 5.0, box_w - 20.0, row_height - 10.0, color_u8!(50, 50, 70, 255));
            }

            // Label
            let text_color = if is_selected { YELLOW } else { WHITE };
            draw_text(label, label_x, y + 20.0, 20.0, text_color);

            if *is_slider {
                // Draw slider background
                draw_rectangle(slider_x, y + 8.0, slider_w, 16.0, color_u8!(40, 40, 50, 255));
                // Draw slider fill
                let fill_w = slider_w * value;
                draw_rectangle(slider_x, y + 8.0, fill_w, 16.0, color_u8!(80, 150, 80, 255));
                draw_rectangle_lines(slider_x, y + 8.0, slider_w, 16.0, 2.0, if is_selected { YELLOW } else { GRAY });
                // Value text
                let pct = format!("{:.0}%", value * 100.0);
                draw_text(&pct, slider_x + slider_w + 10.0, y + 22.0, 18.0, text_color);
            } else {
                // Toggle button
                let toggle_text = if *value > 0.5 { "ON" } else { "OFF" };
                let toggle_color = if *value > 0.5 { GREEN } else { RED };
                draw_text(toggle_text, slider_x, y + 20.0, 20.0, toggle_color);
            }
        }

        // Back button
        let back_y = start_y + 5.0 * row_height;
        let is_back_selected = selected == 5;
        if is_back_selected {
            draw_rectangle(box_x + 10.0, back_y - 5.0, box_w - 20.0, row_height - 10.0, color_u8!(50, 50, 70, 255));
        }
        let back_text = "< Back (Settings Saved)";
        let back_color = if is_back_selected { YELLOW } else { WHITE };
        draw_text(back_text, label_x, back_y + 20.0, 20.0, back_color);

        // Controls hint
        let hint = "Up/Down: Select | Left/Right: Adjust | Enter: Toggle/Back";
        let hint_w = measure_text(hint, None, 14, 1.0).width;
        draw_text(hint, box_x + (box_w - hint_w) / 2.0, box_y + box_h - 15.0, 14.0, GRAY);
    }
}
