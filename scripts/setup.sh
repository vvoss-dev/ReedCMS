#!/usr/bin/env bash
# Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
# SPDX-License-Identifier: Apache-2.0

# ReedCMS Setup Script
# Installs reed binary and man pages based on ENVIRONMENT setting in .env

set -euo pipefail

# Colours for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Colour

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BINARY_PATH="${PROJECT_ROOT}/target/release/reed"
MAN_DIR="${PROJECT_ROOT}/src/man"
ENV_FILE="${PROJECT_ROOT}/.env"

# Read ENVIRONMENT from .env
ENVIRONMENT="dev"
if [ -f "${ENV_FILE}" ]; then
    # shellcheck disable=SC1090
    source "${ENV_FILE}"
fi

echo -e "${BLUE}ReedCMS Setup${NC}"
echo "============="
echo ""
echo "Environment: ${ENVIRONMENT}"
echo ""

# Check if binary exists
if [ ! -f "${BINARY_PATH}" ]; then
    echo -e "${RED}Error: Binary not found at ${BINARY_PATH}${NC}"
    echo "Run 'cargo build --release' first"
    exit 1
fi

# Build man pages if needed
if [ ! -d "${MAN_DIR}" ] || [ -z "$(ls -A ${MAN_DIR}/*.1 2>/dev/null)" ]; then
    echo -e "${YELLOW}Building man pages...${NC}"
    "${SCRIPT_DIR}/build-man-pages.sh"
    echo ""
fi

# Determine installation mode based on ENVIRONMENT
case "${ENVIRONMENT}" in
    dev)
        echo -e "${BLUE}Development Mode: Creating symlinks${NC}"
        echo "Binary and man pages will auto-update when you rebuild"
        echo ""

        # Install binary symlink
        echo "Installing binary symlink..."
        if [ -L /usr/local/bin/reed ]; then
            echo -e "${YELLOW}Removing existing symlink${NC}"
            sudo rm /usr/local/bin/reed
        elif [ -f /usr/local/bin/reed ]; then
            echo -e "${RED}Error: /usr/local/bin/reed exists and is not a symlink${NC}"
            echo "Run 'scripts/uninstall.sh' first or manually remove the file"
            exit 1
        fi

        sudo ln -s "${BINARY_PATH}" /usr/local/bin/reed
        echo -e "${GREEN}✓ Binary symlinked: /usr/local/bin/reed → ${BINARY_PATH}${NC}"

        # Install man page symlinks
        echo ""
        echo "Installing man page symlinks..."
        sudo mkdir -p /usr/local/share/man/man1

        for man_file in "${MAN_DIR}"/*.1; do
            if [ -f "${man_file}" ]; then
                man_name=$(basename "${man_file}")
                target_path="/usr/local/share/man/man1/${man_name}"

                if [ -L "${target_path}" ]; then
                    sudo rm "${target_path}"
                elif [ -f "${target_path}" ]; then
                    echo -e "${YELLOW}Warning: ${target_path} exists and is not a symlink, skipping${NC}"
                    continue
                fi

                sudo ln -s "${man_file}" "${target_path}"
                echo -e "${GREEN}✓ Man page symlinked: ${man_name}${NC}"
            fi
        done
        ;;

    prod)
        echo -e "${BLUE}Production Mode: Installing files${NC}"
        echo "Binary and man pages will be copied to system directories"
        echo ""

        # Check for sudo
        if [ "$EUID" -ne 0 ]; then
            echo -e "${RED}Error: Production installation requires root privileges${NC}"
            echo "Run with: sudo ./scripts/setup.sh"
            exit 1
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
        ;;

    *)
        echo -e "${RED}Error: Invalid ENVIRONMENT='${ENVIRONMENT}' in .env${NC}"
        echo "Valid values: dev | prod"
        exit 1
        ;;
esac

# Update man database
echo ""
echo "Updating man database..."
if command -v mandb &> /dev/null; then
    sudo mandb -q 2>/dev/null || true
    echo -e "${GREEN}✓ Man database updated${NC}"
else
    echo -e "${YELLOW}Warning: mandb not found, man pages may not be indexed${NC}"
fi

# Configure /etc/hosts for development domain (dev mode only)
if [ "${ENVIRONMENT}" = "dev" ]; then
    echo ""
    echo "Configuring development domain..."

    # Read domain from .reed/server.csv
    DEV_DOMAIN=$(grep '^server\.dev\.domain|' "${PROJECT_ROOT}/.reed/server.csv" | cut -d'|' -f2)

    if [ -z "${DEV_DOMAIN}" ]; then
        echo -e "${YELLOW}⚠ No dev domain found in .reed/server.csv, skipping /etc/hosts configuration${NC}"
    else
        HOSTS_FILE="/etc/hosts"
        MARKER_COMMENT="# ReedCMS Development Domain - AUTO-MANAGED by setup.sh"
        HOSTS_ENTRY="127.0.0.1\t${DEV_DOMAIN}"

        # Check if our marker exists
        if grep -q "^${MARKER_COMMENT}" "${HOSTS_FILE}" 2>/dev/null; then
            # Update existing entry
            CURRENT_DOMAIN=$(sudo grep -A 1 "^${MARKER_COMMENT}" "${HOSTS_FILE}" | tail -1 | awk '{print $2}')

            if [ "${CURRENT_DOMAIN}" != "${DEV_DOMAIN}" ]; then
                echo -e "${YELLOW}Updating /etc/hosts: ${CURRENT_DOMAIN} → ${DEV_DOMAIN}${NC}"
                # Remove old entry (marker + next line)
                sudo sed -i '' "/^${MARKER_COMMENT}/,+1d" "${HOSTS_FILE}"
                # Add new entry
                echo -e "${MARKER_COMMENT}\n${HOSTS_ENTRY}" | sudo tee -a "${HOSTS_FILE}" > /dev/null
                echo -e "${GREEN}✓ Development domain updated: ${DEV_DOMAIN}${NC}"
            else
                echo -e "${GREEN}✓ Development domain already configured: ${DEV_DOMAIN}${NC}"
            fi
        else
            # Add new entry
            echo "Adding ${DEV_DOMAIN} to /etc/hosts..."
            echo -e "\n${MARKER_COMMENT}\n${HOSTS_ENTRY}" | sudo tee -a "${HOSTS_FILE}" > /dev/null
            echo -e "${GREEN}✓ Development domain configured: ${DEV_DOMAIN}${NC}"
        fi

        # Verify DNS resolution
        if ping -c 1 "${DEV_DOMAIN}" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ Domain resolves correctly: ${DEV_DOMAIN}${NC}"
        else
            echo -e "${YELLOW}⚠ Domain may not resolve yet, try flushing DNS cache${NC}"
        fi
    fi
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
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo "Usage:"
echo "  reed data:get knowledge.title@en    # Run from anywhere"
echo "  man reed                             # View man page"
echo ""
if [ "${ENVIRONMENT}" = "dev" ]; then
    echo "Development mode: Symlinks update automatically when you rebuild"
fi
echo "To uninstall: ./scripts/uninstall.sh"
