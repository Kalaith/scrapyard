# Improvement 2: Fix Power System Inconsistency

## Problem
There are two conflicting power calculation systems in the codebase that can lead to inconsistent game behavior:

1. `update_power()` in `state.rs` - calculates power from interior rooms based on repair status
2. `update_resources()` in `state.rs` - calculates power from grid modules based on active state

This dual system causes confusion and potential bugs where power availability doesn't match consumption calculations.

## Current Issues

### In `update_power()` (lines 301-322):
```rust
for room in &self.interior.rooms {
    if room.repair_points.is_empty() {
        continue;
    }
    
    let repaired = room.repaired_count() as i32;
    
    if repaired > 0 {
        match room.room_type {
            crate::interior::RoomType::Module(ModuleType::Core) => {
                 // Reactor GENERATES power (1 per point)
                 self.total_power += repaired;
            },
            // ... consumption calculations
        }
    }
}
```

### In `update_resources()` (lines 324-340):
```rust
fn update_resources(&mut self) {
    let mut total_power = 0;
    
    for row in &self.ship.grid {
        for cell in row {
            if let Some(module) = cell {
                if module.state == ModuleState::Active {
                    let stats = self.module_registry.get(module.module_type);
                    total_power += stats.power_consumption;
                }
            }
        }
    }
    
    self.resources.power = total_power;
}
```

## Solution
Consolidate into a single, coherent power system. Since the game uses interior-based repairs for progression, the room-based system should be authoritative.

## Implementation Steps

1. **Remove the grid-based power calculation** from `update_resources()`
2. **Enhance `update_power()`** to be the single source of truth
3. **Update all power consumption checks** to use `total_power` and `used_power` fields
4. **Ensure power generation and consumption are balanced**:
   - Core rooms generate power based on repair points
   - All other module rooms consume power based on repair points
   - Power must be available before repairs can be made

## Code Changes

### Modify `update_power()` to handle all power logic:
```rust
fn update_power(&mut self) {
    self.total_power = 0;
    self.used_power = 0;
    
    for room in &self.interior.rooms {
        if room.repair_points.is_empty() {
            continue;
        }
        
        let repaired = room.repaired_count() as i32;
        
        if repaired > 0 {
            match room.room_type {
                crate::interior::RoomType::Module(ModuleType::Core) => {
                     // Reactor GENERATES power (scales with repair level)
                     self.total_power += repaired * 2; // 2 power per repair point
                },
                crate::interior::RoomType::Module(ModuleType::Weapon) => 
                    self.used_power += repaired * 1,
                crate::interior::RoomType::Module(ModuleType::Defense) => 
                    self.used_power += repaired * 1,
                crate::interior::RoomType::Module(ModuleType::Utility) => 
                    self.used_power += repaired * 1,
                crate::interior::RoomType::Module(ModuleType::Engine) => 
                    self.used_power += repaired * 2, // Engines use more power
                crate::interior::RoomType::Cockpit => 
                    self.used_power += repaired * 1,
                crate::interior::RoomType::Medbay => 
                    self.used_power += repaired * 1,
                _ => {}
            }
        }
    }
}
```

### Update `attempt_interior_repair()` to use unified power system:
```rust
let is_reactor = matches!(room.room_type, 
    crate::interior::RoomType::Module(ModuleType::Core));

if self.resources.scrap < scrap_cost {
    return false;
}

if !is_reactor && (self.used_power + power_cost > self.total_power) {
    return false;
}
```

### Remove redundant `update_resources()` call from `update()`:
```rust
pub fn update(&mut self, dt: f32, events: &mut EventBus) {
    // ... existing code ...
    self.update_power(); // Single power update
    // Remove: self.update_resources();
    // ... rest of update ...
}
```

## Benefits
- Eliminates confusion about which power value to use
- Consistent power calculations throughout the game
- Easier debugging of power-related issues
- Better alignment with the interior repair mechanic

## Testing
- Verify power generation from repaired reactor rooms
- Test power consumption prevents over-repair
- Ensure UI displays correct power values
- Check that engine activation requires sufficient power</content>
<parameter name="filePath">h:\WebHatchery\games\scrapyard\plans\improvement_02_fix_power_system.md