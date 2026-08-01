#!/usr/bin/env pwsh
# Publish paperwork crates to crates.io
# Prerequisites: cargo login <YOUR_CRATES_IO_TOKEN>
# Order matters: core must be published before cli (dependency)

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host "=== Step 1: Publish paperwork-core ===" -ForegroundColor Cyan
cargo publish -p paperwork-core
if ($LASTEXITCODE -ne 0) { throw "paperwork-core publish failed" }

Write-Host "`n=== Step 2: Wait for crates.io index (30s) ===" -ForegroundColor Yellow
Start-Sleep -Seconds 30

Write-Host "`n=== Step 3: Publish paperwork-cli ===" -ForegroundColor Cyan
cargo publish -p paperwork-cli
if ($LASTEXITCODE -ne 0) { throw "paperwork-cli publish failed" }

Write-Host "`n=== Done ===" -ForegroundColor Green
Write-Host "  https://crates.io/crates/paperwork-core"
Write-Host "  https://crates.io/crates/paperwork-cli"
Write-Host "  Install: cargo install paperwork-cli"
