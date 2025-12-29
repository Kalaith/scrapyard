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

use state::GameState;
use assets::AssetManager;
use render::Renderer;

#[macroquad::main("Scrapyard Planet")]
async fn main() {
    let mut game_state = GameState::new();
    let mut asset_manager = AssetManager::new();
    asset_manager.load_assets().await;
    
    let renderer = Renderer::new();
    let mut input_manager = crate::input::InputManager::new();

    loop {
        // Update
        let _dt = get_frame_time(); // Will use later
        
        input_manager.update(&mut game_state);
        game_state.update(_dt);

        // Draw
        clear_background(BLACK);
        renderer.draw(&game_state);

        next_frame().await
    }
}
