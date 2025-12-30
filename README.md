# Scrapyard Planet

Spaceship management and survival game built with Rust and Macroquad.

## Overview

**Genre**: Tower Defense / Resource Management  
**Platform**: Desktop and Mobile (Windows + WebGL)  
**Engine**: Macroquad (Rust)

Scrapyard Planet is a tower defense game where players act as scrapyard operators repairing crashed spaceships on a hostile planet. Rogue nanomachines attack anything generating power, forcing players to balance ship repairs with defense. Destroyed nanomachines provide materials for further repairs, creating a tense risk-reward cycle. The goal is to restore the ship's power core, activate the engines, and escape before being overrun.

## How to Run

Ensure you have Rust installed.

```bash
cargo run
```

## Controls

- **WASD / Arrow Keys**: Move character (Interior View)
- **Tab**: Toggle between Interior and Exterior views
- **E**: Interact (Repair modules)
- **Hold E**: Gather scrap from piles (Interior View)
- **P**: Pause Game
- **Esc**: Return to Menu

## Core Gameplay Loop

1. **Arrival**: Start with a damaged ship and low enemy presence.
2. **Repair**: Spend materials to repair or upgrade ship modules.
3. **Defense**: Activated modules generate power, attracting nanomachine enemies.
4. **Combat**: Ship weapons auto-fire at enemies; enemies drop materials on death.
5. **Escalation**: Higher power increases enemy spawn rate and difficulty.
6. **Escape**: Power the engine to trigger a boss and escape timer.

Players must strategically sequence repairs to build defenses before attracting overwhelming enemy waves.

## Ship Modules

Ships have fixed layouts with 10–16 module slots, starting in states: Destroyed, Damaged, or Offline.

### Weapon Modules
*Kill enemies, generate materials*
- **Pulse Turret**: Medium damage, low power cost.
- **Beam Emitter**: Pierces enemies, high power cost.
- **Missile Rack**: AoE damage, consumes materials.

### Defense Modules
*Slow or mitigate enemies*
- **Shield Generator**: Absorbs damage.
- **EMP Field**: Slows enemies.
- **Decoy Beacon**: Redirects aggro.

### Utility Modules
*Economy and control*
- **Recycler**: +20% materials from kills.
- **Scanner**: Reveals hidden modules.
- **Overclock Node**: Temporary weapon boost (risky).

### Engine
- **Win Condition**: Requires repair and sustained power. Triggers boss spawn and escape countdown.

## Power System

Power is the core difficulty dial. Total active power determines enemy threat:
- **0–5 Power**: Light drones only
- **6–10 Power**: Medium enemies
- **11–15 Power**: Elites and swarms
- **16+ Power**: Boss modifiers

Power spikes from overclocking or events draw immediate surges.

## Enemies

Nanomachines spawn continuously toward the ship core, adapting to defenses.
- **Nanodrone**: Fast, low HP, basic material drop, swarm behavior.
- **Nanoguard**: Medium HP, targets defenses first.
- **Leech Unit**: Attaches to modules, drains power.
- **Siege Construct**: Slow, high damage, attacks hull directly.
- **Boss**: Spawns when engine activates. Overrides targeting, disables modules, splits into units.

## Win/Lose Conditions

### Win
- Power engines.
- Defeat boss.
- Survive escape timer.

### Lose
- Core HP reaches 0.
- Engine destroyed after activation.
- Materials depleted with no defenses.

## Development

Built using the [Macroquad](https://macroquad.rs/) game engine.
