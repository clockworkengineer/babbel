#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/fetch_suite_core.sh" \
    "nst/JSONTestSuite" \
    "https://github.com/nst/JSONTestSuite.git" \
    "https://github.com/nst/JSONTestSuite/archive/refs/heads/master.zip" \
    "crates/json/tests" \
    "JSONTestSuite" \
    "test_parsing"
