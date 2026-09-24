$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'src-tauri\target\release\mistria-tracker.exe'
$outDir = Join-Path $root 'dist-portable'
$zip = Join-Path $root 'MistriaTracker-0.1.7-portable.zip'
if (-not (Test-Path -LiteralPath $exe)) { throw "Release executable not found. Build it first with: pnpm tauri build --no-bundle" }
if (Test-Path -LiteralPath $outDir) { Remove-Item -LiteralPath $outDir -Recurse -Force }
if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force }
New-Item -ItemType Directory -Path $outDir | Out-Null
Copy-Item -LiteralPath $exe -Destination (Join-Path $outDir 'MistriaTracker.exe')
Set-Content -LiteralPath (Join-Path $outDir 'README.txt') -Value "Run MistriaTracker.exe. This portable build does not install services, modify game saves, or require administrator access." -Encoding UTF8
Compress-Archive -Path (Join-Path $outDir '*') -DestinationPath $zip -CompressionLevel Optimal
$hash = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash
Set-Content -LiteralPath ($zip + '.sha256') -Value "$hash  MistriaTracker-0.1.7-portable.zip" -Encoding ASCII
Write-Host "Created $zip"
Write-Host "SHA256: $hash"
