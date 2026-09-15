#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"$SCRIPT_DIR/fetch_suite_core.sh" \
    "kmoneil/hcl-test-suite" \
    "https://github.com/kmoneil/hcl-test-suite.git" \
    "https://github.com/kmoneil/hcl-test-suite/archive/refs/heads/main.zip" \
    "crates/hcl/tests" \
    "hcl-test-suite" \
    "tests"
