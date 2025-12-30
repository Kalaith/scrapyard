# Improvement 3: Replace Unsafe Static Mut Variables

## Problem
In `ai.rs`, enemy spawning uses unsafe static mutable variables for timers:

```rust
static mut SPAWN_TIMER: f32 = 0.0;
static mut GUARD_TIMER: f32 = 0.0;
```

This violates Rust's safety guarantees and makes the code non-thread-safe. Static mut variables require unsafe blocks to access and can lead to data races in concurrent code.

## Current Implementation
```rust
pub fn update_wave_logic(state: &mut GameState, dt: f32, events: &mut EventBus) {
    // ...
    unsafe {
        SPAWN_TIMER += dt;
        GUARD_TIMER += dt;
    }
    
    // ... spawning logic ...
    
    unsafe {
        if SPAWN_TIMER >= drone_interval {
            spawn_drone(state, events);
            SPAWN_TIMER = 0.0;
        }
        
        if power_level >= 6 && GUARD_TIMER >= guard_interval {
            spawn_guard(state, events);
            GUARD_TIMER = 0.0;
        }
    }
}
```

## Solution
Replace unsafe static variables with proper state management by adding a wave state structure to `GameState`.

## Implementation Steps

1. **Create a WaveState struct** to hold timer state
2. **Add wave state to GameState**
3. **Update AI functions** to take wave state as parameter
4. **Remove unsafe code**

## Code Changes

### Add WaveState struct (new file: `src/enemy/wave.rs` or in `ai.rs`)
```rust
#[derive(Debug, Clone)]
pub struct WaveState {
    pub spawn_timer: f32,
    pub guard_timer: f32,
}

impl WaveState {
    pub fn new() -> Self {
        Self {
            spawn_timer: 0.0,
            guard_timer: 0.0,
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        self.spawn_timer += dt;
        self.guard_timer += dt;
    }
    
    pub fn reset_spawn_timer(&mut self) {
        self.spawn_timer = 0.0;
    }
    
    pub fn reset_guard_timer(&mut self) {
        self.guard_timer = 0.0;
    }
}
```

### Add to GameState (in `state.rs`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    // ... existing fields ...
    #[serde(skip)]
    pub wave_state: WaveState,
    // ... rest of fields ...
}

impl GameState {
    pub fn new() -> Self {
        // ... existing initialization ...
        wave_state: WaveState::new(),
        // ... rest of initialization ...
    }
    
    pub fn start_new_game(&mut self) {
        // ... existing reset code ...
        self.wave_state = WaveState::new();
        // ... rest of reset ...
    }
}
```

### Update AI functions (in `ai.rs`)
```rust
pub fn update_wave_logic(
    state: &mut GameState, 
    wave_state: &mut WaveState, 
    dt: f32, 
    events: &mut EventBus
) {
    let power_level = state.total_power;
    
    wave_state.update(dt);
    
    // No enemies spawn until player has enough power
    if power_level < 5 {
        return;
    }
    
    // ... existing interval calculations ...
    
    if wave_state.spawn_timer >= drone_interval {
        spawn_drone(state, events);
        wave_state.reset_spawn_timer();
    }
    
    if power_level >= 6 && wave_state.guard_timer >= guard_interval {
        spawn_guard(state, events);
        wave_state.reset_guard_timer();
    }
}
```

### Update the main update loop (in `state.rs`)
```rust
pub fn update(&mut self, dt: f32, events: &mut EventBus) {
    match self.phase {
        GamePhase::Playing => {
            if !self.paused {
                // ... existing code ...
                crate::ai::update_wave_logic(self, &mut self.wave_state, dt, events);
                // ... rest of update ...
            }
        }
        _ => {}
    }
}
```

### Update save/load system
Add wave state to serialization (though timers can be skipped since they're runtime state):

```rust
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    // ... existing fields ...
    // wave_state not needed - runtime only
}
```

## Benefits
- Eliminates unsafe code
- Thread-safe design
- Better encapsulation of wave logic
- Easier testing and debugging
- Follows Rust best practices

## Testing
- Verify enemy spawning intervals work correctly
- Test game save/load doesn't break wave timing
- Ensure wave state resets properly on new game
- Check that pausing doesn't affect spawn timers</content>
<parameter name="filePath">h:\WebHatchery\games\scrapyard\plans\improvement_03_replace_unsafe_statics.md