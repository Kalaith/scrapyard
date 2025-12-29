use macroquad::prelude::*;
use crate::state::{GameState, GamePhase};
use crate::constants::*;
use crate::ship::{ModuleType, ModuleState};

pub struct Renderer {
    // Keep a reference to assets if needed
}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, state: &GameState) {
        match state.phase {
            GamePhase::Menu => self.draw_menu(),
            GamePhase::Playing => self.draw_gameplay(state),
            GamePhase::GameOver => self.draw_game_over(state),
        }
    }

    fn draw_gameplay(&self, state: &GameState) {
        self.draw_ship_hull(state);
        self.draw_ship_grid(state);
        self.draw_enemies(state);
        self.draw_projectiles(state);
        self.draw_particles(state);
        self.draw_hud(state);
    }

    /// Draws the main menu screen.
    pub fn draw_menu(&self) {
        // Background
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(15, 15, 25, 255));

        // Title
        let title = "SCRAPYARD PLANET";
        let title_size = measure_text(title, None, 64, 1.0);
        draw_text(
            title,
            screen_width() / 2.0 - title_size.width / 2.0,
            screen_height() / 3.0,
            64.0,
            WHITE,
        );

        // Subtitle
        let subtitle = "Repair. Defend. Escape.";
        let sub_size = measure_text(subtitle, None, 24, 1.0);
        draw_text(
            subtitle,
            screen_width() / 2.0 - sub_size.width / 2.0,
            screen_height() / 3.0 + 50.0,
            24.0,
            GRAY,
        );

        // Start button
        let btn_width = 200.0;
        let btn_height = 50.0;
        let btn_x = screen_width() / 2.0 - btn_width / 2.0;
        let btn_y = screen_height() / 2.0 + 50.0;

        // Button background
        draw_rectangle(btn_x, btn_y, btn_width, btn_height, color_u8!(60, 60, 80, 255));
        draw_rectangle_lines(btn_x, btn_y, btn_width, btn_height, 2.0, color_u8!(100, 100, 140, 255));

        // Button text
        let start_text = "START GAME";
        let start_size = measure_text(start_text, None, 28, 1.0);
        draw_text(
            start_text,
            btn_x + btn_width / 2.0 - start_size.width / 2.0,
            btn_y + btn_height / 2.0 + 8.0,
            28.0,
            WHITE,
        );

        // Instructions
        let hint = "Click START GAME or press ENTER to begin";
        let hint_size = measure_text(hint, None, 18, 1.0);
        draw_text(
            hint,
            screen_width() / 2.0 - hint_size.width / 2.0,
            screen_height() - 50.0,
            18.0,
            DARKGRAY,
        );
    }

    /// Returns the start button bounds for click detection.
    pub fn get_start_button_bounds(&self) -> (f32, f32, f32, f32) {
        let btn_width = 200.0;
        let btn_height = 50.0;
        let btn_x = screen_width() / 2.0 - btn_width / 2.0;
        let btn_y = screen_height() / 2.0 + 50.0;
        (btn_x, btn_y, btn_width, btn_height)
    }

    fn draw_game_over(&self, _state: &GameState) {
        // Background
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(20, 10, 10, 255));

        let text = "GAME OVER";
        let size = measure_text(text, None, 64, 1.0);
        draw_text(
            text,
            screen_width() / 2.0 - size.width / 2.0,
            screen_height() / 2.0,
            64.0,
            RED,
        );

        let hint = "Press ENTER to return to menu";
        let hint_size = measure_text(hint, None, 24, 1.0);
        draw_text(
            hint,
            screen_width() / 2.0 - hint_size.width / 2.0,
            screen_height() / 2.0 + 60.0,
            24.0,
            WHITE,
        );
    }

    /// Draws the background ship hull texture/shape.
    fn draw_ship_hull(&self, state: &GameState) {
        let total_width = GRID_WIDTH as f32 * CELL_SIZE;
        let total_height = GRID_HEIGHT as f32 * CELL_SIZE;
        let start_x = (screen_width() - total_width) / 2.0;
        let start_y = (screen_height() - total_height) / 2.0;

        // Outer hull border
        draw_rectangle(
            start_x - 20.0,
            start_y - 20.0,
            total_width + 40.0,
            total_height + 40.0,
            color_u8!(50, 50, 60, 255),
        );

        // Inner hull panel
        draw_rectangle(
            start_x - 10.0,
            start_y - 10.0,
            total_width + 20.0,
            total_height + 20.0,
            color_u8!(30, 30, 40, 255),
        );

        // Hull accent lines
        let accent = color_u8!(70, 70, 80, 255);
        draw_line(start_x - 20.0, start_y - 20.0, start_x + total_width + 20.0, start_y - 20.0, 2.0, accent);
        draw_line(start_x - 20.0, start_y + total_height + 20.0, start_x + total_width + 20.0, start_y + total_height + 20.0, 2.0, accent);
    }

    /// Draws the module grid with slot states.
    fn draw_ship_grid(&self, state: &GameState) {
        let total_width = GRID_WIDTH as f32 * CELL_SIZE;
        let total_height = GRID_HEIGHT as f32 * CELL_SIZE;
        let start_x = (screen_width() - total_width) / 2.0;
        let start_y = (screen_height() - total_height) / 2.0;

        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let px = start_x + x as f32 * CELL_SIZE;
                let py = start_y + y as f32 * CELL_SIZE;

                let module = &state.ship.grid[x][y];

                // Draw module base (slot background)
                self.draw_module_base(px, py, module.is_some());

                // Draw slot outline
                draw_rectangle_lines(px, py, CELL_SIZE, CELL_SIZE, 1.0, COLOR_GRID_LINE);

                if let Some(mod_data) = module {
                    self.draw_module(px, py, mod_data);
                }
            }
        }
    }

    /// Renders the underlying slot state (Empty/Module Present).
    fn draw_module_base(&self, x: f32, y: f32, has_module: bool) {
        let color = if has_module {
            color_u8!(25, 25, 30, 255) // Darker for occupied
        } else {
            color_u8!(40, 40, 50, 255) // Lighter for empty slot
        };
        draw_rectangle(x, y, CELL_SIZE, CELL_SIZE, color);
    }

    /// Draws a module with color and state overlay.
    fn draw_module(&self, x: f32, y: f32, mod_data: &crate::ship::Module) {
        let color = match mod_data.module_type {
            ModuleType::Core => RED,
            ModuleType::Weapon => ORANGE,
            ModuleType::Defense => BLUE,
            ModuleType::Utility => GREEN,
            ModuleType::Engine => PURPLE,
            ModuleType::Empty => COLOR_MODULE_EMPTY,
        };

        let padding = 2.0;
        draw_rectangle(
            x + padding,
            y + padding,
            CELL_SIZE - padding * 2.0,
            CELL_SIZE - padding * 2.0,
            color,
        );

        // State overlays
        match mod_data.state {
            ModuleState::Destroyed => {
                // X marks destroyed
                draw_line(x, y, x + CELL_SIZE, y + CELL_SIZE, 2.0, BLACK);
                draw_line(x + CELL_SIZE, y, x, y + CELL_SIZE, 2.0, BLACK);
            }
            ModuleState::Offline => {
                // Dim overlay for offline
                draw_rectangle(
                    x + padding,
                    y + padding,
                    CELL_SIZE - padding * 2.0,
                    CELL_SIZE - padding * 2.0,
                    color_u8!(0, 0, 0, 120),
                );
            }
            ModuleState::Active => {
                // Subtle glow for active (bright border)
                draw_rectangle_lines(
                    x + padding,
                    y + padding,
                    CELL_SIZE - padding * 2.0,
                    CELL_SIZE - padding * 2.0,
                    2.0,
                    WHITE,
                );
            }
        }
    }

    fn draw_hud(&self, state: &GameState) {
        // Draw Top Bar Background
        draw_rectangle(0.0, 0.0, screen_width(), 40.0, color_u8!(20, 20, 20, 255));

        // Scrap
        draw_text(
            &format!("SCRAP: {}/{}", state.resources.scrap, state.resources.max_scrap),
            20.0,
            28.0,
            24.0,
            WHITE,
        );

        // Power (Center)
        let power_text = format!("POWER: {}", state.resources.power);
        let power_size = measure_text(&power_text, None, 24, 1.0);
        draw_text(
            &power_text,
            screen_width() / 2.0 - power_size.width / 2.0,
            28.0,
            24.0,
            YELLOW,
        );

        // Timer (Right) - Placeholder
        draw_text("TIME: --:--", screen_width() - 150.0, 28.0, 24.0, GRAY);
    }

    fn draw_enemies(&self, state: &GameState) {
        for enemy in &state.enemies {
            let color = match enemy.enemy_type {
                crate::entities::EnemyType::Nanodrone => GREEN,
                crate::entities::EnemyType::Nanoguard => YELLOW,
                crate::entities::EnemyType::Leech => PURPLE,
                crate::entities::EnemyType::Boss => RED,
            };

            // Draw as simple circles for now
            draw_circle(enemy.position.x, enemy.position.y, 8.0, color);

            // Health bar if damaged
            if enemy.health < enemy.max_health {
                let bar_width = 20.0;
                let bar_height = 4.0;
                let pct = enemy.health / enemy.max_health;
                draw_rectangle(
                    enemy.position.x - bar_width / 2.0,
                    enemy.position.y - 15.0,
                    bar_width,
                    bar_height,
                    RED,
                );
                draw_rectangle(
                    enemy.position.x - bar_width / 2.0,
                    enemy.position.y - 15.0,
                    bar_width * pct,
                    bar_height,
                    GREEN,
                );
            }
        }
    }

    fn draw_projectiles(&self, state: &GameState) {
        for proj in &state.projectiles {
            draw_line(
                proj.position.x,
                proj.position.y,
                proj.position.x - proj.velocity.normalize().x * 10.0,
                proj.position.y - proj.velocity.normalize().y * 10.0,
                2.0,
                YELLOW,
            );
        }
    }

    fn draw_particles(&self, state: &GameState) {
        for particle in &state.particles {
            if particle.active {
                let alpha = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
                let color = Color::new(
                    particle.color.r,
                    particle.color.g,
                    particle.color.b,
                    particle.color.a * alpha,
                );
                draw_circle(particle.position.x, particle.position.y, 3.0, color);
            }
        }
    }
}
