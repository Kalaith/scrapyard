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

pub fn draw_stat_card(rect: Rect, label: &str, value: &str, detail: &str, color: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, theme::panel_bg_light());
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, theme::panel_border());
    draw_ui_text(label, rect.x + 10.0, rect.y + 18.0, 16.0, color);
    draw_ui_text(value, rect.x + 10.0, rect.y + 42.0, 24.0, color);

    if !detail.is_empty() {
        draw_ui_text(
            detail,
            rect.x + 10.0,
            rect.y + rect.h - 6.0,
            14.0,
            theme::text_muted(),
        );
    }
}

pub fn draw_progress_bar(rect: Rect, pct: f32, color: Color) {
    let clamped = pct.clamp(0.0, 1.0);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color_u8!(18, 22, 26, 255));
    draw_rectangle(rect.x, rect.y, rect.w * clamped, rect.h, color);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, theme::panel_border());
}
