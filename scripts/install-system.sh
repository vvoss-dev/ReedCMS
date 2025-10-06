#!/usr/bin/env bash
# Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
# SPDX-License-Identifier: Apache-2.0

# ReedCMS System-Wide Installation
# Installs binary and man pages to system directories (requires sudo)

set -euo pipefail

# Colours for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Colour

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BINARY_PATH="${PROJECT_ROOT}/target/release/reed"
MAN_DIR="${PROJECT_ROOT}/man"

echo "ReedCMS System Installation"
echo "==========================="
echo ""

# Check if running with sudo
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script requires root privileges${NC}"
    echo "Please run with sudo: sudo $0"
    exit 1
fi

# Check if binary exists
if [ ! -f "${BINARY_PATH}" ]; then
    echo -e "${RED}Error: Binary not found at ${BINARY_PATH}${NC}"
    echo "Run 'cargo build --release' first"
    exit 1
fi

# Check if man pages exist
if [ ! -d "${MAN_DIR}" ] || [ -z "$(ls -A ${MAN_DIR}/*.1 2>/dev/null)" ]; then
    echo -e "${YELLOW}Warning: Man pages not found in ${MAN_DIR}${NC}"
    echo "Building man pages..."
    "${SCRIPT_DIR}/build-man-pages.sh"
fi

# Install binary
echo "Installing binary..."
if [ -f /usr/local/bin/reed ]; then
    echo -e "${YELLOW}Removing existing installation${NC}"
    rm /usr/local/bin/reed
fi

install -m 755 "${BINARY_PATH}" /usr/local/bin/reed
echo -e "${GREEN}✓ Binary installed: /usr/local/bin/reed${NC}"

# Install man pages
echo ""
echo "Installing man pages..."
mkdir -p /usr/local/share/man/man1

for man_file in "${MAN_DIR}"/*.1; do
    if [ -f "${man_file}" ]; then
        man_name=$(basename "${man_file}")
        install -m 644 "${man_file}" /usr/local/share/man/man1/"${man_name}"
        echo -e "${GREEN}✓ Man page installed: ${man_name}${NC}"
    fi
done

# Update man database
echo ""
echo "Updating man database..."
if command -v mandb &> /dev/null; then
    mandb -q 2>/dev/null || true
    echo -e "${GREEN}✓ Man database updated${NC}"
else
    echo -e "${YELLOW}Warning: mandb not found, man pages may not be indexed${NC}"
fi

# Verification
echo ""
echo "Verification:"
echo "============="
if command -v reed &> /dev/null; then
    echo -e "${GREEN}✓ 'reed' command is available${NC}"
    reed --version 2>/dev/null || echo -e "${YELLOW}  (version command not implemented yet)${NC}"
else
    echo -e "${RED}✗ 'reed' command not found in PATH${NC}"
fi

if man -w reed &> /dev/null; then
    echo -e "${GREEN}✓ 'man reed' is available${NC}"
else
    echo -e "${YELLOW}⚠ 'man reed' not found (may need to restart shell)${NC}"
fi

echo ""
echo -e "${GREEN}System installation complete!${NC}"
echo ""
echo "Usage:"
echo "  reed data:get knowledge.title@en    # Run from anywhere"
echo "  man reed                             # View man page"
echo ""
echo "To uninstall: sudo scripts/uninstall-system.sh"
