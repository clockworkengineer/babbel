#!/usr/bin/env bash
# Master Conformance Test Suite Downloader Dispatcher (Bash)
set -e

FORMAT="${1:-all}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================================="
echo " Babbel Conformance Test Suite Master Downloader"
echo " Target Filter: $FORMAT"
echo "=========================================================="

run_suite() {
    local fmt="$1"
    local script="$2"
    if [ "$FORMAT" = "all" ] || [ "$FORMAT" = "$fmt" ]; then
        if [ -f "$SCRIPT_DIR/$script" ]; then
            bash "$SCRIPT_DIR/$script"
            echo ""
        else
            echo "Missing script: $SCRIPT_DIR/$script"
        fi
    fi
}

run_suite "json" "fetch_json_test_suite.sh"
run_suite "toml" "fetch_toml_test_suite.sh"
run_suite "xml" "fetch_w3c_xmlts.sh"
run_suite "cbor" "fetch_cbor_test_suite.sh"
run_suite "bson" "fetch_bson_test_suite.sh"
run_suite "msgpack" "fetch_msgpack_test_suite.sh"
run_suite "ron" "fetch_ron_test_suite.sh"
run_suite "kdl" "fetch_kdl_test_suite.sh"
run_suite "parquet" "fetch_parquet_test_suite.sh"

echo "All requested conformance test suites checked/downloaded."
