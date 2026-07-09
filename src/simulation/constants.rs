#![allow(dead_code)]

pub const GRID_WIDTH: usize = 20;
pub const GRID_HEIGHT: usize = 15;
pub const CELL_SIZE: f32 = 30.0;
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
pub const BOSS_ABILITY_COOLDOWN: f32 = 8.0; // Seconds between boss EMP pulses
pub const BOSS_SPLIT_COUNT: usize = 3; // Number of fragments spawned on boss death
pub const BOSS_FRAGMENT_HP: f32 = 40.0; // HP of each fragment drone

// Per-module damage: modules soak hits before losing a repair point.
pub const MODULE_POINT_HEALTH: f32 = 60.0; // Damage to knock out one repair point

// Wave Logic. These thresholds use routed system signature, not raw reactor capacity.
pub const WAVE_GRACE_POWER: i32 = 2;
pub const WAVE_T1_POWER: i32 = 5;
pub const WAVE_T2_POWER: i32 = 9;
pub const WAVE_T3_POWER: i32 = 13;

pub const SPAWN_INTERVAL_DRONE_T0: f32 = 14.0; // Much slower initial spawns
pub const SPAWN_INTERVAL_DRONE_T1: f32 = 8.0; // Still manageable
pub const SPAWN_INTERVAL_DRONE_T2: f32 = 5.0; // Getting dangerous
pub const SPAWN_INTERVAL_DRONE_T3: f32 = 3.0; // Intense

pub const SPAWN_INTERVAL_GUARD_T2: f32 = 18.0; // Guards spawn slower
pub const SPAWN_INTERVAL_GUARD_T3: f32 = 8.0;
pub const SPAWN_INTERVAL_LEECH: f32 = 22.0;
pub const SPAWN_INTERVAL_SIEGE: f32 = 28.0;

// Power system
pub const POWER_PER_CORE_POINT: i32 = 1; // Each reactor repair point gives 1 power
                                         // Per-point draw for weapon/defense/utility/engine rooms is now data-driven from
                                         // modules.json (`power_consumption`). Cockpit + medbay aren't grid modules, so they keep
                                         // constant per-point costs here.
pub const POWER_COST_COCKPIT: i32 = 1;
pub const POWER_COST_MEDBAY: i32 = 1;

// Routed system signature
pub const RECYCLER_SIGNATURE_BONUS: i32 = 2;
pub const ENGINE_CHARGE_SIGNATURE_BONUS: i32 = 5;
pub const LIFE_SUPPORT_OFFLINE_SIGNATURE: i32 = 2;
// Divisor on the extractable-value (greed) term of the signature. Lowered from 250 so the
// player's unbanked wealth registers on the threat readout — danger *is* the paycheck.
pub const HIGH_VALUE_SIGNATURE_DIVISOR: i32 = 130;
// Nanite alert above its base leaks into the signature (long/loud runs draw more), and the
// alert climbs faster when zero power is routed — the swarm hunts the silence (anti-turtle).
pub const NANITE_ALERT_SIGNATURE_DIVISOR: f32 = 6.0;
pub const NANITE_ALERT_SILENCE_ACCEL: f32 = 0.4;

// Economy
pub const BASE_ESCAPE_CREDITS: i32 = 500;
pub const CREDIT_BONUS_PER_LEVEL: f32 = 0.25;
pub const SCRAP_EFFICIENCY_BONUS: f32 = 0.20;
pub const RECYCLER_KILL_BONUS: f32 = 0.35;

// Payout
pub const PAYOUT_FULL_REPAIR_BONUS: i32 = 60;
pub const PAYOUT_POWERED_SYSTEM_BONUS: i32 = 35;
pub const PAYOUT_HULL_MAX_BONUS: i32 = 300;
pub const PAYOUT_ENEMY_DESTROYED_BONUS: i32 = 8;
pub const PAYOUT_SCRAP_DIVISOR: i32 = 2;
pub const PAYOUT_SIGNATURE_BONUS: i32 = 12;
// Risk bonus is intentionally uncapped: if danger scales with unbanked value, the reward
// must keep pace, or greed rationally stops while the threat keeps climbing.
pub const PAYOUT_ENGINE_STRESS_PENALTY_PER_POINT: i32 = 4;

