use macroquad::prelude::*;
use std::collections::HashMap;

/// Sprite struct for robust scaling and rotation of textures.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Sprite {
    pub texture: Option<Texture2D>,
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32, // Radians
    pub origin: Vec2,  // Pivot point (0.5, 0.5 for center)
    pub color: Color,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub struct AssetManager {
    textures: HashMap<String, Texture2D>,
    // fonts: HashMap<String, Font>, // TODO: Add fonts later
}

#[allow(dead_code)]
impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            // fonts: HashMap::new(),
        }
    }

    pub async fn load_assets(&mut self) {
        // Placeholder: Load a dummy texture or generate one
        // For now, we rely on procedural drawing, but this is the hook for later.
        
        // Example:
        // let tex = load_texture("assets/ship.png").await.unwrap();
        // self.textures.insert("ship".to_string(), tex);
    }

    pub fn get_texture(&self, name: &str) -> Option<&Texture2D> {
        self.textures.get(name)
    }

    /// Create a sprite from a loaded texture.
    pub fn create_sprite(&self, name: &str) -> Option<Sprite> {
        self.textures.get(name).map(|t| Sprite::new().with_texture(t.clone()))
    }
}
