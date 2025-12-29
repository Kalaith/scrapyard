use macroquad::prelude::*;

mod constants;
mod state;
mod ship;
mod resources;
mod input;
mod assets;
mod render;
mod gameplay;
mod entities;
mod ai;
mod combat;
mod events;
mod settings;

use state::GameState;
use assets::AssetManager;
use render::Renderer;
use events::{EventBus, GameEvent};

#[macroquad::main("Scrapyard Planet")]
async fn main() {
    let mut game_state = GameState::new();
    let mut asset_manager = AssetManager::new();
    asset_manager.load_assets().await;
    
    let mut renderer = Renderer::new();
    let mut input_manager = crate::input::InputManager::new();
    let mut event_bus = EventBus::new();

    loop {
        let dt = get_frame_time();
        
        // 1. Gather input and push UI events
        input_manager.update(&mut game_state, &mut event_bus);
        
        // 2. Process UI events
        state::process_ui_events(&mut game_state, &mut event_bus);
        
        // 3. Update game simulation
        game_state.update(dt, &mut event_bus);
        
        // 4. Update renderer (shake decay)
        renderer.update(dt);
        
        // 5. Process game events for visual feedback
        for event in event_bus.drain_game() {
            match event {
                GameEvent::EnemyKilled { .. } => {
                    renderer.add_trauma(0.1);
                }
                GameEvent::ModuleDamaged { damage, .. } => {
                    renderer.add_trauma(damage * 0.02);
                }
                GameEvent::ModuleDestroyed { .. } => {
                    renderer.add_trauma(0.4);
                }
                GameEvent::CoreDestroyed => {
                    renderer.add_trauma(1.0);
                }
                GameEvent::EngineActivated => {
                    renderer.add_trauma(0.3);
                }
                _ => {}
            }
        }

        // Draw
        clear_background(BLACK);
        renderer.draw(&game_state);

        next_frame().await
    }
}
