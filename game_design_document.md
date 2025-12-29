# Scrapyard Planet - Game Design Document

## Overview

**Genre**: Tower Defense / Resource Management  
**Platform**: Desktop and Mobile (Windows + WebGL)  
**Engine**: Macroquad (Rust)

Scrapyard Planet is a tower defense game where players act as scrapyard operators repairing crashed spaceships on a hostile planet. Rogue nanomachines attack anything generating power, forcing players to balance ship repairs with defense. Destroyed nanomachines provide materials for further repairs, creating a tense risk-reward cycle. The goal is to restore the ship's power core, activate the engines, and escape before being overrun.

## Core Gameplay Loop

1. **Arrival**: Start with a damaged ship and low enemy presence.
2. **Repair**: Spend materials to repair or upgrade ship modules.
3. **Defense**: Activated modules generate power, attracting nanomachine enemies.
4. **Combat**: Ship weapons auto-fire at enemies; enemies drop materials on death.
5. **Escalation**: Higher power increases enemy spawn rate and difficulty.
6. **Escape**: Power the engine to trigger a boss and escape timer.

Players must strategically sequence repairs to build defenses before attracting overwhelming enemy waves.

## Ship and Modules

Ships have fixed layouts with 10–16 module slots, starting in states: Destroyed, Damaged, or Offline.

### Module Categories

- **Weapons**: Kill enemies, generate materials
  - Pulse Turret: Medium damage, low power cost
  - Beam Emitter: Pierces enemies, high power cost
  - Missile Rack: AoE damage, consumes materials

- **Defense**: Slow or mitigate enemies
  - Shield Generator: Absorbs damage
  - EMP Field: Slows enemies
  - Decoy Beacon: Redirects aggro

- **Utility**: Economy and control
  - Recycler: +20% materials from kills
  - Scanner: Reveals hidden modules
  - Overclock Node: Temporary weapon boost (risky)

- **Engine**: Win condition
  - Requires repair and sustained power
  - Triggers boss spawn and escape countdown

Modules have upgrade paths (damage, fire rate, etc.) and consume power when active.

## Enemies

Nanomachines spawn continuously toward the ship core, adapting to defenses.

- **Nanodrone**: Fast, low HP, basic material drop, swarm behavior
- **Nanoguard**: Medium HP, targets defenses first
- **Leech Unit**: Attaches to modules, drains power
- **Siege Construct**: Slow, high damage, attacks hull directly

### Boss System
- Spawns when engine activates
- Behaviors: Overrides targeting, disables modules, splits into units
- Must be defeated to escape

## Power System

Power is the core difficulty dial. Total active power determines enemy threat:

- 0–5 Power: Light drones only
- 6–10 Power: Medium enemies
- 11–15 Power: Elites and swarms
- 16+ Power: Boss modifiers

Power spikes from overclocking or events draw immediate surges. Visible as a danger meter with visual feedback.

## Economy

### Materials
- Dropped by enemies
- Used for repairs and upgrades
- Limited storage cap (excess lost unless recycler active)

### Currency
- Earned from mission rewards
- Used for meta-upgrades between missions

### Reward Calculation
Final payout = Base ship value + active modules bonus + upgrade multipliers - core damage penalty

## Mission Structure

Missions are single-run with escalating phases:

- **Arrival**: Scouting and early repairs
- **Mid-Repair**: Sustained pressure, build defenses
- **Power Commitment**: Engine activation, boss warning
- **Escape Phase**: Maximum threat, countdown to launch

No hard waves; continuous spawning based on power.

## Win/Lose Conditions

### Win
- Power engines
- Defeat boss
- Survive escape timer

### Lose
- Core HP reaches 0
- Engine destroyed after activation
- Materials depleted with no defenses

Campaign continues after failure with reduced rewards and worse starting ships.

## Target Audience and Platform

**Primary Audience**: Males 18-35, fans of sci-fi and indie games like FTL and Space Run. Appeals to strategic gameplay and ship customization.

**Secondary Audience**: 13-18 males, mobile players seeking quick sessions.

**Tertiary Audience**: Broad mobile market, 50/50 gender split, casual TD fans.

**Platform Strategy**: Desktop-first for full game sales, mobile port with free-to-play elements (in-app purchases for currency boosts). Avoids reputation damage from mobile-first approach.

## Similar Games

- **Space Run**: Ship-based TD with directional attacks; differs in fixed ship layouts.
- **FTL**: Strategic resource management under pressure; inspires tense decision-making.