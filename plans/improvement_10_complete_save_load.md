# Improvement 10: Complete Save/Load System

## Problem
The current save/load system is incomplete and inconsistent. It saves some state but not all, and the load function doesn't properly restore the game to a playable state.

## Current Issues

### Incomplete Save Data (`state.rs`):
The `SaveData` struct is missing many fields:
```rust
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub ship: Ship,
    pub resources: Resources,
    pub phase: GamePhase,
    pub engine_state: EngineState,
    pub escape_timer: f32,
    pub enemies: Vec<SavedEnemy>,
    pub projectiles: Vec<SavedProjectile>,
    pub particles: Vec<SavedParticle>,
    pub scrap_piles: Vec<SavedScrapPile>,
    pub frame_count: u64,
    // Missing: tutorial_state, upgrades, interior repairs, etc.
}
```

### Missing State in Save:
- Tutorial progress
- Upgrade levels and templates
- Interior room repair states
- Player position and state
- Wave state (spawn timers)
- View mode
- Ship integrity

### Load Function Issues:
- Doesn't restore tutorial state
- Missing error handling
- Doesn't validate save file version
- Some entities may not load correctly

## Solution
Complete the save/load system to preserve all necessary game state for a proper save/load experience.

## Implementation Steps

1. **Expand SaveData struct** to include all state
2. **Add save version field** for compatibility
3. **Implement comprehensive save/load**
4. **Add save slot system**
5. **Handle save corruption gracefully**

## Code Changes

### Update SaveData Structure (`state.rs`):
```rust
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub timestamp: u64,
    
    // Core game state
    pub ship: Ship,
    pub interior: ShipInterior,
    pub resources: Resources,
    pub phase: GamePhase,
    pub view_mode: ViewMode,
    
    // Player state
    pub player_position: (f32, f32),
    
    // Ship state
    pub ship_integrity: f32,
    pub ship_max_integrity: f32,
    
    // Engine state
    pub engine_state: EngineState,
    pub escape_timer: f32,
    
    // Tutorial state
    pub tutorial_step_index: usize,
    pub tutorial_completed: bool,
    
    // Upgrade state
    pub upgrade_levels: std::collections::HashMap<String, u32>,
    
    // Wave state
    pub spawn_timer: f32,
    pub guard_timer: f32,
    
    // Entities
    pub enemies: Vec<SavedEnemy>,
    pub projectiles: Vec<SavedProjectile>,
    pub particles: Vec<SavedParticle>,
    pub scrap_piles: Vec<SavedScrapPile>,
    
    // Metadata
    pub frame_count: u64,
    pub play_time_seconds: f32,
}

const SAVE_VERSION: u32 = 1;
```

### Update Save Function:
```rust
impl GameState {
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let save_data = SaveData {
            version: SAVE_VERSION,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            
            ship: self.ship.clone(),
            interior: self.interior.clone(),
            resources: self.resources.clone(),
            phase: self.phase,
            view_mode: self.view_mode,
            
            player_position: (self.player.position.x, self.player.position.y),
            
            ship_integrity: self.ship_integrity,
            ship_max_integrity: self.ship_max_integrity,
            
            engine_state: self.engine_state,
            escape_timer: self.escape_timer,
            
            tutorial_step_index: self.tutorial_state.current_step_index,
            tutorial_completed: self.tutorial_state.completed,
            
            upgrade_levels: self.upgrades.levels.clone(),
            
            spawn_timer: self.wave_state.spawn_timer,
            guard_timer: self.wave_state.guard_timer,
            
            enemies: self.enemies.iter().map(|e| SavedEnemy {
                id: e.id,
                enemy_type: e.enemy_type.clone(),
                pos: (e.position.x, e.position.y),
                hp: e.health,
                max_hp: e.max_health,
                speed: e.speed,
                damage: e.damage,
                target: e.target_module,
            }).collect(),
            
            projectiles: self.projectiles.iter().map(|p| SavedProjectile {
                pos: (p.position.x, p.position.y),
                vel: (p.velocity.x, p.velocity.y),
                damage: p.damage,
                active: p.active,
            }).collect(),
            
            particles: self.particles.iter().map(|p| SavedParticle {
                pos: (p.position.x, p.position.y),
                vel: (p.velocity.x, p.velocity.y),
                life: p.lifetime,
                max_life: p.max_lifetime,
                color: (p.color.r, p.color.g, p.color.b, p.color.a),
                active: p.active,
            }).collect(),
            
            scrap_piles: self.scrap_piles.iter().map(|p| SavedScrapPile {
                pos: (p.position.x, p.position.y),
                amount: p.amount,
                active: p.active,
            }).collect(),
            
            frame_count: self.frame_count,
            play_time_seconds: self.frame_count as f32 / 60.0, // Assuming 60 FPS
        };
        
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &save_data)?;
        Ok(())
    }
}
```

