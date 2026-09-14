#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/fetch_suite_core.sh" \
    "cbor/test-vectors" \
    "https://github.com/cbor/test-vectors.git" \
    "https://github.com/cbor/test-vectors/archive/refs/heads/master.zip" \
    "crates/cbor/tests" \
    "test-vectors" \
    "appendix_a.json"
