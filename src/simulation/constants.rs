pub const GRID_WIDTH: usize = 20;
pub const GRID_HEIGHT: usize = 15;
pub const CELL_SIZE: f32 = 40.0;
pub const SCREEN_WIDTH: f32 = 1280.0;
pub const SCREEN_HEIGHT: f32 = 720.0;

pub const COLOR_GRID_LINE: macroquad::color::Color = macroquad::color::GRAY;
pub const COLOR_MODULE_EMPTY: macroquad::color::Color = macroquad::color::DARKGRAY;

// Enemy spawning
pub const MIN_SCRAP_PILES: usize = 8;
pub const MAX_SCRAP_PILES: usize = 12;
pub const SCRAP_PILE_MIN_AMOUNT: i32 = 15;
pub const SCRAP_PILE_MAX_AMOUNT: i32 = 40;
pub const SCRAP_SPAWN_PADDING: f32 = 20.0;

// Combat
pub const ENEMY_ATTACK_RANGE: f32 = 30.0;
pub const ENEMY_HIT_RADIUS_NANODRONE: f32 = 10.0;
pub const ENEMY_HIT_RADIUS_NANOGUARD: f32 = 15.0;
pub const ENEMY_HIT_RADIUS_BOSS: f32 = 40.0;

// Enemy Stats (HP, Speed, Damage)
pub const ENEMY_DRONE_HP: f32 = 10.0;
pub const ENEMY_DRONE_SPEED: f32 = 100.0;
pub const ENEMY_DRONE_DAMAGE: f32 = 5.0;

pub const ENEMY_GUARD_HP: f32 = 50.0;
pub const ENEMY_GUARD_SPEED: f32 = 40.0;
pub const ENEMY_GUARD_DAMAGE: f32 = 15.0;

pub const ENEMY_LEECH_HP: f32 = 30.0;
pub const ENEMY_LEECH_SPEED: f32 = 60.0;
pub const ENEMY_LEECH_DAMAGE: f32 = 2.0;
pub const ENEMY_LEECH_POWER_DRAIN: i32 = 1; // Power drained per tick when attached

pub const ENEMY_SIEGE_HP: f32 = 200.0;
pub const ENEMY_SIEGE_SPEED: f32 = 15.0;
pub const ENEMY_SIEGE_DAMAGE: f32 = 30.0;

pub const ENEMY_BOSS_HP: f32 = 1000.0;
pub const ENEMY_BOSS_SPEED: f32 = 20.0;
pub const ENEMY_BOSS_DAMAGE: f32 = 50.0;
pub const BOSS_ABILITY_COOLDOWN: f32 = 8.0; // Seconds between boss abilities
pub const BOSS_SPLIT_COUNT: usize = 3; // Number of drones spawned on boss death

// Wave Logic
pub const WAVE_GRACE_POWER: i32 = 5;
pub const WAVE_T1_POWER: i32 = 11;
pub const WAVE_T2_POWER: i32 = 21;
pub const WAVE_T3_POWER: i32 = 40;

pub const SPAWN_INTERVAL_DRONE_T0: f32 = 8.0;
pub const SPAWN_INTERVAL_DRONE_T1: f32 = 4.0;
pub const SPAWN_INTERVAL_DRONE_T2: f32 = 2.0;
pub const SPAWN_INTERVAL_DRONE_T3: f32 = 1.0;

pub const SPAWN_INTERVAL_GUARD_T2: f32 = 15.0;
pub const SPAWN_INTERVAL_GUARD_T3: f32 = 5.0;

// Power system
pub const POWER_PER_CORE_POINT: i32 = 2;
pub const POWER_COST_WEAPON: i32 = 1;
pub const POWER_COST_DEFENSE: i32 = 1;
pub const POWER_COST_UTILITY: i32 = 1;
pub const POWER_COST_ENGINE: i32 = 2;
pub const POWER_COST_COCKPIT: i32 = 1;
pub const POWER_COST_MEDBAY: i32 = 1;

// Economy
pub const BASE_ESCAPE_CREDITS: i32 = 500;
pub const CREDIT_BONUS_PER_LEVEL: f32 = 0.25;
pub const SCRAP_EFFICIENCY_BONUS: f32 = 0.20;

// Interaction
pub const INTERACTION_RANGE: f32 = 40.0;
pub const GATHERING_TIME_SECONDS: f32 = 2.0;

// Nano-robots
pub const NANO_REPAIR_RATE_PER_LEVEL: f32 = 2.0;
pub const NANO_REPAIR_INTERVAL_SECONDS: f32 = 2.0;

// Engine system
pub const ENGINE_CHARGE_BASE_TIME: f32 = 60.0;
pub const ENGINE_MIN_REPAIR_PERCENT: f32 = 0.25;

// Screen shake
pub const TRAUMA_DECAY_RATE: f32 = 1.5; // Adjusted to match current renderer.rs
pub const SHAKE_INTENSITY: f32 = 15.0; // Adjusted to match current renderer.rs
pub const MODULE_DAMAGE_TRAUMA: f32 = 0.02;
pub const MODULE_DESTROY_TRAUMA: f32 = 0.4;
pub const CORE_DESTROY_TRAUMA: f32 = 1.0;
pub const ENGINE_ACTIVATE_TRAUMA: f32 = 0.3;
pub const ENEMY_KILL_TRAUMA: f32 = 0.1;
