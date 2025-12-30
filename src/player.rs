// player.rs - Player character for interior view

use macroquad::prelude::*;
use crate::constants::*;

/// Interior scale: each grid cell becomes a large room
/// Room size in pixels = CELL_SIZE * ROOM_SCALE
pub const ROOM_SCALE: f32 = 10.0;  // Each module is a 10x normal size room
pub const TILE_SIZE: f32 = 10.0;   // Visual floor tile size (1/4 of old)
pub const PLAYER_SPEED: f32 = 300.0;
pub const PLAYER_SIZE: f32 = 8.0;

#[derive(Debug, Clone)]
pub struct Player {
    pub position: Vec2,       // World position in interior coordinates
    pub size: f32,
    pub speed: f32,
    pub facing: Vec2,         // Direction player is facing
    pub velocity: Vec2,       // Current velocity (for gathering logic)
    pub nearby_module: Option<(usize, usize)>, // Module player can interact with
}

impl Player {
    pub fn new() -> Self {
        Self::new_at(vec2(300.0, 200.0)) // Default position
    }

    pub fn new_at(pos: Vec2) -> Self {
        Self {
            position: pos,
            size: PLAYER_SIZE,
            speed: PLAYER_SPEED,
            facing: vec2(0.0, -1.0),
            velocity: Vec2::ZERO,
            nearby_module: None,
        }
    }

    /// Update player movement based on input
    pub fn update(&mut self, dt: f32, interior: &crate::interior::ShipInterior) {
        let mut move_dir = Vec2::ZERO;
        
        // WASD and Arrow key movement
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            move_dir.y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            move_dir.y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            move_dir.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            move_dir.x += 1.0;
        }

        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
            self.facing = move_dir;
            self.velocity = move_dir * self.speed;
            
            let new_pos = self.position + self.velocity * dt;
            
            // Check collision with rooms - try full movement first
            if interior.is_walkable(new_pos) {
                self.position = new_pos;
            } else {
                // Try X-only movement
                let new_x = vec2(new_pos.x, self.position.y);
                if interior.is_walkable(new_x) {
                    self.position = new_x;
                }
                // Try Y-only movement
                let new_y = vec2(self.position.x, new_pos.y);
                if interior.is_walkable(new_y) {
                    self.position = new_y;
                }
            }
        } else {
            self.velocity = Vec2::ZERO;
        }
    }

    /// Convert player position to grid coordinates
    pub fn get_grid_position(&self) -> (usize, usize) {
        let gx = (self.position.x / (CELL_SIZE * ROOM_SCALE)).floor() as usize;
        let gy = (self.position.y / (CELL_SIZE * ROOM_SCALE)).floor() as usize;
        (gx.min(GRID_WIDTH - 1), gy.min(GRID_HEIGHT - 1))
    }

    /// Check which module room player is in for interaction
    pub fn update_nearby_module(&mut self, interior: &crate::interior::ShipInterior) {
        // Check if player is in a module room
        if let Some(room) = interior.module_room_at(self.position) {
            self.nearby_module = room.module_index;
        } else {
            self.nearby_module = None;
        }
    }
}
