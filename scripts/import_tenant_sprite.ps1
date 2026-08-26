param(
    [Parameter(Mandatory = $true)] [string]$InputPath,
    [Parameter(Mandatory = $true)] [string]$OutputPath
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
$drawingReferences = @(
    [System.Drawing.Bitmap].Assembly.Location,
    [System.Drawing.Color].Assembly.Location,
    [System.Collections.Queue].Assembly.Location,
    (Join-Path $PSHOME "System.Private.Windows.Core.dll"),
    (Join-Path $PSHOME "System.Private.Windows.GdiPlus.dll")
)
Add-Type -ReferencedAssemblies $drawingReferences -TypeDefinition @'
using System;
using System.Collections;
using System.Drawing;
using System.Drawing.Imaging;

public static class TenantSpriteImporter
{
    public static void RemoveEdgeConnectedCheckerboard(string inputPath, string outputPath)
    {
        using (var source = new Bitmap(inputPath))
        using (var bitmap = new Bitmap(source.Width, source.Height, PixelFormat.Format32bppArgb))
        {
            using (var graphics = Graphics.FromImage(bitmap))
                graphics.DrawImageUnscaled(source, 0, 0);

            var visited = new bool[bitmap.Width * bitmap.Height];
            var queue = new Queue();
            Action<int, int> enqueue = (x, y) => {
                if (x < 0 || y < 0 || x >= bitmap.Width || y >= bitmap.Height) return;
                int index = y * bitmap.Width + x;
                if (visited[index]) return;
                Color color = bitmap.GetPixel(x, y);
                int max = Math.Max(color.R, Math.Max(color.G, color.B));
                int min = Math.Min(color.R, Math.Min(color.G, color.B));
                if (color.R < 225 || color.G < 225 || color.B < 225 || max - min > 12) return;
                visited[index] = true;
                queue.Enqueue(index);
            };

            for (int x = 0; x < bitmap.Width; x++) {
                enqueue(x, 0);
                enqueue(x, bitmap.Height - 1);
            }
            for (int y = 0; y < bitmap.Height; y++) {
                enqueue(0, y);
                enqueue(bitmap.Width - 1, y);
            }

            while (queue.Count > 0) {
                int index = (int)queue.Dequeue();
                int x = index % bitmap.Width;
                int y = index / bitmap.Width;
                Color pixel = bitmap.GetPixel(x, y);
                bitmap.SetPixel(x, y, Color.FromArgb(0, pixel.R, pixel.G, pixel.B));
                enqueue(x - 1, y);
                enqueue(x + 1, y);
                enqueue(x, y - 1);
                enqueue(x, y + 1);
            }

            bitmap.Save(outputPath, ImageFormat.Png);
        }
    }
}
'@

$resolvedInput = (Resolve-Path -LiteralPath $InputPath).Path
$resolvedOutput = [System.IO.Path]::GetFullPath((Join-Path (Get-Location) $OutputPath))
$outputDirectory = Split-Path -Parent $resolvedOutput
if (-not (Test-Path -LiteralPath $outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory | Out-Null
}
[TenantSpriteImporter]::RemoveEdgeConnectedCheckerboard($resolvedInput, $resolvedOutput)

Write-Output "Imported transparent sprite: $resolvedOutput"
