use crate::enemy::entities::EnemyType;
use crate::ship::interior::{Room, RoomType};
use crate::ship::ship::ModuleType;
use crate::simulation::constants::*;
use crate::state::{EngineState, GameState, ViewMode};
use crate::ui::panels;
use crate::ui::renderer::Renderer;
use crate::ui::theme;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const HUD_HEIGHT: f32 = 42.0;
const SAFE_MARGIN: f32 = 12.0;
const PROMPT_WIDTH: f32 = 560.0;
const PROMPT_HEIGHT: f32 = 58.0;
const BOTTOM_HUD_EDGE: f32 = 675.0;
const BOTTOM_STACK_MARGIN: f32 = 180.0;
const ACTION_STACK_GAP: f32 = 8.0;
const ROUTE_PANEL_X: f32 = 12.0;
const ROUTE_PANEL_Y: f32 = 92.0;
const ROUTE_PANEL_W: f32 = 286.0;
const ROUTE_ROW_H: f32 = 28.0;

impl Renderer {
    pub fn draw_ship_ui(&self, state: &GameState) {
        self.draw_top_status_bar(state);
        self.draw_powered_system_strip(state);
        self.draw_power_routing_panel(state);

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
        x = draw_status_item(
            x,
            "SIGNAL",
            &state.threat_signature.to_string(),
            signature_color(state.threat_signature),
            116.0,
        );

        let alert_pct = (state.nanite_alert / 50.0).clamp(0.0, 1.0);
        draw_ui_text("ALERT", x, 25.0, 16.0, alert_color(alert_pct));
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
        draw_ui_text("POWERED", x, y + 16.0, 13.0, theme::text_dim());
        x += 72.0;

        let mut drawn = 0;
        for room in state
            .interior
            .rooms
            .iter()
            .filter(|room| should_show_system(room) && room.powered)
        {
            draw_system_chip(room, x, y);
            x += 34.0;
            drawn += 1;
        }

        if drawn == 0 {
            draw_ui_text("NONE", x, y + 16.0, 13.0, theme::text_dim());
        }
    }

