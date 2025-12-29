# Implementation Plan: [MECHANIC]

**Role**: Gameplay Logic, AI, Numbers, Combat Math.
**Goal**: Make the game fun, fair, and functional.

## Phase 1: The Hull (Data Definitions)
- [x] **Module Registry**
    - Define `ModuleStats` struct:
        - `base_cost`, `power_consumption`, `health`, `range`, `damage`, `fire_rate`.
    - Hardcode/Load stats for: `PulseTurret`, `BeamEmitter`, `Shield`, `Recycler`, `Engine`.
- [x] **Ship Grid Logic**
    - `is_valid_slot(x, y)`
    - `calculate_path_to_core(start_pos)`: Simple BFS or A* from edge to core.

## Phase 2: The Spark (Simulation Loop)
- [ ] **Resource System**
    - `Materials`: Cap at 500 (soft) / Hard cap logic.
    - `Power`: Recalculate every frame: `Sum(Active Modules * PowerCost)`.
    - `Credits`: Meta-currency accumulator.
- [ ] **Interaction Resolvers**
    - `attempt_repair(slot_id)`:
        - Check Cost <= Materials.
        - Deduct Cost.
        - Set State: Destroyed/Offline -> Active (or Offline if manual toggle).
    - `attempt_upgrade(slot_id)`:
        - Check logic for Max Level.
        - Multiply stats by 1.2x (or linear growth).
- [ ] **Engine Logic**
    - State: `Idle` -> `Charging` (Boss Spawn) -> `Escaped` (Win).
    - Timer: 180s countdown once activated.

## Phase 3: The Swarm (AI & Combat)
- [ ] **Wave Manager (The "Director")**
    - Input: `CurrentPower`.
    - Output: Spawn Events.
    - **Logic**:
        - Power 0-5: `Drone` every 3s.
        - Power 6-10: `Drone` every 1s + `Guard` every 10s.
        - Power 16+ (Boss): Stop normal waves, spawn `Boss`.
- [ ] **Enemy Logic**
    - **Behaviors**:
        - `Rusher` (Drone): Move directly to Core.
        - `Tank` (Guard): Move to nearest `Weapon`/`Shield` module first.
        - `Leech`: Move to `Generator` and stay attached (draining power).
- [ ] **Combat Math**
    - `Projectile` resolution:
        - Box Collision check against Enemy list.
        - OnHit: `Enemy.hp -= projectile.damage`.
    - `Enemy Attack`:
        - Range check to Target Module.
        - `Module.hp -= enemy.dps * dt`.
        - If `Module.hp <= 0`: State -> `Destroyed`.

## Phase 4: Polish (Progression)
- [ ] **Meta-Progression**
    - Unlocks: "Unlock Beam Emitter = 500 Credits".
    - Persistent Stat modifiers: "Start with +50 Materials".
- [ ] **Boss Logic**
    - **Phase 1**: Direct attack.
    - **Phase 2 (50% HP)**: "EMP Blast" -> Disable 50% random modules for 10s.
