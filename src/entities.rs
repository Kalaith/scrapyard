use macroquad::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyType {
    Nanodrone,
    Nanoguard,
    Leech,
    Boss,
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub id: u64,
    pub enemy_type: EnemyType,
    pub position: Vec2,
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub damage: f32,
    pub target_module: Option<(usize, usize)>, // Grid coords
}

impl Enemy {
    pub fn new(id: u64, enemy_type: EnemyType, position: Vec2) -> Self {
        let (hp, speed, damage) = match enemy_type {
            EnemyType::Nanodrone => (10.0, 100.0, 5.0),
            EnemyType::Nanoguard => (50.0, 40.0, 15.0),
            EnemyType::Leech => (30.0, 60.0, 2.0), // Low damage but special effect later
            EnemyType::Boss => (1000.0, 20.0, 50.0),
        };

        Self {
            id,
            enemy_type,
            position,
            health: hp,
            max_health: hp,
            speed,
            damage,
            target_module: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Projectile {
    pub position: Vec2,
    pub velocity: Vec2,
    pub damage: f32,
    pub active: bool,
}

impl Projectile {
    pub fn new(position: Vec2, target: Vec2, speed: f32, damage: f32) -> Self {
        let direction = (target - position).normalize_or_zero();
        Self {
            position,
            velocity: direction * speed,
            damage,
            active: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: Color,
    pub active: bool,
}

impl Particle {
    pub fn new(position: Vec2, velocity: Vec2, lifetime: f32, color: Color) -> Self {
        Self {
            position,
            velocity,
            lifetime,
            max_lifetime: lifetime,
            color,
            active: true,
        }
    }
}
#[derive(Debug, Clone)]
pub struct ScrapPile {
    pub position: Vec2, // Room-relative or Global? Global is easier for drawing/collision
    pub amount: i32,
    pub active: bool,
}

impl ScrapPile {
    pub fn new(position: Vec2, amount: i32) -> Self {
        Self {
            position,
            amount,
            active: true,
        }
    }
}
