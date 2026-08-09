#![allow(dead_code, clippy::module_inception, clippy::type_complexity)]

use macroquad::prelude::*;
use macroquad_toolkit::capture;
use macroquad_toolkit::debug::DebugOverlay;

mod data;
mod economy;
mod enemy;
mod ship;
mod simulation;
mod state;
mod ui;

use ship::layout::Layout;
use state::GameState;
// use ui::assets::AssetManager;
use simulation::constants::*;
use simulation::events::{EventBus, GameEvent};
use ui::renderer::Renderer;
use ui::sound_manager::{SoundEffect, SoundManager};

fn window_conf() -> Conf {
    capture::capture_window_conf(
        "SCRAPYARD",
        "Scrapyard Planet",
        SCREEN_WIDTH as i32,
        SCREEN_HEIGHT as i32,
    )
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game_state = GameState::new();
    game_state.assets.load_assets().await;

    let mut sound_manager = SoundManager::new();
    sound_manager.load_sounds().await;
    sound_manager.play_music(&game_state.settings);

    let mut renderer = Renderer::new();
    let mut input_manager = ui::input_manager::InputManager::new();
    let mut event_bus = EventBus::new();
    let mut debug_overlay = DebugOverlay::new();

    // Screenshot harness: when SCRAPYARD_CAPTURE_PATH is set, seed a scene,
    // simulate deterministic frames, write a PNG, and exit.
    let capture_configs = capture::CaptureConfig::all_from_env("SCRAPYARD");
    let next_capture_scene = std::cell::RefCell::new(None::<String>);

    // Whole per-frame body (input, update, events, draw), shared by the
    // capture harness and the interactive loop below.
    let mut frame = |dt: f32| {
        if let Some(scene) = next_capture_scene.borrow_mut().take() {
            game_state.begin_capture_scene(&scene);
        }
        // 1. Gather input and push UI events
        input_manager.update(&mut game_state, &mut event_bus);

        // 2. Process UI events
        state::process_ui_events(&mut game_state, &mut event_bus);

        // 3. Update game simulation
        if !game_state.paused {
            game_state.update(dt, &mut event_bus);
        }

        // 4. Update renderer (shake decay)
        renderer.update(dt);
        debug_overlay.record_frame(dt);
        debug_overlay.visible = game_state.settings.show_fps;

        // 5. Process game events for visual and audio feedback
        // Update sound enabled state based on master volume
        sound_manager.set_enabled(game_state.settings.master_volume > 0.0);

        for event in event_bus.drain_game() {
            game_state.record_event(&event);
            match event {
                GameEvent::EnemyKilled { x, y, .. } => {
                    renderer.add_trauma(ENEMY_KILL_TRAUMA);
                    sound_manager.play_sfx(SoundEffect::EnemyKilled, &game_state.settings);
                    game_state.spawn_burst(vec2(x, y), color_u8!(255, 140, 40, 255), 8, 140.0, 0.5);
                }
                GameEvent::ModuleDamaged { x, y, damage } => {
                    renderer.add_trauma((damage * MODULE_DAMAGE_TRAUMA).min(0.3));
                    sound_manager.play_sfx(SoundEffect::ModuleDamaged, &game_state.settings);
                    let pos = Layout::grid_to_screen_center(x, y);
                    game_state.spawn_burst(pos, color_u8!(255, 220, 80, 255), 6, 110.0, 0.4);
                }
                GameEvent::ModuleDestroyed { x, y } => {
                    renderer.add_trauma(MODULE_DESTROY_TRAUMA);
                    sound_manager.play_sfx(SoundEffect::ModuleDestroyed, &game_state.settings);
                    let pos = Layout::grid_to_screen_center(x, y);
                    game_state.spawn_burst(pos, color_u8!(255, 80, 40, 255), 14, 180.0, 0.7);
                }
                GameEvent::ModuleRepaired { .. } => {
                    sound_manager.play_sfx(SoundEffect::Repair, &game_state.settings);
                }
                GameEvent::CoreDestroyed => {
                    renderer.add_trauma(CORE_DESTROY_TRAUMA);
                    sound_manager.play_sfx(SoundEffect::GameOver, &game_state.settings);
                }
                GameEvent::EngineActivated => {
                    renderer.add_trauma(ENGINE_ACTIVATE_TRAUMA);
                    sound_manager.play_sfx(SoundEffect::EngineCharge, &game_state.settings);
                }
                GameEvent::ThreatEscalated { .. } => {
                    renderer.add_trauma(THREAT_ESCALATE_TRAUMA);
                    sound_manager.play_sfx(SoundEffect::EngineCharge, &game_state.settings);
                }
                GameEvent::EmpPulse => {
                    renderer.add_trauma(EMP_PULSE_TRAUMA);
                    sound_manager.play_sfx(SoundEffect::ModuleDestroyed, &game_state.settings);
                    let center = vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);
                    game_state.spawn_burst(center, color_u8!(120, 200, 255, 255), 20, 260.0, 0.6);
                }
                GameEvent::EscapeSuccess => {
                    sound_manager.play_sfx(SoundEffect::Victory, &game_state.settings);
                }
                _ => {}
            }
        }

        // Draw
        clear_background(BLACK);
        renderer.draw(&game_state);

        // "Show FPS" setting toggles the toolkit's FPS/frame-time overlay.
        debug_overlay.draw(&[]);
    };

    if let Some(configs) = capture_configs {
        for config in configs {
            next_capture_scene.replace(Some(config.scene.clone()));
            capture::run_capture_once(&config, &mut frame).await;
        }
        return;
    }

    loop {
        let dt = get_frame_time();
        frame(dt);
        next_frame().await
    }
}
