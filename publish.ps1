#!/usr/bin/env pwsh
# Publish paperwork crates to crates.io
# Prerequisites: cargo login <YOUR_CRATES_IO_TOKEN>
# Order matters: core must be published before cli (dependency)

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host "=== Step 1: Publish paperwork-core ===" -ForegroundColor Cyan
cargo publish -p paperwork-core
if ($LASTEXITCODE -ne 0) { throw "paperwork-core publish failed" }

Write-Host "`n=== Step 2: Wait for crates.io index (polling, up to 5 min) ===" -ForegroundColor Yellow
$deadline = (Get-Date).AddMinutes(5)
$indexed = $false
while ((Get-Date) -lt $deadline) {
    $hit = cargo search paperwork-core --limit 1 2>$null | Select-String 'paperwork-core\s*=\s*"0\.5\.0"'
    if ($hit) { $indexed = $true; break }
    Start-Sleep -Seconds 10
}
if (-not $indexed) {
    Write-Warning "paperwork-core 0.5.0 not visible on crates.io after ~5 minutes."
    Write-Warning "Re-run 'cargo publish -p paperwork-cli' manually once it appears."
    exit 1
}
Write-Host "paperwork-core 0.5.0 is visible on crates.io" -ForegroundColor Green

Write-Host "`n=== Step 3: Publish paperwork-cli ===" -ForegroundColor Cyan
cargo publish -p paperwork-cli
if ($LASTEXITCODE -ne 0) { throw "paperwork-cli publish failed" }

Write-Host "`n=== Done ===" -ForegroundColor Green
Write-Host "  https://crates.io/crates/paperwork-core"
Write-Host "  https://crates.io/crates/paperwork-cli"
Write-Host "  Install: cargo install paperwork-cli"
