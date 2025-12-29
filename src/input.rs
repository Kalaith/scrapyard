use macroquad::prelude::*;
use crate::constants::*;
use crate::state::{GameState, GamePhase};

pub struct InputManager {
    pub last_mouse_pos: Vec2,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: Vec2::ZERO,
        }
    }
    
    pub fn update(&mut self, state: &mut GameState) {
        self.last_mouse_pos = mouse_position().into();

        match state.phase {
            GamePhase::Menu => self.handle_menu_input(state),
            GamePhase::Playing => self.handle_gameplay_input(state),
            GamePhase::GameOver => self.handle_game_over_input(state),
        }
    }

    fn handle_menu_input(&self, state: &mut GameState) {
        // Check for Enter key
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            state.start_new_game();
            return;
        }

        // Check for button click
        if is_mouse_button_pressed(MouseButton::Left) {
            let btn_width = 200.0;
            let btn_height = 50.0;
            let btn_x = screen_width() / 2.0 - btn_width / 2.0;
            let btn_y = screen_height() / 2.0 + 50.0;

            let mouse = self.last_mouse_pos;
            if mouse.x >= btn_x && mouse.x <= btn_x + btn_width &&
               mouse.y >= btn_y && mouse.y <= btn_y + btn_height {
                state.start_new_game();
            }
        }
    }

    fn handle_game_over_input(&self, state: &mut GameState) {
        // Return to menu on Enter
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            state.phase = GamePhase::Menu;
        }
    }

    fn handle_gameplay_input(&mut self, state: &mut GameState) {
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some((x, y)) = self.screen_to_grid(self.last_mouse_pos) {
                // Determine action (Contextual)
                self.handle_click(x, y, state);
            }
        }

        // Escape to return to menu (for testing)
        if is_key_pressed(KeyCode::Escape) {
            state.phase = GamePhase::Menu;
        }
    }

    fn screen_to_grid(&self, pos: Vec2) -> Option<(usize, usize)> {
        let grid_width_px = GRID_WIDTH as f32 * CELL_SIZE;
        let grid_height_px = GRID_HEIGHT as f32 * CELL_SIZE;
        
        // Assuming centered grid from render.rs logic
        let start_x = (screen_width() - grid_width_px) / 2.0;
        let start_y = (screen_height() - grid_height_px) / 2.0;

        if pos.x < start_x || pos.x > start_x + grid_width_px ||
           pos.y < start_y || pos.y > start_y + grid_height_px {
            return None;
        }

        let x = ((pos.x - start_x) / CELL_SIZE) as usize;
        let y = ((pos.y - start_y) / CELL_SIZE) as usize;

        if x < GRID_WIDTH && y < GRID_HEIGHT {
            Some((x, y))
        } else {
            None
        }
    }

    fn handle_click(&self, x: usize, y: usize, state: &mut GameState) {
        // Simple Interaction: Toggle/Repair
        
        // 1. Get Module Type and Stats (Immutable Borrow)
        let (_module_type, repair_cost, module_name) = {
            if let Some(module) = &state.ship.grid[x][y] {
                let stats = state.module_registry.get(module.module_type);
                (module.module_type, stats.base_cost, stats.name.clone())
            } else {
                return;
            }
        };

        // 2. Mutate Module (Mutable Borrow)
        if let Some(module) = &mut state.ship.grid[x][y] {
            if module.state == crate::ship::ModuleState::Destroyed {
                if state.resources.can_afford(repair_cost) {
                    state.resources.deduct(repair_cost);
                    module.state = crate::ship::ModuleState::Active; 
                    println!("Repaired {} at ({}, {}) for {}", module_name, x, y, repair_cost);
                } else {
                    println!("Not enough scrap! Needed: {}", repair_cost);
                }
            } else if module.state == crate::ship::ModuleState::Active {
                module.state = crate::ship::ModuleState::Offline;
            } else if module.state == crate::ship::ModuleState::Offline {
                module.state = crate::ship::ModuleState::Active;
            }
        }
    }
}