### Update Load Function:
```rust
impl GameState {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let save_data: SaveData = serde_json::from_reader(reader)?;
        
        // Version check
        if save_data.version != SAVE_VERSION {
            return Err(format!("Incompatible save version: {}", save_data.version).into());
        }
        
        // Create base game state
        let mut state = Self::new()?;
        
        // Restore saved state
        state.ship = save_data.ship;
        state.interior = save_data.interior;
        state.resources = save_data.resources;
        state.phase = save_data.phase;
        state.view_mode = save_data.view_mode;
        
        state.player.position = vec2(save_data.player_position.0, save_data.player_position.1);
        
        state.ship_integrity = save_data.ship_integrity;
        state.ship_max_integrity = save_data.ship_max_integrity;
        
        state.engine_state = save_data.engine_state;
        state.escape_timer = save_data.escape_timer;
        
        state.tutorial_state.current_step_index = save_data.tutorial_step_index;
        state.tutorial_state.completed = save_data.tutorial_completed;
        
        state.upgrades.levels = save_data.upgrade_levels;
        
        state.wave_state.spawn_timer = save_data.spawn_timer;
        state.wave_state.guard_timer = save_data.guard_timer;
        
        // Restore entities
        state.enemies = save_data.enemies.into_iter().map(|s| Enemy {
            id: s.id,
            enemy_type: s.enemy_type,
            position: vec2(s.pos.0, s.pos.1),
            health: s.hp,
            max_health: s.max_hp,
            speed: s.speed,
            damage: s.damage,
            target_module: s.target,
        }).collect();
        
        state.projectiles = save_data.projectiles.into_iter().map(|s| Projectile {
            position: vec2(s.pos.0, s.pos.1),
            velocity: vec2(s.vel.0, s.vel.1),
            damage: s.damage,
            active: s.active,
        }).collect();
        
        state.particles = save_data.particles.into_iter().map(|s| Particle {
            position: vec2(s.pos.0, s.pos.1),
            velocity: vec2(s.vel.0, s.vel.1),
            lifetime: s.life,
            max_lifetime: s.max_life,
            color: Color::new(s.color.0, s.color.1, s.color.2, s.color.3),
            active: s.active,
        }).collect();
        
        state.scrap_piles = save_data.scrap_piles.into_iter().map(|s| 
            ScrapPile {
                position: vec2(s.pos.0, s.pos.1),
                amount: s.amount,
                active: s.active,
            }
        ).collect();
        
        state.frame_count = save_data.frame_count;
        
        // Rebuild caches
        state.invalidate_core_cache();
        
        Ok(state)
    }
}
```

### Add Save Slot System:
```rust
impl GameState {
    pub fn get_save_slot_path(slot: usize) -> String {
        format!("save_slot_{}.json", slot)
    }
    
    pub fn save_to_slot(&self, slot: usize) -> std::io::Result<()> {
        let path = Self::get_save_slot_path(slot);
        self.save(&path)
    }
    
    pub fn load_from_slot(slot: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::get_save_slot_path(slot);
        Self::load_from_file(&path)
    }
    
    pub fn list_save_slots() -> Vec<(usize, Option<u64>)> {
        let mut slots = Vec::new();
        for slot in 0..5 { // Support 5 save slots
            let path = Self::get_save_slot_path(slot);
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let timestamp = modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    slots.push((slot, Some(timestamp)));
                } else {
                    slots.push((slot, None));
                }
            } else {
                slots.push((slot, None));
            }
        }
        slots
    }
}
```

### Add Autosave Functionality:
```rust
impl GameState {
    pub fn autosave(&self) -> std::io::Result<()> {
        let path = "autosave.json";
        self.save(path)
    }
    
    pub fn load_autosave() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_from_file("autosave.json")
    }
    
    // Call autosave periodically
    pub fn update(&mut self, dt: f32, events: &mut EventBus) {
        // ... existing update logic ...
        
        // Autosave every 5 minutes of play time
        if self.frame_count % (5 * 60 * 60) == 0 {
            if let Err(e) = self.autosave() {
                eprintln!("Failed to autosave: {}", e);
            }
        }
    }
}
```

## Benefits
- Complete game state preservation
- Multiple save slots
- Autosave functionality
- Version checking for compatibility
- Better error handling

## Testing
- Test saving and loading all game states
- Verify tutorial progress persists
- Check upgrade levels are restored
- Test save slot system
- Ensure corrupted saves are handled gracefully
- Verify autosave doesn't impact performance</content>
<parameter name="filePath">h:\WebHatchery\games\scrapyard\plans\improvement_10_complete_save_load.md