# Improvement 9: Make Tutorial System Data-Driven

## Problem
The tutorial system is hardcoded in `state.rs` with magic strings and rigid logic. This makes it difficult to modify, localize, or extend the tutorial.

## Current Issues

### Hardcoded Tutorial Steps (`state.rs`):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialStep {
    Welcome,
    RepairReactor,
    RepairShields,
    RepairWeapon,
    RepairEngine,
    Complete,
}

impl TutorialStep {
    pub fn target_room(&self) -> Option<usize> {
        match self {
            TutorialStep::RepairReactor => Some(12),
            TutorialStep::RepairShields => Some(5),
            TutorialStep::RepairWeapon => Some(1),
            TutorialStep::RepairEngine => Some(20),
            _ => None,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            TutorialStep::Welcome => "Welcome aboard! Your ship is damaged.\nUse WASD to move. Press E near orange repair points.",
            TutorialStep::RepairReactor => "First, repair the REACTOR to restore power.\nFollow the highlighted path to the central room.",
            TutorialStep::RepairShields => "Good! Now repair the SHIELDS for defense.\nHead to the shield room above.",
            TutorialStep::RepairWeapon => "Shields online! Repair a WEAPON to fight back.\nGo to the left weapon bay.",
            TutorialStep::RepairEngine => "Weapons ready! Finally, repair the ENGINE.\nHead to the engine room below.",
            TutorialStep::Complete => "All systems operational! Enemies will now attack.\nPress Tab to view exterior. Good luck!",
        }
    }

    pub fn next(&self) -> TutorialStep {
        match self {
            TutorialStep::Welcome => TutorialStep::RepairReactor,
            TutorialStep::RepairReactor => TutorialStep::RepairShields,
            TutorialStep::RepairShields => TutorialStep::RepairWeapon,
            TutorialStep::RepairWeapon => TutorialStep::RepairEngine,
            TutorialStep::RepairEngine => TutorialStep::Complete,
            TutorialStep::Complete => TutorialStep::Complete,
        }
    }
}
```

### Hardcoded Advancement Logic:
```rust
// In input.rs
if state.attempt_interior_repair(room_idx, point_idx) {
    // Advance tutorial if this is the target room (just need 1 repair)
    if let Some(target) = state.tutorial_step.target_room() {
        if room_idx == target {
            state.tutorial_step = state.tutorial_step.next();
        }
    }
}
```

## Solution
Move tutorial configuration to JSON and make the system data-driven.

## Implementation Steps

1. **Create tutorial JSON configuration**
2. **Replace hardcoded enum with data structure**
3. **Update tutorial logic to use configuration**
4. **Add tutorial state management**

## Code Changes

### Create `assets/tutorial.json`:
```json
{
  "steps": [
    {
      "id": "welcome",
      "title": "Welcome Aboard",
      "message": "Welcome aboard! Your ship is damaged.\nUse WASD to move. Press E near orange repair points.",
      "target_room": null,
      "auto_advance": false,
      "show_highlight": false
    },
    {
      "id": "repair_reactor",
      "title": "Repair Reactor",
      "message": "First, repair the REACTOR to restore power.\nFollow the highlighted path to the central room.",
      "target_room": 12,
      "auto_advance": true,
      "show_highlight": true,
      "trigger_condition": "room_repaired"
    },
    {
      "id": "repair_shields",
      "title": "Repair Shields",
      "message": "Good! Now repair the SHIELDS for defense.\nHead to the shield room above.",
      "target_room": 5,
      "auto_advance": true,
      "show_highlight": true,
      "trigger_condition": "room_repaired"
    },
    {
      "id": "repair_weapon",
      "title": "Repair Weapons",
      "message": "Shields online! Repair a WEAPON to fight back.\nGo to the left weapon bay.",
      "target_room": 1,
      "auto_advance": true,
      "show_highlight": true,
      "trigger_condition": "room_repaired"
    },
    {
      "id": "repair_engine",
      "title": "Repair Engine",
      "message": "Weapons ready! Finally, repair the ENGINE.\nHead to the engine room below.",
      "target_room": 20,
      "auto_advance": true,
      "show_highlight": true,
      "trigger_condition": "room_repaired"
    },
    {
      "id": "complete",
      "title": "Tutorial Complete",
      "message": "All systems operational! Enemies will now attack.\nPress Tab to view exterior. Good luck!",
      "target_room": null,
      "auto_advance": false,
      "show_highlight": false
    }
  ]
}
```

### Create Tutorial Configuration Structs (`src/state/tutorial.rs`):
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TutorialStepConfig {
    pub id: String,
    pub title: String,
    pub message: String,
    pub target_room: Option<usize>,
    pub auto_advance: bool,
    pub show_highlight: bool,
    pub trigger_condition: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TutorialConfig {
    pub steps: Vec<TutorialStepConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialState {
    pub current_step_index: usize,
    pub completed: bool,
    pub timer: f32,
}

impl TutorialState {
    pub fn new() -> Self {
        Self {
            current_step_index: 0,
            completed: false,
            timer: 0.0,
        }
    }
    
    pub fn current_step<'a>(&self, config: &'a TutorialConfig) -> Option<&'a TutorialStepConfig> {
        config.steps.get(self.current_step_index)
    }
    
    pub fn advance(&mut self, config: &TutorialConfig) {
        if self.current_step_index + 1 < config.steps.len() {
            self.current_step_index += 1;
        } else {
            self.completed = true;
        }
    }
    
    pub fn can_advance(&self, config: &TutorialConfig, condition: &str) -> bool {
        if let Some(step) = self.current_step(config) {
            if step.auto_advance {
                return step.trigger_condition.as_ref() == Some(&condition.to_string());
            }
        }
        false
    }
}
```

