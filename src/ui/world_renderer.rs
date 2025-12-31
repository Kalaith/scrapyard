use macroquad::prelude::*;
use crate::state::{GameState, ViewMode};
use crate::simulation::constants::*;
use crate::ship::ship::{ModuleType, ModuleState, Module};
use crate::ship::interior::{REPAIR_POINT_SIZE, RoomType};
use crate::ui::renderer::Renderer;

impl Renderer {
    pub fn draw_gameplay(&self, state: &GameState) {
        // Get screen shake offset for trauma feedback
        let shake = self.get_shake_offset();
        
        match state.view_mode {
            ViewMode::Exterior => {
                // self.draw_ship_hull(state); // Removed per user request
                self.draw_ship_grid(state);
                self.draw_enemies(state, shake);
                self.draw_projectiles(state, shake);
                self.draw_particles(state, shake);
            }
            ViewMode::Interior => {
                self.draw_interior(state);
            }
        }
        
        // Draw HUD with stats (always visible)
        self.draw_hud(state);
        
        // View mode indicator
        let mode_text = match state.view_mode {
            ViewMode::Exterior => "EXTERIOR [Tab]",
            ViewMode::Interior => "INTERIOR [Tab]",
        };
        draw_text(mode_text, screen_width() - 150.0, screen_height() - 20.0, 18.0, GRAY);
        
        // Tutorial overlay
        if !state.tutorial_state.is_complete() {
            self.draw_tutorial(state);
        }
    }
    
    fn draw_hud(&self, state: &GameState) {
        // HUD background bar at top
        draw_rectangle(0.0, 0.0, screen_width(), 35.0, color_u8!(0, 0, 0, 180));
        
        // Power info
        // Power info
        let max_power: i32 = state.interior.rooms.iter()
            .filter(|r| matches!(r.room_type, crate::ship::interior::RoomType::Module(ModuleType::Core)))
            .map(|r| r.repair_points.len() as i32 * POWER_PER_CORE_POINT)
            .sum();

        let power_color = if state.used_power <= state.total_power { GREEN } else { RED };
        let power_text = format!("Power: {}/{} [{}]", state.used_power, state.total_power, max_power);
        draw_text(&power_text, 20.0, 24.0, 20.0, power_color);
        
        // Scrap
        let scrap_text = format!("Scrap: {}", state.resources.scrap);
        draw_text(&scrap_text, 180.0, 24.0, 20.0, ORANGE);
        
        // Credits
        let credits_text = format!("Credits: {}", state.resources.credits);
        draw_text(&credits_text, 320.0, 24.0, 20.0, YELLOW);
        
        // Ship integrity
        let hp_pct = state.ship_integrity / state.ship_max_integrity;
        let hp_color = if hp_pct > 0.6 { GREEN } else if hp_pct > 0.3 { YELLOW } else { RED };
        let hp_text = format!("Hull: {:.0}/{:.0}", state.ship_integrity, state.ship_max_integrity);
        draw_text(&hp_text, 480.0, 24.0, 20.0, hp_color);

        // Engine Status
        let (stress_text, stress_color) = if state.engine_stress >= STRESS_THRESHOLD_CRITICAL {
            ("ENGINE: CASCADE", RED)
        } else if state.engine_stress >= STRESS_THRESHOLD_UNSTABLE {
            ("ENGINE: UNSTABLE", ORANGE)
        } else if state.engine_stress >= STRESS_THRESHOLD_STRAINED {
            ("ENGINE: STRAINED", YELLOW)
        } else {
             if state.engine_stress > 0.0 {
                 ("ENGINE: WARM", GREEN)
             } else {
                 ("ENGINE: STABLE", BLUE)
             }
        };
        // Shake text if critical
        let (dx, dy) = if state.engine_stress >= STRESS_THRESHOLD_CRITICAL { 
             (macroquad::rand::gen_range(-2.0, 2.0), macroquad::rand::gen_range(-2.0, 2.0))
        } else { (0.0, 0.0) };
        draw_text(stress_text, 680.0 + dx, 24.0 + dy, 20.0, stress_color);
        
        // Nanite Alert
        let alert_x = 900.0;
        draw_text("Alert:", alert_x, 24.0, 20.0, WHITE);
        draw_rectangle(alert_x + 60.0, 10.0, 100.0, 14.0, DARKGRAY);
        let alert_pct = (state.nanite_alert / 50.0).clamp(0.0, 1.0);
        draw_rectangle(alert_x + 60.0, 10.0, 100.0 * alert_pct, 14.0, RED);
        
        // Engine/Escape timer (if charging)
        if state.engine_state == crate::state::EngineState::Charging {
            let mins = (state.escape_timer / 60.0).floor() as i32;
            let secs = (state.escape_timer % 60.0).floor() as i32;
            let escape_text = format!("ESCAPE: {:02}:{:02}", mins, secs);
            draw_text(&escape_text, screen_width() - 180.0, 48.0, 20.0, SKYBLUE);
        }
    }

