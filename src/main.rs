use macroquad::prelude::*;

mod ship;
mod enemy;
mod economy;
mod simulation;
mod state;
mod ui;
mod data;
mod util;

use state::GameState;
use ui::assets::AssetManager;
use ui::renderer::Renderer;
use simulation::events::{EventBus, GameEvent};
use simulation::constants::*;

#[macroquad::main("Scrapyard Planet")]
async fn main() {
    let mut game_state = GameState::new();
    let mut asset_manager = AssetManager::new();
    asset_manager.load_assets().await;
    
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
        
        // 5. Process game events for visual feedback
        for event in event_bus.drain_game() {
            match event {
                GameEvent::EnemyKilled { .. } => {
                    renderer.add_trauma(ENEMY_KILL_TRAUMA);
                }
                GameEvent::ModuleDamaged { damage, .. } => {
                    renderer.add_trauma(damage * MODULE_DAMAGE_TRAUMA);
                }
                GameEvent::ModuleDestroyed { .. } => {
                    renderer.add_trauma(MODULE_DESTROY_TRAUMA);
                }
                GameEvent::CoreDestroyed => {
                    renderer.add_trauma(CORE_DESTROY_TRAUMA);
                }
                GameEvent::EngineActivated => {
                    renderer.add_trauma(ENGINE_ACTIVATE_TRAUMA);
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
