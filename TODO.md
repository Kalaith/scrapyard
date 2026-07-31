# TODO — Scrapyard Planet

## Architecture & testing

- Separate ship interior simulation from HUD/world rendering so repair and routing behaviour can be tested without draw calls.
- Integration tests for `PowerRouted` events, life-support timer updates, threat-signature changes, and rapid system toggles.
- Scenario fixtures for low-hull escape, high-risk salvage, and combat-heavy runs.

## Dead code & data

- Unused BFS path-to-core helper in `ship/ship.rs`.
- `spawn_rules` in `assets/enemies.json` is never read — wave pacing lives in `simulation/constants.rs`.
- Legacy `TutorialStep` enum kept "for backwards compatibility during transition"; the transition is done.
- Stray scratch comments in `ui/world_renderer.rs` around the exterior-grid module centring math.
- The vestigial exterior tower-defence layer should be deleted rather than revived — the interior loop carries the game.

## Accessibility & UX

- Key remapping (the last outstanding accessibility item).
- Exterior view watermark: "AUTOMATED DEFENSE — TAB TO RETURN", so players stop hunting for input that isn't there.
