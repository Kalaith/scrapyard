use crate::state::GameState;
use crate::ui::renderer::Renderer;
use crate::ui::theme;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

impl Renderer {
    pub fn draw_menu(&self, state: &GameState) {
        let has_background = self.draw_menu_background(state);

        if !has_background {
            self.draw_fallback_menu_title();
        }

        self.draw_menu_buttons();
        self.draw_menu_records(state);
    }

    /// Best-run records shown top-left on the menu — score-attack framing.
    fn draw_menu_records(&self, state: &GameState) {
        let p = &state.profile;
        if p.runs_completed == 0 && p.credits_left_behind == 0 {
            return; // Nothing earned yet; keep the first-run menu clean.
        }
        let best_time = p
            .best_time
            .map(|t| format!("{:02}:{:02}", (t / 60.0) as i32, (t % 60.0) as i32))
            .unwrap_or_else(|| "--:--".to_string());
        let lines = [
            "SALVAGE RECORD".to_string(),
            format!("Escapes: {}", p.runs_completed),
            format!("Best payout: {} CR", p.best_payout.unwrap_or(0)),
            format!("Best time: {}", best_time),
            format!("Left behind: {} CR", p.credits_left_behind),
        ];
        let x = 24.0;
        let mut y = 90.0;
        for (i, line) in lines.iter().enumerate() {
            let (size, color) = if i == 0 {
                (18.0, theme::cyan())
            } else {
                (16.0, theme::text_muted())
            };
            draw_ui_text(line, x, y, size, color);
            y += size + 8.0;
        }
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
        let title_size = measure_ui_text(title, None, 64, 1.0);
        draw_ui_text(
            title,
            screen_width() / 2.0 - title_size.width / 2.0,
            screen_height() / 3.0,
            64.0,
            WHITE,
        );

        let subtitle = "Repair. Defend. Escape.";
        let sub_size = measure_ui_text(subtitle, None, 24, 1.0);
        draw_ui_text(
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
        let size = measure_ui_text(text, None, 64, 1.0);
        draw_ui_text(
            text,
            screen_width() / 2.0 - size.width / 2.0,
            screen_height() / 3.0,
            64.0,
            RED,
        );

        let stats_y = screen_height() / 2.0 - 40.0;
        let minutes = (state.time_survived / 60.0).floor() as i32;
        let seconds = (state.time_survived % 60.0).floor() as i32;

        // Death forensics: prove the death was a chosen death, not an unfair one.
        // The honest "value left behind" is the full escape payout the player forfeited by
        // not launching — nothing is banked on death.
        let foregone = state.calculate_payout().total;
        let (headline, headline_color) = if let Some(ready) = state.engine_ready_at {
            let rm = (ready / 60.0).floor() as i32;
            let rs = (ready % 60.0).floor() as i32;
            (
                format!("ESCAPE WAS AVAILABLE AT {:02}:{:02} — YOU STAYED", rm, rs),
                color_u8!(255, 180, 80, 255),
            )
        } else {
            (
                "THE ENGINE NEVER REACHED ESCAPE-READY".to_string(),
                color_u8!(200, 120, 120, 255),
            )
        };
        let head_size = measure_ui_text(&headline, None, 22, 1.0);
        draw_ui_text(
            &headline,
            screen_width() / 2.0 - head_size.width / 2.0,
            stats_y - 34.0,
            22.0,
            headline_color,
        );

        let mut stats: Vec<String> = Vec::new();
        if let Some(ready) = state.engine_ready_at {
            let extra = (state.time_survived - ready).max(0.0) as i32;
            stats.push(format!("You stayed {}s longer, chasing the payout", extra));
        }
        stats.push(format!("Value left behind: {} credits", foregone));
        stats.push(format!("Died holding: {} scrap", state.resources.scrap));
        stats.push(format!("Kills: {}", state.enemies_destroyed));
        stats.push(format!("Time Survived: {:02}:{:02}", minutes, seconds));
        stats.push("Banked this run: 0 — a death costs everything".to_string());

        for (i, stat) in stats.iter().enumerate() {
            let s = measure_ui_text(stat, None, 24, 1.0);
            draw_ui_text(
                stat,
                screen_width() / 2.0 - s.width / 2.0,
                stats_y + i as f32 * 30.0,
                24.0,
                GRAY,
            );
        }

        let hint = "Press ENTER to return to menu";
        let hint_size = measure_ui_text(hint, None, 24, 1.0);
        draw_ui_text(
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
        let size = measure_ui_text(text, None, 64, 1.0);
        draw_ui_text(
            text,
            screen_width() / 2.0 - size.width / 2.0,
            screen_height() / 3.0,
            64.0,
            GREEN,
        );

        let subtitle = "You made it off the planet!";
        let sub_size = measure_ui_text(subtitle, None, 28, 1.0);
        draw_ui_text(
            subtitle,
            screen_width() / 2.0 - sub_size.width / 2.0,
            screen_height() / 3.0 + 50.0,
            28.0,
            color_u8!(150, 255, 150, 255),
        );

        let stats_y = screen_height() / 2.0 - 40.0;
        let payout = state
            .last_payout
            .unwrap_or_else(|| state.calculate_payout());
        let stats = [
            format!("Contract Base: +{} credits", payout.base),
            format!("Systems Repaired: +{} credits", payout.repaired_bonus),
            format!("Systems Powered: +{} credits", payout.powered_bonus),
            format!("Hull Condition: +{} credits", payout.hull_bonus),
            format!("Scrap Reserve: +{} credits", payout.scrap_bonus),
            format!("Combat Bonus: +{} credits", payout.combat_bonus),
            format!("Risk Bonus: +{} credits", payout.risk_bonus),
            format!("Stress Penalties: -{} credits", payout.penalties),
            format!("Total Paid: {} credits", payout.total),
        ];

        for (i, stat) in stats.iter().enumerate() {
            let s = measure_ui_text(stat, None, 24, 1.0);
            draw_ui_text(
                stat,
                screen_width() / 2.0 - s.width / 2.0,
                stats_y + i as f32 * 30.0,
                24.0,
                WHITE,
            );
        }

        let hint = "Press ENTER to continue to Upgrades";
        let hint_size = measure_ui_text(hint, None, 24, 1.0);
        draw_ui_text(
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
        let title = "SALVAGE GUILD UPGRADES";
        let title_w = measure_ui_text(title, None, 48, 1.0).width;
        draw_ui_text(title, (screen_width() - title_w) / 2.0, 60.0, 48.0, WHITE);

        let credits_text = format!(
            "BANKED CREDITS: {}   LIFETIME: {}   ESCAPES: {}",
            state.profile.banked_credits,
            state.profile.lifetime_credits,
            state.profile.runs_completed
        );
        let cred_w = measure_ui_text(&credits_text, None, 24, 1.0).width;
        draw_ui_text(
            &credits_text,
            (screen_width() - cred_w) / 2.0,
            100.0,
            24.0,
            GREEN,
        );

        let start_y = 146.0;
        let card_w = 560.0;
        let card_h = 76.0;
        let spacing_x = 24.0;
        let spacing_y = 12.0;
        let total_w = card_w * 2.0 + spacing_x;
        let card_x = (screen_width() - total_w) / 2.0;

        for (i, template) in state.upgrade_templates.iter().enumerate() {
            let col = i % 2;
            let row = i / 2;
            let x = card_x + col as f32 * (card_w + spacing_x);
            let y = start_y + row as f32 * (card_h + spacing_y);
            let current_level = state.profile.upgrade_level(&template.id);
            let is_max = current_level >= template.max_level;
            let cost = state.profile.upgrade_cost(template);
            let can_afford = state.profile.banked_credits >= cost && !is_max;

            let bg_color = if is_max {
                color_u8!(40, 50, 40, 255)
            } else if can_afford {
                color_u8!(40, 40, 60, 255)
            } else {
                color_u8!(30, 30, 35, 255)
            };
            let surface = macroquad_toolkit::ui::SurfaceStyle::new(bg_color)
                .with_border(2.0, if can_afford { YELLOW } else { GRAY });
            macroquad_toolkit::ui::draw_surface(Rect::new(x, y, card_w, card_h), &surface);

            draw_ui_text(
                &format!(
                    "[{}] {} ({}/{})",
                    i + 1,
                    template.name,
                    current_level,
                    template.max_level
                ),
                x + 15.0,
                y + 30.0,
                24.0,
                WHITE,
            );
            let description = fit_card_text(&template.description, card_w - 150.0, 15);
            draw_ui_text(&description, x + 15.0, y + 55.0, 15.0, GRAY);

            if is_max {
                draw_ui_text("MAX", x + card_w - 68.0, y + 44.0, 20.0, GREEN);
            } else {
                let cost_color = if can_afford { WHITE } else { RED };
                draw_ui_text(
                    &format!("{} CR", cost),
                    x + card_w - 105.0,
                    y + 32.0,
                    20.0,
                    cost_color,
                );
            }
        }

        let footer =
            "Number keys buy permanent upgrades | Enter starts next run | Esc returns to menu";
        let footer_w = measure_ui_text(footer, None, 20, 1.0).width;
        draw_ui_text(
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

    let text_size = measure_ui_text(label, None, 24, 1.0);
    draw_ui_text(
        label,
        x + (w - text_size.width) / 2.0,
        y + h / 2.0 + 8.0,
        24.0,
        theme::text_primary(),
    );
}

fn fit_card_text(text: &str, max_width: f32, font_size: u16) -> String {
    if measure_ui_text(text, None, font_size, 1.0).width <= max_width {
        return text.to_string();
    }

    let mut fitted = text.to_string();
    while fitted.len() > 4 {
        fitted.pop();
        let candidate = format!("{}...", fitted.trim_end());
        if measure_ui_text(&candidate, None, font_size, 1.0).width <= max_width {
            return candidate;
        }
    }

    "...".to_string()
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
