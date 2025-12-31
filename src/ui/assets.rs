use macroquad::prelude::*;
use std::collections::HashMap;

/// Sprite struct for robust scaling and rotation of textures.
#[derive(Debug, Clone)]

pub struct Sprite {
    pub texture: Option<Texture2D>,
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32, // Radians
    pub origin: Vec2,  // Pivot point (0.5, 0.5 for center)
    pub color: Color,
}


impl Sprite {
    pub fn new() -> Self {
        Self {
            texture: None,
            position: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation: 0.0,
            origin: vec2(0.5, 0.5),
            color: WHITE,
        }
    }

    pub fn with_texture(mut self, texture: Texture2D) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = vec2(x, y);
        self
    }

    pub fn scaled(mut self, sx: f32, sy: f32) -> Self {
        self.scale = vec2(sx, sy);
        self
    }

    pub fn rotated(mut self, angle: f32) -> Self {
        self.rotation = angle;
        self
    }

    pub fn tinted(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn draw(&self) {
        if let Some(tex) = &self.texture {
            let w = tex.width() * self.scale.x;
            let h = tex.height() * self.scale.y;
            let ox = w * self.origin.x;
            let oy = h * self.origin.y;

            draw_texture_ex(
                tex,
                self.position.x - ox,
                self.position.y - oy,
                self.color,
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    rotation: self.rotation,
                    pivot: Some(self.position),
                    ..Default::default()
                },
            );
        }
    }

    /// Draw a colored rectangle as a placeholder when no texture is available.
    pub fn draw_placeholder(&self, width: f32, height: f32, color: Color) {
        let w = width * self.scale.x;
        let h = height * self.scale.y;
        let ox = w * self.origin.x;
        let oy = h * self.origin.y;

        draw_rectangle(
            self.position.x - ox,
            self.position.y - oy,
            w,
            h,
            color,
        );
    }
}


pub struct AssetManager {
    textures: HashMap<String, Texture2D>,
    // fonts: HashMap<String, Font>, // TODO: Add fonts later
}


impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            // fonts: HashMap::new(),
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
            match load_texture(&path).await {
                Ok(tex) => {
                    tex.set_filter(FilterMode::Nearest);
                    self.textures.insert(name.to_string(), tex);
                },
                Err(e) => eprintln!("Failed to load texture {}: {}", path, e),
            }
        }
    }

    pub fn get_texture(&self, name: &str) -> Option<&Texture2D> {
        self.textures.get(name)
    }

    /// Create a sprite from a loaded texture.
    pub fn create_sprite(&self, name: &str) -> Option<Sprite> {
        self.textures.get(name).map(|t| Sprite::new().with_texture(t.clone()))
    }
}
