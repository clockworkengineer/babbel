#!/usr/bin/env bash
# Fetch official toml-test (TOML v1.0.0 Conformance Test Suite)
# URL: https://github.com/toml-lang/toml-test

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_DIR="$ROOT_DIR/crates/toml/tests"
SUITE_DIR="$TARGET_DIR/toml-test"
REPO_URL="https://github.com/toml-lang/toml-test.git"
ZIP_URL="https://github.com/toml-lang/toml-test/archive/refs/heads/master.zip"
ZIP_PATH="$TARGET_DIR/toml-test-master.zip"

echo "=== toml-lang/toml-test Conformance Suite Downloader ==="

if [ -d "$SUITE_DIR/tests" ]; then
    echo "[OK] toml-test already present at: $SUITE_DIR"
    exit 0
fi

mkdir -p "$TARGET_DIR"

if command -v git &> /dev/null; then
    echo "Cloning $REPO_URL via git..."
    git clone --depth 1 "$REPO_URL" "$SUITE_DIR"
else
    echo "git not found. Downloading archive from $ZIP_URL ..."
    curl -sSL "$ZIP_URL" -o "$ZIP_PATH"
    echo "Extracting to $TARGET_DIR ..."
    unzip -q "$ZIP_PATH" -d "$TARGET_DIR"
    if [ -d "$TARGET_DIR/toml-test-master" ]; then
        mv "$TARGET_DIR/toml-test-master" "$SUITE_DIR"
    fi
    rm -f "$ZIP_PATH"
fi

if [ -d "$SUITE_DIR/tests" ]; then
    echo "[SUCCESS] toml-test successfully installed at: $SUITE_DIR"
else
    echo "[WARNING] tests directory was not found at expected location: $SUITE_DIR"
fi
