#!/usr/bin/env bash
# Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
# SPDX-License-Identifier: Apache-2.0

# ReedCMS User Installation
# Installs binary and man pages to user directories (no sudo required)

set -euo pipefail

# Colours for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Colour

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BINARY_PATH="${PROJECT_ROOT}/target/release/reed"
MAN_DIR="${PROJECT_ROOT}/src/man"

USER_BIN_DIR="${HOME}/.local/bin"
USER_MAN_DIR="${HOME}/.local/share/man/man1"

echo "ReedCMS User Installation"
echo "========================="
echo ""

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

# Create user directories
echo "Creating user directories..."
mkdir -p "${USER_BIN_DIR}"
mkdir -p "${USER_MAN_DIR}"

# Install binary
echo "Installing binary..."
if [ -f "${USER_BIN_DIR}/reed" ]; then
    echo -e "${YELLOW}Removing existing installation${NC}"
    rm "${USER_BIN_DIR}/reed"
fi

install -m 755 "${BINARY_PATH}" "${USER_BIN_DIR}/reed"
echo -e "${GREEN}✓ Binary installed: ${USER_BIN_DIR}/reed${NC}"

# Install man pages
echo ""
echo "Installing man pages..."

for man_file in "${MAN_DIR}"/*.1; do
    if [ -f "${man_file}" ]; then
        man_name=$(basename "${man_file}")
        install -m 644 "${man_file}" "${USER_MAN_DIR}/${man_name}"
        echo -e "${GREEN}✓ Man page installed: ${man_name}${NC}"
    fi
done

# Check if ~/.local/bin is in PATH
echo ""
if [[ ":$PATH:" != *":${USER_BIN_DIR}:"* ]]; then
    echo -e "${YELLOW}Warning: ${USER_BIN_DIR} is not in your PATH${NC}"
    echo ""
    echo "Add this line to your shell configuration file:"
    echo ""

    if [ -n "${BASH_VERSION:-}" ]; then
        echo "  # For Bash (~/.bashrc or ~/.bash_profile):"
        echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
    elif [ -n "${ZSH_VERSION:-}" ]; then
        echo "  # For Zsh (~/.zshrc):"
        echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
    else
        echo "  # For your shell configuration file:"
        echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
    fi

    echo ""
    echo "Then reload your shell or run: source ~/.bashrc"
    echo ""
fi

# Check if user man directory is in MANPATH
if ! man -w &> /dev/null || ! man -w 2>&1 | grep -q "${USER_MAN_DIR}"; then
    echo -e "${YELLOW}Note: ${USER_MAN_DIR} may not be in your MANPATH${NC}"
    echo ""
    echo "If 'man reed' doesn't work, add this line to your shell configuration:"
    echo "  export MANPATH=\"\${HOME}/.local/share/man:\${MANPATH}\""
    echo ""
fi

# Verification
echo "Verification:"
echo "============="
if command -v reed &> /dev/null; then
    echo -e "${GREEN}✓ 'reed' command is available${NC}"
    reed --version 2>/dev/null || echo -e "${YELLOW}  (version command not implemented yet)${NC}"
else
    echo -e "${YELLOW}⚠ 'reed' command not found in PATH (see PATH instructions above)${NC}"
fi

if man -w reed &> /dev/null 2>&1; then
    echo -e "${GREEN}✓ 'man reed' is available${NC}"
else
    echo -e "${YELLOW}⚠ 'man reed' not found (may need to configure MANPATH or restart shell)${NC}"
fi

echo ""
echo -e "${GREEN}User installation complete!${NC}"
echo ""
echo "Installed to:"
echo "  Binary: ${USER_BIN_DIR}/reed"
echo "  Man pages: ${USER_MAN_DIR}/"
echo ""
echo "To uninstall: scripts/uninstall-user.sh"
