# Babbel Local Quality Check Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Running Babbel Local Verification Suite" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

Write-Host "[1/5] Checking Formatting (rustfmt)..." -ForegroundColor Yellow
cargo fmt --all -- --check

Write-Host "[2/5] Running Clippy Lints..." -ForegroundColor Yellow
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "[3/5] Checking No-Default-Features (Embedded no_std + alloc)..." -ForegroundColor Yellow
cargo check -p babbel_core --no-default-features --features alloc

Write-Host "[4/5] Running All Workspace Tests..." -ForegroundColor Yellow
cargo test --workspace --all-targets

Write-Host "[5/5] Building Workspace Documentation..." -ForegroundColor Yellow
cargo doc --workspace --no-deps

Write-Host "==========================================================" -ForegroundColor Green
Write-Host " All Babbel Quality Checks Passed Cleanly!" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
