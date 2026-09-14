#!/usr/bin/env bash
# Fetch official cbor/test-vectors (CBOR RFC 7049 Appendix A Conformance Suite)
# URL: https://github.com/cbor/test-vectors

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_DIR="$ROOT_DIR/crates/cbor/tests"
SUITE_DIR="$TARGET_DIR/test-vectors"
REPO_URL="https://github.com/cbor/test-vectors.git"
ZIP_URL="https://github.com/cbor/test-vectors/archive/refs/heads/master.zip"
ZIP_PATH="$TARGET_DIR/test-vectors-master.zip"

echo "=== cbor/test-vectors Conformance Suite Downloader ==="

if [ -f "$SUITE_DIR/appendix_a.json" ]; then
    echo "[OK] test-vectors already present at: $SUITE_DIR"
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
    if [ -d "$TARGET_DIR/test-vectors-master" ]; then
        mv "$TARGET_DIR/test-vectors-master" "$SUITE_DIR"
    fi
    rm -f "$ZIP_PATH"
fi

if [ -f "$SUITE_DIR/appendix_a.json" ]; then
    echo "[SUCCESS] test-vectors successfully installed at: $SUITE_DIR"
else
    echo "[WARNING] appendix_a.json was not found at expected location: $SUITE_DIR"
fi
