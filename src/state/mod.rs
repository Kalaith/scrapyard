pub mod game_state;
mod game_update;     // Update logic (impl GameState)
mod game_actions;    // Player actions (impl GameState)
#[cfg(not(target_arch = "wasm32"))]
mod game_persistence; // Save/load (impl GameState)
pub mod persistence;
pub mod tutorial;
pub mod profile;

pub use game_state::{GameState, GamePhase, EngineState, ViewMode};
pub use tutorial::TutorialStep;
pub use profile::PlayerProfile;

use crate::simulation::events::{EventBus, UIEvent};

pub fn process_ui_events(state: &mut GameState, events: &mut EventBus) {
    for event in events.drain_ui() {
        match event {
            UIEvent::StartGame => {
                state.start_new_game();
            }
            UIEvent::ReturnToMenu => {
                state.paused = false;
                state.phase = GamePhase::Menu;
            }
            UIEvent::Pause => {
                state.paused = true;
                state.pause_menu_selection = 0;
            }
            UIEvent::Resume => {
                state.paused = false;
            }
            UIEvent::Repair(x, y) => {
                state.attempt_repair(x, y, events);
            }
            UIEvent::Upgrade(x, y) => {
                state.attempt_upgrade(x, y, events);
            }
            UIEvent::Toggle(x, y) => {
                state.toggle_module(x, y);
            }
            UIEvent::PurchaseUpgrade(id) => {
                if state.phase == GamePhase::Victory {
                    state.phase = GamePhase::InterRound;
                } else {
                    state.purchase_upgrade(&id);
                }
            }
            UIEvent::NextRound => {
                state.start_new_game();
            }
            UIEvent::SaveGame(slot) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Err(e) = state.save_to_slot(slot) {
                        eprintln!("Failed to save: {}", e);
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = slot; // Suppress unused warning
                    eprintln!("Save not supported in WebGL");
                }
                state.paused = false;
            }
            UIEvent::LoadGame(slot) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Ok(loaded) = GameState::load_from_slot(slot) {
                        *state = loaded;
                    } else {
                        eprintln!("Failed to load slot {}", slot);
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = slot; // Suppress unused warning
                    eprintln!("Load not supported in WebGL");
                }
            }
            UIEvent::ExitGame => {
                std::process::exit(0);
            }
        }
    }
}
