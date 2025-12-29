use macroquad::prelude::*;
use crate::state::{GameState, GamePhase};
use crate::constants::*;
use crate::ship::{ModuleType, ModuleState};

pub struct Renderer {
    pub trauma: f32, // Screen shake trauma (0.0 - 1.0)
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            trauma: 0.0,
        }
    }

    /// Add trauma for screen shake (clamped to 1.0)
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }

    /// Update trauma decay
    pub fn update(&mut self, dt: f32) {
        self.trauma = (self.trauma - dt * 2.0).max(0.0);
    }

    /// Get current shake offset
    fn get_shake_offset(&self) -> Vec2 {
        if self.trauma > 0.0 {
            let shake = self.trauma * self.trauma; // Quadratic for smoother feel
            let max_offset = 8.0;
            vec2(
                rand::gen_range(-1.0, 1.0) * max_offset * shake,
                rand::gen_range(-1.0, 1.0) * max_offset * shake,
            )
        } else {
            Vec2::ZERO
        }
    }

    pub fn draw(&self, state: &GameState) {
        match state.phase {
            GamePhase::Menu => self.draw_menu(),
            GamePhase::Playing => self.draw_gameplay(state),
            GamePhase::GameOver => self.draw_game_over(state),
            GamePhase::Victory => self.draw_victory(state),
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

    fn draw_game_over(&self, state: &GameState) {
        // Background with vignette effect
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(15, 5, 5, 255));
        
        // Vignette
        for i in 0..5 {
            let alpha = 100 - i * 20;
            let offset = i as f32 * 30.0;
            draw_rectangle_lines(
                offset, offset, 
                screen_width() - offset * 2.0, 
                screen_height() - offset * 2.0,
                3.0,
                color_u8!(80, 0, 0, alpha as u8),
            );
        }

        // Title
        let text = "CORE DESTROYED";
        let size = measure_text(text, None, 64, 1.0);
        draw_text(
            text,
            screen_width() / 2.0 - size.width / 2.0,
            screen_height() / 3.0,
            64.0,
            RED,
        );

        // Stats
        let stats_y = screen_height() / 2.0;
        let stats = [
            format!("Scrap Collected: {}", state.resources.scrap + 100), // Add starting scrap
            format!("Credits Earned: {}", state.resources.credits),
            format!("Frames Survived: {}", state.frame_count),
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

    fn draw_victory(&self, state: &GameState) {
        // Bright background
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(10, 20, 30, 255));
        
        // Glow effect
        for i in 0..8 {
            let alpha = 60 - i * 7;
            let offset = i as f32 * 20.0;
            draw_rectangle_lines(
                offset, offset, 
                screen_width() - offset * 2.0, 
                screen_height() - offset * 2.0,
                2.0,
                color_u8!(100, 200, 255, alpha as u8),
            );
        }

        // Title
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

        // Stats
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
            draw_text(
                stat,
                screen_width() / 2.0 - s.width / 2.0,
                stats_y + i as f32 * 30.0,
                24.0,
                WHITE,
            );
        }

        let hint = "Press ENTER to return to menu";
        let hint_size = measure_text(hint, None, 24, 1.0);
        draw_text(
            hint,
            screen_width() / 2.0 - hint_size.width / 2.0,
            screen_height() - 80.0,
            24.0,
            GRAY,
        );
    }

    /// Draws the background ship hull texture/shape.
    fn draw_ship_hull(&self, _state: &GameState) {
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
        draw_rectangle(0.0, 0.0, screen_width(), 50.0, color_u8!(20, 20, 20, 255));

        // === LEFT SECTION: Resources ===
        // Scrap with capacity warning
        let scrap_pct = state.resources.scrap as f32 / state.resources.max_scrap as f32;
        let scrap_color = if scrap_pct >= 1.0 {
            RED
        } else if scrap_pct >= 0.9 {
            YELLOW
        } else {
            WHITE
        };
        draw_text(
            &format!("SCRAP: {}/{}", state.resources.scrap, state.resources.max_scrap),
            20.0,
            32.0,
            24.0,
            scrap_color,
        );

        // Credits
        draw_text(
            &format!("CREDITS: {}", state.resources.credits),
            20.0,
            48.0,
            18.0,
            color_u8!(200, 200, 100, 255),
        );

        // === CENTER SECTION: Power Meter ===
        let meter_width = 200.0;
        let meter_height = 20.0;
        let meter_x = screen_width() / 2.0 - meter_width / 2.0;
        let meter_y = 15.0;

        // Power meter background
        draw_rectangle(meter_x, meter_y, meter_width, meter_height, color_u8!(40, 40, 40, 255));
        
        // Power level (assuming max power around 20 for scaling)
        let power = state.resources.power;
        let power_pct = (power.abs() as f32 / 20.0).min(1.0);
        
        // Color based on power level (threat)
        let power_color = if power >= 16 {
            RED
        } else if power >= 10 {
            ORANGE
        } else if power >= 5 {
            YELLOW
        } else {
            GREEN
        };
        
        draw_rectangle(meter_x, meter_y, meter_width * power_pct, meter_height, power_color);
        draw_rectangle_lines(meter_x, meter_y, meter_width, meter_height, 2.0, WHITE);
        
        // Power text
        let power_text = format!("POWER: {}", power);
        let power_size = measure_text(&power_text, None, 16, 1.0);
        draw_text(
            &power_text,
            meter_x + meter_width / 2.0 - power_size.width / 2.0,
            meter_y + 15.0,
            16.0,
            WHITE,
        );

        // === RIGHT SECTION: Timer & Status ===
        use crate::state::EngineState;
        
        match state.engine_state {
            EngineState::Idle => {
                draw_text("ENGINE: IDLE", screen_width() - 180.0, 28.0, 20.0, GRAY);
            }
            EngineState::Charging => {
                let mins = (state.escape_timer / 60.0).floor() as i32;
                let secs = (state.escape_timer % 60.0).floor() as i32;
                let timer_text = format!("ESCAPE: {:02}:{:02}", mins, secs);
                
                // Pulse red when < 20s
                let timer_color = if state.escape_timer < 20.0 {
                    let pulse = ((get_time() * 4.0).sin() * 0.5 + 0.5) as f32;
                    Color::new(1.0, pulse * 0.3, pulse * 0.3, 1.0)
                } else {
                    ORANGE
                };
                
                draw_text(&timer_text, screen_width() - 180.0, 28.0, 24.0, timer_color);
                draw_text("BOSS ACTIVE", screen_width() - 180.0, 46.0, 16.0, RED);
            }
            EngineState::Escaped => {
                draw_text("ESCAPED!", screen_width() - 180.0, 28.0, 24.0, GREEN);
            }
        }

        // === PAUSE OVERLAY ===
        if state.paused {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(0, 0, 0, 150));
            let pause_text = "PAUSED";
            let pause_size = measure_text(pause_text, None, 64, 1.0);
            draw_text(
                pause_text,
                screen_width() / 2.0 - pause_size.width / 2.0,
                screen_height() / 2.0,
                64.0,
                WHITE,
            );
            let hint = "Press P to resume, ESC to quit";
            let hint_size = measure_text(hint, None, 24, 1.0);
            draw_text(
                hint,
                screen_width() / 2.0 - hint_size.width / 2.0,
                screen_height() / 2.0 + 50.0,
                24.0,
                GRAY,
            );
        }

        // === LEFT SIDEBAR: Core Health ===
        self.draw_sidebar(state);
    }

    fn draw_sidebar(&self, state: &GameState) {
        let sidebar_width = 150.0;
        let sidebar_x = 10.0;
        let sidebar_y = 60.0;

        // Sidebar background
        draw_rectangle(sidebar_x, sidebar_y, sidebar_width, 120.0, color_u8!(25, 25, 35, 220));
        draw_rectangle_lines(sidebar_x, sidebar_y, sidebar_width, 120.0, 1.0, color_u8!(60, 60, 80, 255));

        // Find core and display health
        if let Some(core_pos) = state.ship.find_core() {
            if let Some(core) = &state.ship.grid[core_pos.0][core_pos.1] {
                // Core Health label
                draw_text("CORE STATUS", sidebar_x + 10.0, sidebar_y + 20.0, 16.0, WHITE);
                
                // Health bar
                let bar_x = sidebar_x + 10.0;
                let bar_y = sidebar_y + 30.0;
                let bar_width = sidebar_width - 20.0;
                let bar_height = 16.0;
                let hp_pct = core.health / core.max_health;

                draw_rectangle(bar_x, bar_y, bar_width, bar_height, color_u8!(60, 20, 20, 255));
                draw_rectangle(bar_x, bar_y, bar_width * hp_pct, bar_height, RED);
                draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 1.0, WHITE);

                // HP text
                draw_text(
                    &format!("{:.0}/{:.0}", core.health, core.max_health),
                    bar_x + 5.0,
                    bar_y + 12.0,
                    14.0,
                    WHITE,
                );

                // Level indicator
                draw_text(
                    &format!("Level: {}", core.level),
                    sidebar_x + 10.0,
                    sidebar_y + 65.0,
                    14.0,
                    GRAY,
                );
            }
        }

        // Enemy count
        draw_text(
            &format!("Enemies: {}", state.enemies.len()),
            sidebar_x + 10.0,
            sidebar_y + 90.0,
            14.0,
            ORANGE,
        );

        // Projectile count
        draw_text(
            &format!("Projectiles: {}", state.projectiles.len()),
            sidebar_x + 10.0,
            sidebar_y + 108.0,
            14.0,
            YELLOW,
        );
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
