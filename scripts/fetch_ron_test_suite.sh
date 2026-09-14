#!/usr/bin/env bash
# Fetch official starfederation/ron (RON Conformance Test Suite)
# URL: https://github.com/starfederation/ron

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_DIR="$ROOT_DIR/crates/ron/tests"
SUITE_DIR="$TARGET_DIR/ron-upstream"
REPO_URL="https://github.com/starfederation/ron.git"
ZIP_URL="https://github.com/starfederation/ron/archive/refs/heads/main.zip"
ZIP_PATH="$TARGET_DIR/ron-upstream-main.zip"

echo "=== starfederation/ron Conformance Suite Downloader ==="

if [ -f "$SUITE_DIR/testdata/conformance/manifest.json" ]; then
    echo "[OK] starfederation/ron already present at: $SUITE_DIR"
    exit 0
fi

mkdir -p "$TARGET_DIR"

if command -v git &> /dev/null; then
    echo "Cloning $REPO_URL via git..."
    git clone --depth 1 "$REPO_URL" "$SUITE_DIR"
else
    echo "git not found. Downloading archive from $ZIP_URL ..."
    curl -fsSL "$ZIP_URL" -o "$ZIP_PATH"

    echo "Extracting to $TARGET_DIR ..."
    unzip -q "$ZIP_PATH" -d "$TARGET_DIR"

    if [ -d "$TARGET_DIR/ron-main" ]; then
        mv "$TARGET_DIR/ron-main" "$SUITE_DIR"
    fi

    rm -f "$ZIP_PATH"
fi

if [ -f "$SUITE_DIR/testdata/conformance/manifest.json" ]; then
    echo "[SUCCESS] starfederation/ron successfully installed at: $SUITE_DIR"
else
    echo "[WARNING] manifest.json was not found at expected location: $SUITE_DIR"
fi
