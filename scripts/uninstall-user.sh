#!/usr/bin/env bash
# Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
# SPDX-License-Identifier: Apache-2.0

# ReedCMS User Uninstallation
# Removes binary and man pages from user directories (no sudo required)

set -euo pipefail

# Colours for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Colour

USER_BIN_DIR="${HOME}/.local/bin"
USER_MAN_DIR="${HOME}/.local/share/man/man1"

echo "ReedCMS User Uninstallation"
echo "==========================="
echo ""

# Remove binary
echo "Removing binary..."
if [ -f "${USER_BIN_DIR}/reed" ]; then
    rm "${USER_BIN_DIR}/reed"
    echo -e "${GREEN}✓ Binary removed: ${USER_BIN_DIR}/reed${NC}"
else
    echo -e "${YELLOW}Binary not found at ${USER_BIN_DIR}/reed${NC}"
fi

# Remove man pages
echo ""
echo "Removing man pages..."
removed_count=0

if [ -d "${USER_MAN_DIR}" ]; then
    for man_file in "${USER_MAN_DIR}"/reed*.1; do
        if [ -f "${man_file}" ]; then
            rm "${man_file}"
            man_name=$(basename "${man_file}")
            echo -e "${GREEN}✓ Man page removed: ${man_name}${NC}"
            ((removed_count++))
        fi
    done
fi

if [ ${removed_count} -eq 0 ]; then
    echo -e "${YELLOW}No man pages found in ${USER_MAN_DIR}${NC}"
fi

# Verification
echo ""
echo "Verification:"
echo "============="
if command -v reed &> /dev/null; then
    echo -e "${YELLOW}⚠ 'reed' command still found in PATH${NC}"
    echo "  Location: $(which reed)"
    echo "  (This may be a system installation)"
else
    echo -e "${GREEN}✓ 'reed' command removed${NC}"
fi

if man -w reed &> /dev/null 2>&1; then
    echo -e "${YELLOW}⚠ 'man reed' still found${NC}"
    echo "  Location: $(man -w reed)"
    echo "  (This may be a system installation)"
else
    echo -e "${GREEN}✓ 'man reed' removed${NC}"
fi

echo ""
echo -e "${GREEN}User uninstallation complete!${NC}"
