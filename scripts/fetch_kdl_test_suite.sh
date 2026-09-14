#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/fetch_suite_core.sh" \
    "kdl-org/kdl-test" \
    "https://github.com/kdl-org/kdl-test.git" \
    "https://github.com/kdl-org/kdl-test/archive/refs/heads/main.zip" \
    "crates/kdl/tests" \
    "kdl-test" \
    "test_cases"
