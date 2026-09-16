#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"$SCRIPT_DIR/fetch_suite_core.sh" \
    "drnice/AvroTest" \
    "https://github.com/drnice/AvroTest.git" \
    "https://github.com/drnice/AvroTest/archive/refs/heads/master.zip" \
    "crates/avro/tests" \
    "avro-test-suite" \
    "users2.avro"
