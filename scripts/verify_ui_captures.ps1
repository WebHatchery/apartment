<#
.SYNOPSIS
    Validate Second Story's committed UI verification images.

.DESCRIPTION
    Rejects missing desktop/narrow evidence, narrow captures that are not
    640x480, and byte-identical PNGs committed under different scene names.
    Pixel hashes are deliberately not compared to a golden manifest because
    live character animation samples frame time during capture.
#>
param(
    [string]$CaptureDir = "docs\verification"
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$resolvedDir = Join-Path $gameDir $CaptureDir

if (-not (Test-Path -LiteralPath $resolvedDir -PathType Container)) {
    throw "Capture directory not found: $resolvedDir"
}

Add-Type -AssemblyName System.Drawing
$files = @(Get-ChildItem -LiteralPath $resolvedDir -Filter "ui_*.png" -File)
if ($files.Count -eq 0) {
    throw "No ui_*.png captures found in $resolvedDir"
}

$required = @(
    "ui_menu.png",
    "ui_showcase.png",
    "ui_finances_showcase.png",
    "ui_city_showcase.png",
    "ui_inbox_showcase.png",
    "ui_tasks_showcase.png",
    "ui_career_showcase.png",
    "ui_event_showcase.png",
    "ui_menu_tiny.png",
    "ui_showcase_tiny.png",
    "ui_finances_tiny.png",
    "ui_city_tiny.png",
    "ui_inbox_tiny.png",
    "ui_tasks_tiny.png",
    "ui_career_tiny.png",
    "ui_event_tiny.png"
)
$names = @{}
foreach ($file in $files) {
    $names[$file.Name] = $true
}
$missing = @($required | Where-Object { -not $names.ContainsKey($_) })
if ($missing.Count -gt 0) {
    throw "Missing required captures: $($missing -join ', ')"
}

$errors = [System.Collections.Generic.List[string]]::new()
$hashGroups = @{}
foreach ($file in $files) {
    $image = [System.Drawing.Image]::FromFile($file.FullName)
    try {
        if ($file.Name -like "*_tiny.png" -and `
            ($image.Width -ne 640 -or $image.Height -ne 480)) {
            $errors.Add("$($file.Name) is $($image.Width)x$($image.Height), expected 640x480")
        }
    }
    finally {
        $image.Dispose()
    }

    $hash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
    if (-not $hashGroups.ContainsKey($hash)) {
        $hashGroups[$hash] = [System.Collections.Generic.List[string]]::new()
    }
    $hashGroups[$hash].Add($file.Name)
}

foreach ($group in $hashGroups.Values) {
    if ($group.Count -gt 1) {
        $errors.Add("Byte-identical named scenes: $($group -join ', ')")
    }
}

if ($errors.Count -gt 0) {
    throw "UI capture verification failed:`n - $($errors -join "`n - ")"
}

Write-Output "UI capture verification passed: $($files.Count) distinct images; all *_tiny.png files are 640x480."
