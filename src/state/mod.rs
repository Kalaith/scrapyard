mod game_actions; // Player actions (impl GameState)
mod game_persistence; // Save/load (impl GameState)
pub mod game_state;
mod game_update; // Update logic (impl GameState)
pub mod persistence;
pub mod profile;
pub mod tutorial;

pub use game_state::{EngineState, GamePhase, GameState, ViewMode};

use crate::simulation::events::{EventBus, UIEvent};

pub fn process_ui_events(state: &mut GameState, events: &mut EventBus) {
    for event in events.drain_ui() {
        match event {
            UIEvent::StartGame => {
                state.settings_open = false;
                state.start_new_game();
            }
            UIEvent::ReturnToMenu => {
                state.paused = false;
                state.settings_open = false;
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
                    state.sync_upgrades_from_profile();
                } else {
                    state.purchase_upgrade(&id);
                }
            }
            UIEvent::NextRound => {
                state.start_new_game();
            }
            UIEvent::SaveGame(slot) => {
                if let Err(e) = state.save_to_slot(slot) {
                    eprintln!("Failed to save: {}", e);
                }
                state.paused = false;
            }
            UIEvent::LoadGame(slot) => {
                if let Ok(loaded) = GameState::load_from_slot(slot) {
                    *state = loaded;
                } else {
                    eprintln!("Failed to load slot {}", slot);
                }
            }
            UIEvent::ExitGame => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    std::process::exit(0);
                }
                #[cfg(target_arch = "wasm32")]
                {
                    state.paused = false;
                    state.settings_open = false;
                    state.phase = GamePhase::Menu;
                }
            }
        }
    }
}
