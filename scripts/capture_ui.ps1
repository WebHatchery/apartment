<#
.SYNOPSIS
    Headless screenshot harness for Second Story.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (APARTMENT_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. "menu" captures the building-select main menu; the default
    "gameplay" scene seeds a fresh game on the first configured building
    template.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @(
        "menu",
        "gameplay",
        "showcase",
        "unit_showcase",
        "hallway_showcase",
        "applications_showcase",
        "ownership_showcase",
        "tenants_showcase",
        "finances_showcase",
        "city_showcase",
        "market_showcase",
        "inbox_showcase",
        "tasks_showcase",
        "pause_showcase",
        "history_showcase",
        "career_showcase",
        "tutorial_showcase",
        "event_showcase",
        "notification_showcase"
    ),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
