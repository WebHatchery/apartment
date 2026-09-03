# RustGames project itch.io publisher wrapper.
# Configuration lives in itch.json; shared behavior lives in the workspace
# rust_management/publish-itch.ps1 script.

param(
    [ValidateSet("all", "html5", "windows")]
    [string]$Channel = "all",
    [string]$ButlerPath = "",
    [string]$UserVersion = "",
    [switch]$Preview,
    [switch]$Status,
    [switch]$DryRun,
    [switch]$SkipDemoBuild,
    [switch]$Help
)

$ErrorActionPreference = "Stop"
$rootPublisher = Join-Path (Split-Path $PSScriptRoot -Parent) "publish-itch.ps1"
if (-not (Test-Path $rootPublisher)) {
    Write-Error "RustGames itch publisher not found: $rootPublisher"
    exit 1
}

$needsHtml5Package = $Channel -in @("all", "html5") -and -not $Status -and -not $Help
$distDir = Join-Path $PSScriptRoot "dist"
$webglDir = Join-Path $distDir "webgl"
$demoDir = Join-Path $distDir "demo\webgl"
$backupDir = $null
$swapped = $false

function Remove-GeneratedDirectory {
    param([string]$Parent, [string]$Child)

    if (-not (Test-Path $Child)) { return }
    $parentPath = [System.IO.Path]::GetFullPath($Parent).TrimEnd('\', '/')
    $childPath = [System.IO.Path]::GetFullPath($Child).TrimEnd('\', '/')
    $prefix = $parentPath + [System.IO.Path]::DirectorySeparatorChar
    if (-not $childPath.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to remove generated directory outside dist: $childPath"
    }
    Remove-Item -LiteralPath $childPath -Recurse -Force
}

try {
    if ($needsHtml5Package) {
        if (-not (Test-Path $webglDir -PathType Container)) {
            throw "Full WebGL artifacts not found: $webglDir. Run .\publish.ps1 first."
        }

        if (-not $SkipDemoBuild) {
            & cargo build --release --features demo --target wasm32-unknown-unknown --target-dir target-demo
            if ($LASTEXITCODE -ne 0) { throw "Demo WebGL build failed with exit code $LASTEXITCODE." }
        }

        $demoWasm = Join-Path $PSScriptRoot "target-demo\wasm32-unknown-unknown\release\apartment.wasm"
        if (-not (Test-Path $demoWasm -PathType Leaf)) {
            throw "Demo WebGL module not found: $demoWasm"
        }

        Remove-GeneratedDirectory $distDir $demoDir
        New-Item -ItemType Directory -Path $demoDir -Force | Out-Null
        Copy-Item -Path (Join-Path $webglDir "*") -Destination $demoDir -Recurse -Force
        Copy-Item -LiteralPath $demoWasm -Destination (Join-Path $demoDir "apartment.wasm") -Force

        $backupDir = Join-Path $distDir ("webgl-full-" + [guid]::NewGuid().ToString("N"))
        Move-Item -LiteralPath $webglDir -Destination $backupDir
        Move-Item -LiteralPath $demoDir -Destination $webglDir
        $swapped = $true
    }

    & $rootPublisher -ProjectDir $PSScriptRoot -Channel $Channel `
        -ButlerPath $ButlerPath -UserVersion $UserVersion -Preview:$Preview `
        -Status:$Status -DryRun:$DryRun -Help:$Help
    if (-not $?) { exit 1 }
} finally {
    if ($swapped -and (Test-Path $webglDir -PathType Container)) {
        Move-Item -LiteralPath $webglDir -Destination $demoDir
    }
    if ($null -ne $backupDir -and (Test-Path $backupDir -PathType Container)) {
        if (Test-Path $webglDir) {
            throw "Cannot restore full WebGL artifacts because the destination exists: $webglDir"
        }
        Move-Item -LiteralPath $backupDir -Destination $webglDir
    }
}
