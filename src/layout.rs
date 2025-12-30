use macroquad::prelude::*;
use crate::constants::{GRID_WIDTH, CELL_SIZE, GRID_HEIGHT};

pub struct Layout;

impl Layout {
    /// Convert grid coordinates to screen position (top-left of cell)
    pub fn grid_to_screen(x: usize, y: usize) -> Vec2 {
        let start_x = (screen_width() - GRID_WIDTH as f32 * CELL_SIZE) / 2.0;
        let start_y = (screen_height() - GRID_HEIGHT as f32 * CELL_SIZE) / 2.0;
        vec2(
            start_x + x as f32 * CELL_SIZE,
            start_y + y as f32 * CELL_SIZE,
        )
    }

    /// Convert grid coordinates to screen position (center of cell)
    pub fn grid_to_screen_center(x: usize, y: usize) -> Vec2 {
        let pos = Self::grid_to_screen(x, y);
        vec2(pos.x + CELL_SIZE / 2.0, pos.y + CELL_SIZE / 2.0)
    }

    /// Convert screen coordinates to grid coordinates
    pub fn screen_to_grid(pos: Vec2) -> Option<(usize, usize)> {
        let start_x = (screen_width() - GRID_WIDTH as f32 * CELL_SIZE) / 2.0;
        let start_y = (screen_height() - GRID_HEIGHT as f32 * CELL_SIZE) / 2.0;
        
        let grid_width_px = GRID_WIDTH as f32 * CELL_SIZE;
        let grid_height_px = GRID_HEIGHT as f32 * CELL_SIZE;

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

    /// Convert screen coordinates to grid coordinates (clamped to nearest valid cell)
    /// Useful for drag and drop operations or proximity checks
    pub fn screen_to_grid_clamped(pos: Vec2) -> (usize, usize) {
        let start_x = (screen_width() - GRID_WIDTH as f32 * CELL_SIZE) / 2.0;
        let start_y = (screen_height() - GRID_HEIGHT as f32 * CELL_SIZE) / 2.0;
        
        let x = ((pos.x - start_x) / CELL_SIZE).floor() as i32;
        let y = ((pos.y - start_y) / CELL_SIZE).floor() as i32;

        (
            x.clamp(0, GRID_WIDTH as i32 - 1) as usize,
            y.clamp(0, GRID_HEIGHT as i32 - 1) as usize
        )
    }
}
