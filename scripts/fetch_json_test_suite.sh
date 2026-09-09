#!/usr/bin/env bash
# Fetch official JSONTestSuite (RFC 8259 Conformance Test Suite)
# URL: https://github.com/nst/JSONTestSuite

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_DIR="$ROOT_DIR/crates/json/tests"
SUITE_DIR="$TARGET_DIR/JSONTestSuite"
REPO_URL="https://github.com/nst/JSONTestSuite.git"
ZIP_URL="https://github.com/nst/JSONTestSuite/archive/refs/heads/master.zip"
ZIP_PATH="$TARGET_DIR/JSONTestSuite-master.zip"

echo "=== nst/JSONTestSuite Conformance Suite Downloader ==="

if [ -d "$SUITE_DIR/test_parsing" ]; then
    echo "[OK] JSONTestSuite already present at: $SUITE_DIR"
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
    unzip -q -o "$ZIP_PATH" -d "$TARGET_DIR"

    if [ -d "$TARGET_DIR/JSONTestSuite-master" ]; then
        mv "$TARGET_DIR/JSONTestSuite-master" "$SUITE_DIR"
    fi

    rm -f "$ZIP_PATH"
fi

if [ -d "$SUITE_DIR/test_parsing" ]; then
    echo "[SUCCESS] JSONTestSuite successfully installed at: $SUITE_DIR"
else
    echo "[WARNING] test_parsing was not found at expected location: $SUITE_DIR"
fi
