use macroquad::prelude::*;
use serde::{Serialize, Deserialize};
use crate::simulation::constants::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyType {
    Nanodrone,
    Nanoguard,
    Leech,
    SiegeConstruct, // Slow, high HP, attacks hull directly
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
    pub attached_to: Option<(usize, usize)>,   // For Leech: module it's attached to
    pub ability_timer: f32,                     // For Boss: cooldown for special abilities
    pub attacking: bool,                        // Tracks if currently dealing damage (for sound throttling)
}

impl Enemy {
    pub fn new(id: u64, enemy_type: EnemyType, position: Vec2) -> Self {
        let (hp, speed, damage) = match enemy_type {
            EnemyType::Nanodrone => (ENEMY_DRONE_HP, ENEMY_DRONE_SPEED, ENEMY_DRONE_DAMAGE),
            EnemyType::Nanoguard => (ENEMY_GUARD_HP, ENEMY_GUARD_SPEED, ENEMY_GUARD_DAMAGE),
            EnemyType::Leech => (ENEMY_LEECH_HP, ENEMY_LEECH_SPEED, ENEMY_LEECH_DAMAGE),
            EnemyType::SiegeConstruct => (ENEMY_SIEGE_HP, ENEMY_SIEGE_SPEED, ENEMY_SIEGE_DAMAGE),
            EnemyType::Boss => (ENEMY_BOSS_HP, ENEMY_BOSS_SPEED, ENEMY_BOSS_DAMAGE),
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
            attached_to: None,
            ability_timer: 0.0,
            attacking: false,
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
