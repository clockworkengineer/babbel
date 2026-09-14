#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/fetch_suite_core.sh" \
    "mpaland/bsonfy" \
    "https://github.com/mpaland/bsonfy.git" \
    "https://github.com/mpaland/bsonfy/archive/refs/heads/master.zip" \
    "crates/bson/tests" \
    "bsonfy" \
    "test/spec/bson_test.ts"
