#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/fetch_suite_core.sh" \
    "apache/parquet-testing" \
    "https://github.com/apache/parquet-testing.git" \
    "https://github.com/apache/parquet-testing/archive/refs/heads/master.zip" \
    "crates/parquet/tests" \
    "parquet-testing" \
    "data"
