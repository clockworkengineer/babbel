#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/fetch_suite_core.sh" \
    "skystrife/toml-test" \
    "https://github.com/skystrife/toml-test.git" \
    "https://github.com/skystrife/toml-test/archive/refs/heads/master.zip" \
    "crates/toml/tests" \
    "toml-test" \
    "tests"
