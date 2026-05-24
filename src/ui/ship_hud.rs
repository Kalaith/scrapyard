use crate::enemy::entities::EnemyType;
use crate::ship::interior::{Room, RoomType};
use crate::ship::ship::ModuleType;
use crate::simulation::constants::*;
use crate::state::{EngineState, GameState, ViewMode};
use crate::ui::panels;
use crate::ui::renderer::Renderer;
use crate::ui::theme;
use macroquad::prelude::*;

const HUD_HEIGHT: f32 = 42.0;
const SAFE_MARGIN: f32 = 12.0;
const PROMPT_WIDTH: f32 = 560.0;
const PROMPT_HEIGHT: f32 = 58.0;
const BOTTOM_HUD_EDGE: f32 = 675.0;
const BOTTOM_STACK_MARGIN: f32 = 180.0;
const ACTION_STACK_GAP: f32 = 8.0;

impl Renderer {
    pub fn draw_ship_ui(&self, state: &GameState) {
        self.draw_top_status_bar(state);
        self.draw_powered_system_strip(state);

        if !state.enemies.is_empty() {
            self.draw_compact_radar(state);
        }

        if is_key_down(KeyCode::Tab) {
            self.draw_system_details(state);
        }

        self.draw_bottom_prompt(state);
        self.draw_available_actions(state);
    }

