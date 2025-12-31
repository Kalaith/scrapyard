# Scrapyard Planet: Game Balance Document

## 1. Economy & Power

### Resources
- **Scrap**: Primary currency for repairs.
  - Per Pile: 15-40 scrap
  - Per Resource Crate: Varied
  - Enemy Drops:
    - Nanodrone: 3
    - Leech: 5
    - Nanoguard: 10
    - Siege Construct: 25
    - Boss: 100
- **Credits**: Score/Meta-currency.
  - Earned via escaping (Base 500 + Bonus per Upgrade Level).
  - Roughly 50% of scrap value also given as credits on kill.

### Power & Upkeep
- **Reactor Output**: 1 Power per repaired reactor point (Max 16).
- **Module Costs**:
  - Weapons: 1 Power
  - Shields: 1 Power (per point, up to 4?) - *Note: Code checks repaired count * 1. But room usually has 4 points.*
  - Systems/Utility: 1 Power
  - Engines: 1 Power
  - Cockpit/Medbay: 1 Power

### Repairs
- **Cost**: 10 Scrap per repair point.
- **Auto-Repair**: 2.0 HP/sec per Robotics Level (Module Health, not interior).

---

## 2. Player Ship & Modules

### Ship Stats
- **Base Integrity**: 1000 HP
- **Hull Upgrades**: +200 HP per level.

### Modules (Base Stats)
All modules max level: 5.
Upgrade Scaling: +50% HP, +50% Upgrade Cost per level.

| Module | Cost (Credits?) | HP | Power Use | Key Stats |
| :--- | :--- | :--- | :--- | :--- |
| **Reactor** | 0 | 1000 | -10 (Gen) | Generates power |
| **Weapon** | 20 | 100 | 2 | Dmg: 10, Rng: 300, Rate: 7.0s |
| **Shield** | 30 | 150 | 3 | Str: 50, Rech: 5.0 |
| **Utility** | 25 | 80 | 1 | +20% Scrap Bonus |
| **Engine** | 500 | 500 | 50 | Charge Time: 60s |

*Note: Power Consumption in `modules.json` (e.g. 50 for engine) differs from `constants.rs` dynamic upkeep (1 per point). The game currently uses the interior point based system (1 per point).*

---

## 3. Enemies & Combat

### Enemy Types

| Enemy | HP | Speed | Dmg | Special |
| :--- | :--- | :--- | :--- | :--- |
| **Nanodrone** | 10 | 100 | 5 | Fast swarmer |
| **Leech** | 30 | 60 | 2 | Drains 1 Power/tick |
| **Nanoguard** | 50 | 40 | 15 | Prioritizes Weapons/Shields |
| **Siege** | 200 | 15 | 30 | Slow tank |
| **Core Eater** | 1000 | 20 | 50 | Boss. Spawns 3 drones on death |

### Wave Progression

1.  **Grace Period**: 0 - 3 Power. No spawns.
2.  **Tier 0**: 4+ Power.
    - Drones spawn every 15s.
3.  **Tier 1**: 16+ Power.
    - Drones every 8s.
4.  **Tier 2**: 24+ Power.
    - Drones every 4s.
    - Guards spawn every 20s.
5.  **Tier 3**: 40+ Power.
    - Drones every 2s.
    - Guards every 5s.

### Combat Mechanics
- **Weapon Range**: 300 base. Scales 50%-100% based on repair status.
- **Shields**: Block up to 80% damage (scaled by number of active shield points).
- **Engine Charge**: 60s base time. Reduced by engine repair %.

---

## 4. Starter Ship Configuration

Based on `starter_ship.json`:

### Repairable Systems
| System | Type | Repair Points (Levels) | Power Cost (Max) | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **Main Reactor** | Core | 16 | **+16 (Gen)** | The heart of the ship. |
| **Hyperdrive** | Engine | 8 | 8 | Victory condition. |
| **Left Pulse Turret** | Weapon | 4 | 4 | Primary offense. |
| **Right Pulse Turret** | Weapon | 4 | 4 | Secondary offense. |
| **Shield Generator** | Defense | 4 | 4 | Critical defense. |
| **Recycler** | Utility | 4 | 4 | Boosts scrap income. |
| **Cockpit** | Cockpit| 3 | 3 | Navigation/View? |
| **Medbay** | Medbay | 3 | 3 | Healing? |

### Power Balance
- **Max Possible Generation**: **16 Power** (Fully repaired Reactor)
- **Max Possible Demand**: **30 Power** (All systems fully active)
- **Deficit**: **-14 Power**

**Strategic Note**: The player CANNOT run all systems at once. They must manage power routing, typically choosing between:
- Full combat (Weapons + Shields)
- Escape (Engines + minimal defense)
- Economy (Utility active early on)
