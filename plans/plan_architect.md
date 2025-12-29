# Implementation Plan: [ARCHITECT]

**Role**: Core Systems, Data Structures, State Management, Memory Safety.
**Goal**: Build the rigid skeleton and nervous system of the game.

## Phase 1: The Hull (Core Arch)
- [x] **Application Skeleton**
    - `main.rs`: Entry point, window config (1280x720, resizable).
    - `Constants`: Global defines for Grid Size, Cell Size, Colors.
- [x] **State Management**
    - `GameState` struct:
        - `ship`: `Ship`
        - `resources`: `Resources`
        - `enemies`: `Vec<Enemy>`
        - `projectiles`: `Vec<Projectile>`
        - `particles`: `Vec<Particle>`
        - `game_phase`: `Enum { Arrival, Defense, Escape, GameOver, Victory }`
- [x] **Data Serialization (Serde)**
    - Implement `SaveData` struct mapping for `GameState`.
    - JSON Load/Save for persisting run state (optional but good architecture).

## Phase 2: The Spark (Input & Events)
- [ ] **Input Manager**
    - `InputState` struct: Captures MousePos, Clicks, Keys.
    - `Raycaster`: `screen_to_world(mouse_pos) -> Option<GridCoord>`.
    - `UIEvent` Enum: `Repair(Coord)`, `Upgrade(Coord)`, `Pause`, `Resume`.
- [ ] **Event Bus (Optional but recommended)**
    - Simple `Vec<GameEvent>` queue for decoupling UI clicks from Game Logic updates.

## Phase 3: The Swarm (Optimization)
- [ ] **Spatial Hash Grid**
    - If implementation shows >100 entities, implement a spatial lookup to avoid O(N²) collision checks.
    - `SpatialMap`: Map GridCoord -> Vec<EntityIndex>.
- [ ] **Object Pooling**
    - `ProjectilePool`: Pre-allocate 1000 projectiles to avoid runtime allocation.
    - `ParticlePool`: Pre-allocate visual effects.

## Phase 4: Polish (System)
- [ ] **Settings System**
    - `Settings` struct: Volume (SFX/Music), Fullscreen toggle.
    - Save specific settings to `config.json`.
- [ ] **Error Handling**
    - Graceful crash handling (logging panic info to file).
