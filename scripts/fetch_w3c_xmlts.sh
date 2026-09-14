#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/fetch_suite_core.sh" \
    "W3C XML Conformance Test Suite (XML TS 20130923)" \
    "" \
    "https://www.w3.org/XML/Test/xmlts20130923.zip" \
    "crates/xml/tests" \
    "xmlconf" \
    "xmlconf.xml"
