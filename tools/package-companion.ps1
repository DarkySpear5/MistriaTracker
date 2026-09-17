$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$source = Join-Path $root 'companion\mistria_tracker_companion'
$stage = Join-Path $root 'dist-companion\MistriaTrackerCompanion'
$zip = Join-Path $root 'MistriaTracker-Companion-0.1.3.zip'
if (-not (Test-Path -LiteralPath (Join-Path $source 'manifest.json'))) { throw 'Companion manifest not found.' }
if (Test-Path -LiteralPath (Join-Path $root 'dist-companion')) { Remove-Item -LiteralPath (Join-Path $root 'dist-companion') -Recurse -Force }
if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force }
New-Item -ItemType Directory -Path $stage -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $source '*') -Destination $stage -Recurse
Compress-Archive -LiteralPath (Join-Path $stage '*') -DestinationPath $zip -CompressionLevel Optimal
Write-Host "Created $zip"
