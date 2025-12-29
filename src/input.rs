use macroquad::prelude::*;
use crate::constants::*;
use crate::state::{GameState, GamePhase};
use crate::events::{EventBus, UIEvent};

/// Captures current input state for the frame
#[derive(Debug, Clone)]
pub struct InputState {
    pub mouse_pos: Vec2,
    pub mouse_world_pos: Option<(usize, usize)>,
    pub left_click: bool,
    pub right_click: bool,
    pub escape_pressed: bool,
    pub enter_pressed: bool,
    pub space_pressed: bool,
    pub pause_pressed: bool,
}

impl InputState {
    pub fn capture() -> Self {
        Self {
            mouse_pos: mouse_position().into(),
            mouse_world_pos: None, // Set after screen_to_grid
            left_click: is_mouse_button_pressed(MouseButton::Left),
            right_click: is_mouse_button_pressed(MouseButton::Right),
            escape_pressed: is_key_pressed(KeyCode::Escape),
            enter_pressed: is_key_pressed(KeyCode::Enter),
            space_pressed: is_key_pressed(KeyCode::Space),
            pause_pressed: is_key_pressed(KeyCode::P),
        }
    }
}

pub struct InputManager {
    pub last_mouse_pos: Vec2,
    pub hovered_module: Option<(usize, usize)>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: Vec2::ZERO,
            hovered_module: None,
        }
    }
    
    pub fn update(&mut self, state: &mut GameState, events: &mut EventBus) {
        let mut input = InputState::capture();
        self.last_mouse_pos = input.mouse_pos;
        
        // Convert mouse to grid coords
        input.mouse_world_pos = self.screen_to_grid(input.mouse_pos);
        self.hovered_module = input.mouse_world_pos;

        match state.phase {
            GamePhase::Menu => self.handle_menu_input(&input, events),
            GamePhase::Playing => self.handle_gameplay_input(&input, state, events),
            GamePhase::GameOver | GamePhase::Victory => self.handle_game_over_input(&input, events),
        }
    }

    fn handle_menu_input(&self, input: &InputState, events: &mut EventBus) {
        // Check for Enter key
        if input.enter_pressed || input.space_pressed {
            events.push_ui(UIEvent::StartGame);
            return;
        }

        // Check for button click
        if input.left_click {
            let btn_width = 200.0;
            let btn_height = 50.0;
            let btn_x = screen_width() / 2.0 - btn_width / 2.0;
            let btn_y = screen_height() / 2.0 + 50.0;

            if input.mouse_pos.x >= btn_x && input.mouse_pos.x <= btn_x + btn_width &&
               input.mouse_pos.y >= btn_y && input.mouse_pos.y <= btn_y + btn_height {
                events.push_ui(UIEvent::StartGame);
            }
        }
    }

    fn handle_game_over_input(&self, input: &InputState, events: &mut EventBus) {
        if input.enter_pressed || input.space_pressed {
            events.push_ui(UIEvent::ReturnToMenu);
        }
    }

    fn handle_gameplay_input(&mut self, input: &InputState, state: &GameState, events: &mut EventBus) {
        // Pause toggle
        if input.pause_pressed {
            if state.paused {
                events.push_ui(UIEvent::Resume);
            } else {
                events.push_ui(UIEvent::Pause);
            }
        }

        // Escape to return to menu
        if input.escape_pressed {
            events.push_ui(UIEvent::ReturnToMenu);
        }

        // Left click on grid
        if input.left_click {
            if let Some((x, y)) = input.mouse_world_pos {
                self.handle_grid_click(x, y, state, events);
            }
        }

        // Right click for upgrade (if holding a module)
        if input.right_click {
            if let Some((x, y)) = input.mouse_world_pos {
                events.push_ui(UIEvent::Upgrade(x, y));
            }
        }
    }

    fn handle_grid_click(&self, x: usize, y: usize, state: &GameState, events: &mut EventBus) {
        if let Some(module) = &state.ship.grid[x][y] {
            match module.state {
                crate::ship::ModuleState::Destroyed => {
                    events.push_ui(UIEvent::Repair(x, y));
                }
                crate::ship::ModuleState::Active | crate::ship::ModuleState::Offline => {
                    // Check if this is the engine
                    if module.module_type == crate::ship::ModuleType::Engine 
                       && module.state == crate::ship::ModuleState::Active {
                        events.push_ui(UIEvent::ActivateEngine);
                    } else {
                        events.push_ui(UIEvent::Toggle(x, y));
                    }
                }
            }
        }
    }

    fn screen_to_grid(&self, pos: Vec2) -> Option<(usize, usize)> {
        let grid_width_px = GRID_WIDTH as f32 * CELL_SIZE;
        let grid_height_px = GRID_HEIGHT as f32 * CELL_SIZE;
        
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
}
