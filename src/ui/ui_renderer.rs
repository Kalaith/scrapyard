use crate::state::GameState;
use crate::ui::renderer::Renderer;
use crate::ui::theme;
use macroquad::prelude::*;

impl Renderer {
    pub fn draw_menu(&self, state: &GameState) {
        let has_background = self.draw_menu_background(state);

        if !has_background {
            self.draw_fallback_menu_title();
        }

        self.draw_menu_buttons();
    }

    fn draw_menu_background(&self, state: &GameState) -> bool {
        let Some(texture) = state.assets.get_texture("menu_start_background") else {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                color_u8!(12, 12, 18, 255),
            );
            return false;
        };

        draw_texture_cover(
            texture,
            Rect::new(0.0, 0.0, screen_width(), screen_height()),
        );
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            color_u8!(0, 0, 0, 56),
        );
        draw_rectangle(
            0.0,
            screen_height() * 0.54,
            screen_width(),
            screen_height() * 0.46,
            color_u8!(0, 0, 0, 76),
        );
        true
    }

    fn draw_fallback_menu_title(&self) {
        let title = "SCRAPYARD PLANET";
        let title_size = measure_text(title, None, 64, 1.0);
        draw_text(
            title,
            screen_width() / 2.0 - title_size.width / 2.0,
            screen_height() / 3.0,
            64.0,
            WHITE,
        );

        let subtitle = "Repair. Defend. Escape.";
        let sub_size = measure_text(subtitle, None, 24, 1.0);
        draw_text(
            subtitle,
            screen_width() / 2.0 - sub_size.width / 2.0,
            screen_height() / 3.0 + 50.0,
            24.0,
            GRAY,
        );
    }

    fn draw_menu_buttons(&self) {
        let buttons = [
            (
                "NEW GAME",
                self.get_new_game_button_bounds(),
                theme::warning(),
            ),
            ("SETTINGS", self.get_settings_button_bounds(), theme::cyan()),
            ("EXIT GAME", self.get_exit_button_bounds(), theme::danger()),
        ];

        for (label, bounds, accent) in buttons {
            draw_menu_button(bounds, label, accent);
        }
    }

    pub fn get_new_game_button_bounds(&self) -> (f32, f32, f32, f32) {
        menu_button_bounds(0)
    }

    pub fn get_settings_button_bounds(&self) -> (f32, f32, f32, f32) {
        menu_button_bounds(1)
    }

    pub fn get_exit_button_bounds(&self) -> (f32, f32, f32, f32) {
        menu_button_bounds(2)
    }

    pub fn get_start_button_bounds(&self) -> (f32, f32, f32, f32) {
        self.get_new_game_button_bounds()
    }

    pub fn draw_game_over(&self, state: &GameState) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            color_u8!(15, 5, 5, 255),
        );
        for i in 0..5 {
            let alpha = 100 - i * 20;
            let offset = i as f32 * 30.0;
            draw_rectangle_lines(
                offset,
                offset,
                screen_width() - offset * 2.0,
                screen_height() - offset * 2.0,
                3.0,
                color_u8!(80, 0, 0, alpha as u8),
            );
        }

        let text = "CORE DESTROYED";
        let size = measure_text(text, None, 64, 1.0);
        draw_text(
            text,
            screen_width() / 2.0 - size.width / 2.0,
            screen_height() / 3.0,
            64.0,
            RED,
        );

        let stats_y = screen_height() / 2.0;
        let minutes = (state.time_survived / 60.0).floor() as i32;
        let seconds = (state.time_survived % 60.0).floor() as i32;
        let stats = [
            format!("Scrap Collected: {}", state.resources.scrap + 100),
            format!("Credits Earned: {}", state.resources.credits),
            format!("Time Survived: {:02}:{:02}", minutes, seconds),
        ];

        for (i, stat) in stats.iter().enumerate() {
            let s = measure_text(stat, None, 24, 1.0);
            draw_text(
                stat,
                screen_width() / 2.0 - s.width / 2.0,
                stats_y + i as f32 * 30.0,
                24.0,
                GRAY,
            );
        }

        let hint = "Press ENTER to return to menu";
        let hint_size = measure_text(hint, None, 24, 1.0);
        draw_text(
            hint,
            screen_width() / 2.0 - hint_size.width / 2.0,
            screen_height() - 80.0,
            24.0,
            WHITE,
        );
    }

    pub fn draw_victory(&self, state: &GameState) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            color_u8!(10, 20, 30, 255),
        );
        for i in 0..8 {
            let alpha = 60 - i * 7;
            let offset = i as f32 * 20.0;
            draw_rectangle_lines(
                offset,
                offset,
                screen_width() - offset * 2.0,
                screen_height() - offset * 2.0,
                2.0,
                color_u8!(100, 200, 255, alpha as u8),
            );
        }

        let text = "ESCAPE SUCCESSFUL!";
        let size = measure_text(text, None, 64, 1.0);
        draw_text(
            text,
            screen_width() / 2.0 - size.width / 2.0,
            screen_height() / 3.0,
            64.0,
            GREEN,
        );

        let subtitle = "You made it off the planet!";
        let sub_size = measure_text(subtitle, None, 28, 1.0);
        draw_text(
            subtitle,
            screen_width() / 2.0 - sub_size.width / 2.0,
            screen_height() / 3.0 + 50.0,
            28.0,
            color_u8!(150, 255, 150, 255),
        );

        let stats_y = screen_height() / 2.0;
        let stats = [
            format!("Total Credits: {}", state.resources.credits),
            format!(
                "Core Health Remaining: {:.0}%",
                if let Some(pos) = state.ship.find_core() {
                    if let Some(core) = &state.ship.grid[pos.0][pos.1] {
                        (core.health / core.max_health) * 100.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            ),
        ];

        for (i, stat) in stats.iter().enumerate() {
            let s = measure_text(stat, None, 24, 1.0);
            draw_text(
                stat,
                screen_width() / 2.0 - s.width / 2.0,
                stats_y + i as f32 * 30.0,
                24.0,
                WHITE,
            );
        }

        let hint = "Press ENTER to continue to Upgrades";
        let hint_size = measure_text(hint, None, 24, 1.0);
        draw_text(
            hint,
            screen_width() / 2.0 - hint_size.width / 2.0,
            screen_height() - 80.0,
            24.0,
            YELLOW,
        );
    }

    pub fn draw_upgrade_screen(&self, state: &GameState) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            color_u8!(15, 20, 30, 255),
        );
        let title = "SHIP IMPROVEMENTS";
        let title_w = measure_text(title, None, 48, 1.0).width;
        draw_text(title, (screen_width() - title_w) / 2.0, 60.0, 48.0, WHITE);

        let credits_text = format!("AVAILABLE CREDITS: {}", state.resources.credits);
        let cred_w = measure_text(&credits_text, None, 24, 1.0).width;
        draw_text(
            &credits_text,
            (screen_width() - cred_w) / 2.0,
            100.0,
            24.0,
            GREEN,
        );

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

            let bg_color = if is_max {
                color_u8!(40, 50, 40, 255)
            } else if can_afford {
                color_u8!(40, 40, 60, 255)
            } else {
                color_u8!(30, 30, 35, 255)
            };
            let surface = macroquad_toolkit::ui::SurfaceStyle::new(bg_color)
                .with_border(2.0, if can_afford { YELLOW } else { GRAY });
            macroquad_toolkit::ui::draw_surface(Rect::new(card_x, y, card_w, card_h), &surface);

            draw_text(
                &format!(
                    "{} (Level {}/{})",
                    template.name, current_level, template.max_level
                ),
                card_x + 15.0,
                y + 30.0,
                24.0,
                WHITE,
            );
            draw_text(&template.description, card_x + 15.0, y + 55.0, 16.0, GRAY);

            if is_max {
                draw_text("MAX LEVEL", card_x + card_w - 120.0, y + 45.0, 20.0, GREEN);
            } else {
                let cost_color = if can_afford { WHITE } else { RED };
                draw_text(
                    &format!("Cost: {} Cr", cost),
                    card_x + card_w - 150.0,
                    y + 35.0,
                    20.0,
                    cost_color,
                );
                if can_afford {
                    draw_text(
                        &format!("[{}] Buy", i + 1),
                        card_x + card_w - 150.0,
                        y + 60.0,
                        20.0,
                        YELLOW,
                    );
                } else {
                    draw_text(
                        "Insufficient Funds",
                        card_x + card_w - 150.0,
                        y + 60.0,
                        16.0,
                        RED,
                    );
                }
            }
        }

        let footer = "Press [ENTER] to start next round | Press [ESC] for Menu";
        let footer_w = measure_text(footer, None, 20, 1.0).width;
        draw_text(
            footer,
            (screen_width() - footer_w) / 2.0,
            screen_height() - 40.0,
            20.0,
            DARKGRAY,
        );
    }
}