    fn draw_top_status_bar(&self, state: &GameState) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            HUD_HEIGHT,
            color_u8!(0, 0, 0, 190),
        );
        draw_line(
            0.0,
            HUD_HEIGHT,
            screen_width(),
            HUD_HEIGHT,
            1.0,
            color_u8!(68, 74, 76, 170),
        );

        let mut x = SAFE_MARGIN;
        x = draw_status_item(
            x,
            "HULL",
            &format!(
                "{:.0}/{:.0}",
                state.ship_integrity, state.ship_max_integrity
            ),
            hull_color(state),
            172.0,
        );
        x = draw_status_item(
            x,
            "SCRAP",
            &state.resources.scrap.to_string(),
            theme::warning(),
            112.0,
        );
        x = draw_status_item(
            x,
            "POWER",
            &format!("{}/{}", state.used_power, state.total_power),
            power_color(state),
            142.0,
        );

        let alert_pct = (state.nanite_alert / 50.0).clamp(0.0, 1.0);
        draw_text("ALERT", x, 25.0, 16.0, alert_color(alert_pct));
        panels::draw_segmented_bar(
            Rect::new(x + 66.0, 14.0, 132.0, 10.0),
            alert_pct,
            10,
            alert_color(alert_pct),
        );
        x += 224.0;

        let (engine_label, engine_color) = engine_status(state);
        draw_status_item(x, "ENGINE", engine_label, engine_color, 190.0);
    }

    fn draw_powered_system_strip(&self, state: &GameState) {
        let y = HUD_HEIGHT + 8.0;
        let mut x = SAFE_MARGIN;
        draw_text("POWERED", x, y + 16.0, 13.0, theme::text_dim());
        x += 72.0;

        let mut drawn = 0;
        for room in state
            .interior
            .rooms
            .iter()
            .filter(|room| should_show_system(room) && room.repaired_count() > 0)
        {
            draw_system_chip(room, x, y);
            x += 34.0;
            drawn += 1;
        }

        if drawn == 0 {
            draw_text("NONE", x, y + 16.0, 13.0, theme::text_dim());
        }
    }

    fn draw_compact_radar(&self, state: &GameState) {
        let w = 132.0;
        let h = 86.0;
        let x = screen_width() - w - SAFE_MARGIN;
        let y = HUD_HEIGHT + 8.0;
        let rect = Rect::new(x, y, w, h);

        panels::draw_panel(rect, "", theme::danger());
        draw_text(
            &format!("ATTACK {}", state.enemies.len()),
            x + 10.0,
            y + 18.0,
            14.0,
            theme::danger(),
        );

        let map = Rect::new(x + 10.0, y + 26.0, w - 20.0, h - 36.0);
        draw_rectangle(map.x, map.y, map.w, map.h, color_u8!(3, 7, 8, 230));
        draw_rectangle_lines(map.x, map.y, map.w, map.h, 1.0, color_u8!(46, 58, 60, 200));
        draw_rectangle(
            map.x + map.w / 2.0 - 5.0,
            map.y + map.h / 2.0 - 3.0,
            10.0,
            6.0,
            theme::text_primary(),
        );

        for enemy in &state.enemies {
            let px = map.x + (enemy.position.x / screen_width()).clamp(0.0, 1.0) * map.w;
            let py = map.y + (enemy.position.y / screen_height()).clamp(0.0, 1.0) * map.h;
            draw_circle(px, py, 3.0, enemy_color(&enemy.enemy_type));
        }
    }

    fn draw_system_details(&self, state: &GameState) {
        let rect = Rect::new(SAFE_MARGIN, HUD_HEIGHT + 42.0, 260.0, 212.0);
        panels::draw_panel(rect, "SYSTEMS", theme::text_primary());

        let mut y = rect.y + 50.0;
        for room in state
            .interior
            .rooms
            .iter()
            .filter(|room| should_show_system(room))
        {
            if y > rect.y + rect.h - 16.0 {
                break;
            }
            draw_system_detail_row(state, room, rect.x + 12.0, y);
            y += 24.0;
        }
    }

    fn draw_bottom_prompt(&self, state: &GameState) {
        let x = (screen_width() - PROMPT_WIDTH) / 2.0;
        let y = bottom_prompt_y();
        let rect = Rect::new(x, y, PROMPT_WIDTH, PROMPT_HEIGHT);
        let (title, body, color) = prompt_text(state);

        draw_rectangle(rect.x, rect.y, rect.w, rect.h, color_u8!(0, 0, 0, 190));
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, color);
        draw_rectangle(rect.x, rect.y, rect.w, 2.0, color);
        draw_text(title, rect.x + 16.0, rect.y + 23.0, 18.0, color);
        let fitted_body = fit_text(&body, rect.w - 32.0, 15);
        draw_text(
            &fitted_body,
            rect.x + 16.0,
            rect.y + 46.0,
            15.0,
            theme::text_primary(),
        );
    }

    fn draw_available_actions(&self, state: &GameState) {
        let mut actions = Vec::new();
        if let Some((room_idx, point_idx)) = active_repair_target(state) {
            actions.push(("E", "Repair", theme::warning()));
            if let Some((_, power_cost)) = state.get_repair_cost(room_idx, point_idx) {
                if power_cost > 0 && state.used_power + power_cost <= state.total_power {
                    actions.push(("AUTO", "Route Power", theme::success()));
                }
            }
        }
        actions.push(("TAB", view_action_label(state), theme::cyan()));

        let slot_w = 138.0;
        let slot_h = 26.0;
        let gap = 8.0;
        let total_w =
            actions.len() as f32 * slot_w + (actions.len().saturating_sub(1)) as f32 * gap;
        let mut x = (screen_width() - total_w) / 2.0;
        let y = bottom_prompt_y() + PROMPT_HEIGHT + ACTION_STACK_GAP;

        for (key, label, color) in actions {
            draw_action_chip(Rect::new(x, y, slot_w, slot_h), key, label, color);
            x += slot_w + gap;
        }
    }
}

fn hud_bottom_edge() -> f32 {
    let height = screen_height();
    if height < BOTTOM_HUD_EDGE {
        height
    } else {
        BOTTOM_HUD_EDGE
    }
}

fn bottom_prompt_y() -> f32 {
    hud_bottom_edge() - PROMPT_HEIGHT - BOTTOM_STACK_MARGIN
}

fn draw_status_item(x: f32, label: &str, value: &str, color: Color, width: f32) -> f32 {
    draw_text(label, x, 16.0, 13.0, theme::text_muted());
    draw_text(value, x, 33.0, 18.0, color);
    x + width
}

fn draw_system_chip(room: &Room, x: f32, y: f32) {
    let color = system_state_color(room);
    draw_rectangle(x, y, 26.0, 22.0, color_u8!(0, 0, 0, 175));
    draw_rectangle_lines(x, y, 26.0, 22.0, 1.0, color);
    draw_text(system_code(room), x + 8.0, y + 16.0, 13.0, color);
}

