use macroquad::prelude::*;
use crate::constants::*;
use crate::state::{GameState, GamePhase, ViewMode};
use crate::events::{EventBus, UIEvent};
use crate::layout::Layout;

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
        input.mouse_world_pos = Layout::screen_to_grid(input.mouse_pos);
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
                
                // Scrap Gathering Logic
                // If E is held, increment timer. If released/moving, reset.
                if is_key_down(KeyCode::E) {
                     // Check if player is NOT moving (must stay still to gather)
                     if state.player.velocity.length() < 0.1 {
                         // Find nearest pile if not already targeting one
                         if state.gathering_target.is_none() {
                             let mut nearest = None;
                             let mut min_dist = 40.0; // Interaction range
                             
                             for (i, pile) in state.scrap_piles.iter().enumerate() {
                                 if !pile.active { continue; }
                                 let d = pile.position.distance(state.player.position);
                                 if d < min_dist {
                                     min_dist = d;
                                     nearest = Some(i);
                                 }
                             }
                             state.gathering_target = nearest;
                         }
                         
                         // Process gathering
                         if let Some(target_idx) = state.gathering_target {
                             state.gathering_timer += get_frame_time();
                             
                             // 2.0 seconds to gather regular pile
                             if state.gathering_timer >= 2.0 {
                                 if target_idx < state.scrap_piles.len() {
                                     let amount = state.scrap_piles[target_idx].amount;
                                     state.resources.add_scrap(amount);
                                     state.scrap_piles[target_idx].active = false; // Mark collected
                                     events.push_ui(UIEvent::Toggle(0, 0)); // Dummy event or maybe add a Collected event later
                                     
                                     // Reset
                                     state.gathering_target = None;
                                     state.gathering_timer = 0.0;
                                 }
                             }
                         }
                     } else {
                         // Moving cancels gathering
                         state.gathering_target = None;
                         state.gathering_timer = 0.0;
                     }
                } else {
                    // Key released cancels gathering
                    state.gathering_target = None;
                    state.gathering_timer = 0.0;
                }

                // E key for interaction with repair points (Single press interact, distinct from hold-to-gather)
                // We prioritize manual repair if pressed once.
                // To avoid conflict, repair is instant click, gathering is hold.
                // But wait, attempt_interior_repair is instant?
                // Yes. So if we just click E, we might repair.
                // Let's keep repair as "Pressed" (one frame). Gathering requires "Down".
                if input.interact_pressed {
                    use crate::state::TutorialStep;
                    
                    // Handle welcome message dismissal
                    if state.tutorial_step == TutorialStep::Welcome {
                        state.tutorial_step = TutorialStep::RepairReactor;
                        // return; // Don't return, allow interaction same frame
                    }
                    
                    // Find current room and repair point
                    if let Some(room_idx) = state.interior.rooms.iter().position(|r| r.contains(state.player.position)) {
                        let room = &state.interior.rooms[room_idx];
                        if let Some(point_idx) = room.repair_point_at(state.player.position) {
                            if state.attempt_interior_repair(room_idx, point_idx) {
                                // Advance tutorial if this is the target room (just need 1 repair)
                                if let Some(target) = state.tutorial_step.target_room() {
                                    if room_idx == target {
                                        state.tutorial_step = state.tutorial_step.next();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ViewMode::Exterior => {
                // Interaction removed as per request
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
}
