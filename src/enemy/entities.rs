use crate::simulation::constants::*;
use macroquad::prelude::*;
use macroquad_toolkit::timing::Cooldown;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyType {
    Nanodrone,
    Nanoguard,
    Leech,
    SiegeConstruct, // Slow, high HP, attacks hull directly
    Burrower,       // Fragile, sprints at a system and chews its repair points
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
    pub ability_cooldown: Cooldown,            // For Boss/Leech: cooldown for special abilities
    pub attacking: bool, // Tracks if currently dealing damage (for sound throttling)
}

impl Enemy {
    pub fn new(id: u64, enemy_type: EnemyType, position: Vec2) -> Self {
        let (hp, speed, damage) = match enemy_type {
            EnemyType::Nanodrone => (ENEMY_DRONE_HP, ENEMY_DRONE_SPEED, ENEMY_DRONE_DAMAGE),
            EnemyType::Nanoguard => (ENEMY_GUARD_HP, ENEMY_GUARD_SPEED, ENEMY_GUARD_DAMAGE),
            EnemyType::Leech => (ENEMY_LEECH_HP, ENEMY_LEECH_SPEED, ENEMY_LEECH_DAMAGE),
            EnemyType::SiegeConstruct => (ENEMY_SIEGE_HP, ENEMY_SIEGE_SPEED, ENEMY_SIEGE_DAMAGE),
            EnemyType::Burrower => (
                ENEMY_BURROWER_HP,
                ENEMY_BURROWER_SPEED,
                ENEMY_BURROWER_DAMAGE,
            ),
            EnemyType::Boss => (ENEMY_BOSS_HP, ENEMY_BOSS_SPEED, ENEMY_BOSS_DAMAGE),
        };

        let ability_duration = Self::ability_cooldown_duration(enemy_type.clone());
        let ability_cooldown = if ability_duration > 0.0 {
            Cooldown::new_armed(ability_duration)
        } else {
            Cooldown::new(0.0)
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
            ability_cooldown,
            attacking: false,
        }
    }

    /// Duration of this enemy type's special-ability cooldown (0 when the type has none:
    /// only Boss EMP pulses and Leech power-drain ticks use `ability_cooldown`).
    pub fn ability_cooldown_duration(enemy_type: EnemyType) -> f32 {
        match enemy_type {
            EnemyType::Boss => BOSS_ABILITY_COOLDOWN,
            EnemyType::Leech => LEECH_DRAIN_INTERVAL,
            _ => 0.0,
        }
    }

    /// Reconstructs `ability_cooldown` from a saved "seconds elapsed since last trigger"
    /// value (the format persisted in `SavedEnemy::ability_timer`).
    pub fn ability_cooldown_from_elapsed(enemy_type: EnemyType, elapsed: f32) -> Cooldown {
        let duration = Self::ability_cooldown_duration(enemy_type);
        if duration <= 0.0 {
            return Cooldown::new(0.0);
        }
        let mut cooldown = Cooldown::new_armed(duration);
        cooldown.tick(elapsed.clamp(0.0, duration));
        cooldown
    }

    /// Seconds elapsed since `ability_cooldown` last triggered, for persistence.
    pub fn ability_elapsed(&self) -> f32 {
        let duration = Self::ability_cooldown_duration(self.enemy_type.clone());
        (duration - self.ability_cooldown.remaining()).max(0.0)
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