fn draw_system_detail_row(state: &GameState, room: &Room, x: f32, y: f32) {
    let color = system_state_color(room);
    let repaired = room.repaired_count();
    let total = room.repair_points.len().max(1);
    let status = room_status(state, room);

    draw_text(room.name(), x, y, 13.0, theme::text_muted());
    panels::draw_segmented_bar(
        Rect::new(x + 88.0, y - 10.0, 82.0, 8.0),
        repaired as f32 / total as f32,
        total,
        color,
    );
    draw_text(status.0, x + 184.0, y, 13.0, status.1);
}

fn draw_action_chip(rect: Rect, key: &str, label: &str, color: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color_u8!(0, 0, 0, 170));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, color);
    draw_text(key, rect.x + 10.0, rect.y + 18.0, 14.0, color);
    draw_text(
        label,
        rect.x + 42.0,
        rect.y + 18.0,
        14.0,
        theme::text_primary(),
    );
}

fn prompt_text(state: &GameState) -> (&'static str, String, Color) {
    if let Some((room_idx, point_idx)) = active_repair_target(state) {
        let room = &state.interior.rooms[room_idx];
        let (scrap_cost, power_cost) = state.get_repair_cost(room_idx, point_idx).unwrap_or((0, 0));
        let can_repair = state.resources.scrap >= scrap_cost
            && (power_cost == 0 || state.used_power + power_cost <= state.total_power);
        let body = if can_repair {
            let cost = if power_cost > 0 {
                format!("{scrap_cost} scrap and {power_cost} power")
            } else {
                format!("{scrap_cost} scrap")
            };
            format!(
                "Press E to repair {} point {} for {}.",
                room.name(),
                point_idx + 1,
                cost
            )
        } else if state.resources.scrap < scrap_cost {
            format!("Need {} scrap to repair {}.", scrap_cost, room.name())
        } else {
            "Need more reactor power before repairing this system.".to_string()
        };
        return (
            "REPAIR",
            body,
            if can_repair {
                theme::warning()
            } else {
                theme::danger()
            },
        );
    }

    if let Some(step) = state.tutorial_state.current_step(&state.tutorial_config) {
        if !state.tutorial_state.is_complete() {
            return (
                "OBJECTIVE",
                tutorial_prompt(&step.id, &step.message),
                theme::warning(),
            );
        }
    }

    (
        "OBJECTIVE",
        current_objective(state).to_string(),
        objective_color(state),
    )
}

fn current_objective(state: &GameState) -> &'static str {
    if !room_repaired_any(state, RoomType::Module(ModuleType::Core)) {
        "Restore reactor power."
    } else if !room_fully_repaired(state, RoomType::Module(ModuleType::Engine)) {
        "Repair engines so escape can begin."
    } else if state.engine_state == EngineState::Charging {
        "Survive until the escape timer completes."
    } else {
        "Engines are ready. Prepare for escape."
    }
}

fn tutorial_prompt(id: &str, fallback: &str) -> String {
    match id {
        "welcome" => "Move with WASD. Repair orange points with E.".to_string(),
        "repair_reactor" => "Repair the reactor first to restore usable power.".to_string(),
        "repair_shields" => "Repair shields next so the ship can survive attacks.".to_string(),
        "repair_weapon" => "Repair a weapon so the ship can fight back.".to_string(),
        "repair_engine" => "Repair the engine to begin escape charging.".to_string(),
        "complete" => "Systems online. Press TAB for exterior view and survive.".to_string(),
        _ => fallback.replace('\n', " "),
    }
}

fn objective_color(state: &GameState) -> Color {
    if state.engine_state == EngineState::Charging {
        theme::cyan()
    } else if !state.enemies.is_empty() {
        theme::danger()
    } else {
        theme::warning()
    }
}

fn active_repair_target(state: &GameState) -> Option<(usize, usize)> {
    let room = state.interior.room_at(state.player.position)?;
    let point_idx = room.repair_point_at(state.player.position)?;
    if room.repair_points[point_idx].repaired {
        return None;
    }
    let room_idx = state
        .interior
        .rooms
        .iter()
        .position(|candidate| candidate.id == room.id)?;
    Some((room_idx, point_idx))
}