// Persistent upgrade effects
pub const META_STARTING_SCRAP_PER_LEVEL: i32 = 15;
pub const META_REACTOR_POINTS_PER_LEVEL: usize = 2;
pub const META_RESTORED_POINTS_PER_LEVEL: usize = 2;
pub const META_STORAGE_PER_LEVEL: i32 = 120;
pub const META_REPAIR_DISCOUNT_PER_LEVEL: i32 = 1;

// Interaction
pub const INTERACTION_RANGE: f32 = 40.0;
pub const GATHERING_TIME_SECONDS: f32 = 2.0;
pub const LIFE_SUPPORT_REPAIR_DISCOUNT: i32 = 2;
pub const LIFE_SUPPORT_GRACE_SECONDS: f32 = 8.0;
pub const LIFE_SUPPORT_OFFLINE_DAMAGE_PER_SEC: f32 = 4.0;
pub const MEDBAY_HULL_REPAIR_PER_SEC: f32 = 5.0;
pub const MEDBAY_STRESS_RELIEF_PER_SEC: f32 = 2.5;

// Engine system
pub const ENGINE_CHARGE_BASE_TIME: f32 = 60.0;
pub const ENGINE_MIN_REPAIR_PERCENT: f32 = 1.0;

// Screen shake
pub const TRAUMA_DECAY_RATE: f32 = 1.5; // Adjusted to match current renderer.rs
pub const SHAKE_INTENSITY: f32 = 15.0; // Adjusted to match current renderer.rs
pub const MODULE_DAMAGE_TRAUMA: f32 = 0.02;
pub const MODULE_DESTROY_TRAUMA: f32 = 0.4;
pub const CORE_DESTROY_TRAUMA: f32 = 1.0;
pub const ENGINE_ACTIVATE_TRAUMA: f32 = 0.3;
pub const ENEMY_KILL_TRAUMA: f32 = 0.1;
pub const THREAT_ESCALATE_TRAUMA: f32 = 0.18; // Sting when the signal crosses a tier
pub const EMP_PULSE_TRAUMA: f32 = 0.35; // Boss EMP knocks power offline

// Ship
pub const SHIP_BASE_INTEGRITY: f32 = 1000.0;
pub const HULL_UPGRADE_BONUS: f32 = 200.0; // HP added per hull upgrade level

// Module upgrades
pub const MODULE_MAX_LEVEL: u8 = 5;
pub const MODULE_UPGRADE_HP_MULTIPLIER: f32 = 1.5; // HP multiplier per upgrade level
                                                   // In-run upgrade (E on a fully-repaired weapon/shield): scrap cost scales with level, and
                                                   // each level adds this fraction to turret damage/fire-rate and shield coverage — the
                                                   // late-run scrap sink the economy was missing.
pub const MODULE_UPGRADE_SCRAP_BASE: i32 = 45;
pub const MODULE_UPGRADE_EFFECT_PER_LEVEL: f32 = 0.5;

// Interior hazards: when the hull crosses these fractions, a breach knocks out a repair
// point in a live system, forcing the player back into the interior to re-repair.
pub const HULL_BREACH_THRESHOLDS: [f32; 3] = [0.6, 0.4, 0.2];

// Repair costs
pub const REPAIR_SCRAP_COST: i32 = 10; // Scrap cost per interior repair point

// Engine Stress System
// Guaranteed-floor tuning: 8 engine points x 3.0 = 24 stress, well under the 46
// cascade threshold, so a natural-pace engine repair can never cascade on its own.
pub const STRESS_GAIN_PER_REPAIR: f32 = 3.0;
// Per-second idle decay of engine stress.
pub const STRESS_DECAY_IDLE: f32 = 2.0;
// Base stress per second while charging (scaled by nanite alert). Tuned so a quiet run
// (alert near base) climbs ~21 stress across the 60s charge from a cool engine, finishing
// under the cascade threshold; a loud/long run pushes alert up and tips into cascade.
pub const STRESS_CHARGE_GAIN_BASE: f32 = 0.35;
pub const STRESS_THRESHOLD_STRAINED: f32 = 16.0;
pub const STRESS_THRESHOLD_UNSTABLE: f32 = 31.0;
pub const STRESS_THRESHOLD_CRITICAL: f32 = 46.0;
pub const CASCADE_DAMAGE_PER_SEC: f32 = 50.0; // Rapid internal damage during cascade
pub const NANITE_ALERT_BASE: f32 = 16.0; // Base divisor for charging stress
