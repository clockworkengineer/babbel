#!/usr/bin/env bash
# Babbel Local Quality Check Script
set -euo pipefail

echo "=========================================================="
echo " Running Babbel Local Verification Suite"
echo "=========================================================="

echo "[1/5] Checking Formatting (rustfmt)..."
cargo fmt --all -- --check

echo "[2/5] Running Clippy Lints..."
cargo clippy --workspace --all-targets -- -D warnings

echo "[3/5] Checking No-Default-Features (Embedded no_std + alloc)..."
cargo check -p babbel_core --no-default-features --features alloc

echo "[4/5] Running All Workspace Tests..."
cargo test --workspace --all-targets

echo "[5/5] Building Workspace Documentation..."
cargo doc --workspace --no-deps

echo "=========================================================="
echo " All Babbel Quality Checks Passed Cleanly!"
echo "=========================================================="