fn should_show_system(room: &Room) -> bool {
    !room.name().is_empty() && !room.repair_points.is_empty()
}

fn room_status(state: &GameState, room: &Room) -> (&'static str, Color) {
    if room.room_type == RoomType::Module(ModuleType::Engine) {
        let (label, color) = engine_status(state);
        if label != "LOCKED" {
            return (label, color);
        }
    }

    let repaired = room.repaired_count();
    if repaired == 0 {
        ("DAMAGED", theme::danger())
    } else if room.is_fully_repaired() {
        ("WORKING", theme::success())
    } else {
        ("PARTIAL", theme::warning())
    }
}

fn system_state_color(room: &Room) -> Color {
    if room.repaired_count() == 0 {
        theme::danger()
    } else if room.is_fully_repaired() {
        theme::success()
    } else {
        theme::warning()
    }
}

fn system_code(room: &Room) -> &'static str {
    match room.room_type {
        RoomType::Module(ModuleType::Core) => "R",
        RoomType::Module(ModuleType::Weapon) => "W",
        RoomType::Module(ModuleType::Defense) => "S",
        RoomType::Module(ModuleType::Engine) => "E",
        RoomType::Module(ModuleType::Utility) => "U",
        RoomType::Cockpit => "C",
        RoomType::Medbay => "M",
        RoomType::Storage => "H",
        _ => "?",
    }
}

fn power_color(state: &GameState) -> Color {
    if state.used_power <= state.total_power {
        theme::success()
    } else {
        theme::danger()
    }
}

fn hull_color(state: &GameState) -> Color {
    let pct = state.ship_integrity / state.ship_max_integrity;
    if pct > 0.6 {
        theme::success()
    } else if pct > 0.3 {
        theme::warning()
    } else {
        theme::danger()
    }
}

fn alert_color(alert_pct: f32) -> Color {
    if alert_pct > 0.66 {
        theme::danger()
    } else if alert_pct > 0.35 {
        theme::warning()
    } else {
        theme::success()
    }
}

fn engine_status(state: &GameState) -> (&'static str, Color) {
    if state.engine_stress >= STRESS_THRESHOLD_CRITICAL {
        ("CASCADE", theme::danger())
    } else if state.engine_stress >= STRESS_THRESHOLD_UNSTABLE {
        ("UNSTABLE", theme::warning())
    } else if state.engine_stress >= STRESS_THRESHOLD_STRAINED {
        ("STRAINED", theme::warning())
    } else if state.engine_state == EngineState::Charging {
        ("CHARGING", theme::cyan())
    } else if room_fully_repaired(state, RoomType::Module(ModuleType::Engine)) {
        ("READY", theme::cyan())
    } else {
        ("LOCKED", theme::text_muted())
    }
}

fn enemy_color(enemy_type: &EnemyType) -> Color {
    match enemy_type {
        EnemyType::Boss => theme::danger(),
        EnemyType::Nanoguard | EnemyType::SiegeConstruct => theme::warning(),
        EnemyType::Leech => theme::cyan(),
        EnemyType::Nanodrone => theme::danger(),
    }
}

fn room_repaired_any(state: &GameState, room_type: RoomType) -> bool {
    state
        .interior
        .rooms
        .iter()
        .any(|room| room.room_type == room_type && room.repaired_count() > 0)
}

fn room_fully_repaired(state: &GameState, room_type: RoomType) -> bool {
    state
        .interior
        .rooms
        .iter()
        .any(|room| room.room_type == room_type && room.is_fully_repaired())
}

fn view_action_label(state: &GameState) -> &'static str {
    match state.view_mode {
        ViewMode::Exterior => "Interior",
        ViewMode::Interior => "Exterior",
    }
}

fn fit_text(text: &str, max_width: f32, font_size: u16) -> String {
    if measure_text(text, None, font_size, 1.0).width <= max_width {
        return text.to_string();
    }

    let mut fitted = text.to_string();
    while fitted.len() > 4 {
        fitted.pop();
        let candidate = format!("{}...", fitted.trim_end());
        if measure_text(&candidate, None, font_size, 1.0).width <= max_width {
            return candidate;
        }
    }

    "...".to_string()
}
