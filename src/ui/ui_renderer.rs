use macroquad::prelude::*;
use crate::state::GameState;
use crate::ui::renderer::Renderer;

impl Renderer {
    pub fn draw_tutorial(&self, state: &GameState) {
        let step = match state.tutorial_state.current_step(&state.tutorial_config) {
            Some(s) => s,
            None => return, // Tutorial complete
        };
        
        let box_height = 80.0;
        draw_rectangle(0.0, 0.0, screen_width(), box_height, color_u8!(0, 0, 0, 200));
        
        let lines: Vec<&str> = step.message.split('\n').collect();
        
        for (i, line) in lines.iter().enumerate() {
            let text_w = measure_text(line, None, 20, 1.0).width;
            draw_text(line, (screen_width() - text_w) / 2.0, 25.0 + i as f32 * 24.0, 20.0, WHITE);
        }
        
        // Step counter (exclude welcome and complete from count)
        let step_num = state.tutorial_state.current_index;
        let total_steps = state.tutorial_config.steps.len().saturating_sub(2); // Exclude welcome/complete
        if step_num > 0 && step_num <= total_steps {
            let step_text = format!("Step {}/{}", step_num, total_steps);
            draw_text(&step_text, 20.0, box_height - 10.0, 16.0, GRAY);
        }
        
        if state.tutorial_state.is_welcome() {
            draw_text("[Press E to continue]", screen_width() - 180.0, box_height - 10.0, 14.0, YELLOW);
        }
    }

    pub fn draw_menu(&self) {
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(15, 15, 25, 255));
        let title = "SCRAPYARD PLANET";
        let title_size = measure_text(title, None, 64, 1.0);
        draw_text(title, screen_width() / 2.0 - title_size.width / 2.0, screen_height() / 3.0, 64.0, WHITE);

        let subtitle = "Repair. Defend. Escape.";
        let sub_size = measure_text(subtitle, None, 24, 1.0);
        draw_text(subtitle, screen_width() / 2.0 - sub_size.width / 2.0, screen_height() / 3.0 + 50.0, 24.0, GRAY);

        let btn_width = 200.0;
        let btn_height = 50.0;
        let btn_x = screen_width() / 2.0 - btn_width / 2.0;
        
        // Check if save file exists
        let has_save = std::path::Path::new("save_slot_0.json").exists();
        
        // Continue button (only if save exists)
        let mut next_y = screen_height() / 2.0 + 20.0;
        if has_save {
            let btn_y = next_y;
            draw_rectangle(btn_x, btn_y, btn_width, btn_height, color_u8!(40, 80, 60, 255));
            draw_rectangle_lines(btn_x, btn_y, btn_width, btn_height, 2.0, color_u8!(100, 180, 140, 255));
            let continue_text = "CONTINUE";
            let continue_size = measure_text(continue_text, None, 28, 1.0);
            draw_text(continue_text, btn_x + btn_width / 2.0 - continue_size.width / 2.0, btn_y + btn_height / 2.0 + 8.0, 28.0, WHITE);
            next_y += btn_height + 15.0;
        }

        // New Game button
        let btn_y = next_y;
        draw_rectangle(btn_x, btn_y, btn_width, btn_height, color_u8!(60, 60, 80, 255));
        draw_rectangle_lines(btn_x, btn_y, btn_width, btn_height, 2.0, color_u8!(100, 100, 140, 255));
        let start_text = "NEW GAME";
        let start_size = measure_text(start_text, None, 28, 1.0);
        draw_text(start_text, btn_x + btn_width / 2.0 - start_size.width / 2.0, btn_y + btn_height / 2.0 + 8.0, 28.0, WHITE);

