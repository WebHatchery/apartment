<#
.SYNOPSIS
    Validate Second Story's graphics manifests and generated texture files.

.DESCRIPTION
    Ensures the two image-generation manifests agree, IDs are unique, every
    declared texture exists, and each file has the dimensions declared by the
    batch manifest. JPEG substitutions are accepted for the large painterly
    textures that the runtime deliberately loads as compressed assets.
#>
$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$assetsDir = Join-Path $gameDir "assets"
$textureDir = Join-Path $assetsDir "textures"
$batchPath = Join-Path $assetsDir "graphics_batch.json"
$promptsPath = Join-Path $assetsDir "graphics_prompts.json"

$batch = Get-Content -LiteralPath $batchPath -Raw | ConvertFrom-Json
$prompts = Get-Content -LiteralPath $promptsPath -Raw | ConvertFrom-Json
$batchImages = @($batch.image_prompts)
$promptImages = @(
    $prompts.categories.PSObject.Properties | ForEach-Object {
        @($_.Value.images)
    }
)

$errors = [System.Collections.Generic.List[string]]::new()
foreach ($group in @($batchImages | Group-Object id | Where-Object Count -gt 1)) {
    $errors.Add("Duplicate batch ID: $($group.Name)")
}
foreach ($group in @($promptImages | Group-Object id | Where-Object Count -gt 1)) {
    $errors.Add("Duplicate prompt ID: $($group.Name)")
}

$batchIds = @($batchImages.id | Sort-Object)
$promptIds = @($promptImages.id | Sort-Object)
foreach ($difference in @(Compare-Object $batchIds $promptIds)) {
    $side = if ($difference.SideIndicator -eq "<=") { "batch only" } else { "prompt only" }
    $errors.Add("Manifest ID mismatch ($side): $($difference.InputObject)")
}

Add-Type -AssemblyName System.Drawing
foreach ($entry in $batchImages) {
    $paths = @(
        Join-Path $textureDir "$($entry.id).png"
        Join-Path $textureDir "$($entry.id).jpg"
    )
    $path = $paths | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
    if (-not $path) {
        $errors.Add("Missing texture for manifest ID: $($entry.id)")
        continue
    }

    $image = [System.Drawing.Image]::FromFile($path)
    try {
        if ($image.Width -ne $entry.Width -or $image.Height -ne $entry.Height) {
            $errors.Add(
                "$(Split-Path -Leaf $path) is $($image.Width)x$($image.Height), " +
                "expected $($entry.Width)x$($entry.Height)"
            )
        }
    }
    finally {
        $image.Dispose()
    }
}

if ($errors.Count -gt 0) {
    throw "Graphics verification failed:`n - $($errors -join "`n - ")"
}

Write-Output "Graphics verification passed: $($batchImages.Count) manifest textures exist at their declared dimensions."
