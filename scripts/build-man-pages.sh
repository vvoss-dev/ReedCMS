#!/usr/bin/env bash
# Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
# SPDX-License-Identifier: Apache-2.0

# ReedCMS Man Page Builder
# Compiles .ronn source files to .1 man page format

set -euo pipefail

# Colours for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Colour

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
MAN_SOURCE_DIR="${PROJECT_ROOT}/src/man"

echo "ReedCMS Man Page Builder"
echo "========================"
echo ""

# Check for ronn or ronn-ng
RONN_CMD=""
if command -v ronn-ng &> /dev/null; then
    RONN_CMD="ronn-ng"
elif command -v ronn &> /dev/null; then
    RONN_CMD="ronn"
else
    echo -e "${RED}Error: ronn or ronn-ng not found${NC}"
    echo ""
    echo "Install with:"
    echo "  gem install ronn-ng"
    echo ""
    echo "Alternative:"
    echo "  gem install ronn"
    exit 1
fi

echo "Using: ${RONN_CMD}"
echo ""

# Check if source directory exists
if [ ! -d "${MAN_SOURCE_DIR}" ]; then
    echo -e "${RED}Error: Man source directory not found: ${MAN_SOURCE_DIR}${NC}"
    exit 1
fi

# Count .ronn files
ronn_count=$(find "${MAN_SOURCE_DIR}" -name "*.ronn" -type f | wc -l | tr -d ' ')

if [ "${ronn_count}" -eq 0 ]; then
    echo -e "${YELLOW}Warning: No .ronn files found in ${MAN_SOURCE_DIR}${NC}"
    exit 0
fi

echo "Found ${ronn_count} .ronn source file(s)"
echo ""

# Build each .ronn file
built_count=0
failed_count=0

for ronn_file in "${MAN_SOURCE_DIR}"/*.ronn; do
    if [ -f "${ronn_file}" ]; then
        basename=$(basename "${ronn_file}" .ronn)
        output_file="${MAN_SOURCE_DIR}/${basename}"

        echo "Building ${basename}..."

        if ${RONN_CMD} --roff --pipe "${ronn_file}" > "${output_file}" 2>/dev/null; then
            echo -e "${GREEN}✓ ${basename}${NC}"
            ((built_count++))
        else
            echo -e "${RED}✗ ${basename} - Build failed${NC}"
            ((failed_count++))
        fi
    fi
done

echo ""
echo "Summary:"
echo "========="
echo -e "${GREEN}Built: ${built_count}${NC}"
if [ ${failed_count} -gt 0 ]; then
    echo -e "${RED}Failed: ${failed_count}${NC}"
fi

echo ""
echo "Man pages compiled to: ${MAN_SOURCE_DIR}"

if [ ${failed_count} -gt 0 ]; then
    exit 1
fi
