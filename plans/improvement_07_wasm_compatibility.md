# Improvement 7: Fix WASM Compatibility Issues

## Problem
Several parts of the codebase use `std::fs` which doesn't work in WASM builds. The AGENTS.md specifies that WASM cannot use `std::fs` and must use `include_str!` for JSON configs.

## Current Issues

### In `settings.rs`:
```rust
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn load() -> Self {
    match File::open(CONFIG_PATH) {
        Ok(file) => {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_default()
        }
        Err(_) => Self::default(),
    }
}

pub fn save(&self) -> std::io::Result<()> {
    let file = File::create(CONFIG_PATH)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, self)?;
    Ok(())
}
```

### In `state.rs` (save/load functions):
```rust
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn save(&self, path: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    // ...
}

pub fn load_from_file(path: &str) -> std::io::Result<Self> {
    let file = File::open(path)?;
    // ...
}
```

## Solution
Implement conditional compilation to use different approaches for WASM vs native builds, following the AGENTS.md pattern.

## Implementation Steps

1. **Replace file I/O with conditional compilation**
2. **Use browser storage APIs for WASM settings**
3. **Disable save/load functionality for WASM** or implement alternative
4. **Ensure all asset loading uses `include_str!`**

## Code Changes

### Update `settings.rs` for WASM compatibility:
```rust
use serde::{Deserialize, Serialize};

const CONFIG_PATH: &str = "config.json";

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufReader, BufWriter};

#[cfg(target_arch = "wasm32")]
use web_sys::{window, Storage};

impl Settings {
    pub fn load() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::load_from_web_storage().unwrap_or_default()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::load_from_file().unwrap_or_default()
        }
    }
    
    #[cfg(target_arch = "wasm32")]
    fn load_from_web_storage() -> Option<Self> {
        let window = window()?;
        let storage = window.local_storage().ok()??;
        let json = storage.get_item("scrapyard_settings").ok()??;
        serde_json::from_str(&json).ok()
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn load_from_file() -> Option<Self> {
        File::open(CONFIG_PATH)
            .ok()
            .and_then(|file| {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader).ok()
            })
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_arch = "wasm32")]
        {
            self.save_to_web_storage()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.save_to_file()
        }
    }
    
    #[cfg(target_arch = "wasm32")]
    fn save_to_web_storage(&self) -> Result<(), Box<dyn std::error::Error>> {
        let window = window().ok_or("No window")?;
        let storage = window.local_storage().map_err(|_| "No storage")?;
        let json = serde_json::to_string(self)?;
        storage.set_item("scrapyard_settings", &json)
            .map_err(|_| "Failed to save")?;
        Ok(())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(CONFIG_PATH)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}
```

### Update save/load in `state.rs`:
```rust
impl GameState {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        // ... existing save implementation ...
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn save(&self, _path: &str) -> std::io::Result<()> {
        // WASM save not implemented - could use IndexedDB in future
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported, 
            "Save not supported in web version"
        ))
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        // ... existing load implementation ...
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn load_from_file(_path: &str) -> std::io::Result<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported, 
            "Load not supported in web version"
        ))
    }
}
```

### Ensure all JSON loading uses `include_str!`:
Verify that all asset loading in the codebase follows the pattern:

```rust
#[cfg(target_arch = "wasm32")]
let json = include_str!("../../assets/config.json");

#[cfg(not(target_arch = "wasm32"))]
let json = std::fs::read_to_string("assets/config.json")
    .unwrap_or_else(|_| include_str!("../../assets/config.json").to_string());
```

### Update main.rs to handle WASM limitations:
```rust
#[macroquad::main("Scrapyard Planet")]
async fn main() {
    // ... initialization ...
    
    // Disable save/load UI elements for WASM
    #[cfg(target_arch = "wasm32")]
    {
        println!("Note: Save/Load not available in web version");
    }
    
    // ... main loop ...
}
```

### Add WASM-specific dependencies to `Cargo.toml`:
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
web-sys = { version = "0.3", features = [
    "Window",
    "Storage",
    "console",
] }
wasm-bindgen = "0.2"
```

## Benefits
- WASM builds work without crashes
- Proper platform-specific behavior
- Graceful degradation of features
- Compliance with AGENTS.md requirements
- Better user experience across platforms

## Testing
- Test native build save/load functionality
- Verify WASM build doesn't crash on settings
- Check web storage persistence
- Ensure graceful handling of unsupported features
- Test both build targets compile successfully</content>
<parameter name="filePath">h:\WebHatchery\games\scrapyard\plans\improvement_07_wasm_compatibility.md