    pub fn draw_interior(&self, state: &GameState) {
        let interior = &state.interior;
        
        // Camera offset to center on player
        let cam_x = if interior.width < screen_width() {
            (screen_width() - interior.width) / 2.0
        } else {
            (screen_width() / 2.0 - state.player.position.x)
                .clamp(screen_width() - interior.width, 0.0)
        };
        let cam_y = if interior.height < screen_height() {
            (screen_height() - interior.height) / 2.0
        } else {
            (screen_height() / 2.0 - state.player.position.y)
                .clamp(screen_height() - interior.height, 0.0)
        };
        
        // Background (void)
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color_u8!(10, 10, 15, 255));
        
        // Draw all interior elements
        self.draw_rooms(state, cam_x, cam_y);
        self.draw_player(state, cam_x, cam_y);
        self.draw_scrap_piles(state, cam_x, cam_y);
        self.draw_repair_prompt(state, cam_x, cam_y);
    }
    
    fn draw_rooms(&self, state: &GameState, cam_x: f32, cam_y: f32) {
        for room in &state.interior.rooms {
            let rx = cam_x + room.x;
            let ry = cam_y + room.y;
            
            draw_rectangle(rx, ry, room.width, room.height, room.color());
            
            // Tutorial highlight
            let is_target = state.tutorial_state.should_highlight(&state.tutorial_config, room.id);
            if is_target && !room.is_fully_repaired() {
                let pulse = ((state.frame_count as f32 * 0.1).sin() * 0.5 + 0.5) * 155.0 + 100.0;
                draw_rectangle_lines(rx - 2.0, ry - 2.0, room.width + 4.0, room.height + 4.0, 4.0, 
                    Color::new(1.0, 1.0, 0.0, pulse / 255.0));
            } else {
                draw_rectangle_lines(rx, ry, room.width, room.height, 2.0, color_u8!(70, 70, 80, 255));
            }
            
            // Repair points
            for point in &room.repair_points {
                let px = rx + point.x;
                let py = ry + point.y;
                let half = REPAIR_POINT_SIZE / 2.0;
                
                if point.repaired {
                    draw_rectangle(px - half, py - half, half * 2.0, half * 2.0, color_u8!(30, 100, 30, 255));
                    draw_rectangle_lines(px - half, py - half, half * 2.0, half * 2.0, 2.0, GREEN);
                } else {
                    draw_rectangle(px - half, py - half, half * 2.0, half * 2.0, color_u8!(100, 40, 30, 255));
                    draw_rectangle_lines(px - half, py - half, half * 2.0, half * 2.0, 2.0, ORANGE);
                }
            }
            
            // Room name and progress
            let name = room.name();
            if !name.is_empty() {
                let text_size = 18.0;
                let text_w = measure_text(name, None, text_size as u16, 1.0).width;
                draw_text(name, rx + (room.width - text_w) / 2.0, ry + 24.0, text_size, WHITE);
                
                if !room.repair_points.is_empty() {
                    let progress = format!("{}/{}", room.repaired_count(), room.repair_points.len());
                    let prog_w = measure_text(&progress, None, 14, 1.0).width;
                    draw_text(&progress, rx + (room.width - prog_w) / 2.0, ry + 42.0, 14.0, 
                        if room.is_fully_repaired() { GREEN } else { ORANGE });
                }
            }
        }
    }
    
    fn draw_player(&self, state: &GameState, cam_x: f32, cam_y: f32) {
        let player_screen_x = cam_x + state.player.position.x;
        let player_screen_y = cam_y + state.player.position.y;
        
        draw_circle(player_screen_x, player_screen_y, state.player.size, color_u8!(100, 200, 255, 255));
        draw_circle_lines(player_screen_x, player_screen_y, state.player.size, 2.0, WHITE);
        
        let facing_end = vec2(player_screen_x, player_screen_y) + state.player.facing * state.player.size;
        draw_line(player_screen_x, player_screen_y, facing_end.x, facing_end.y, 2.0, WHITE);
        
        // Gathering progress bar
        if state.gathering_target.is_some() && state.gathering_timer > 0.0 {
            let progress = (state.gathering_timer / GATHERING_TIME_SECONDS).clamp(0.0, 1.0);
            let bar_w = 40.0;
            let bar_h = 6.0;
            let px = player_screen_x - bar_w / 2.0;
            let py = player_screen_y - 30.0;
            
            draw_rectangle(px, py, bar_w, bar_h, BLACK);
            draw_rectangle(px, py, bar_w * progress, bar_h, GREEN);
        }
    }
    
    fn draw_scrap_piles(&self, state: &GameState, cam_x: f32, cam_y: f32) {
        for pile in &state.scrap_piles {
            if !pile.active { continue; }
            let screen_pos_x = cam_x + pile.position.x;
            let screen_pos_y = cam_y + pile.position.y;
            
            draw_circle(screen_pos_x, screen_pos_y, 8.0, BROWN);
            draw_circle(screen_pos_x, screen_pos_y, 6.0, DARKBROWN);
            
            if pile.position.distance(state.player.position) < INTERACTION_RANGE {
                draw_circle_lines(screen_pos_x, screen_pos_y, 12.0, 2.0, YELLOW);
                if state.gathering_target.is_none() {
                    draw_text("[Hold E] Scavenge", screen_pos_x - 40.0, screen_pos_y - 15.0, 16.0, WHITE);
                }
            }
        }
    }
    
    fn draw_repair_prompt(&self, state: &GameState, cam_x: f32, cam_y: f32) {
        let interior = &state.interior;
        let Some(room) = interior.room_at(state.player.position) else { return };
        let Some(point_idx) = room.repair_point_at(state.player.position) else { return };
        if room.repair_points[point_idx].repaired { return; }
        
        let Some(room_idx) = interior.rooms.iter().position(|r| r.id == room.id) else { return };
        let Some((scrap_cost, power_cost)) = state.get_repair_cost(room_idx, point_idx) else { return };
        
        let player_screen_x = cam_x + state.player.position.x;
        let player_screen_y = cam_y + state.player.position.y;
        
        let is_reactor = power_cost == 0;
        let can_afford_scrap = state.resources.scrap >= scrap_cost;
        let can_afford_power = is_reactor || (state.used_power + power_cost <= state.total_power);
        
        let cost_text = if is_reactor {
            format!("{scrap_cost} Scrap")
        } else {
            format!("{scrap_cost} Scrap + {power_cost} Power")
        };
        
        let label = if can_afford_scrap && can_afford_power {
            format!("[E] Repair ({})", cost_text)
        } else if !can_afford_scrap {
            format!("Need {scrap_cost} Scrap")
        } else {
            format!("Need {power_cost} Power (Repair Reactor)")
        };
        
        let color = if can_afford_scrap && can_afford_power { YELLOW } else { RED };
        draw_text(&label, player_screen_x - 60.0, player_screen_y - 20.0, 16.0, color);
    }

    pub fn draw_ship_hull(&self, _state: &GameState) {
        let total_width = GRID_WIDTH as f32 * CELL_SIZE;
        let total_height = GRID_HEIGHT as f32 * CELL_SIZE;
        let start_x = (screen_width() - total_width) / 2.0;
        let start_y = (screen_height() - total_height) / 2.0;

        draw_rectangle(start_x - 20.0, start_y - 20.0, total_width + 40.0, total_height + 40.0, color_u8!(50, 50, 60, 255));
        draw_rectangle(start_x - 10.0, start_y - 10.0, total_width + 20.0, total_height + 20.0, color_u8!(30, 30, 40, 255));

        let accent = color_u8!(70, 70, 80, 255);
        draw_line(start_x - 20.0, start_y - 20.0, start_x + total_width + 20.0, start_y - 20.0, 2.0, accent);
        draw_line(start_x - 20.0, start_y + total_height + 20.0, start_x + total_width + 20.0, start_y + total_height + 20.0, 2.0, accent);
    }

    pub fn draw_ship_grid(&self, state: &GameState) {
        let total_width = GRID_WIDTH as f32 * CELL_SIZE;
        let total_height = GRID_HEIGHT as f32 * CELL_SIZE;
        let start_x = (screen_width() - total_width) / 2.0;
        let start_y = (screen_height() - total_height) / 2.0;



        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let px = start_x + x as f32 * CELL_SIZE;
                let py = start_y + y as f32 * CELL_SIZE;
                let module = &state.ship.grid[x][y];

                if let Some(mod_data) = module {
                    self.draw_module_base(px, py, true);
                    draw_rectangle_lines(px, py, CELL_SIZE, CELL_SIZE, 1.0, COLOR_GRID_LINE);
                    self.draw_module(px, py, mod_data);
                } else {
                    // Draw nothing for empty space
                }
            }
        }
        
        // Draw weapon ranges OVER grid
        self.draw_weapon_ranges(state, start_x, start_y);
    }

    fn draw_weapon_ranges(&self, state: &GameState, start_x: f32, start_y: f32) {
        let base_range = state.module_registry.get(ModuleType::Weapon).range;
        
        for room in &state.interior.rooms {
            if room.room_type == RoomType::Module(ModuleType::Weapon) {
                if !room.repair_points.is_empty() {
                    let repaired = room.repaired_count();
                    if repaired > 0 {
                        let repair_pct = repaired as f32 / room.repair_points.len() as f32;
                        let effective_range = base_range * (0.5 + 0.5 * repair_pct);
                        
                        if let Some((gx, gy)) = room.module_index {
                             // Locate module center on screen
                             // Since we don't have Layout::grid_to_screen_center easily here without offsets,
                             // we calculate it using start_x/y
                             // Note: We need width/height of module to center properly.
                             // Assuming module spans room's grid cells.
                             // Actually, module_index points to top-left of module in grid.
                             // We should check the module grid size? Or just use the grid coord + center offset
                             // Weapon modules are typically 2x2 in starter ship (256x256 room -> 256/40 ~ 6.4 cells? Wait)
                             // Ship interior coordinates are different from Grid coordinates.
                             // Ship Exterior Grid is 20x15.
                             
                             // Wait, I need to know how many cells the module occupies in the Exterior Grid.
                             // room.module_index (gx, gy) are Exterior Grid coordinates.
                             // To center the circle, we need the center of the module in Exterior Grid space.
                             
                             // Let's assume typical weapon is 2x2? 
                             // Better: Check the grid at gx, gy. If it's part of a multi-cell module, how do we find center?
                             // Since we iterate rooms, and room has module_index (gx, gy), 
                             // lets just assume center of that specific cell (gx, gy) or center of the room in grid space?
                             // In `combat.rs`, `fire_towers` uses `Layout::grid_to_screen_center(gx, gy)`.
                             // Layout uses generic constants.
                             // Here we have `start_x, start_y`.
                             
                             // Let's rely on the fact that for now, weapons might be 1x1 or we just draw from top-left offset.
                             // Ideally we draw from the center of the module.
                             // Let's try to infer module size or just reuse `module_index` which is the "primary" cell.
                             // If `combat.rs` uses `grid_to_screen_center(gx, gy)`, I should match that.
                             
                             // calculate center of cell (gx, gy)
                             let cx = start_x + gx as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                             let cy = start_y + gy as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                             
                             // Draw circle
                             draw_circle_lines(cx, cy, effective_range, 2.0, color_u8!(255, 100, 0, 100)); // Faint Orange
                        }
                    }
                }
            }
        }
    }

    pub fn draw_module_base(&self, x: f32, y: f32, has_module: bool) {
        let color = if has_module { color_u8!(25, 25, 30, 255) } else { color_u8!(40, 40, 50, 255) };
        draw_rectangle(x, y, CELL_SIZE, CELL_SIZE, color);
    }

    pub fn draw_module(&self, x: f32, y: f32, mod_data: &Module) {
        let color = match mod_data.module_type {
            ModuleType::Core => RED,
            ModuleType::Weapon => ORANGE,
            ModuleType::Defense => BLUE,
            ModuleType::Utility => GREEN,
            ModuleType::Engine => PURPLE,
            ModuleType::Empty => COLOR_MODULE_EMPTY,
        };

        let padding = 2.0;
        draw_rectangle(x + padding, y + padding, CELL_SIZE - padding * 2.0, CELL_SIZE - padding * 2.0, color);

        match mod_data.state {
            ModuleState::Destroyed => {
                draw_line(x, y, x + CELL_SIZE, y + CELL_SIZE, 2.0, BLACK);
                draw_line(x + CELL_SIZE, y, x, y + CELL_SIZE, 2.0, BLACK);
            }
            ModuleState::Offline => {
                draw_rectangle(x + padding, y + padding, CELL_SIZE - padding * 2.0, CELL_SIZE - padding * 2.0, color_u8!(0, 0, 0, 120));
            }
            ModuleState::Active => {
                draw_rectangle_lines(x + padding, y + padding, CELL_SIZE - padding * 2.0, CELL_SIZE - padding * 2.0, 2.0, WHITE);
            }
        }
    }

    pub fn draw_enemies(&self, state: &GameState, shake: Vec2) {
        for enemy in &state.enemies {
            let color = match enemy.enemy_type {
                crate::enemy::entities::EnemyType::Nanodrone => GREEN,
                crate::enemy::entities::EnemyType::Nanoguard => YELLOW,
                crate::enemy::entities::EnemyType::Leech => PURPLE,
                crate::enemy::entities::EnemyType::SiegeConstruct => DARKGRAY,
                crate::enemy::entities::EnemyType::Boss => RED,
            };

            let ex = enemy.position.x + shake.x;
            let ey = enemy.position.y + shake.y;
            draw_circle(ex, ey, 8.0, color);

            if enemy.health < enemy.max_health {
                let bar_width = 20.0;
                let bar_height = 4.0;
                let pct = enemy.health / enemy.max_health;
                draw_rectangle(ex - bar_width / 2.0, ey - 15.0, bar_width, bar_height, RED);
                draw_rectangle(ex - bar_width / 2.0, ey - 15.0, bar_width * pct, bar_height, GREEN);
            }
        }
    }

    pub fn draw_projectiles(&self, state: &GameState, shake: Vec2) {
        for proj in &state.projectiles {
            let px = proj.position.x + shake.x;
            let py = proj.position.y + shake.y;
            draw_line(
                px,
                py,
                px - proj.velocity.normalize().x * 10.0,
                py - proj.velocity.normalize().y * 10.0,
                2.0,
                YELLOW,
            );
        }
    }

    pub fn draw_particles(&self, state: &GameState, shake: Vec2) {
        for particle in &state.particles {
            if particle.active {
                let alpha = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
                let color = Color::new(particle.color.r, particle.color.g, particle.color.b, particle.color.a * alpha);
                draw_circle(particle.position.x + shake.x, particle.position.y + shake.y, 3.0, color);
            }
        }
    }
}
