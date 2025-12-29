# Agent Instructions (AGENTS.md)

**Project**: Scrapyard Planet  
**Engine**: Macroquad (Rust)  
**Platform**: Windows + WebGL (itch.io)

This document provides instructions for AI agents working on this project.

---

## 1. Critical Rules

### 1.1 No Cargo Commands
**Never run cargo commands** (cargo run, cargo build, cargo check, cargo test). The user will run these manually.

### 1.2 Follow CODE_STANDARDS.md
All code must align with the project's Rust coding standards. Key highlights:
- Readability over cleverness
- Module responsibilities are strict (see Section 2.1 of CODE_STANDARDS.md)
- Target 200-400 lines per file, max 800
- Functions target 20-50 lines, max 100
- UI code is "dumb" - reads state, emits actions, no business logic

---

## 2. Project Structure

```
src/
├── ship/         # Ship modules, layouts, repairs
├── enemy/        # Enemy types, AI, spawning, bosses
├── economy/      # Materials, rewards, upgrades, currency
├── mission/      # Mission flow, win/lose conditions, campaigns
├── simulation/   # Game tick, power system, difficulty scaling
├── state/        # Game state, views, persistence
├── ui/           # All rendering (read-only)
├── data/         # Config loaders, templates
└── util/         # Cross-platform utilities
assets/
├── *.json        # Configuration files (data-driven)
└── textures/     # PNG images
```

---

## 3. WebGL/WASM Builds

### 3.1 File Loading for WASM
**WASM cannot use `std::fs`**. Use `include_str!` for JSON configs:

```rust
#[cfg(target_arch = "wasm32")]
let json = include_str!("../../assets/config.json");

#[cfg(not(target_arch = "wasm32"))]
let json = std::fs::read_to_string("assets/config.json")
    .unwrap_or_else(|_| include_str!("../../assets/config.json").to_string());
```

### 3.2 Random Numbers
Use `macroquad::rand` (not the `rand` crate) for WASM compatibility:
```rust
use macroquad::rand::gen_range;
let value = gen_range(0, 100);
```

### 3.3 Asset Paths
Use **relative paths** (no leading `/`):
```rust
// Correct
let path = format!("assets/textures/{}.png", id);

// Wrong - absolute path breaks itch.io
let path = format!("/assets/textures/{}.png", id);
```

---

## 4. Publishing

### 4.1 Build Script
Use `publish.ps1` to create distributable packages:
```powershell
.\publish.ps1              # Windows + WebGL
.\publish.ps1 -WindowsOnly # Windows only
.\publish.ps1 -WebGLOnly   # WebGL only
```

### 4.2 Itch.io Settings
For WebGL uploads on itch.io:
- Enable "This file will be played in the browser"
- Set viewport dimensions: **1280 x 720**
- SharedArrayBuffer: OFF

---

## 5. Graphics & Assets

### 5.1 Requesting Graphics
When graphics are needed, create a prompt request using these guidelines:

**Prompt Template:**
```
Create a [SIZE] pixel art image for [SUBJECT].
Style: [STYLE DESCRIPTION]
Background: [transparent/solid color]
Purpose: [in-game icon/portrait/background/etc]
```

**Example Prompts:**
```
Create a 64x64 pixel art icon of a pulse turret.
Style: Sci-fi, metallic, glowing energy
Background: Transparent
Purpose: Weapon module icon

Create a 256x256 pixel art portrait of a nanodrone enemy.
Style: Mechanical, hostile, red glow
Background: Transparent
Purpose: Enemy archetype portrait
```

### 5.2 Asset Naming Convention
```
module_[type].png          # module_weapon.png
enemy_[type].png           # enemy_nanodrone.png
icon_[function].png        # icon_materials.png
ship_[layout].png          # ship_crashed.png
background_[scene].png     # background_planet.png
```

### 5.3 Asset Sizes
- **Icons**: 32x32 or 64x64
- **Portraits**: 256x256
- **Backgrounds**: 1280x720 or larger
- **Ship layouts**: 1280x720

---

## 6. Data-Driven Design

Configuration lives in JSON files under `assets/`:
- `config.json` - Core game balance (power scaling, spawn rates)
- `modules.json` - Module definitions (weapons, defense, utility)
- `enemies.json` - Enemy types and behaviors
- `ships.json` - Ship layouts and starting states
- `missions.json` - Mission definitions and campaigns

**Prefer adding to JSON over modifying Rust code** when possible.

---

## 7. Common Patterns

### 7.1 UI Actions
UI returns actions, game state handles them:
```rust
pub enum UiAction {
    RepairModule { module_id: usize },
    UpgradeModule { module_id: usize },
    ActivateEngine,
    PauseGame,
}
```

### 7.2 Game Events
Events are logged and displayed:
```rust
pub enum GameEvent {
    ModuleRepaired { name: String, power_cost: i32 },
    EnemyKilled { enemy_type: String, materials_dropped: i32 },
    BossSpawned,
    MissionComplete { reward: i32 },
}
```

### 7.3 Flags System
Ships and modules use string flags:
```rust
ship.flags.insert("engine_online".to_string());
if module.flags.contains("damaged") { ... }
```

---

## 8. Testing

Focus tests on:
- Power and difficulty calculations
- Enemy spawning and AI
- Economy and reward systems
- Mission win/lose logic

Do NOT write tests for UI rendering.

---

## 9. Debugging Tips

- Check the browser console (F12) for WASM errors
- JSON parse errors usually mean malformed config files
- 403 errors on itch.io = assets not being served (check zip structure)
- Black screen in WebGL = likely WASM loading failure

---

## 10. Quick Reference

| Task | Command/Action |
|------|----------------|
| Build Windows | User runs `cargo build --release` |
| Build WebGL | User runs `cargo build --release --target wasm32-unknown-unknown` |
| Package for release | `.\publish.ps1` |
| Test locally (WebGL) | `python -m http.server 8080` in dist/webgl |
| Add new module | Edit assets/modules.json |
| Add new enemy | Edit assets/enemies.json |
| Add new ship layout | Edit assets/ships.json |
| Add new mission | Edit assets/missions.json |
