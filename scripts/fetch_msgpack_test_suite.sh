#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/fetch_suite_core.sh" \
    "kawanet/msgpack-test-suite" \
    "https://github.com/kawanet/msgpack-test-suite.git" \
    "https://github.com/kawanet/msgpack-test-suite/archive/refs/heads/master.zip" \
    "crates/msgpack/tests" \
    "msgpack-test-suite" \
    "dist/msgpack-test-suite.json"
