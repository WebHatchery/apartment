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
        "resident_sprites",
        "unit_showcase",
        "hallway_showcase",
        "applications_showcase",
        "applications_more_showcase",
        "ownership_showcase",
        "tenants_showcase",
        "tenants_more_showcase",
        "finances_showcase",
        "city_showcase",
        "city_filtered_showcase",
        "market_showcase",
        "market_filtered_showcase",
        "purchase_review_showcase",
        "inbox_showcase",
        "tasks_showcase",
        "requests_more_showcase",
        "pause_showcase",
        "history_showcase",
        "career_showcase",
        "career_more_showcase",
        "tutorial_showcase",
        "tutorial_coach_showcase",
        "event_showcase",
        "event_notice_showcase",
        "notification_showcase"
    ),
    [int]$Frames = 150,
    [int]$WindowWidth = 0,
    [int]$WindowHeight = 0,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames `
    -WindowWidth $WindowWidth -WindowHeight $WindowHeight `
    -OutputDir $OutputDir -SkipBuild:$SkipBuild
