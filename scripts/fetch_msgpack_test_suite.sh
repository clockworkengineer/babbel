#!/usr/bin/env bash
# Fetch official msgpack-test-suite (MessagePack Conformance Test Suite)
# URL: https://github.com/kawanet/msgpack-test-suite

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_DIR="$ROOT_DIR/crates/msgpack/tests"
SUITE_DIR="$TARGET_DIR/msgpack-test-suite"
REPO_URL="https://github.com/kawanet/msgpack-test-suite.git"
ZIP_URL="https://github.com/kawanet/msgpack-test-suite/archive/refs/heads/master.zip"
ZIP_PATH="$TARGET_DIR/msgpack-test-suite-master.zip"

echo "=== kawanet/msgpack-test-suite Conformance Suite Downloader ==="

if [ -f "$SUITE_DIR/dist/msgpack-test-suite.json" ] || [ -d "$SUITE_DIR/src" ]; then
    echo "[OK] msgpack-test-suite already present at: $SUITE_DIR"
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
    if [ -d "$TARGET_DIR/msgpack-test-suite-master" ]; then
        mv "$TARGET_DIR/msgpack-test-suite-master" "$SUITE_DIR"
    fi
    rm -f "$ZIP_PATH"
fi

if [ -f "$SUITE_DIR/dist/msgpack-test-suite.json" ] || [ -d "$SUITE_DIR/src" ]; then
    echo "[SUCCESS] msgpack-test-suite successfully installed at: $SUITE_DIR"
else
    echo "[WARNING] test files were not found at expected location: $SUITE_DIR"
fi
