# REED-04-13: System Setup Script

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Testing**: Test scripts must validate all operations
- **Avoid**: Swiss Army knife functions
- **Avoid**: Generic file names like `handler.sh`, `utils.sh`, `setup.sh`

## Ticket Information
- **ID**: REED-04-13
- **Title**: System Setup Script
- **Layer**: CLI Layer (REED-04)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-04-01 (CLI Foundation), REED-04-11 (Man Pages)

## Objective
Create a system setup script that enables:
1. **Binary Installation**: Install `reed` binary to system PATH (`/usr/local/bin/reed`)
2. **Man Page Installation**: Install man pages to enable `man reed` system-wide
3. **Development Setup**: Configure development environment with proper symlinks
4. **Uninstall Support**: Clean removal of all installed components

## Rationale

### Current Limitations
- Users must invoke `./target/release/reed` with full path
- Man pages exist but aren't accessible via `man reed`
- No standardised installation procedure
- Manual setup required for each development machine

### Professional Tool Standards
Professional CLI tools provide installation scripts:
- **cargo**: Installs via `rustup` to `~/.cargo/bin`
- **docker**: System-wide installation with man pages
- **git**: Complete system integration
- **npm**: Global installation with executable linking

### User Experience Goals
```bash
# After running setup script:
reed data:get knowledge.title@en   # Works from any directory
man reed                            # Displays man page
reed --version                      # Shows version
which reed                          # Shows /usr/local/bin/reed
```

## Implementation Strategy

### Installation Modes

**1. Development Mode** (`scripts/setup-dev.sh`):
- Symlinks binary from `target/release/reed` → `/usr/local/bin/reed`
- Symlinks man pages from `src/man/*.1` → `/usr/local/share/man/man1/`
- No file copying, uses symlinks for live development
- Updates automatically when `cargo build --release` runs

**2. System Installation** (`scripts/install-system.sh`):
- Copies binary to `/usr/local/bin/reed`
- Copies man pages from `src/man/*.1` to `/usr/local/share/man/man1/`
- Sets proper permissions (755 for binary, 644 for man pages)
- Updates man database with `mandb`
- Production-ready installation

**3. User Installation** (`scripts/install-user.sh`):
- Installs to user directories (no sudo required)
- Binary → `~/.local/bin/reed`
- Man pages from `src/man/*.1` → `~/.local/share/man/man1/`
- Updates user's shell PATH if needed

### Directory Structure
```
scripts/
├── setup-dev.sh              # Development mode setup (symlinks)
├── install-system.sh         # System-wide installation (requires sudo)
├── install-user.sh           # User-local installation (no sudo)
├── uninstall-system.sh       # Remove system installation
├── uninstall-user.sh         # Remove user installation
├── build-man-pages.sh        # Build man pages from .ronn sources
└── README.md                 # Installation documentation
```

## Implementation Files

### Development Setup Script (`scripts/setup-dev.sh`)

```bash
#!/usr/bin/env bash
# Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
# SPDX-License-Identifier: Apache-2.0

# ReedCMS Development Setup
# Creates symlinks for binary and man pages to enable system-wide access during development

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

echo "ReedCMS Development Setup"
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

# Install binary symlink
echo "Installing binary symlink..."
if [ -L /usr/local/bin/reed ]; then
    echo -e "${YELLOW}Removing existing symlink${NC}"
    sudo rm /usr/local/bin/reed
elif [ -f /usr/local/bin/reed ]; then
    echo -e "${RED}Error: /usr/local/bin/reed exists and is not a symlink${NC}"
    echo "Run 'scripts/uninstall-system.sh' first or manually remove the file"
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

# Update man database
echo ""
echo "Updating man database..."
if command -v mandb &> /dev/null; then
    sudo mandb -q 2>/dev/null || true
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
echo -e "${GREEN}Development setup complete!${NC}"
echo ""
echo "Usage:"
echo "  reed data:get knowledge.title@en    # Run from anywhere"
echo "  man reed                             # View man page"
echo ""
echo "Note: Symlinks will update automatically when you run 'cargo build --release'"
