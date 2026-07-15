use crate::ui::theme;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub fn draw_panel(rect: Rect, title: &str, accent: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, theme::panel_bg());
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, theme::panel_border());
    draw_rectangle(rect.x, rect.y, rect.w, 2.0, accent);

    if !title.is_empty() {
        draw_ui_text(title, rect.x + 12.0, rect.y + 22.0, 18.0, accent);
        draw_line(
            rect.x,
            rect.y + 34.0,
            rect.x + rect.w,
            rect.y + 34.0,
            1.0,
            color_u8!(45, 52, 58, 210),
        );
    }
}
