#!/usr/bin/env bash
# Fetch official W3C XML Conformance Test Suite (XML TS 20130923)
# URL: https://www.w3.org/XML/Test/

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_DIR="$ROOT_DIR/crates/xml/tests"
ZIP_PATH="$TARGET_DIR/xmlts20130923.zip"
XMLCONF_DIR="$TARGET_DIR/xmlconf"
URL="https://www.w3.org/XML/Test/xmlts20130923.zip"

echo "=== W3C XML Conformance Test Suite Downloader ==="

if [ -f "$XMLCONF_DIR/xmlconf.xml" ]; then
    echo "[OK] W3C XML test suite already present at: $XMLCONF_DIR"
    exit 0
fi

echo "Downloading $URL ..."
curl -sSL "$URL" -o "$ZIP_PATH"

echo "Extracting to $TARGET_DIR ..."
unzip -q -o "$ZIP_PATH" -d "$TARGET_DIR"

rm -f "$ZIP_PATH"

if [ -f "$XMLCONF_DIR/xmlconf.xml" ]; then
    echo "[SUCCESS] W3C XML Conformance Test Suite successfully installed at: $XMLCONF_DIR"
else
    echo "[WARNING] Archive extracted, but xmlconf.xml was not found at expected location: $XMLCONF_DIR"
fi
