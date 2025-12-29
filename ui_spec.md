# Scrapyard Planet – UI Specification

## Screen Layout Overview

### Perspective
- Single-screen gameplay
- Ship centered horizontally, slightly lower than center vertically
- Enemies approach primarily from right side (can expand later)

### UI Philosophy
- Everything critical is always visible
- Danger escalates visually as power increases
- No modal popups during combat

## Top HUD (Global Status Bar)

### Position
- Top of screen, full width, height ~10% of screen

### Elements (Left → Right)

1. **Materials Counter**
   - Icon: Scrap / cog
   - Format: 185 / 500
   - Color:
     - White normal
     - Yellow when near cap
     - Red flashing when full

2. **Currency Counter**
   - Icon: Credit coin
   - Format: 1,200
   - Static during mission

3. **Escape Timer**
   - Label: ESCAPE TIMER
   - Format: MM:SS
   - Hidden until engines activated
   - Color:
     - White > 60s
     - Yellow < 60s
     - Red pulsing < 20s

4. **Wave / Threat Indicator**
   - Example: WAVE 7
   - Replaced by BOSS INCOMING! when applicable
   - Boss warning uses:
     - Red text
     - Shake animation
     - Alarm sound

## Left Sidebar (Ship Status)

### Position
- Left side, vertical stack, width ~20% of screen

### Power Meter
- **Label**: POWER
- **Bar**: Horizontal segmented bar
  - Colors:
    - Green: safe
    - Yellow: increased threat
    - Red: extreme danger
- **Text**: 75 / 100
- **Behavior**:
  - Bar shakes slightly at high power
  - Background glows red when spawning elites

### Core Health
- **Label**: CORE HEALTH
- **Bar**: Red HP bar
- **Format**: 280 / 400
- **Behavior**:
  - Screen vignette flashes red when damaged
  - Warning sound below 25%

## Ship Interaction Layer (Center)

### Ship Modules
- Each module is a clickable UI panel attached to ship geometry

#### Module Panel Elements
- Icon (weapon / defense / utility)
- Name (e.g. LASER TURRET)
- Level (LVL 2)
- State:
  - Destroyed
  - Offline
  - Active
- Buttons:
  - REPAIR (gray → green when affordable)
  - UPGRADE (appears when repaired)
  - ONLINE indicator when active

#### Visual States
- **Destroyed**: sparks, smoke, darkened
- **Damaged**: flickering lights
- **Active**: glow + animation
- **Disabled (boss)**: red static overlay

### Engine Module (Special)
- Larger panel
- Shows:
  - ENGINE
  - Level
  - Status: OFFLINE / ONLINE
- **Activation**:
  - Clicking ONLINE:
    - Triggers boss
    - Starts escape timer
    - Locks further engine upgrades

### Enemy Indicators
- Enemies do not have UI bars by default
- Boss has:
  - Large HP bar at top center
  - Name + icon
- Elite enemies glow slightly

## Bottom Bar (Global Controls)

### Position
- Bottom of screen, height ~10%

### Buttons
- **Upgrades**
  - Bottom-left
  - Opens between-mission upgrades (disabled during combat)
- **Pause**
  - Bottom-center
  - Pauses game
  - Shows overlay menu
- **Exit**
  - Bottom-right
  - Confirm dialog

## Contextual UI Elements

### Tooltips
- Appear on hover / long-press
- Show:
  - Power cost
  - Material cost
  - Reward contribution
  - Enemy threat increase

### Victory Condition Panel
- Bottom-right floating panel
- Text:
  ```
  VICTORY CONDITIONS:
  POWER UP THE ENGINES & SURVIVE
  ```
- Fades out once engine is activated.

## Feedback & Animation Rules
- **Every repair**: Sound + particle burst
- **Power increase**: UI hum intensifies
- **Boss spawn**: Screen shake + Music switch
- **Near defeat**: UI flicker + Alarm pulse

## Macroquad Implementation Notes
- All UI uses screen-space coordinates
- Use rectangles + text first, sprites later
- Module panels stored as:
  ```rust
  struct ModuleUI {
      bounds: Rect,
      module_id: usize,
  }
  ```
- Hit detection via `Rect::contains(mouse_position())`

## What This UI Achieves
- Immediate readability
- Constant pressure via power meter
- Clear risk-reward feedback
- Mobile-compatible layout with scaling