# ReedCMS Installation Scripts

This directory contains installation and setup scripts for ReedCMS CLI integration.

## Available Scripts

### Development Setup

**`setup-dev.sh`** - Development mode (symlinks, auto-updates)
```bash
# Creates symlinks to target/release/reed and man pages
# Symlinks update automatically when you rebuild
# Requires: sudo (for /usr/local/bin access)

./scripts/setup-dev.sh
```

**Use when:**
- Actively developing ReedCMS
- Testing changes frequently
- Want automatic updates after rebuilds

**Locations:**
- Binary: `/usr/local/bin/reed` → `target/release/reed` (symlink)
- Man pages: `/usr/local/share/man/man1/reed*.1` → `man/*.1` (symlinks)

---

### System-Wide Installation

**`install-system.sh`** - Production installation (copies files)
```bash
# Copies binary and man pages to system directories
# Requires: sudo

sudo ./scripts/install-system.sh
```

**Use when:**
- Installing ReedCMS for all users
- Production server deployment
- Stable installation needed

**Locations:**
- Binary: `/usr/local/bin/reed` (file copy)
- Man pages: `/usr/local/share/man/man1/reed*.1` (file copies)

**Uninstall:**
```bash
sudo ./scripts/uninstall-system.sh
```

---

### User-Local Installation

**`install-user.sh`** - User-only installation (no sudo required)
```bash
# Installs to user's home directory
# No root privileges needed

./scripts/install-user.sh
```

**Use when:**
- No sudo access available
- Personal installation only
- Don't want to affect other users

**Locations:**
- Binary: `~/.local/bin/reed`
- Man pages: `~/.local/share/man/man1/reed*.1`

**Requirements:**
- `~/.local/bin` must be in your `PATH`
- `~/.local/share/man` should be in your `MANPATH`

**Shell configuration (if needed):**
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="${HOME}/.local/bin:${PATH}"
export MANPATH="${HOME}/.local/share/man:${MANPATH}"
```

**Uninstall:**
```bash
./scripts/uninstall-user.sh
```

---

## Prerequisites

### For All Scripts
1. **Build ReedCMS binary:**
   ```bash
   cargo build --release
   ```

2. **Man pages exist:**
   ```bash
   # Man pages should be in man/ directory
   ls man/*.1
   
   # If missing, build them:
   ./scripts/build-man-pages.sh
   ```

### For Man Page Building
Install `ronn-ng` (Ruby gem):
```bash
gem install ronn-ng
```

---

## Installation Comparison

| Feature | setup-dev.sh | install-system.sh | install-user.sh |
|---------|-------------|-------------------|-----------------|
| **Requires sudo** | Yes | Yes | No |
| **File type** | Symlinks | Copies | Copies |
| **Auto-updates** | Yes | No | No |
| **All users** | Yes | Yes | No |
| **Best for** | Development | Production | Limited access |

---

## Verification

After installation, verify with:

```bash
# Check binary location
which reed

# Test command
reed --version

# View man page
man reed

# Check man page location
man -w reed
```

---

## Manual Installation

If scripts don't work for your system, install manually:

### Binary
```bash
# System-wide (requires sudo)
sudo install -m 755 target/release/reed /usr/local/bin/reed

# User-local (no sudo)
install -m 755 target/release/reed ~/.local/bin/reed
```

### Man Pages
```bash
# System-wide (requires sudo)
sudo install -m 644 man/reed.1 /usr/local/share/man/man1/reed.1
sudo mandb

# User-local (no sudo)
install -m 644 man/reed.1 ~/.local/share/man/man1/reed.1
```

---

## Troubleshooting

### "reed: command not found"

**For system installation:**
- Ensure `/usr/local/bin` is in your PATH
- Check: `echo $PATH | grep /usr/local/bin`
- Add if missing: `export PATH="/usr/local/bin:$PATH"`

**For user installation:**
- Ensure `~/.local/bin` is in your PATH
- Add to shell config: `export PATH="${HOME}/.local/bin:${PATH}"`
- Reload shell: `source ~/.bashrc` or `source ~/.zshrc`

### "man reed" not working

**Check if man pages are installed:**
```bash
# System installation
ls /usr/local/share/man/man1/reed*.1

# User installation
ls ~/.local/share/man/man1/reed*.1
```

**Update man database:**
```bash
# System-wide
sudo mandb

# For user installation, ensure MANPATH is set
export MANPATH="${HOME}/.local/share/man:${MANPATH}"
```

### "Permission denied" errors

**For system scripts:**
- Use `sudo`: `sudo ./scripts/install-system.sh`

**For user scripts:**
- Don't use sudo: `./scripts/install-user.sh`
- Ensure `~/.local/bin` and `~/.local/share/man/man1` are writable

### Symlinks not updating (dev setup)

**Rebuild binary:**
```bash
cargo build --release
```

**Verify symlink:**
```bash
ls -la /usr/local/bin/reed
# Should show: /usr/local/bin/reed -> /path/to/ReedCMS/target/release/reed
```

---

## Script Permissions

Make scripts executable:
```bash
chmod +x scripts/*.sh
```

---

## Platform Support

**Tested on:**
- macOS (Darwin)
- Linux (Debian/Ubuntu)
- Linux (Fedora/RHEL)

**Not tested:**
- Windows (use WSL)
- BSD variants

---

## Development Workflow

**Recommended setup for ReedCMS development:**

1. **Initial setup:**
   ```bash
   cargo build --release
   ./scripts/setup-dev.sh
   ```

2. **Make changes to code**

3. **Rebuild:**
   ```bash
   cargo build --release
   # Binary automatically updates via symlink
   ```

4. **Test immediately:**
   ```bash
   reed data:get test.key@en
   ```

**No reinstallation needed!** Symlinks ensure `reed` command always uses latest build.

---

## CI/CD Integration

**In CI/CD pipelines:**

```yaml
# GitHub Actions example
- name: Build ReedCMS
  run: cargo build --release

- name: Install for testing
  run: sudo ./scripts/install-system.sh

- name: Run integration tests
  run: |
    reed --version
    man reed
```

---

## License

Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
SPDX-License-Identifier: Apache-2.0
