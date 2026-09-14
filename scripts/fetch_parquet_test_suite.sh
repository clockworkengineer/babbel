#!/usr/bin/env bash
# Fetch official Apache Parquet test suite (parquet-testing)
# URLs:
#   https://github.com/apache/parquet-testing.git (primary test files)
#   https://github.com/apache/parquet-format.git (format specification)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
TARGET_DIR="${ROOT_DIR}/crates/parquet/tests"
SUITE_DIR="${TARGET_DIR}/parquet-testing"
REPO_URL="https://github.com/apache/parquet-testing.git"
ZIP_URL="https://github.com/apache/parquet-testing/archive/refs/heads/master.zip"
ZIP_PATH="${TARGET_DIR}/parquet-testing-master.zip"

echo "=== Apache Parquet Conformance Suite Downloader ==="

if [ -d "${SUITE_DIR}/data" ]; then
    echo "[OK] parquet-testing already present at: ${SUITE_DIR}"
    exit 0
fi

mkdir -p "${TARGET_DIR}"

if command -v git &> /dev/null; then
    echo "Cloning ${REPO_URL} via git..."
    git clone --depth 1 "${REPO_URL}" "${SUITE_DIR}"
else
    echo "git not found. Downloading archive from ${ZIP_URL} ..."
    if command -v curl &> /dev/null; then
        curl -fsSL "${ZIP_URL}" -o "${ZIP_PATH}"
    elif command -v wget &> /dev/null; then
        wget -q "${ZIP_URL}" -O "${ZIP_PATH}"
    else
        echo "Error: git, curl, or wget is required." >&2
        exit 1
    fi

    echo "Extracting to ${TARGET_DIR} ..."
    unzip -q -o "${ZIP_PATH}" -d "${TARGET_DIR}"

    if [ -d "${TARGET_DIR}/parquet-testing-master" ]; then
        mv "${TARGET_DIR}/parquet-testing-master" "${SUITE_DIR}"
    fi

    rm -f "${ZIP_PATH}"
fi

if [ -d "${SUITE_DIR}/data" ]; then
    echo "[SUCCESS] parquet-testing successfully installed at: ${SUITE_DIR}"
else
    echo "[WARNING] data directory was not found at expected location: ${SUITE_DIR}"
fi
