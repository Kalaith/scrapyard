# Improvement 6: Implement Proper Error Handling

## Problem
The codebase uses `unwrap()` and `expect()` calls throughout, which will panic on errors. This is not robust for a production game, especially for WASM builds where graceful error handling is crucial.

## Current Issues

### In `gameplay.rs`:
```rust
let weapon_configs: HashMap<String, WeaponConfig> = serde_json::from_str(weapon_json)
    .expect("Failed to parse weapons.json");
```

### In `interior.rs`:
```rust
let data: ShipData = serde_json::from_str(json_str)
    .expect("Failed to parse ship JSON");
```

### In `settings.rs`:
```rust
let file = File::open(CONFIG_PATH)?;
let reader = BufReader::new(file);
let save_data: SaveData = serde_json::from_reader(reader).unwrap_or_default();
```

### In `state.rs`:
```rust
let upgrade_templates: Vec<UpgradeTemplate> = serde_json::from_str(include_str!("../assets/upgrades.json"))
    .expect("Failed to load upgrades.json");
```

## Solution
Implement proper error handling using `Result<T, E>` and the `anyhow` crate for ergonomic error handling.

## Implementation Steps

1. **Add error handling dependencies** to `Cargo.toml`
2. **Create custom error types** or use `anyhow::Result`
3. **Replace unwrap/expect** with proper error propagation
4. **Handle errors gracefully** in different contexts
5. **Add logging** for debugging

## Code Changes

### Update `Cargo.toml`:
```toml
[dependencies]
# ... existing deps ...
anyhow = "1.0"
thiserror = "1.0"
```

### Create custom error types (new file: `src/util/error.rs`):
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Failed to load asset: {path}")]
    AssetLoad { path: String },
    
    #[error("Failed to parse JSON: {source}")]
    JsonParse {
        #[from]
        source: serde_json::Error
    },
    
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error
    },
    
    #[error("Invalid game state: {message}")]
    InvalidState { message: String },
    
    #[error("Configuration error: {message}")]
    Config { message: String },
}

pub type Result<T> = anyhow::Result<T, GameError>;
```

### Update asset loading (in `gameplay.rs`):
```rust
use crate::util::error::{GameError, Result};

impl ModuleRegistry {
    pub fn new() -> Result<Self> {
        let weapon_json = include_str!("../assets/stats/weapons.json");
        let weapon_configs: HashMap<String, WeaponConfig> = 
            serde_json::from_str(weapon_json)
                .map_err(|e| GameError::JsonParse { source: e })?;
        
        // ... rest of initialization ...
        Ok(Self { stats })
    }
}
```

### Update ship loading (in `interior.rs`):
```rust
impl ShipInterior {
    pub fn from_json(json_str: &str) -> Result<Self> {
        let data: ShipData = serde_json::from_str(json_str)
            .map_err(|e| GameError::JsonParse { source: e })?;
        
        // ... rest of construction ...
        Ok(Self { rooms, width, height })
    }
    
    pub fn starter_ship() -> Self {
        const SHIP_JSON: &str = include_str!("../assets/ships/starter_ship.json");
        Self::from_json(SHIP_JSON)
            .unwrap_or_else(|e| {
                eprintln!("Failed to load starter ship: {}", e);
                // Return a minimal fallback ship
                Self {
                    rooms: Vec::new(),
                    width: 1000.0,
                    height: 600.0,
                }
            })
    }
}
```

### Update settings loading (in `settings.rs`):
```rust
impl Settings {
    pub fn load() -> Self {
        match Self::load_from_file() {
            Ok(settings) => settings,
            Err(e) => {
                eprintln!("Failed to load settings, using defaults: {}", e);
                Self::default()
            }
        }
    }
    
    fn load_from_file() -> Result<Self> {
        let file = File::open(CONFIG_PATH)?;
        let reader = BufReader::new(file);
        let settings = serde_json::from_reader(reader)?;
        Ok(settings)
    }
    
    pub fn save(&self) -> Result<()> {
        let file = File::create(CONFIG_PATH)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}
```

### Update GameState initialization (in `state.rs`):
```rust
impl GameState {
    pub fn new() -> Result<Self> {
        let interior = ShipInterior::starter_ship();
        let player = Player::new_at(interior.player_start_position());
        
        let module_registry = ModuleRegistry::new()?;
        
        // ... rest of initialization ...
        
        let upgrade_templates: Vec<UpgradeTemplate> = 
            serde_json::from_str(include_str!("../assets/upgrades.json"))?;
        
        Ok(Self {
            // ... fields ...
        })
    }
}
```

### Update main.rs to handle initialization errors:
```rust
#[macroquad::main("Scrapyard Planet")]
async fn main() -> Result<()> {
    let mut game_state = GameState::new()
        .unwrap_or_else(|e| {
            eprintln!("Failed to initialize game: {}", e);
            std::process::exit(1);
        });
    
    let mut asset_manager = AssetManager::new();
    if let Err(e) = asset_manager.load_assets().await {
        eprintln!("Failed to load assets: {}", e);
        // Continue with placeholder assets
    }
    
    // ... rest of main loop ...
    Ok(())
}
```

## Benefits
- Graceful error handling instead of panics
- Better debugging with detailed error messages
- Robust WASM builds that don't crash
- Easier troubleshooting for players
- Professional error handling practices

## Testing
- Test with corrupted JSON files
- Verify graceful fallback behavior
- Check error logging in console
- Ensure WASM builds handle errors properly
- Test save/load error scenarios</content>
<parameter name="filePath">h:\WebHatchery\games\scrapyard\plans\improvement_06_error_handling.md