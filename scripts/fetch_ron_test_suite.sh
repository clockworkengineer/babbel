#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/fetch_suite_core.sh" \
    "starfederation/ron" \
    "https://github.com/starfederation/ron.git" \
    "https://github.com/starfederation/ron/archive/refs/heads/main.zip" \
    "crates/ron/tests" \
    "ron-upstream" \
    "testdata/conformance/manifest.json"
