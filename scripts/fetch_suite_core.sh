#!/usr/bin/env bash
# Parameterized Conformance Test Suite Downloader Engine (Bash)
set -e

NAME="$1"
REPO_URL="$2"
ZIP_URL="$3"
TARGET_RELATIVE_DIR="$4"
SUITE_DIR_NAME="$5"
MARKER_FILE="$6"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_DIR="$ROOT_DIR/$TARGET_RELATIVE_DIR"
SUITE_DIR="$TARGET_DIR/$SUITE_DIR_NAME"
ZIP_PATH="$TARGET_DIR/$SUITE_DIR_NAME-temp.zip"

echo "=== $NAME Conformance Suite Downloader ==="

if [ -e "$SUITE_DIR/$MARKER_FILE" ]; then
    echo "[OK] $NAME already present at: $SUITE_DIR"
    exit 0
fi

mkdir -p "$TARGET_DIR"

if [ -n "$REPO_URL" ] && command -v git &> /dev/null; then
    echo "Cloning $REPO_URL via git..."
    git clone --depth 1 "$REPO_URL" "$SUITE_DIR"
else
    echo "Downloading archive from $ZIP_URL ..."
    if command -v curl &> /dev/null; then
        curl -sSL "$ZIP_URL" -o "$ZIP_PATH"
    elif command -v wget &> /dev/null; then
        wget -q "$ZIP_URL" -O "$ZIP_PATH"
    else
        echo "Error: neither curl nor wget found."
        exit 1
    fi

    echo "Extracting to $TARGET_DIR ..."
    unzip -q -o "$ZIP_PATH" -d "$TARGET_DIR"

    REPO_BASE="$(basename "$REPO_URL" .git)"
    for CAND in "$TARGET_DIR/$SUITE_DIR_NAME-master" "$TARGET_DIR/$SUITE_DIR_NAME-main" "$TARGET_DIR/$REPO_BASE-master" "$TARGET_DIR/$REPO_BASE-main"; do
        if [ -d "$CAND" ] && [ "$CAND" != "$SUITE_DIR" ]; then
            mv "$CAND" "$SUITE_DIR"
            break
        fi
    done

    rm -f "$ZIP_PATH"
fi

if [ -e "$SUITE_DIR/$MARKER_FILE" ]; then
    echo "[SUCCESS] $NAME successfully installed at: $SUITE_DIR"
else
    echo "[WARNING] Marker '$MARKER_FILE' was not found at expected location: $SUITE_DIR"
fi
