#!/usr/bin/env bash
# Fetch official kdl-org/kdl-test (KDL Conformance Test Suite)
# URL: https://github.com/kdl-org/kdl-test

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_DIR="$ROOT_DIR/crates/kdl/tests"
SUITE_DIR="$TARGET_DIR/kdl-test"
REPO_URL="https://github.com/kdl-org/kdl-test.git"
ZIP_URL="https://github.com/kdl-org/kdl-test/archive/refs/heads/main.zip"
ZIP_PATH="$TARGET_DIR/kdl-test-main.zip"

echo "=== kdl-org/kdl-test Conformance Suite Downloader ==="

if [ -d "$SUITE_DIR/test_cases" ]; then
    echo "[OK] kdl-org/kdl-test already present at: $SUITE_DIR"
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

    if [ -d "$TARGET_DIR/kdl-test-main" ]; then
        mv "$TARGET_DIR/kdl-test-main" "$SUITE_DIR"
    fi

    rm -f "$ZIP_PATH"
fi

if [ -d "$SUITE_DIR/test_cases" ]; then
    echo "[SUCCESS] kdl-org/kdl-test successfully installed at: $SUITE_DIR"
else
    echo "[WARNING] test_cases was not found at expected location: $SUITE_DIR"
fi
