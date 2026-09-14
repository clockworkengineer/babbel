#!/usr/bin/env bash
# Fetch official mpaland/bsonfy (BSON Conformance Test Suite)
# URL: https://github.com/mpaland/bsonfy

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_DIR="$ROOT_DIR/crates/bson/tests"
SUITE_DIR="$TARGET_DIR/bsonfy"
REPO_URL="https://github.com/mpaland/bsonfy.git"
ZIP_URL="https://github.com/mpaland/bsonfy/archive/refs/heads/master.zip"
ZIP_PATH="$TARGET_DIR/bsonfy-master.zip"

echo "=== mpaland/bsonfy Conformance Suite Downloader ==="

if [ -f "$SUITE_DIR/test/spec/bson_test.ts" ]; then
    echo "[OK] bsonfy already present at: $SUITE_DIR"
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
    if [ -d "$TARGET_DIR/bsonfy-master" ]; then
        mv "$TARGET_DIR/bsonfy-master" "$SUITE_DIR"
    fi
    rm -f "$ZIP_PATH"
fi

if [ -f "$SUITE_DIR/test/spec/bson_test.ts" ]; then
    echo "[SUCCESS] bsonfy successfully installed at: $SUITE_DIR"
else
    echo "[WARNING] bson_test.ts was not found at expected location: $SUITE_DIR"
fi
