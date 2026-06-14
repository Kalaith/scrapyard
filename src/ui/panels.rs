use crate::ui::theme;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

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

pub fn draw_segmented_bar(rect: Rect, pct: f32, segments: usize, color: Color) {
    let gap = 3.0;
    let segments = segments.max(1);
    let segment_w = (rect.w - gap * (segments as f32 - 1.0)) / segments as f32;
    let filled = (pct.clamp(0.0, 1.0) * segments as f32).ceil() as usize;

    for i in 0..segments {
        let x = rect.x + i as f32 * (segment_w + gap);
        let fill = if i < filled {
            color
        } else {
            color_u8!(25, 28, 31, 255)
        };
        draw_rectangle(x, rect.y, segment_w, rect.h, fill);
        draw_rectangle_lines(
            x,
            rect.y,
            segment_w,
            rect.h,
            1.0,
            color_u8!(63, 67, 72, 190),
        );
    }
}

pub fn draw_checkbox(x: f32, y: f32, checked: bool) {
    let color = if checked {
        theme::success()
    } else {
        theme::text_dim()
    };
    draw_rectangle_lines(x, y, 12.0, 12.0, 1.0, color);

    if checked {
        draw_line(x + 2.0, y + 6.0, x + 5.0, y + 10.0, 2.0, color);
        draw_line(x + 5.0, y + 10.0, x + 11.0, y + 2.0, 2.0, color);
    }
}

pub fn draw_keycap(rect: Rect, label: &str, color: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color_u8!(18, 21, 25, 245));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, color);
    let size = measure_ui_text(label, None, 16, 1.0);
    draw_ui_text(
        label,
        rect.x + (rect.w - size.width) / 2.0,
        rect.y + rect.h / 2.0 + 6.0,
        16.0,
        color,
    );
}
