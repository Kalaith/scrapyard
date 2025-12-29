# Scrapyard Planet: Master Implementation Plan

## Goal
Implement "Scrapyard Planet", a tower defense/resource management game where the map is the player's ship. This plan coordinates the efforts of four specialized agent roles.

## Agent Role Plans
Access the detailed task lists for each role below:

- **[ARCHITECT Plan](plans/plan_architect.md)**
  *Systems, Data Structures, State Management*

- **[MECHANIC Plan](plans/plan_mechanic.md)**
  *Gameplay Logic, Combat Math, AI Loop*

- **[CONSTRUCT Plan](plans/plan_construct.md)**
  *Rendering, UI, Audio, Visual Feedback*

- **[OBSERVER Plan](plans/plan_observer.md)**
  *Quality Assurance, Verification, Balancing*

---

## Phase Overview

### Phase 1: The Hull (Foundation)
Establishing the core engine, window, and static data structures.
*Key Deliverable*: A window displaying the ship grid.

### Phase 2: The Spark (Interactivity)
Enabling mouse interaction, resource tracking, and module activation.
*Key Deliverable*: Click to repair modules, resources deduct.

### Phase 3: The Swarm (Combat)
Implementing the game loop: Spawn Enemies -> Fire Weapons -> Destroy -> Loot.
*Key Deliverable*: Win/Loss condition achievable.

### Phase 4: Polish
Adding "juice", narrative elements, and balancing.
*Key Deliverable*: Narrative tutorial and polished UI.