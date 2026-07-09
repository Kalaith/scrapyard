# Scrapyard Planet — Design Review & Improvement Report

*Senior design/production review — 2026-07-09. Based on full source analysis (src/), data files (assets/*.json), README, and commit history.*

---

# 1. Project Overview

## Project Name

**Scrapyard Planet** (`scrapyard`)

## Genre

Roguelite survival tower-defense / ship-management hybrid. Single-run "repair, defend, extract" loop with persistent meta-progression between runs.

## Core Concept

The player is a lone engineer walking the interior of a crashed spaceship (FTL-style room view, WASD + E), repairing subsystems with scavenged scrap and routing limited reactor power to them. Rogue nanomachines attack anything that emits power. The twist that defines the game:

> **Every system you repair and power makes you stronger AND louder.** A derived "threat signature" (routed power + recycler bonus + engine charging + hoarded wealth) directly sets enemy spawn tiers and pacing. There is no wave counter — *the player is the difficulty dial.*

Escaping requires fully repairing the engine, routing power to it, and surviving a 60-second charge that spikes threat, summons a boss, and races an engine-stress meter that can cascade into catastrophic failure. The escape payout formula explicitly rewards greed (risk bonus scales with signature; bonuses for repaired/powered systems, hull, scrap, and kills), so the core fantasy is: **how loud and how rich do I dare to get before I commit to launch?**

- **Player fantasy:** desperate engineer triaging a dying ship under siege; greed vs. survival.
- **What makes it different:** difficulty is self-inflicted and legible (the SIGNAL stat on the HUD), and escape is a *commitment* rather than a finish line.
- **Target player:** fans of FTL, Dome Keeper, and short-session roguelites; comfortable with systems/management play, 15–30 minute runs.

## Current State

**Early development — the strategy layer is feature-complete; the action layer is a stub.**

Evidence:
- The full loop works: tutorial → repair/route → signature-driven waves → engine charge/stress/cascade → victory/defeat payout screens → meta-upgrade shop → next run. Persistence, save/load (native), settings, and a data-driven tutorial all exist.
- But moment-to-moment combat is passive and shallow: turrets auto-fire, the player cannot be attacked, enemies drain a single global hull pool (they never damage or destroy individual modules), the boss's ability/split constants are defined but never used, the particle system is never invoked, and the exterior "defense view" accepts no input at all.
- Significant vestigial code (exterior grid repair/upgrade actions with no input path, unused BFS pathfinding, `modules.json` fields silently ignored, duplicated spawn thresholds in `enemies.json` that the code doesn't read) indicates the design pivoted mid-development from an exterior tower-defense to the interior engineer game — and the old layer was never removed or finished.

---

# 2. Core Gameplay Analysis

## Main Gameplay Loop

> Scavenge scrap → Repair repair-points (E) → Route limited power (1–8) → Signature rises → Waves escalate → Turrets auto-defend / hull drains → Repair engine → Commit to 60s charge (boss + stress gamble) → Escape → Payout → Meta-upgrades → Repeat

**Is the loop clear?** Yes — unusually so for this stage. The HUD surfaces SIGNAL, POWER, and even a live payout preview ("VALUE {n} CR") in the routing panel, and the 5-step tutorial explicitly teaches the thesis ("Escape is a commitment. Stay longer for a bigger payout, or launch before the signal overwhelms you"). This is strong design communication.

**Is it satisfying?** Partially. The *decisions* are satisfying: with ~16 power max and every routed system costing 1 power + signature, choosing weapons (income) vs. shields (survival) vs. recycler/medbay (economy/sustain) vs. engine (escape) is a real dilemma, and the auto-shed priority when power drops adds pressure. The *action* is not: the player walks, holds E, presses number keys — and then watches. Combat resolves itself off-screen (or on a spectator screen). Kills, hits, and deaths have no particles, no module loss, no interior consequence. The loop's tension is intellectual but never visceral.

**Meaningful decisions?** Yes, three good ones: what to power (loadout under a budget), when to launch (greed timer), and the meta `targeting_tier` upgrade (opt-in +50%/level spawn speed for +100/level payout — a genuinely smart risk-for-reward purchase). 

**Variety?** No. One ship layout (`starter_ship.json`), one fixed set of systems, deterministic pre-repair order from meta-upgrades, scrap piles as the only randomness. Run 5 plays like run 2.

**Long-term motivation?** Weak. The meta tree is 8 flat, parallel stat tracks (cheaper repairs, more hull, more scrap) with no unlocks, no new content, no branches. Nothing new *happens* on later runs except `targeting_tier`.

A note on failed runs: losses bank **zero** credits (the 20% "recovery value" is computed and shown on the game-over screen but never written to the profile). Zero-banking is the *right* call under the game's intended contract — *a modest escape is nearly promised; every credit of extractable value you reach for raises the danger; every death is therefore a chosen death* — but that contract only holds if the modest escape actually is nearly promised. Today it isn't (see the stress-trap finding under Escape Sequence below), so failure is currently an ignorance tax, not a greed tax. Separately, showing a recovery number that is never banked is misleading UI, and mid-run kill credits being silently overwritten at run start is dead code either way.

---

# 3. Existing Systems Review

## Threat Signature / Wave Director

### Purpose
Replaces a wave counter with a player-controlled aggro economy — the game's identity.

### Current Implementation
`signature = used_power + recycler(+2) + engine_charging(+5) + life_support_offline(+2) + wealth_term((repaired×60 + scrap/2 + kills×8)/250)`. Tiers at 2/5/9/13 gate spawn types and shrink intervals (drones 14s→3s; guards, leeches, sieges each have their own conditions, e.g. leeches only if a support system is "hot"). A slow global `nanite_alert` (+0.1/s) escalates infinitely long runs. `targeting_tier` divides all intervals by up to 2.5×.

### Strengths
- Coherent, legible, and *taught* (tutorial + HUD SIGNAL readout).
- Conditional spawn rules (leeches only when support systems are hot, sieges when the engine is powered) make enemies feel like a reaction to the player, not a script.
- The payout risk bonus (`signature×12`, cap 240) closes the loop: loudness is literally money.

### Weaknesses
- The band is narrow: with all costs flat at 1 power, signature ≈ number of routed systems, so the whole strategic space is ~6 integers. The wealth term is heavily divided (/250) and barely registers.
- The optimal strategy may be degenerate: stay under grace (signature < 2), quietly repair everything with the 8–12 scrap piles, then flip everything on at once for the payout snapshot. Nothing punishes turtling except passive `nanite_alert` (+0.1/s — 6 points in a 60-second wait, very slow).
- `enemies.json` `spawn_rules` (10/16/24/40) are dead data; the real thresholds (2/5/9/13) live in `constants.rs` — a tuning trap.

### Improvement Ideas
- **Rebase SIGNAL on extractable value.** The game already computes the player's unrealized paycheck every frame (the "VALUE {n} CR" payout preview). Deriving threat from that number — *your danger is literally your unbanked wealth* — is the purest implementation of the design thesis, and the HUD already shows both figures side by side. Two caveats: build the threat term from the *greed* lines only (scrap held, repairs beyond the survival minimum, powered systems, risk bonus) and exclude the *survival* lines — the payout includes hull% × 300, and wealth-driven threat naively applied would make a healthy ship draw more enemies than a wreck, a perverse incentive to fly broken. And uncap the risk bonus (currently capped at 240) — if value is difficulty, reward must keep scaling with what you survived, or greed rationally stops at the cap while the danger keeps climbing. **Impact: Game-changing. Cost: Medium.**
- Make system power costs asymmetric again (engine 2–3, weapons 2, support 1) so signature has texture and "loud loadouts" exist. **Impact: High. Cost: Small.**
- Add a soft anti-turtle: scrap piles decay, or nanite_alert accelerates while total routed power is 0 ("they're hunting the silence"). **Impact: Medium. Cost: Small.**
- Move spawn thresholds into `enemies.json` for real (the repo's own data-driven rule) and delete the dead copy. **Impact: Low (dev velocity). Cost: Small.**

## Power Routing & Ship Systems

### Purpose
The loadout layer — spend a 0–16 power budget across weapons/shields/recycler/life-support/medbay/engine.

### Current Implementation
Core repair points each grant +1 capacity (16 max). All systems cost a flat 1. Over-capacity auto-sheds in priority order (utility first, engine last). Life support offline chews hull 4/s after 8s grace; medbay heals 5/s; recycler +35% kill scrap +2 signature.

### Strengths
- Auto-shed priority is elegant — a leech draining your reactor causes a believable rolling brownout.
- Life support as "powered cockpit" with a repair discount gives a normally-boring system a real cost/benefit.
- The FIX/NO PWR/ON/OFF routing panel with live payout preview is genuinely good UI.

### Weaknesses
- Flat 1-power costs make routing almost trivially affordable once the core is half-repaired; the interesting scarcity phase is only the first few minutes.
- `modules.json` promises depth it doesn't deliver (engine consumption 50, shield_strength, recharge_rate — all ignored, hardcoded elsewhere). Anyone tuning the JSON changes nothing.
- In-run module *upgrades* (levels 1–5, ×1.5 HP) exist in code with no input path — an orphaned system.

### Improvement Ideas
- Restore asymmetric costs from `modules.json` and actually parse those fields — one change fixes scarcity, data-driven compliance, and signature texture simultaneously. **Impact: High. Cost: Small.**
- Either wire in-run module upgrades to the interior loop (E on a fully-repaired module → spend scrap → +damage/+shield) or delete the system. Wiring it adds the missing mid-run scrap sink — currently, once repairs are done, scrap has nothing to buy but the payout bonus. **Impact: High. Cost: Medium.**

## Combat & Enemies

### Purpose
The pressure that makes routing/greed decisions matter.

### Current Implementation
5 enemy types (drone 10 HP rusher, guard 50 HP anti-turret, leech 30 HP power-drainer, siege 200 HP tank, boss 1000 HP). Turrets auto-fire from powered weapon rooms, scaled by repair % (damage, fire rate, range). Shields reduce damage up to 80%. All enemy damage drains one global `ship_integrity` pool.

### Strengths
- The Leech is the standout: it attaches and forcibly un-routes a powered system every 2s, creating rolling brownouts the player must respond to. This is the only enemy whose fantasy is fully mechanically realized.
- Turret effectiveness scaling with repair % elegantly rewards partial investment.

### Weaknesses
- **Enemies cannot damage modules.** Guards "target weapons" and sieges "attack hull," but mechanically every enemy just drains the same global pool — `ModuleDestroyed`/`CoreDamaged` events are defined and handled but never emitted, module HP is never applied. Defense has no spatial texture: it never matters *where* enemies attack, so it never matters where your turrets or shields are.
- **The boss is a 1000-HP sponge.** `ability_cooldown` and `split_count` are loaded and ticked but never used. The climactic escape sequence — the game's best moment on paper — plays out against the game's most boring enemy.
- The player is invulnerable and combat is fully automated; there is no failure state a player can *feel* coming except a number going down.
- No particles, no hit feedback, no death effects (the particle system exists and is never fed).

### Improvement Ideas
- Implement per-module damage: enemies chew the module their AI already targets; a destroyed module goes offline and loses repair points until re-repaired. This single change makes turret placement, guard targeting, and repair triage all real, and creates the interior gameplay the game is missing (run to the weapon room *because it's dying*). **Impact: Game-changing. Cost: Medium** (the targeting, events, module HP, and repair loop all already exist — they just aren't connected).
- Implement the boss's designed abilities (split into 3 on death, 8s ability — e.g., an EMP that un-routes everything, doubling down on the power fantasy). **Impact: High. Cost: Small–Medium.**
- Feed the particle system on kill/hit/repair; add the already-toggleable screen shake properly. **Impact: Medium (feel). Cost: Small.**

## Escape Sequence / Engine Stress

### Purpose
Converts "win condition" into the game's signature gamble.

### Current Implementation
Engine must be 100% repaired + powered. 60s charge scaled by repair % (moot, since 100% is required — `ENGINE_MIN_REPAIR_PERCENT = 1.0` makes the scaling dead code). Charging adds +5 signature and spawns the boss. Each engine repair point adds +6 stress; stress grows with nanite alert while charging; ≥46 triggers cascade (50 hull/s, boss, escape timer *reverses*, forced shutdown with hysteresis until stress cools below 31).

### Strengths
- Mechanically rich and thematically perfect: the engine literally fights you if you rushed it, and repairing it *when* matters (stress decays at 2/s idle, so repairing the engine early and letting stress cool is a real strategy the game never tells you about).
- Hysteresis and cascade reversal are sophisticated anti-cheese design.

### Weaknesses
- **The floor escape is mathematically booby-trapped.** The engine's 8 repair points at +6 stress each total **48 stress — above the 46 cascade threshold** (`constants.rs:137-141`). Stress decays at only 2/s idle, and hysteresis blocks charging until stress cools below 31. So even a modest, quiet player who repairs the engine at a normal pace either trips a cascade (50 hull/s + boss spawn) or ends up locked out of launch waiting through a cool-down the game never explains — and then the mandatory charge boss arrives against turrets they were told not to power. This makes the intended contract ("a modest escape is nearly promised; deaths come from greed") impossible to honor: new-player failure is currently an ignorance tax, not a greed tax.
- Almost none of this is communicated. Stress gain per repair, the cool-down-before-charging strategy, and cascade thresholds are invisible until they kill you. STRAINED/UNSTABLE/CASCADE labels exist on the HUD but the *causality* (you repaired 8 points back-to-back) doesn't.
- `engine_stress` and `nanite_alert` are **not saved** — save mid-charge, reload, and the gamble state is wiped. Exploitable and inconsistent.
- 60s is a flat timer against a boring boss; the final minute risks being "stand in the medbay and wait."

### Improvement Ideas
- **Design the guaranteed floor.** Retune stress so a first-timer repairing the engine at a natural pace cannot cascade by accident (e.g., +4/point, or a higher critical threshold, or faster idle decay), make the charge sequence survivable with near-zero systems powered, and have the tutorial state the contract outright: "You can always leave early — poor." Zero-banking on death is only fair once this exists. **Impact: Game-changing. Cost: Small–Medium** (tuning + tutorial text, no new systems).
- Surface stress in the repair prompt ("REPAIR — 10 scrap, +6 STRESS") and add a one-line stress explanation to the tutorial's engine step. **Impact: High. Cost: Small.**
- Persist stress/alert in saves. **Impact: Medium (integrity). Cost: Small.**
- Script the charge minute: at T-40 and T-15, spawn surge waves and a leech squad targeting the engine — forcing the player to physically defend/re-route during the countdown instead of hiding. **Impact: High. Cost: Medium.**

## Economy & Meta-progression

### Purpose
In-run scrap drives repair triage; escape credits fund 8 permanent upgrades across runs.

### Current Implementation
Start with 50 scrap; repairs cost 10 (less discounts); full ship ≈ 46 points ≈ 460 scrap; 8–12 finite piles (15–40 each); kills pay 3–100 scrap. Payout: base 500 + repaired×60 + powered×35 + hull%×300 + scrap/2 + kills×8 + risk bonus − stress/low-hull penalties, ×credit multiplier. Meta: pre-repairs, scrap/storage/hull/cost/payout boosts, targeting_tier.

### Strengths
- The payout breakdown screen is excellent — every line item teaches a strategy ("powered_bonus ×35: I should have kept more online at launch").
- `targeting_tier` is the best upgrade in the tree: it buys difficulty.
- Repair-cost discounts stacking from life support + meta is a nice quiet synergy.

### Weaknesses
- **Losses bank nothing — which is defensible, but currently unearned.** `record_victory` is only called on win; the 20% failure payout is display-only. Under the guaranteed-floor contract this is correct design (death must cost everything for greed to mean anything — Lethal Company's wipes work the same way). But the contract's precondition — a nearly-promised modest escape — doesn't hold yet (see the stress trap under Escape Sequence), and showing a "recovery value" number that is never actually banked is misleading UI regardless.
- **No death forensics.** For zero-banking to teach "you got greedy" rather than "this game is unfair," the death screen must prove the death was chosen. It currently shows stats but never says *escape was available and you stayed*.
- Mid-run kill credits (`combat.rs` adds scrap/2 to credits) are overwritten by `banked_credits` at run start — dead/confusing code.
- **Farming risk once the floor exists:** if a safe run always succeeds, its payout must be a consolation, not a career — otherwise the optimal strategy is grinding the boring floor run to max the meta shop before ever engaging with greed. Keep the interesting meta purchases priced against greed money.
- Scrap has no sink after repairs finish; the endgame economy is "hold scrap for the /2 payout line."
- The tree is flat stats only — no new toys, so no "one more run to unlock X" pull.

### Improvement Ideas
- **Death-screen forensics:** "Escape was available at 08:42. You stayed for 380 more credits. You died holding 1,240." Plus a "credits left behind" line to weaponize regret. Track the timestamp when engine-ready conditions were first met. **Impact: High. Cost: Small.**
- Keep zero-banking, but remove the misleading "recovery value" display (or relabel it as "value lost"). **Impact: Medium. Cost: Small.**
- Delete or properly design mid-run credit income. **Impact: Low. Cost: Small.**
- Add 2–3 unlock-type meta purchases (new starting ship layout, a fourth routable system, an alternate turret type) so later runs *play* differently. **Impact: High. Cost: Large.**

## Interior / Player Layer

### Purpose
Embodies the strategy layer — you *are* the repair cursor, so travel time is a resource.

### Current Implementation
22-room FTL-style layout, 300px/s movement, wall-sliding collision, hold-E scavenging (2s, movement cancels), per-point repairs at prop locations, camera follow, tutorial highlights.

### Strengths
- Making repairs positional is what elevates this above a menu game — walking from the engine bay to a leeched recycler *is* the gameplay.
- Props tinting red when broken is good silent signposting.

### Weaknesses
- Nothing in the interior ever threatens or even reacts to the player. No boarding, no hazards, no fires, no hull breaches — so once piles are gathered, walking is pure downtime, and the Tab-to-exterior view is more interesting to watch than the space you occupy.
- One layout forever.

### Improvement Ideas
- Interior consequences of exterior damage: when hull drops below thresholds (or a module is destroyed, per the combat fix above), spawn interior hazards — a burst pipe prop that damages hull until E-fixed, sparking doorways that slow movement. Reuses the repair-point interaction verbatim. **Impact: High. Cost: Medium.**
- 2–3 additional ship layouts (the JSON format already supports it) chosen or unlocked per run. **Impact: High. Cost: Medium** (content, not code).

## UI / UX / Accessibility

### Purpose
Communicate a numbers-driven game clearly.

### Current Implementation
Strong HUD (hull/scrap/power/SIGNAL/ALERT/engine state), routing panel with live payout preview, radar, hold-Tab systems overlay, contextual prompts, full menu/pause/settings/victory/defeat/shop flow. Settings: volumes, fullscreen, screen shake, FPS.

### Strengths
For this stage, the UI is the most finished part of the game — the routing panel plus payout preview is shipping-quality design thinking.

### Weaknesses
- The screen-shake toggle is not wired up (shake always applies), and "Show FPS" shows a sound debug label, not FPS. Broken settings are worse than absent ones.
- Controls are hardcoded; no remapping, no colorblind consideration (enemy identity is largely color-coded dots on the radar).
- The exterior view accepts no input and communicates that nowhere — players will click it.

### Improvement Ideas
- Wire the two broken settings. **Impact: Low–Medium. Cost: Small.**
- Add an idle "AUTOMATED DEFENSE — TAB TO RETURN" watermark to the exterior view until it gains interactivity. **Impact: Low. Cost: Small.**

---

# 4. Similar Games & Lessons

## FTL: Faster Than Light

- **Similar:** room-based ship, power routing under scarcity, per-system repair, crew-as-cursor.
- **Does better:** systems have *interior consequences* — fires, breaches, boarding — so the ship layout is a battlefield, not a menu. Power costs are asymmetric (engines 1–8, weapons 1–4), giving routing texture.
- **Adapt:** interior hazards tied to damage; asymmetric power costs; per-system damage.
- **Don't copy:** FTL's breadth (dozens of ships/weapons/events). Scrapyard is one ship, one siege — keep it that way for now.

## Dome Keeper

- **Similar:** alternate between resource-gathering downtime and defense; greed timing is the core skill.
- **Does better:** the gather phase is *itself* under time pressure (dig deeper = risk missing the wave), and every wave physically threatens the thing you return to. Scrapyard's gather phase has no clock ticking against it below the grace threshold.
- **Adapt:** make quiet time cost something (pile decay or accelerating alert) so gathering is a bet, not a freebie.
- **Don't copy:** manual combat aiming — Scrapyard's identity is routing, not twitch.

## They Are Billions

- **Similar:** noise/expansion attracts the horde — the direct ancestor of the threat-signature idea.
- **Does better:** losses are *local and cascading* (one breached wall infects a district), which makes spatial defense decisions terrifying. Scrapyard's global hull pool has no locality at all.
- **Adapt:** per-module damage so a breach *somewhere specific* matters; let the SIGNAL number spike visibly when the player does something loud, with an audio sting.
- **Don't copy:** the scale and the brutal no-mid-run-save difficulty philosophy — Scrapyard is a shorter, more forgiving loop.

## Lethal Company / extraction-genre loop

- **Similar:** "quota greed vs. leave alive" is exactly the escape-commitment decision; payout screens as the dopamine hit. It also banks nothing on a wipe — validation for Scrapyard's zero-banking stance.
- **Does better:** the *option to leave early* is always genuinely available and everyone knows it, so every wipe is legibly self-inflicted. That legibility, not the loot math, is what makes total loss feel fair.
- **Adapt:** the guarantee ("you can always leave, poor") and the regret framing — show "credits left behind" and "escape was available at T" on the game-over screen so every death reads as a choice.
- **Don't copy:** multiplayer. The tension here is solitary triage; co-op would dissolve the routing dilemma.

---

# 5. Feature Improvement List

## Critical Improvements

*Design contract (agreed 2026-07-09): a modest escape is nearly promised; every credit of extractable value raises the danger; dying banks nothing because every death is a chosen death. The first three rows below are what make that contract true — deliberately **instead of** banking failure payouts, which would dilute the greed dial the game is built on.*

| Priority | Feature | Description | Player Benefit | Development Cost |
|---|---|---|---|---|
| High | Per-module enemy damage | Enemies damage the module their AI already targets; destroyed modules go offline and lose repair points. Emit the already-handled `ModuleDestroyed`/`CoreDamaged` events. | Defense becomes spatial; guards/sieges finally differ; interior repair becomes reactive mid-combat gameplay instead of setup-phase busywork | Medium |
| High | Guaranteed-floor escape | Fix the stress trap (8 engine points × +6 = 48 stress > 46 cascade threshold — the modest run is currently impossible to launch cleanly): retune stress gain/threshold/decay so natural-pace engine repair can't cascade; make the charge survivable with near-zero systems; tutorial states the contract: "You can always leave early — poor" | Zero-banking on death becomes fair; new-player failure changes from ignorance tax to chosen greed | Small–Medium |
| High | Death-screen forensics | Game-over shows "Escape was available at T. You stayed for +X credits. You died holding Y — left behind." Track when engine-ready was first met | Every death reads as a choice, not unfairness; loss aversion fuels retry instead of churn | Small |
| High | SIGNAL rebased on extractable value | Derive threat from the live payout preview's greed lines (scrap held, extra repairs, powered systems, risk) — excluding survival lines like the hull bonus — and uncap the 240 risk-bonus ceiling so reward keeps pace with unbounded danger | "Every dollar of value adds difficulty" becomes literally true and visible: danger *is* the unbanked paycheck on the HUD | Medium |
| High | Boss abilities | Implement the designed-but-dead split-on-death (3 fragments) and 8s ability (suggest: EMP un-routes all systems) | The escape climax gets a real antagonist; leech-style pressure at the moment of maximum tension | Small–Medium |
| High | Combat/repair feedback (juice) | Feed the existing dead particle system (kill bursts, hit sparks, repair flashes); wire the screen-shake setting | The strategy layer's stakes become visible and felt; cheapest fun-per-hour in the backlog | Small |

## High Value Improvements

| Priority | Feature | Description | Player Benefit | Development Cost |
|---|---|---|---|---|
| High | Asymmetric power costs | Parse `modules.json` consumption for real (engine 2–3, weapons 2, support 1); delete dead JSON fields that stay unused | Routing regains scarcity; signature gets texture; data-driven tuning actually works | Small |
| High | Scripted escape-charge events | Surge waves at T-40/T-15, leeches targeting the engine during charge | The final 60s becomes the game's best minute instead of a wait | Medium |
| Medium | Interior hazards from damage | Hull/module damage spawns burst pipes / sparking doors fixed with the existing E-interaction | Downtime walking becomes triage; interior view stops being safer *and* duller than the spectator view | Medium |
| Medium | Wire in-run module upgrades | Give the orphaned upgrade system (levels 1–5) an interior input path: E on a fully repaired module | Mid/late-run scrap sink; something to do with wealth besides hold it | Medium |
| Medium | Stress transparency | Show "+6 STRESS" on engine repair prompt; add stress line to tutorial; persist stress/alert in saves | The game's deepest mechanic becomes learnable before it kills you, instead of after | Small |
| Medium | 2–3 ship layouts | Additional `ships/*.json` variants (the loader already supports it), unlocked via meta shop | Run-to-run variety; gives the meta tree its missing content pull | Medium |

## Nice To Have

| Priority | Feature | Description | Player Benefit | Development Cost |
|---|---|---|---|---|
| Low | Anti-turtle pressure | Scrap pile decay or alert acceleration at zero routed power | Closes the "silent full-repair" degenerate line | Small |
| Low | Exterior view watermark | "AUTOMATED DEFENSE — TAB TO RETURN" label | Stops players hunting for input that isn't there | Small |
| Low | Menu records | Profile tracks and displays best payout alongside best time | Score-attack framing, nearly free | Small |
| Low | Colorblind-safe enemy markers | Shape-code radar dots (triangle/square/etc.), not just color | Accessibility | Small |
| Low | Dead-code cleanup | Remove/unify: unused BFS pathfinding, exterior grid repair/upgrade UI events, `enemies.json` spawn_rules vs `constants.rs`, `update_resources()` no-op, legacy tutorial enum | Prevents tuning traps and future bug reports; repo standards compliance | Small |

## Avoid / Do Not Add

| Feature | Why avoid |
|---|---|
| Multiplayer / co-op | The routing dilemma and escape commitment are solitary tensions; co-op dissolves them and multiplies scope enormously |
| Manual player combat (guns, dodging) | Would convert the game into a mediocre twin-stick shooter and bury the routing identity; the player-as-invulnerable-engineer is a feature — *pressure* should come through the systems |
| Building/placing new turrets | The fixed ship with fixed sockets is the puzzle; free placement turns it into a generic TD and invalidates the layout design |
| Procedural ship generation | Hand-authored layouts are cheap in the existing JSON format and better tuned; proc-gen is a large cost for variety the game doesn't yet know how to use |
| More resource types | Scrap+power+credits is already the right complexity; a fourth currency adds bookkeeping, not decisions |
| Story/dialogue systems | The premise is fully carried by mechanics and the tutorial's five lines; narrative content would delay the loop fixes that actually matter |

---

# 6. Missing Gameplay Elements

## Threat to the player / interior danger

- **Why expected:** the fantasy is "hostile planet, ship under siege," yet the besieged interior is the safest place in the game — the player is literally invulnerable and nothing inside ever changes.
- **Needed?** Yes — it's the biggest gap between fantasy and mechanics, and the fix (interior hazards, module damage bleeding inward) reuses existing interactions rather than adding new systems.
- **Implementation:** per-module damage (Critical list) + hazard props (High Value list). Boarding enemies are a possible later step but not required.
- **Priority: High.**

## Run-to-run variety

- **Why expected:** roguelite framing (runs, meta shop, payouts) promises varied runs; the game delivers identical ones.
- **Needed?** Yes, but modestly — 2–3 layouts plus boss abilities plus asymmetric costs already multiplies the decision space. Full proc-gen is not needed.
- **Implementation:** ship layout unlocks; optionally a small per-run modifier ("ion storms: shields cost 2") drawn from a JSON list.
- **Priority: Medium.**

## Mid-run failure feedback loop

- **Why expected:** players expect to *see* why they're losing (which system is under attack, what's about to break).
- **Needed?** Yes — currently the only failure signal is the hull number. Module damage + particles + audio stings on signature tier-ups cover this.
- **Priority: High** (covered by Critical items).

## Difficulty options

- **Why expected:** genre-standard; and the game already has the perfect diegetic version in `targeting_tier`.
- **Needed?** Not as a settings menu. Extend the contract metaphor instead (a "safe contract" tier below baseline for struggling players).
- **Priority: Low.**

## Not missing (deliberately)

Crafting, inventory, player weapons, multiplayer, story — all correctly absent. The game should defend this scope.

---

# 7. Content & Replayability Analysis

**Current sources of replayability:** meta-upgrade grind (8 flat stat tracks), `targeting_tier` self-selected difficulty, best-time tracking, payout optimization as an implicit score-attack, and the risk-bonus formula rewarding experimentation with loud loadouts.

**Assessment:**

- **Variety:** the weakest axis. One ship, one enemy roster with mostly interchangeable behavior, no events, no modifiers. Randomness is limited to scrap pile placement/amounts.
- **Progression:** functional but flat — every purchase is "number bigger," and a losing player earns nothing at all (see Critical fix). No unlock creates a *new situation*.
- **Player choices / strategies:** the strongest axis on paper — quiet-scavenger vs. loud-farmer vs. rush-engine are all viable archetypes, and the payout breakdown lets players self-evaluate builds. But flat power costs and turtling-friendly pacing compress the space in practice.
- **Emergent gameplay:** the leech un-routing power → auto-shed brownouts → life support offline → hull bleed chain is genuinely emergent and excellent. It's the only such chain; per-module damage would create several more (guard kills turret → DPS drops → siege arrives intact → core exposed).
- **Long-term goals:** best time and lifetime credits exist but aren't celebrated; there is no run-history, no highest-payout record, no challenge framing.

**Improvements, in order of leverage:** guaranteed-floor escape + death forensics (makes every death legible and chosen, so total loss motivates rather than churns) → asymmetric costs + module damage (widens live strategy space) → SIGNAL-as-unbanked-wealth (makes greed the explicit difficulty dial) → ship layout unlocks (new situations) → payout/best-run records on the menu (score-attack framing, nearly free) → per-run modifiers (cheap variety multiplier, later).

---

# 8. Player Experience Review

## First 10 Minutes

The tutorial is well-built: five data-driven steps with room highlighting that teach movement, the reactor-capacity-vs-routing distinction, and — impressively — the actual *thesis* of the game ("Escape is a commitment"). The HUD supports it with FIX/NO PWR states and a live payout preview. A new player will understand what to do and why.

**What could be improved:** the first *felt* consequence is missing. The player repairs, routes, and then… numbers change. The first enemy kill has no particle, no impact; the first hull damage is a number ticking down. The player understands the game but hasn't yet *felt* it. Also, nothing warns that repairing the engine adds stress — the game's first betrayal is invisible.

## First Hour

This is where the current build loses people. Runs 2–4 are identical to run 1 minus the tutorial: same ship, same order, same waves. And if the player *died* in runs 1–3, they have zero credits and — worse — no way to see that the deaths were avoidable, because the floor escape is booby-trapped (stress trap) and the death screen never says "you could have left." Under the intended contract, repeated early deaths should read as "I reached too far"; today they read as "this game is unfair." The hook that should carry this hour — "this time I'll run louder for the risk bonus" — exists in the payout math but the game never dares the player to try it (no post-run "you played it safe; risk bonus 12" nudge).

**Fixes:** the guaranteed-floor escape and death forensics (Critical list) are precisely first-hour fixes; surface the risk bonus on the payout screen as an explicit challenge; make `targeting_tier` purchasable earlier (500 base cost gates the most interesting purchase behind ~1–2 wins).

## Long-Term

Currently: maxing eight stat tracks, which is done in perhaps 10–15 successful runs with nothing new seen after run 1. Long-term play needs situations, not stats — layouts, boss behavior, modifiers, and records. The good news is the identity is strong enough that a modest amount of content (2–3 layouts, 1 real boss, per-run modifiers) would support 10+ hours for the genre's audience.

---

# 9. Development Roadmap

## Phase 1: Make It Fun (the consequence patch)

**Goal:** make the existing strategy layer produce *felt* moment-to-moment gameplay, and stop punishing new players.

- Per-module enemy damage + `ModuleDestroyed` emission (the keystone change — everything else in the phase amplifies it)
- Guaranteed-floor escape: fix the 48-vs-46 stress trap, make the minimal charge survivable, tutorialize "you can always leave early — poor"
- Death-screen forensics ("escape was available at T; you died holding X")
- Feed the particle system; wire screen shake; audio sting on signature tier-up
- Boss split + EMP ability
- Stress shown on engine repair prompt; persist stress/alert in saves

**Why first:** every item here connects systems that already exist. This is weeks of work that converts a working simulation into a game, before any new content is authored.

## Phase 2: Add Depth (the decision patch)

**Goal:** widen the strategy space so runs diverge by player intent.

- SIGNAL rebased on extractable value (greed lines of the payout preview, survival lines excluded; uncap the risk bonus) — do this after the floor exists, since it sharpens the same contract
- Asymmetric power costs parsed from `modules.json` (and delete the dead JSON/dead code inventory from §5)
- Scripted escape-charge events (surges, engine-targeting leeches)
- Wire in-run module upgrades as the late-run scrap sink
- Interior hazards from hull/module damage
- Anti-turtle pressure (pile decay or silence-hunting alert)

**Why second:** depth changes need Phase 1's consequences in place to matter — module upgrades are pointless until modules can be lost; hazards need the damage events.

## Phase 3: Add Content (the variety patch)

**Goal:** make run 10 differ from run 2.

- 2–3 additional ship layouts as meta unlocks
- 1–2 additional enemy variants exploiting the new module-damage system (e.g., a burrower that spawns interior hazards directly)
- Per-run contract modifiers (JSON list: "ion storm," "rich derelict," "hostile scans")
- Menu records: best payout, best time, credits-left-behind

**Why third:** content multiplies whatever the systems are; authoring it before Phases 1–2 would multiply shallowness.

## Phase 4: Polish

**Goal:** shipping quality for the WebHatchery catalog.

- Balance pass across the tier thresholds/payout formula with the new costs (add the README's requested payout unit tests here — they guard the tuning)
- Separate simulation from rendering per the README's own improvement notes (enables the test scenarios)
- Accessibility: shape-coded radar, remappable keys, fix Show FPS
- WASM save support via the toolkit's keyed storage (currently native-only)
- Fresh `catalog_thumbnail.png` and title-screen pass

**Why last:** polish stabilizes; it shouldn't precede the systems it stabilizes.

---

# 10. Final Assessment

## Strongest Idea

The **threat-signature economy**: the player's own power routing is the difficulty dial *and* the payout multiplier, converging in an escape sequence that is a priced commitment rather than a finish line. This is a real, coherent, teachable design identity — rarer and more valuable than any amount of content. The engine-stress/cascade/hysteresis system is a second, underexposed gem.

## Biggest Risk

**The consequence gap.** The game is currently a good spreadsheet wearing a game's clothes: invulnerable player, global damage pool, inert boss, silent combat, identical runs. If development continues to add strategy-layer features without making outcomes spatial and felt, it ships as "interesting but flat" — and the booby-trapped floor escape (48 engine-repair stress against a 46 cascade threshold, with zero-banking on death) will bounce exactly the new players a catalog release attracts, because their unavoidable early deaths cost everything and the game never shows them the death was avoidable.

## Missing Ingredient

**Per-module damage.** One medium-cost change makes enemy roles real, makes defense spatial, gives the interior view a purpose during combat, creates emergent failure chains, and gives repair its mid-run second act. Nearly every other recommendation in this review compounds on it.

## Unique Selling Point

"In this tower defense, *you* are the wave counter." No other small game in this space lets the player price their own danger and then charges them for greed at the exit. Protect this; market with this.

## Recommendation

**Continue development — with targeted redesign of the combat/consequence layer, not the core.**

Reasoning: the hard part of game design — a distinctive, coherent, teachable core tension — is done and implemented, and the codebase's gaps are mostly *connections between systems that already exist* (module HP exists, targeting exists, events exist, the particle system exists, the boss's abilities are half-loaded). That is an unusually favorable ratio of remaining-work to existing-value. The vestigial exterior-TD layer should be deleted rather than revived, and the flat meta tree needs unlocks eventually. The death economy should stay harsh (zero banked on death) — but only after the guaranteed-floor escape and death forensics land, since the stress trap currently makes that harshness dishonest. Do Phase 1 before adding any new content; if per-module damage lands and playtests still feel flat, reassess before Phase 3 spend.
