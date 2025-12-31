//! Sound Manager for game audio
//!
//! Handles loading and playing sound effects with volume control from settings.

use macroquad::audio::{Sound, PlaySoundParams, play_sound, load_sound};
use std::collections::HashMap;

/// Sound effect identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    Repair,
    EnemyKilled,
    ModuleDamaged,
    ModuleDestroyed,
    TurretFire,
    ScrapCollected,
    ButtonClick,
    EngineCharge,
    Victory,
    GameOver,
}

pub struct SoundManager {
    sounds: HashMap<SoundEffect, Sound>,
    enabled: bool,
}

impl SoundManager {
    pub fn new() -> Self {
        Self {
            sounds: HashMap::new(),
            enabled: true,
        }
    }

    /// Load all sound effects asynchronously
    pub async fn load_sounds(&mut self) {
        // Map of sound effects to their file paths
        let sound_paths = [
            (SoundEffect::Repair, "assets/sounds/repair.wav"),
            (SoundEffect::EnemyKilled, "assets/sounds/enemy_killed.wav"),
            (SoundEffect::ModuleDamaged, "assets/sounds/damage.wav"),
            (SoundEffect::ModuleDestroyed, "assets/sounds/explosion.wav"),
            (SoundEffect::TurretFire, "assets/sounds/laser.wav"),
            (SoundEffect::ScrapCollected, "assets/sounds/pickup.wav"),
            (SoundEffect::ButtonClick, "assets/sounds/click.wav"),
            (SoundEffect::EngineCharge, "assets/sounds/engine.wav"),
            (SoundEffect::Victory, "assets/sounds/victory.wav"),
            (SoundEffect::GameOver, "assets/sounds/gameover.wav"),
        ];

        for (effect, path) in sound_paths {
            match load_sound(path).await {
                Ok(sound) => {
                    self.sounds.insert(effect, sound);
                }
                Err(_) => {
                    // Sound file not found - silent failure is OK
                    // Game will work without sounds
                }
            }
        }
    }

    /// Play a sound effect with the given volume (0.0 - 1.0)
    pub fn play(&self, effect: SoundEffect, volume: f32) {
        if !self.enabled { return; }
        
        if let Some(sound) = self.sounds.get(&effect) {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: volume.clamp(0.0, 1.0),
                }
            );
        }
    }

    /// Play a sound using settings-based volume
    pub fn play_sfx(&self, effect: SoundEffect, settings: &crate::data::settings::Settings) {
        self.play(effect, settings.effective_sfx_volume());
    }

    /// Play background music (placeholder for future implementation)
    pub fn play_music(&self, settings: &crate::data::settings::Settings) {
        let _vol = settings.effective_music_volume();
        // TODO: Implement background music
    }

    /// Enable or disable all sounds
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if any sounds were loaded
    pub fn has_sounds(&self) -> bool {
        !self.sounds.is_empty()
    }
}
