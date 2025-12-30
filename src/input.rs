use macroquad::prelude::*;
use crate::constants::*;
use crate::state::{GameState, GamePhase, ViewMode};
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
    pub tab_pressed: bool,
    pub interact_pressed: bool,
}

impl InputState {
    pub fn capture() -> Self {
        Self {
            mouse_pos: mouse_position().into(),
            mouse_world_pos: None,
            left_click: is_mouse_button_pressed(MouseButton::Left),
            right_click: is_mouse_button_pressed(MouseButton::Right),
            escape_pressed: is_key_pressed(KeyCode::Escape),
            enter_pressed: is_key_pressed(KeyCode::Enter),
            space_pressed: is_key_pressed(KeyCode::Space),
            pause_pressed: is_key_pressed(KeyCode::P),
            tab_pressed: is_key_pressed(KeyCode::Tab),
            interact_pressed: is_key_pressed(KeyCode::E),
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
        if input.enter_pressed || input.space_pressed {
            events.push_ui(UIEvent::StartGame);
            return;
        }

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

    fn handle_gameplay_input(&mut self, input: &InputState, state: &mut GameState, events: &mut EventBus) {
        // Tab toggles view mode
        if input.tab_pressed {
            state.view_mode = match state.view_mode {
                ViewMode::Interior => ViewMode::Exterior,
                ViewMode::Exterior => ViewMode::Interior,
            };
        }

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

        // View-specific input
        match state.view_mode {
            ViewMode::Interior => {
                // Player movement is handled in player.update()
                // E key for interaction with repair points
                if input.interact_pressed {
                    use crate::state::TutorialStep;
                    
                    // Handle welcome message dismissal
                    if state.tutorial_step == TutorialStep::Welcome {
                        state.tutorial_step = TutorialStep::RepairReactor;
                        return;
                    }
                    
                    // Find current room and repair point
                    if let Some(room_idx) = state.interior.rooms.iter().position(|r| r.contains(state.player.position)) {
                        let room = &state.interior.rooms[room_idx];
                        if let Some(point_idx) = room.repair_point_at(state.player.position) {
                            if !room.repair_points[point_idx].repaired {
                                // Determine costs
                                let scrap_cost = 10;
                                let is_reactor = match room.room_type {
                                    crate::interior::RoomType::Module(crate::ship::ModuleType::Core) => true,
                                    _ => false,
                                };
                                
                                let power_cost = match room.room_type {
                                    crate::interior::RoomType::Module(crate::ship::ModuleType::Core) => 0,
                                    crate::interior::RoomType::Module(crate::ship::ModuleType::Weapon) => 2,
                                    crate::interior::RoomType::Module(crate::ship::ModuleType::Defense) => 2,
                                    crate::interior::RoomType::Module(crate::ship::ModuleType::Utility) => 1,
                                    crate::interior::RoomType::Module(crate::ship::ModuleType::Engine) => 3,
                                    crate::interior::RoomType::Cockpit => 2,
                                    crate::interior::RoomType::Medbay => 1,
                                    _ => 0,
                                };
                                
                                // Check affordability
                                if state.resources.scrap < scrap_cost {
                                    // TODO: Show visual feedback "Not enough scrap" (maybe via UI event)
                                    return;
                                }
                                
                                // Check power limit (unless it's the reactor itself)
                                if !is_reactor {
                                    if state.used_power + power_cost > state.total_power {
                                        // TODO: Show visual feedback "Not enough power"
                                        return;
                                    }
                                }
                                
                                // Deduct costs and repair
                                state.resources.scrap -= scrap_cost;
                                state.interior.rooms[room_idx].repair_points[point_idx].repaired = true;
                                
                                if power_cost > 0 {
                                    state.used_power += power_cost; // Update immediate usage tracking locally if needed, but next frame update_power fixes it
                                }
                                
                                // Advance tutorial if this is the target room (just need 1 repair)
                                if let Some(target) = state.tutorial_step.target_room() {
                                    if room_idx == target {
                                        state.tutorial_step = state.tutorial_step.next();
                                    }
                                }
                                
                                // Check if room is now fully repaired - activate module
                                if state.interior.rooms[room_idx].is_fully_repaired() {
                                    if let Some((gx, gy)) = state.interior.rooms[room_idx].module_index {
                                        if let Some(module) = &mut state.ship.grid[gx][gy] {
                                            module.state = crate::ship::ModuleState::Active;
                                            module.health = module.max_health;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ViewMode::Exterior => {
                // Mouse-based interaction (legacy)
                if input.left_click {
                    if let Some((x, y)) = input.mouse_world_pos {
                        self.handle_grid_click(x, y, state, events);
                    }
                }

                if input.right_click {
                    if let Some((x, y)) = input.mouse_world_pos {
                        events.push_ui(UIEvent::Upgrade(x, y));
                    }
                }
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
