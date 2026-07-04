<#
.SYNOPSIS
    Headless screenshot harness for Scrapyard Planet.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (SCRAPYARD_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Scenes seed GameState via GameState::begin_capture_scene:
    "menu" (main menu), "gameplay" (a fresh salvage run), "pause" (gameplay
    paused).

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Scenes gameplay -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("menu", "gameplay", "pause"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