    fn draw_compact_radar(&self, state: &GameState) {
        let w = 132.0;
        let h = 86.0;
        let x = screen_width() - w - SAFE_MARGIN;
        let y = HUD_HEIGHT + 8.0;
        let rect = Rect::new(x, y, w, h);

        panels::draw_panel(rect, "", theme::danger());
        draw_ui_text(
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
        let rect = Rect::new(
            ROUTE_PANEL_X + ROUTE_PANEL_W + 12.0,
            HUD_HEIGHT + 42.0,
            260.0,
            212.0,
        );
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

    fn draw_power_routing_panel(&self, state: &GameState) {
        let route_indices = state.routeable_room_indices();
        let rows = route_indices.len().max(1) as f32;
        let rect = Rect::new(
            ROUTE_PANEL_X,
            ROUTE_PANEL_Y,
            ROUTE_PANEL_W,
            84.0 + rows * ROUTE_ROW_H,
        );
        panels::draw_panel(rect, "POWER ROUTING", theme::cyan());

        let payout = state.calculate_payout();
        draw_ui_text(
            &format!("VALUE {} CR", payout.total),
            rect.x + 12.0,
            rect.y + 42.0,
            14.0,
            theme::warning(),
        );
        draw_ui_text(
            &format!("SIGNAL {}", state.threat_signature),
            rect.x + 150.0,
            rect.y + 42.0,
            14.0,
            signature_color(state.threat_signature),
        );

        let mut y = rect.y + 64.0;
        for (slot, room_idx) in route_indices.iter().enumerate() {
            let room = &state.interior.rooms[*room_idx];
            draw_route_row(state, room, slot, rect.x + 10.0, y, rect.w - 20.0);
            y += ROUTE_ROW_H;
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
        draw_ui_text(title, rect.x + 16.0, rect.y + 23.0, 18.0, color);
        let fitted_body = fit_text(&body, rect.w - 32.0, 15);
        draw_ui_text(
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
            if let Some((_, power_draw)) = state.get_repair_cost(room_idx, point_idx) {
                if power_draw > 0 {
                    actions.push(("1-8", "Route", theme::success()));
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
    draw_ui_text(label, x, 16.0, 13.0, theme::text_muted());
    draw_ui_text(value, x, 33.0, 18.0, color);
    x + width
}

fn draw_system_chip(room: &Room, x: f32, y: f32) {
    let color = system_state_color(room);
    draw_rectangle(x, y, 26.0, 22.0, color_u8!(0, 0, 0, 175));
    draw_rectangle_lines(x, y, 26.0, 22.0, 1.0, color);
    draw_ui_text(system_code(room), x + 8.0, y + 16.0, 13.0, color);
}

fn draw_system_detail_row(state: &GameState, room: &Room, x: f32, y: f32) {
    let color = system_state_color(room);
    let repaired = room.repaired_count();
    let total = room.repair_points.len().max(1);
    let status = room_status(state, room);

    draw_ui_text(room.name(), x, y, 13.0, theme::text_muted());
    panels::draw_segmented_bar(
        Rect::new(x + 88.0, y - 10.0, 82.0, 8.0),
        repaired as f32 / total as f32,
        total,
        color,
    );
    draw_ui_text(status.0, x + 184.0, y, 13.0, status.1);
}

fn draw_route_row(state: &GameState, room: &Room, slot: usize, x: f32, y: f32, w: f32) {
    let repaired = room.repaired_count();
    let total = room.repair_points.len().max(1);
    let draw = GameState::room_power_need(room);
    let can_power = repaired > 0 && (room.powered || state.used_power + draw <= state.total_power);
    let color = if room.powered {
        theme::success()
    } else if repaired == 0 {
        theme::danger()
    } else if can_power {
        theme::warning()
    } else {
        theme::text_muted()
    };
    let bg = if room.powered {
        color_u8!(12, 34, 23, 218)
    } else {
        color_u8!(5, 9, 12, 190)
    };

    draw_rectangle(x, y - 18.0, w, 24.0, bg);
    draw_rectangle_lines(x, y - 18.0, w, 24.0, 1.0, color);
    draw_ui_text(&(slot + 1).to_string(), x + 8.0, y, 14.0, color);
    draw_ui_text(route_label(room), x + 30.0, y, 13.0, theme::text_primary());
    panels::draw_segmented_bar(
        Rect::new(x + 116.0, y - 10.0, 58.0, 7.0),
        repaired as f32 / total as f32,
        total,
        color,
    );
    draw_ui_text(
        &format!("{}P", draw),
        x + 184.0,
        y,
        13.0,
        theme::text_muted(),
    );
    draw_ui_text(
        route_state_label(room, can_power),
        x + 224.0,
        y,
        13.0,
        color,
    );
}

fn draw_action_chip(rect: Rect, key: &str, label: &str, color: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color_u8!(0, 0, 0, 170));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, color);
    draw_ui_text(key, rect.x + 10.0, rect.y + 18.0, 14.0, color);
    draw_ui_text(
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
        let (scrap_cost, power_draw) = state.get_repair_cost(room_idx, point_idx).unwrap_or((0, 0));
        let can_repair = state.resources.scrap >= scrap_cost;
        let body = if can_repair {
            let route_note = if power_draw > 0 {
                format!(" Route draw after repair: {}P.", power_draw + 1)
            } else {
                String::new()
            };
            format!(
                "Press E to repair {} point {} for {} scrap.{}",
                room.name(),
                point_idx + 1,
                scrap_cost,
                route_note
            )
        } else if state.resources.scrap < scrap_cost {
            format!("Need {} scrap to repair {}.", scrap_cost, room.name())
        } else {
            "Repair unavailable.".to_string()
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
        "Patch the reactor to create power capacity."
    } else if state.used_power == 0 {
        "Repair a system, then route power with the panel."
    } else if !room_fully_repaired(state, RoomType::Module(ModuleType::Engine)) {
        "Repair more systems for payout, or prepare the engine."
    } else if state.engine_state == EngineState::Charging {
        "Survive until the escape timer completes."
    } else {
        "Route power to the engine when ready to launch."
    }
}

fn tutorial_prompt(id: &str, fallback: &str) -> String {
    match id {
        "welcome" => "Move with WASD. Repair orange points with E.".to_string(),
        "repair_reactor" => "Patch reactor points to create routing capacity.".to_string(),
        "choose_power" => {
            "Choose what to power: weapons earn scrap, shields buy time, support improves payout."
                .to_string()
        }
        "repair_engine" => {
            "Repair and power the engine only when the value is worth the attack.".to_string()
        }
        "complete" => {
            "Stay for more payout or launch before the signal overwhelms you.".to_string()
        }
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
    } else if room.powered || room.room_type == RoomType::Module(ModuleType::Core) {
        ("POWERED", theme::success())
    } else if room.is_fully_repaired() {
        ("READY", theme::warning())
    } else {
        ("OFFLINE", theme::text_muted())
    }
}

fn system_state_color(room: &Room) -> Color {
    if room.repaired_count() == 0 {
        theme::danger()
    } else if room.powered || room.room_type == RoomType::Module(ModuleType::Core) {
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
        RoomType::Cockpit => "L",
        RoomType::Medbay => "M",
        RoomType::Storage => "H",
        _ => "?",
    }
}

fn route_label(room: &Room) -> &'static str {
    match room.room_type {
        RoomType::Module(ModuleType::Weapon) => "WEAPONS",
        RoomType::Module(ModuleType::Defense) => "SHIELDS",
        RoomType::Module(ModuleType::Utility) => "RECYCLER",
        RoomType::Module(ModuleType::Engine) => "ENGINE",
        RoomType::Cockpit => "LIFE SUP",
        RoomType::Medbay => "MEDBAY",
        _ => room.name(),
    }
}

fn route_state_label(room: &Room, can_power: bool) -> &'static str {
    if room.powered {
        "ON"
    } else if room.repaired_count() == 0 {
        "FIX"
    } else if can_power {
        "OFF"
    } else {
        "NO PWR"
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

fn signature_color(signature: i32) -> Color {
    if signature >= WAVE_T3_POWER {
        theme::danger()
    } else if signature >= WAVE_T1_POWER {
        theme::warning()
    } else {
        theme::success()
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
    } else if room_fully_repaired(state, RoomType::Module(ModuleType::Engine))
        && !state.engine_powered()
    {
        ("ROUTE", theme::warning())
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
    if measure_ui_text(text, None, font_size, 1.0).width <= max_width {
        return text.to_string();
    }

    let mut fitted = text.to_string();
    while fitted.len() > 4 {
        fitted.pop();
        let candidate = format!("{}...", fitted.trim_end());
        if measure_ui_text(&candidate, None, font_size, 1.0).width <= max_width {
            return candidate;
        }
    }

    "...".to_string()
}
