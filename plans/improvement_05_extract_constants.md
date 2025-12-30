# Improvement 5: Extract Magic Numbers to Constants

## Problem
The codebase contains many "magic numbers" - hardcoded numeric values without clear meaning. This makes code hard to understand, maintain, and balance.

## Examples of Magic Numbers Found

### In `ai.rs`:
```rust
let count = ::rand::random::<usize>() % 5 + 8; // 8-12 scrap piles
// ...
let amount = macroquad::rand::gen_range(15, 40); // scrap amount
// ...
let fire_chance = effective_fire_rate * dt; // no named constant
```

### In `combat.rs`:
```rust
let attack_range = 30.0; // enemy attack range
// ...
let hit_radius = match enemy.enemy_type {
    EnemyType::Boss => 40.0,
    EnemyType::Nanoguard => 15.0,
    _ => 10.0,
};
```

### In `state.rs`:
```rust
let repair_amount = robotics_level as f32 * 2.0; // nano-robot repair rate
// ...
let base_credits = 500;
let bonus_mult = 1.0 + (self.upgrades.get_level("credit_bonus") as f32 * 0.25);
// ...
self.escape_timer -= dt * engine_repair_pct; // no base time constant
```

### In `input.rs`:
```rust
let min_dist = 40.0; // interaction range
// ...
if state.gathering_timer >= 2.0 { // gathering time
```

## Solution
Extract all magic numbers into named constants in `constants.rs` or domain-specific constant files.

## Implementation Steps

1. **Audit all source files** for magic numbers
2. **Categorize constants** by domain
3. **Add constants to appropriate files**
4. **Replace magic numbers** with constant references
5. **Update any calculations** that depend on these values

## New Constants to Add

### Game Balance Constants (`constants.rs`)
```rust
// Enemy spawning
pub const MIN_SCRAP_PILES: usize = 8;
pub const MAX_SCRAP_PILES: usize = 12;
pub const SCRAP_PILE_MIN_AMOUNT: i32 = 15;
pub const SCRAP_PILE_MAX_AMOUNT: i32 = 40;

// Combat
pub const ENEMY_ATTACK_RANGE: f32 = 30.0;
pub const ENEMY_HIT_RADIUS_NANODRONE: f32 = 10.0;
pub const ENEMY_HIT_RADIUS_NANOGUARD: f32 = 15.0;
pub const ENEMY_HIT_RADIUS_BOSS: f32 = 40.0;

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

// Tutorial and UI
pub const INTERACTION_RANGE: f32 = 40.0;
pub const GATHERING_TIME_SECONDS: f32 = 2.0;
pub const TUTORIAL_FADE_TIME: f32 = 3.0;

// Nano-robots
pub const NANO_REPAIR_RATE_PER_LEVEL: f32 = 2.0;
pub const NANO_REPAIR_INTERVAL_SECONDS: f32 = 2.0;

// Engine system
pub const ENGINE_CHARGE_BASE_TIME: f32 = 60.0;
pub const ENGINE_MIN_REPAIR_PERCENT: f32 = 0.25;

// Screen shake
pub const TRAUMA_DECAY_RATE: f32 = 2.0;
pub const MAX_SHAKE_OFFSET: f32 = 8.0;
pub const MODULE_DAMAGE_TRAUMA: f32 = 0.02;
pub const MODULE_DESTROY_TRAUMA: f32 = 0.4;
pub const CORE_DESTROY_TRAUMA: f32 = 1.0;
pub const ENGINE_ACTIVATE_TRAUMA: f32 = 0.3;
pub const ENEMY_KILL_TRAUMA: f32 = 0.1;
```

### Update Code Examples

#### In `ai.rs`:
```rust
// Before
let count = ::rand::random::<usize>() % 5 + 8;
let amount = macroquad::rand::gen_range(15, 40);

// After
let count = macroquad::rand::gen_range(MIN_SCRAP_PILES, MAX_SCRAP_PILES + 1);
let amount = macroquad::rand::gen_range(SCRAP_PILE_MIN_AMOUNT, SCRAP_PILE_MAX_AMOUNT + 1);
```

#### In `combat.rs`:
```rust
// Before
let attack_range = 30.0;
let hit_radius = match enemy.enemy_type {
    EnemyType::Boss => 40.0,
    EnemyType::Nanoguard => 15.0,
    _ => 10.0,
};

// After
let attack_range = ENEMY_ATTACK_RANGE;
let hit_radius = match enemy.enemy_type {
    EnemyType::Boss => ENEMY_HIT_RADIUS_BOSS,
    EnemyType::Nanoguard => ENEMY_HIT_RADIUS_NANOGUARD,
    _ => ENEMY_HIT_RADIUS_NANODRONE,
};
```

#### In `state.rs`:
```rust
// Before
let repair_amount = robotics_level as f32 * 2.0;
let base_credits = 500;
let bonus_mult = 1.0 + (self.upgrades.get_level("credit_bonus") as f32 * 0.25);

// After
let repair_amount = robotics_level as f32 * NANO_REPAIR_RATE_PER_LEVEL;
let base_credits = BASE_ESCAPE_CREDITS;
let bonus_mult = 1.0 + (self.upgrades.get_level("credit_bonus") as f32 * CREDIT_BONUS_PER_LEVEL);
```

## Benefits
- Improved code readability
- Easier game balancing
- Centralized configuration
- Better documentation of game mechanics
- Reduced bugs from inconsistent values

## Testing
- Verify all gameplay mechanics work with new constants
- Test edge cases with minimum/maximum values
- Ensure UI displays correct values
- Check that upgrades apply proper bonuses</content>
<parameter name="filePath">h:\WebHatchery\games\scrapyard\plans\improvement_05_extract_constants.md