fn menu_button_bounds(index: usize) -> (f32, f32, f32, f32) {
    let width = 270.0;
    let height = 46.0;
    let gap = 14.0;
    let x = screen_width() / 2.0 - width / 2.0;
    let start_y = screen_height() * 0.58;
    let y = start_y + index as f32 * (height + gap);

    (x, y, width, height)
}

fn draw_menu_button(bounds: (f32, f32, f32, f32), label: &str, accent: Color) {
    let (x, y, w, h) = bounds;
    draw_rectangle(x, y, w, h, color_u8!(5, 7, 9, 210));
    draw_rectangle_lines(x, y, w, h, 1.0, color_u8!(160, 164, 158, 180));
    draw_rectangle(x, y, 4.0, h, accent);

    let text_size = measure_text(label, None, 24, 1.0);
    draw_text(
        label,
        x + (w - text_size.width) / 2.0,
        y + h / 2.0 + 8.0,
        24.0,
        theme::text_primary(),
    );
}

fn draw_texture_cover(texture: &Texture2D, dest: Rect) {
    let texture_ratio = texture.width() / texture.height();
    let dest_ratio = dest.w / dest.h;

    let source = if texture_ratio > dest_ratio {
        let source_w = texture.height() * dest_ratio;
        Rect::new(
            (texture.width() - source_w) / 2.0,
            0.0,
            source_w,
            texture.height(),
        )
    } else {
        let source_h = texture.width() / dest_ratio;
        Rect::new(
            0.0,
            (texture.height() - source_h) / 2.0,
            texture.width(),
            source_h,
        )
    };

    draw_texture_ex(
        texture,
        dest.x,
        dest.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(dest.w, dest.h)),
            source: Some(source),
            ..Default::default()
        },
    );
}
