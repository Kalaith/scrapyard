# Scrapyard Planet

Scrapyard Planet is a survival tower-defense game about repairing a crashed spaceship on a hostile planet while rogue nanomachines attack anything that generates power.

Every repair makes escape more possible, but powered modules attract more danger. The central tension is deciding what to fix first and how much threat your defenses can handle.

## Gameplay

- Move through the ship interior to repair modules and gather scrap.
- Toggle between interior management and exterior defense views.
- Activate weapons, engines, and support systems.
- Fight off nanomachine waves drawn by power output.
- Use dropped materials to repair and upgrade more systems.
- Prepare for the final escape sequence.

## Goal

Restore the ship's power core, activate the engines, survive the final pressure, and escape before being overrun.

## Controls

- WASD: move character in interior view.
- Tab: toggle interior and exterior views.
- E: interact or repair modules.
- Hold E: gather scrap from piles.
- P: pause game.
- Esc: return to menu.

## Current Scope

Playable repair-and-defense loop with ship modules, scrap gathering, escalating waves, auto-firing weapons, and escape pressure.
# Practical Future Improvements

- Add unit tests for payout breakdown math covering base, repaired, powered, hull, scrap, combat, risk, penalties, and total.
- Add integration tests for PowerRouted events, life-support timer updates, threat signature changes, and rapid system toggles.
- Separate ship interior simulation from HUD/world rendering so repair and routing behavior can be tested without draw calls.
- Create scenario fixtures for low-hull escape, high-risk salvage, and combat-heavy runs.

