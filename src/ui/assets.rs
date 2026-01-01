use macroquad::prelude::*;

// Re-export Sprite from toolkit
pub use macroquad_toolkit::sprite::Sprite;

// AssetManager wrapper that adds game-specific methods
pub struct AssetManager {
    inner: macroquad_toolkit::assets::AssetManager,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            inner: macroquad_toolkit::assets::AssetManager::new(),
        }
    }

    pub async fn load_assets(&mut self) {
        let textures = vec![
            "enemy_nanodrone", "enemy_nanoguard", "enemy_leech", "enemy_siege_construct", "enemy_boss",
            "ship_hull_scavenger",
            "weapon_turret_base", "weapon_pulse_turret", "weapon_beam_emitter", "weapon_missile_rack",
            "tile_floor_core", "tile_floor_weapon", "tile_floor_defense", "tile_floor_engine",
            "tile_floor_utility", "tile_floor_medbay", "tile_floor_cockpit", "tile_floor_storage",
            "tile_floor_corridor", "tile_wall_tech",
            "prop_console_wall", "prop_console_desk", "prop_server_rack",
            "prop_pipe_burst", "prop_engine_valve", "prop_generator_coil",
            "prop_ammo_loader", "prop_capacitor_bank",
            "prop_med_scanner", "prop_cryo_pod",
            "prop_shield_emitter"
        ];

        for name in textures {
            let path = format!("assets/{}.png", name);
            if let Err(e) = self.inner.load_texture(name, &path).await {
                eprintln!("Failed to load texture: {}", e);
            }
        }
    }

    pub fn get_texture(&self, name: &str) -> Option<&Texture2D> {
        self.inner.get_texture(name)
    }

    /// Create a sprite from a loaded texture.
    pub fn create_sprite(&self, name: &str) -> Option<Sprite> {
        self.get_texture(name).map(|t| Sprite::new().with_texture(t.clone()))
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