### Update GameState to use Tutorial System (`state.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    // ... existing fields ...
    #[serde(skip)]
    pub tutorial_config: TutorialConfig,
    pub tutorial_state: TutorialState,
    // ... rest ...
}

impl GameState {
    pub fn new() -> Self {
        // ... existing code ...
        
        let tutorial_config: TutorialConfig = serde_json::from_str(
            include_str!("../assets/tutorial.json")
        ).expect("Failed to load tutorial.json");
        
        let tutorial_state = TutorialState::new();
        
        Self {
            // ... existing fields ...
            tutorial_config,
            tutorial_state,
            // ... rest ...
        }
    }
    
    pub fn start_new_game(&mut self) {
        // ... existing reset ...
        self.tutorial_state = TutorialState::new();
    }
    
    // Helper methods
    pub fn current_tutorial_step(&self) -> Option<&TutorialStepConfig> {
        self.tutorial_state.current_step(&self.tutorial_config)
    }
    
    pub fn advance_tutorial(&mut self, condition: &str) {
        if self.tutorial_state.can_advance(&self.tutorial_config, condition) {
            self.tutorial_state.advance(&self.tutorial_config);
        }
    }
}
```

### Update Tutorial Rendering (`render.rs`):
```rust
fn draw_tutorial(&self, state: &GameState) {
    if let Some(step) = state.current_tutorial_step() {
        // Semi-transparent background
        let box_height = 100.0;
        draw_rectangle(0.0, 0.0, screen_width(), box_height, 
            color_u8!(0, 0, 0, 200));
        
        // Title
        draw_text(&step.title, 20.0, 25.0, 18.0, WHITE);
        
        // Message
        let lines: Vec<&str> = step.message.split('\n').collect();
        for (i, line) in lines.iter().enumerate() {
            draw_text(line, 20.0, 50.0 + i as f32 * 20.0, 16.0, WHITE);
        }
        
        // Highlight target room if needed
        if step.show_highlight {
            if let Some(room_idx) = step.target_room {
                if let Some(room) = state.interior.rooms.get(room_idx) {
                    draw_rectangle_lines(room.x, room.y, room.width, room.height, 
                        3.0, YELLOW);
                }
            }
        }
    }
}
```

### Update Tutorial Advancement Logic (`input.rs`):
```rust
if state.attempt_interior_repair(room_idx, point_idx) {
    // Check if this completes a tutorial step
    if let Some(step) = state.current_tutorial_step() {
        if step.target_room == Some(room_idx) {
            state.advance_tutorial("room_repaired");
        }
    }
}
```

## Benefits
- Easily modifiable tutorial content
- Support for localization
- Configurable advancement conditions
- Better tutorial highlighting
- Extensible for future tutorial features

## Testing
- Verify tutorial steps advance correctly
- Test room highlighting functionality
- Check JSON loading and parsing
- Ensure tutorial state saves/loads properly
- Test edge cases (corrupted JSON, missing steps)</content>
<parameter name="filePath">h:\WebHatchery\games\scrapyard\plans\improvement_09_tutorial_data_driven.md