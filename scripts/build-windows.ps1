param(
    [switch]$SkipTests
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

if (-not $SkipTests) {
    cargo test --workspace --all-targets
    if ($LASTEXITCODE -ne 0) { throw "Tests failed." }
}

cargo build --release --package lucid-browser
if ($LASTEXITCODE -ne 0) { throw "Release build failed." }

$exe = Join-Path (Get-Location) "target\release\lucid-browser.exe"
if (-not (Test-Path $exe)) { throw "Expected executable not found: $exe" }

Write-Host "Build successful: $exe" -ForegroundColor Green
