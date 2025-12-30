# Improvement 1: Reorganize Code into Module Structure

## Problem
The codebase violates the AGENTS.md project structure by keeping everything flat in `src/`. All modules are in a single directory instead of the organized hierarchy specified in the project standards.

## Current Structure
```
src/
├── ai.rs
├── assets.rs
├── combat.rs
├── constants.rs
├── entities.rs
├── events.rs
├── gameplay.rs
├── input.rs
├── interior.rs
├── layout.rs
├── main.rs
├── player.rs
├── render.rs
├── resources.rs
├── settings.rs
├── ship.rs
├── state.rs
```

## Target Structure
```
src/
├── ship/         # Ship modules, layouts, repairs
│   ├── mod.rs
│   ├── ship.rs
│   ├── interior.rs
│   └── layout.rs
├── enemy/        # Enemy types, AI, spawning, bosses
│   ├── mod.rs
│   ├── ai.rs
│   ├── entities.rs
│   └── combat.rs
├── economy/      # Materials, rewards, upgrades, currency
│   ├── mod.rs
│   ├── resources.rs
│   └── upgrades.rs (extract from state.rs)
├── simulation/   # Game tick, power system, difficulty scaling
│   ├── mod.rs
│   ├── gameplay.rs
│   └── constants.rs
├── state/        # Game state, views, persistence
│   ├── mod.rs
│   ├── game_state.rs (split from state.rs)
│   ├── persistence.rs (save/load logic)
│   └── tutorial.rs (tutorial system)
├── ui/           # All rendering (read-only)
│   ├── mod.rs
│   ├── render.rs
│   ├── input.rs
│   └── assets.rs
├── data/         # Config loaders, templates
│   ├── mod.rs
│   └── settings.rs
└── util/         # Cross-platform utilities
    └── mod.rs
```

## Implementation Steps

1. Create the directory structure
2. Move existing files to appropriate directories
3. Create `mod.rs` files for each module
4. Update `main.rs` module declarations
5. Update `Cargo.toml` if needed
6. Update all `use` statements throughout the codebase
7. Run tests to ensure everything compiles

## Benefits
- Better code organization and maintainability
- Clearer separation of concerns
- Easier navigation for developers
- Compliance with project standards
- Reduced merge conflicts in team development

## Files to Modify
- All `src/*.rs` files (move and update imports)
- `src/main.rs` (update mod declarations)
- `Cargo.toml` (if needed for module structure)</content>
<parameter name="filePath">h:\WebHatchery\games\scrapyard\plans\improvement_01_reorganize_modules.md