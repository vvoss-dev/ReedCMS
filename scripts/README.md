# ReedCMS Installation Scripts

This directory contains installation scripts for ReedCMS CLI integration.

## Quick Start

**The installation mode is controlled by the `ENVIRONMENT` variable in `.env`:**

```bash
# 1. Build the binary
cargo build --release

# 2. Check your .env file
cat .env
# Should show: ENVIRONMENT=dev (or prod)

# 3. Run setup
./scripts/setup.sh
```

---

## Installation Modes

### Development Mode (ENVIRONMENT=dev)

**Uses symlinks** - automatically updates when you rebuild

```bash
# In .env:
ENVIRONMENT=dev

# Run setup:
./scripts/setup.sh

# Now rebuilding updates the command automatically:
cargo build --release  # reed command uses new binary immediately
```

**What it does:**
- Creates symlink: `/usr/local/bin/reed` → `target/release/reed`
- Creates symlinks: `/usr/local/share/man/man1/*.1` → `src/man/*.1`
- Requires: sudo (for `/usr/local/bin` access)
- Auto-updates: Yes - symlinks always point to latest build

**Best for:**
- Active development
- Testing changes frequently
- Local machine setup

---

### Production Mode (ENVIRONMENT=prod)

**Copies files** - stable installation

```bash
# In .env:
ENVIRONMENT=prod

# Run setup with sudo:
sudo ./scripts/setup.sh
```

**What it does:**
- Copies binary to `/usr/local/bin/reed` (755 permissions)
- Copies man pages to `/usr/local/share/man/man1/` (644 permissions)
- Updates man database with `mandb`
- Requires: sudo
- Auto-updates: No - files are copies

**Best for:**
- Production deployment
- Server installation
- Stable system-wide installation

---

##Scripts

| Script | Purpose |
|--------|---------|
| `setup.sh` | Install reed binary and man pages (reads ENVIRONMENT from .env) |
| `uninstall.sh` | Remove all installed files (binary + man pages) |
| `build-man-pages.sh` | Compile `.ronn` sources to `.1` man pages |

---

## Usage After Installation

```bash
# Run from anywhere
reed data:get knowledge.title@en

# View man page
man reed

# Check installation
which reed              # Shows: /usr/local/bin/reed
man -w reed             # Shows: /usr/local/share/man/man1/reed.1
```

---

## Uninstall

```bash
sudo ./scripts/uninstall.sh
```

Removes:
- `/usr/local/bin/reed` (file or symlink)
- `/usr/local/share/man/man1/reed*.1` (all reed man pages)
- Updates man database

---

## Troubleshooting

### "reed: command not found"

**Check installation:**
```bash
ls -la /usr/local/bin/reed
```

**If symlink is broken (dev mode):**
```bash
# Rebuild binary
cargo build --release

# Or re-run setup
./scripts/setup.sh
```

###"man reed" not working

**Update man database:**
```bash
sudo mandb
```

**Check man page exists:**
```bash
ls -la /usr/local/share/man/man1/reed.1
```

### Permission errors

**setup.sh requires sudo:**
```bash
# Both dev and prod modes need sudo for /usr/local/bin access
./scripts/setup.sh  # Will prompt for sudo when needed
```

### Switching between dev and prod

```bash
# 1. Uninstall current
sudo ./scripts/uninstall.sh

# 2. Change .env
# Edit .env: ENVIRONMENT=dev (or prod)

# 3. Reinstall
./scripts/setup.sh
```

---

## Development Workflow

**Recommended for ReedCMS development:**

```bash
# One-time setup
cargo build --release
./scripts/setup.sh      # With ENVIRONMENT=dev in .env

# Daily work
# 1. Make code changes
# 2. Rebuild
cargo build --release

# 3. Test immediately (symlink auto-updates!)
reed data:get test.key@en
```

**No reinstallation needed!** Dev mode symlinks ensure `reed` command always uses latest build.

---

## CI/CD Integration

```yaml
# GitHub Actions example
- name: Build ReedCMS
  run: cargo build --release

- name: Set production mode
  run: echo "ENVIRONMENT=prod" > .env

- name: Install for testing
  run: sudo ./scripts/setup.sh

- name: Run integration tests
  run: |
    reed --version
    man reed
```

---

## Manual Installation

If scripts don't work, install manually:

```bash
# Binary
sudo cp target/release/reed /usr/local/bin/reed
sudo chmod 755 /usr/local/bin/reed

# Man page
sudo cp src/man/reed.1 /usr/local/share/man/man1/reed.1
sudo chmod 644 /usr/local/share/man/man1/reed.1
sudo mandb
```

---

## License

Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
SPDX-License-Identifier: Apache-2.0
