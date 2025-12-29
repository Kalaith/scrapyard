# Implementation Plan: [CONSTRUCT]

**Role**: Rendering, UI, Visual Feedback, Audio, "Juice".
**Goal**: Make the game look and feel responsive and premium.

## Phase 1: The Hull (Foundation)
- [x] **Asset Management**
    - Create `AssetManager` struct (Texture/Font caching).
    - Load minimal placeholder assets (using geometric primitives until art arrives).
    - Implement `Sprite` struct for robust scaling/rotation.
- [x] **Ship Rendering (Static)**
    - `draw_ship_hull()`: Render the background ship texture/shape.
    - `draw_grid()`: Render module slots (10-16 slots).
    - `draw_module_base()`: Render the underlying slot state (Empty/Module Present).

## Phase 2: The Spark (UI & Interactivity)
**Reference**: `ui_spec.md`
- [ ] **Top HUD (Global Status)**
    - `draw_top_bar()`: Height ~10% screen.
    - **Materials**: Icon + "Current/Cap" text.
        - *Logic*: Turn Yellow at 90% cap, Red flash at 100%.
    - **Currency**: Static display.
    - **Wave/Threat**: "WAVE X" text (Standard) or "BOSS INCOMING" (Red/Shaking).
    - **Escape Timer**: Hidden until Engines ON. Format "MM:SS".
        - *Logic*: Pulse Red when < 20s.
- [ ] **Left Sidebar (Ship Status)**
    - Side panel ~20% width.
    - **Power Meter**:
        - Segmented Horizontal Bar.
        - Colors: Green (Safe) -> Yellow (Threat) -> Red (Danger).
        - Animation: Shake bars at High Power.
    - **Core Health**:
        - Red HP Bar "Current/Max".
        - Feedback: Vignette red flash on damage.
- [ ] **Ship Interaction Layer (Module UI)**
    - **Module Panels** (World Space, attached to grid):
        - `draw_module_card` with: Name, Level, Status Icon.
        - **Buttons**:
            - [REPAIR]: Gray if poor, Green if affordable. Costs displayed.
            - [UPGRADE]: Visible only when active.
            - [ONLINE/OFFLINE]: Toggle button.
    - **Engine Module Special**:
        - Larger panel.
        - [ACTIVATE]: Triggers "Are you sure?" -> Starts Boss/Timer.
    - **States**:
        - *Destroyed*: Darkened, "Sparks" particle effect.
        - *Damaged*: Flickering overlay.
        - *Active*: Glow effect.
- [ ] **Bottom Bar (Controls)**
    - [UPGRADES] (Left): Open Upgrade Shop overlay.
    - [PAUSE] (Center): Freeze logic tier.
    - [EXIT] (Right): Confirm Dialog.
- [ ] **Context Tooltips**
    - Hover logic to show: Power Cost, Threat Increase, Reward Contribution.

## Phase 3: The Swarm (Dynamic Visuals)
- [ ] **Entity Rendering**
    - **Enemies**:
        - `Nanodrone`: Small, swarm movements.
        - `Nanoguard`: Larger, slower.
        - `Boss`: Large HP Bar at Top Center.
    - **Projectiles**:
        - Lasers: `draw_line` with bloom bloom.
        - Missiles: Sprite + Smoke trail particles.
- [ ] **Feedback Systems** (Juice)
    - **Screen Shake**: `Camera` offset based on trauma value (explosions/heavy hits).
    - **Damage Numbers**: Floating text scaling up and fading out.
    - **Sounds**: (Placeholder logic)
        - Audio triggers for: `Repair_Success`, `Weapon_Fire`, `Enemy_Explode`, `Alarm_Loop`.

## Phase 4: Polish
- [ ] **Narrative Overlay**
    - `TutorialManager` visual hook:
        - "Comm Link" Dialogue Box (Portrait + Text).
        - Typewriter text effect.
- [ ] **Victory/Defeat Screens**
    - Victory: "Engines Online" animation -> Fade to white.
    - Defeat: "Core Critical" -> Slow fade to black/static.
