use macroquad::prelude::*;

mod ship;
mod enemy;
mod economy;
mod simulation;
mod state;
mod ui;
mod data;

use state::GameState;
// use ui::assets::AssetManager;
use ui::renderer::Renderer;
use ui::sound_manager::{SoundManager, SoundEffect};
use simulation::events::{EventBus, GameEvent};
use simulation::constants::*;

#[macroquad::main("Scrapyard Planet")]
async fn main() {
    let mut game_state = GameState::new();
    game_state.assets.load_assets().await;
    
    let mut sound_manager = SoundManager::new();
    sound_manager.load_sounds().await;
    sound_manager.play_music(&game_state.settings);
    
    let mut renderer = Renderer::new();
    let mut input_manager = ui::input_manager::InputManager::new();
    let mut event_bus = EventBus::new();

    loop {
        let dt = get_frame_time();
        
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
        
        // 5. Process game events for visual and audio feedback
        // Update sound enabled state based on master volume
        sound_manager.set_enabled(game_state.settings.master_volume > 0.0);
        
        for event in event_bus.drain_game() {
            match event {
                GameEvent::EnemyKilled { .. } => {
                    renderer.add_trauma(ENEMY_KILL_TRAUMA);
                    sound_manager.play_sfx(SoundEffect::EnemyKilled, &game_state.settings);
                }
                GameEvent::ModuleDamaged { damage, .. } => {
                    renderer.add_trauma(damage * MODULE_DAMAGE_TRAUMA);
                    sound_manager.play_sfx(SoundEffect::ModuleDamaged, &game_state.settings);
                }
                GameEvent::ModuleDestroyed { .. } => {
                    renderer.add_trauma(MODULE_DESTROY_TRAUMA);
                    sound_manager.play_sfx(SoundEffect::ModuleDestroyed, &game_state.settings);
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
                GameEvent::EscapeSuccess => {
                    sound_manager.play_sfx(SoundEffect::Victory, &game_state.settings);
                }
                _ => {}
            }
        }

        // Draw
        clear_background(BLACK);
        renderer.draw(&game_state);
        
        // Debug: show sound status
        if game_state.settings.show_fps && sound_manager.has_sounds() {
            macroquad::prelude::draw_text("♪ Sound ON", 10.0, 30.0, 16.0, GREEN);
        }

        next_frame().await
    }
}
