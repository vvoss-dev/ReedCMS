#!/usr/bin/env bash
# Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
# SPDX-License-Identifier: Apache-2.0

# ReedCMS System Uninstallation
# Removes binary and man pages from system directories (requires sudo)

set -euo pipefail

# Colours for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Colour

echo "ReedCMS System Uninstallation"
echo "============================="
echo ""

# Check if running with sudo
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script requires root privileges${NC}"
    echo "Please run with sudo: sudo $0"
    exit 1
fi

# Remove binary
echo "Removing binary..."
if [ -f /usr/local/bin/reed ]; then
    rm /usr/local/bin/reed
    echo -e "${GREEN}✓ Binary removed: /usr/local/bin/reed${NC}"
elif [ -L /usr/local/bin/reed ]; then
    rm /usr/local/bin/reed
    echo -e "${GREEN}✓ Binary symlink removed: /usr/local/bin/reed${NC}"
else
    echo -e "${YELLOW}Binary not found at /usr/local/bin/reed${NC}"
fi

# Remove man pages
echo ""
echo "Removing man pages..."
removed_count=0

for man_file in /usr/local/share/man/man1/reed*.1; do
    if [ -f "${man_file}" ] || [ -L "${man_file}" ]; then
        rm "${man_file}"
        man_name=$(basename "${man_file}")
        echo -e "${GREEN}✓ Man page removed: ${man_name}${NC}"
        ((removed_count++))
    fi
done

if [ ${removed_count} -eq 0 ]; then
    echo -e "${YELLOW}No man pages found in /usr/local/share/man/man1/${NC}"
fi

# Update man database
echo ""
echo "Updating man database..."
if command -v mandb &> /dev/null; then
    mandb -q 2>/dev/null || true
    echo -e "${GREEN}✓ Man database updated${NC}"
else
    echo -e "${YELLOW}Warning: mandb not found${NC}"
fi

# Verification
echo ""
echo "Verification:"
echo "============="
if command -v reed &> /dev/null; then
    echo -e "${YELLOW}⚠ 'reed' command still found in PATH${NC}"
    echo "  Location: $(which reed)"
else
    echo -e "${GREEN}✓ 'reed' command removed from system${NC}"
fi

if man -w reed &> /dev/null 2>&1; then
    echo -e "${YELLOW}⚠ 'man reed' still found${NC}"
    echo "  Location: $(man -w reed)"
else
    echo -e "${GREEN}✓ 'man reed' removed from system${NC}"
fi

echo ""
echo -e "${GREEN}System uninstallation complete!${NC}"