        let hint = if has_save { "Click CONTINUE to load or NEW GAME to start fresh" } 
                   else { "Click NEW GAME or press ENTER to begin" };
        let hint_size = measure_text(hint, None, 18, 1.0);
        draw_text(hint, screen_width() / 2.0 - hint_size.width / 2.0, screen_height() - 50.0, 18.0, DARKGRAY);
    }

    pub fn get_menu_button_bounds(&self) -> (Option<(f32, f32, f32, f32)>, (f32, f32, f32, f32)) {
        let btn_width = 200.0;
        let btn_height = 50.0;
        let btn_x = screen_width() / 2.0 - btn_width / 2.0;
        let has_save = std::path::Path::new("save_slot_0.json").exists();
        
        let mut next_y = screen_height() / 2.0 + 20.0;
        let continue_bounds = if has_save {
            let bounds = (btn_x, next_y, btn_width, btn_height);
            next_y += btn_height + 15.0;
            Some(bounds)
        } else {
            None
        };
        
        let new_game_bounds = (btn_x, next_y, btn_width, btn_height);
        (continue_bounds, new_game_bounds)
    }

    pub fn get_start_button_bounds(&self) -> (f32, f32, f32, f32) {
        let (_, new_game) = self.get_menu_button_bounds();
        new_game
    }

    pub fn draw_game_over(&self, state: &GameState) {
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(15, 5, 5, 255));
        for i in 0..5 {
            let alpha = 100 - i * 20;
            let offset = i as f32 * 30.0;
            draw_rectangle_lines(offset, offset, screen_width() - offset * 2.0, screen_height() - offset * 2.0, 3.0, color_u8!(80, 0, 0, alpha as u8));
        }

        let text = "CORE DESTROYED";
        let size = measure_text(text, None, 64, 1.0);
        draw_text(text, screen_width() / 2.0 - size.width / 2.0, screen_height() / 3.0, 64.0, RED);

        let stats_y = screen_height() / 2.0;
        let stats = [
            format!("Scrap Collected: {}", state.resources.scrap + 100),
            format!("Credits Earned: {}", state.resources.credits),
            format!("Frames Survived: {}", state.frame_count),
        ];
        
        for (i, stat) in stats.iter().enumerate() {
            let s = measure_text(stat, None, 24, 1.0);
            draw_text(stat, screen_width() / 2.0 - s.width / 2.0, stats_y + i as f32 * 30.0, 24.0, GRAY);
        }

        let hint = "Press ENTER to return to menu";
        let hint_size = measure_text(hint, None, 24, 1.0);
        draw_text(hint, screen_width() / 2.0 - hint_size.width / 2.0, screen_height() - 80.0, 24.0, WHITE);
    }

    pub fn draw_victory(&self, state: &GameState) {
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(10, 20, 30, 255));
        for i in 0..8 {
            let alpha = 60 - i * 7;
            let offset = i as f32 * 20.0;
            draw_rectangle_lines(offset, offset, screen_width() - offset * 2.0, screen_height() - offset * 2.0, 2.0, color_u8!(100, 200, 255, alpha as u8));
        }

        let text = "ESCAPE SUCCESSFUL!";
        let size = measure_text(text, None, 64, 1.0);
        draw_text(text, screen_width() / 2.0 - size.width / 2.0, screen_height() / 3.0, 64.0, GREEN);

        let subtitle = "You made it off the planet!";
        let sub_size = measure_text(subtitle, None, 28, 1.0);
        draw_text(subtitle, screen_width() / 2.0 - sub_size.width / 2.0, screen_height() / 3.0 + 50.0, 28.0, color_u8!(150, 255, 150, 255));

        let stats_y = screen_height() / 2.0;
        let stats = [
            format!("Total Credits: {}", state.resources.credits),
            format!("Core Health Remaining: {:.0}%", 
                if let Some(pos) = state.ship.find_core() {
                    if let Some(core) = &state.ship.grid[pos.0][pos.1] {
                        (core.health / core.max_health) * 100.0
                    } else { 0.0 }
                } else { 0.0 }
            ),
        ];
        
        for (i, stat) in stats.iter().enumerate() {
            let s = measure_text(stat, None, 24, 1.0);
            draw_text(stat, screen_width() / 2.0 - s.width / 2.0, stats_y + i as f32 * 30.0, 24.0, WHITE);
        }

        let hint = "Press ENTER to continue to Upgrades";
        let hint_size = measure_text(hint, None, 24, 1.0);
        draw_text(hint, screen_width() / 2.0 - hint_size.width / 2.0, screen_height() - 80.0, 24.0, YELLOW);
    }

    pub fn draw_upgrade_screen(&self, state: &GameState) {
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(15, 20, 30, 255));
        let title = "SHIP IMPROVEMENTS";
        let title_w = measure_text(title, None, 48, 1.0).width;
        draw_text(title, (screen_width() - title_w) / 2.0, 60.0, 48.0, WHITE);
        
        let credits_text = format!("AVAILABLE CREDITS: {}", state.resources.credits);
        let cred_w = measure_text(&credits_text, None, 24, 1.0).width;
        draw_text(&credits_text, (screen_width() - cred_w) / 2.0, 100.0, 24.0, GREEN);

        let start_y = 150.0;
        let card_w = 600.0;
        let card_h = 80.0;
        let spacing = 20.0;
        let card_x = (screen_width() - card_w) / 2.0;

        for (i, template) in state.upgrade_templates.iter().enumerate() {
            let y = start_y + i as f32 * (card_h + spacing);
            let current_level = state.upgrades.get_level(&template.id);
            let is_max = current_level >= template.max_level;
            let cost = state.upgrades.get_cost(template);
            let can_afford = state.resources.credits >= cost && !is_max;

            let bg_color = if is_max { color_u8!(40, 50, 40, 255) } else if can_afford { color_u8!(40, 40, 60, 255) } else { color_u8!(30, 30, 35, 255) };
            draw_rectangle(card_x, y, card_w, card_h, bg_color);
            draw_rectangle_lines(card_x, y, card_w, card_h, 2.0, if can_afford { YELLOW } else { GRAY });

            draw_text(&format!("{} (Level {}/{})", template.name, current_level, template.max_level), card_x + 15.0, y + 30.0, 24.0, WHITE);
            draw_text(&template.description, card_x + 15.0, y + 55.0, 16.0, GRAY);

            if is_max {
                draw_text("MAX LEVEL", card_x + card_w - 120.0, y + 45.0, 20.0, GREEN);
            } else {
                let cost_color = if can_afford { WHITE } else { RED };
                draw_text(&format!("Cost: {} Cr", cost), card_x + card_w - 150.0, y + 35.0, 20.0, cost_color);
                if can_afford {
                    draw_text(&format!("[{}] Buy", i + 1), card_x + card_w - 150.0, y + 60.0, 20.0, YELLOW);
                } else {
                    draw_text("Insufficient Funds", card_x + card_w - 150.0, y + 60.0, 16.0, RED);
                }
            }
        }

        let footer = "Press [ENTER] to start next round | Press [ESC] for Menu";
        let footer_w = measure_text(footer, None, 20, 1.0).width;
        draw_text(footer, (screen_width() - footer_w) / 2.0, screen_height() - 40.0, 20.0, DARKGRAY);
    }
}
