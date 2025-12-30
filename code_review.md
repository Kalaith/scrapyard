# Scrapyard Planet Code Review - 10 Potential Issues

Based on a comprehensive code review of the Scrapyard Planet codebase, here are 10 potential issues identified, with a focus on unimplemented or conflicting features:

## 1. Missing Core Configuration Files
The AGENTS.md documentation specifies several JSON config files that should exist but are missing:
- `config.json` (referenced in settings.rs for game settings)
- `modules.json` (for module definitions beyond hardcoded values)
- `enemies.json` (for enemy stats and behaviors)
- `missions.json` (for campaign structure)

Currently, settings fail to load silently, and module/enemy data is hardcoded in Rust instead of being data-driven.

## 2. Conflicting Engine Activation Systems
There are two mutually exclusive engine activation mechanisms:
- **Interior-based**: Engine activates when `ENGINE_MIN_REPAIR_PERCENT` (25%) of engine room is repaired
- **Grid-based**: Player can click "Activate Engine" on active engine modules

This creates gameplay confusion and potential exploits where players could activate engines without proper repairs.

## 3. No Mission/Campaign System
The game design document specifies a mission structure with phases (Arrival → Mid-Repair → Power Commitment → Escape), but no mission system exists. The game runs in a single endless mode without:
- Mission objectives
- Phase transitions
- Campaign progression
- Win/loss rewards scaling

## 4. Incomplete Enemy Variety
The GDD specifies 4 enemy types, but only 3 are implemented:
- **Missing**: Leech Unit (attaches to modules, drains power)
- **Missing**: Siege Construct (slow, high damage, attacks hull directly)
- **Incomplete**: Boss has no special AI behaviors (should override targeting, disable modules, split into units)

## 5. Inconsistent Module Upgrade System
Two upgrade systems exist but don't integrate:
- **Grid upgrades**: Level up modules with scrap costs, increasing max health
- **Interior repairs**: Only activate modules when fully repaired, no upgrade mechanics

Players can't upgrade modules through the interior view, breaking the intended repair/upgrade workflow.

## 6. Redundant Power Calculation Logic
The power system has conflicting calculations:
- `update_power()` calculates from interior room repairs
- `update_resources()` redundantly sets `resources.power = self.used_power`
- Some code still references the old grid-based power system

This creates maintenance issues and potential calculation errors.

## 7. Settings Not Persisted
Settings are loaded from `config.json` but never saved back:
- Volume changes, fullscreen toggles, etc. are lost on restart
- The `Settings::save()` method exists but is never called
- No UI exists to modify settings in-game

## 8. Tutorial Advancement Logic Flawed
Tutorial steps advance on keypress without verifying completion:
- Players can skip "repair reactor" without actually repairing it
- No validation that tutorial objectives are met before progression
- Could lead to players getting stuck in later steps

## 9. WASM Compatibility Issues
Asset loading doesn't follow WASM requirements:
- Code uses `std::fs::read_to_string()` without `#[cfg(target_arch = "wasm32")]` checks
- Should use `include_str!()` for JSON files in WASM builds
- Random number generation uses both `macroquad::rand` and `rand` crate inconsistently

## 10. Boss Fight Is Just a Big Enemy
The boss enemy lacks any special mechanics:
- No overriding of weapon targeting
- No module disabling abilities  
- No splitting into smaller units
- Just spawns as a high-HP enemy with no unique behavior

These issues represent gaps between the implemented code and the design document, potentially breaking core gameplay loops and player experience.