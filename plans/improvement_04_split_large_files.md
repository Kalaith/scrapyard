# Improvement 4: Split Large Files

## Problem
Several files exceed the CODE_STANDARDS.md guidelines of 200-400 lines per file (max 800):

- `state.rs`: 769 lines (should be split)
- `render.rs`: 922 lines (should be split)  
- `input.rs`: 265 lines (could be split)

Large files make code harder to navigate, maintain, and review.

## Current File Sizes
- `state.rs`: 769 lines
- `render.rs`: 922 lines
- `input.rs`: 265 lines
- Other files are within acceptable ranges

## Splitting Strategy

### Split `state.rs` (769 lines) into:
1. **`game_state.rs`** (200-300 lines): Core GameState struct and basic methods
2. **`power_system.rs`** (100-150 lines): Power calculation and management
3. **`tutorial_system.rs`** (100-150 lines): Tutorial logic and state
4. **`persistence.rs`** (150-200 lines): Save/load functionality

### Split `render.rs` (922 lines) into:
1. **`renderer.rs`** (150-200 lines): Main Renderer struct and coordination
2. **`ui_renderer.rs`** (300-400 lines): UI drawing functions
3. **`world_renderer.rs`** (300-400 lines): Game world rendering

### Split `input.rs` (265 lines) into:
1. **`input_manager.rs`** (100-150 lines): InputManager struct and core logic
2. **`ui_input.rs`** (80-120 lines): UI-specific input handling
3. **`gameplay_input.rs`** (80-120 lines): Gameplay input handling

## Implementation Steps

### For `state.rs` split:

1. **Create `src/state/game_state.rs`**:
   - Move `GameState` struct definition
   - Move basic methods: `new()`, `start_new_game()`, `update()`
   - Keep core game loop logic

2. **Create `src/state/power_system.rs`**:
   - Move `update_power()`, `update_resources()`
   - Move power-related helper methods

3. **Create `src/state/tutorial_system.rs`**:
   - Move `TutorialStep` enum and methods
   - Move tutorial-related state and logic

4. **Create `src/state/persistence.rs`**:
   - Move `save()`, `load_from_file()`
   - Move serialization structs (`SaveData`, etc.)

5. **Update `src/state/mod.rs`**:
   ```rust
   pub mod game_state;
   pub mod power_system;
   pub mod tutorial_system;
   pub mod persistence;
   
   pub use game_state::*;
   pub use power_system::*;
   pub use tutorial_system::*;
   pub use persistence::*;
   ```

### For `render.rs` split:

1. **Create `src/ui/renderer.rs`**:
   - Move `Renderer` struct and basic methods
   - Keep main `draw()` coordination method

2. **Create `src/ui/ui_renderer.rs`**:
   - Move all UI drawing functions (`draw_menu`, `draw_hud`, etc.)
   - Move tutorial overlay rendering

3. **Create `src/ui/world_renderer.rs`**:
   - Move world rendering (`draw_ship_hull`, `draw_enemies`, etc.)
   - Move particle and projectile rendering

### For `input.rs` split:

1. **Create `src/ui/input_manager.rs`**:
   - Move `InputManager` struct and core update logic

2. **Create `src/ui/ui_input.rs`**:
   - Move menu, game over, victory input handling

3. **Create `src/ui/gameplay_input.rs`**:
   - Move gameplay input handling (movement, interactions)

## Benefits
- Easier navigation and code review
- Better separation of concerns
- Reduced merge conflicts
- Compliance with coding standards
- Improved maintainability

## Migration Checklist
- [ ] Create new module files
- [ ] Move code sections carefully
- [ ] Update all import statements
- [ ] Update module declarations in `main.rs`
- [ ] Test compilation after each move
- [ ] Verify all functionality still works
- [ ] Update any documentation references

## Testing
- Ensure all rendering works correctly
- Test input handling in all game states
- Verify save/load functionality
- Check tutorial progression
- Confirm power calculations work</content>
<parameter name="filePath">h:\WebHatchery\games\scrapyard\plans\improvement_04_split_large_files.md