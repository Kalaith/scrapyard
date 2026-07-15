use crate::ship::interior::RoomType;
use crate::ship::ship::ModuleType;
use macroquad::prelude::*;

pub const TOP_BAR_HEIGHT: f32 = 42.0;
pub const SIDE_PANEL_TOP: f32 = 54.0;
pub const SIDE_PANEL_WIDTH: f32 = 210.0;
pub const BOTTOM_PANEL_HEIGHT: f32 = 96.0;

pub fn panel_bg() -> Color {
    color_u8!(4, 7, 10, 218)
}

pub fn panel_border() -> Color {
    color_u8!(72, 82, 92, 185)
}

pub fn text_primary() -> Color {
    color_u8!(232, 238, 232, 255)
}

pub fn text_muted() -> Color {
    color_u8!(145, 152, 150, 255)
}

pub fn text_dim() -> Color {
    color_u8!(82, 88, 92, 255)
}

pub fn danger() -> Color {
    color_u8!(255, 52, 52, 255)
}

pub fn warning() -> Color {
    color_u8!(255, 176, 48, 255)
}

pub fn success() -> Color {
    color_u8!(44, 226, 104, 255)
}

pub fn cyan() -> Color {
    color_u8!(38, 210, 235, 255)
}

pub fn purple() -> Color {
    color_u8!(186, 84, 255, 255)
}

pub fn steel() -> Color {
    color_u8!(74, 86, 96, 255)
}

pub fn system_color(room_type: RoomType) -> Color {
    match room_type {
        RoomType::Module(ModuleType::Core) => color_u8!(255, 124, 32, 255),
        RoomType::Module(ModuleType::Weapon) => color_u8!(240, 205, 48, 255),
        RoomType::Module(ModuleType::Defense) => color_u8!(34, 178, 255, 255),
        RoomType::Module(ModuleType::Engine) => color_u8!(255, 70, 58, 255),
        RoomType::Module(ModuleType::Utility) => color_u8!(44, 218, 104, 255),
        RoomType::Cockpit => color_u8!(62, 210, 240, 255),
        RoomType::Medbay => color_u8!(220, 238, 230, 255),
        RoomType::Storage => color_u8!(210, 160, 70, 255),
        RoomType::Corridor => steel(),
        _ => text_dim(),
    }
}

pub fn room_glow(room_type: RoomType) -> Color {
    let color = system_color(room_type);
    Color::new(color.r, color.g, color.b, 0.18)
}
