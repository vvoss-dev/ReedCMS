# Config Commands

> Manage Reed.toml configuration

**Implementation:** REED-04-08  
**Status:** ✅ Complete  
**File:** `src/reedcms/cli/config_commands.rs`

---

## Overview

Config commands manage `Reed.toml` TOML configuration file, synchronising settings to CSV files for runtime use.

**Global patterns:** See [Common Patterns](common-patterns.md) for flags, output formats, error codes.

---

## Commands

### `reed config:init`

Create a new `Reed.toml` configuration file.

```bash
reed config:init [--file <path>] [--force]
```

**Flags:**
- `--file` - Path to create (default: `Reed.toml`)
- `--force` - Overwrite existing file

**Example:**
```bash
reed config:init                      # Create Reed.toml
reed config:init --file custom.toml   # Custom location
reed config:init --force              # Overwrite existing
```

**Generated file:** Complete Reed.toml template with all sections documented

**Performance:** < 50ms

---

### `reed config:sync`

Synchronise `Reed.toml` configuration to `.reed/*.csv` files.

**⚠️ WARNING:** Overwrites current CSV values with Reed.toml values!

```bash
reed config:sync [--file <path>] [--force]
```

**Flags:**
- `--file` - Path to Reed.toml (default: `Reed.toml`)
- `--force` - Skip confirmation prompt

**Example:**
```bash
reed config:export backup.json  # Backup first!
reed config:sync --force        # Sync to CSV
```

**Updates:** `.reed/project.csv`, `.reed/server.csv`

**Confirmation prompt:** Required unless `--force` used

**Performance:** < 200ms

---

### `reed config:show`

Display current `Reed.toml` configuration.

```bash
reed config:show [--file <path>] [--format <format>]
```

**Flags:**
- `--file` - Path to Reed.toml (default: `Reed.toml`)
- `--format` - Output: `toml` (default), `json`

**Examples:**
```bash
reed config:show              # TOML format
reed config:show --format json  # JSON format
```

**Output (TOML):**
```toml
[project]
name = "My Website"
url = "https://example.com"

[server]
workers = 8
timeout = 30
```

**Performance:** < 10ms

---

### `reed config:validate`

Validate `Reed.toml` syntax and values.

```bash
reed config:validate [--file <path>]
```

**Example:**
```bash
reed config:validate
```

**Checks:**
- TOML syntax validity
- Required fields present
- Value types correct
- Port ranges valid
- URL format valid

**Output:**
```
✓ Configuration valid
  - project.name: "My Website"
  - server.workers: 8
  - server.timeout: 30
```

**Performance:** < 50ms

---

### `reed config:export`

Export current CSV configuration to JSON.

```bash
reed config:export <output_file>
```

**Example:**
```bash
reed config:export backup.json
reed config:export config-$(date +%Y%m%d).json  # Dated backup
```

**Output format:**
```json
{
  "project": {
    "name": "My Website",
    "url": "https://example.com"
  },
  "server": {
    "workers": 8,
    "timeout": 30
  }
}
```

**Use case:** Backup before `config:sync`

**Performance:** < 100ms

---

## Reed.toml Format

### Project Section

```toml
[project]
name = "My Website"
url = "https://example.com"
default_language = "en"
available_languages = ["en", "de", "fr"]
timezone = "Europe/Berlin"
```

### Server Section

```toml
[server]
workers = 8              # Worker threads (default: CPU cores)
timeout = 30             # Request timeout in seconds
enable_cors = false      # CORS support
max_body_size = 10485760 # 10MB max request body
```

**Note:** Server binding (port/socket) controlled by `ENVIRONMENT` in `.env`, not Reed.toml.

### Environment-Specific

```toml
[dev]
debug = true
hot_reload = true

[prod]
debug = false
hot_reload = false
compression = true
```

**Complete template:** Created by `config:init`

---

## Configuration Workflow

### Initial Setup

```bash
# 1. Create configuration
reed config:init

# 2. Edit Reed.toml
vim Reed.toml

# 3. Validate
reed config:validate

# 4. Sync to CSV (runtime)
reed config:sync --force
```

### Update Configuration

```bash
# 1. Backup current state
reed config:export backup.json

# 2. Edit Reed.toml
vim Reed.toml

# 3. Validate changes
reed config:validate

# 4. Sync to CSV
reed config:sync --force
```

### Restore from Backup

```bash
# Manual restoration (CSV files are under .reed/)
# Restore from .reed/backups/*.csv.xz
```

---

## TOML vs CSV

**Reed.toml (TOML):**
- Human-readable configuration
- Version control friendly
- Documentation in comments
- Single source for initial setup

**.reed/*.csv:**
- Runtime data source
- Fast O(1) lookups
- Modified by CLI commands
- Automatic backups

**Workflow:**
1. Edit `Reed.toml` for structural changes
2. Run `reed config:sync` to update CSV
3. CSV is truth at runtime
4. CLI commands modify CSV directly

---

## Validation Rules

**Project Name:**
- 1-100 characters
- Any characters allowed

**URL:**
- Valid HTTP/HTTPS URL
- Must include protocol

**Worker Count:**
- 1-1024 threads
- Default: CPU core count

**Timeout:**
- 1-300 seconds
- Default: 30

**Max Body Size:**
- 1-104857600 bytes (100MB)
- Default: 10485760 (10MB)

**Validation:** See [Common Patterns → Common Validation Rules](common-patterns.md#common-validation-rules)

---

## Common Workflows

### Multi-Environment Setup

```bash
# Development Reed.toml
[dev]
debug = true
workers = 2

# Production Reed.toml
[prod]
debug = false
workers = 16
compression = true

# Sync based on environment
ENVIRONMENT=dev reed config:sync --force
ENVIRONMENT=prod reed config:sync --force
```

### Configuration Audit

```bash
# Show current config
reed config:show

# Validate
reed config:validate

# Export for review
reed config:export audit-$(date +%Y%m%d).json
```

---

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| `init` | < 50ms | File write |
| `sync` | < 200ms | Parses TOML, updates CSV |
| `show` | < 10ms | File read |
| `validate` | < 50ms | Full validation |
| `export` | < 100ms | JSON serialisation |

---

## Best Practices

**Version control Reed.toml:**
```bash
# ✅ Good - track configuration
git add Reed.toml
git commit -m "docs: update server configuration"
```

**Backup before sync:**
```bash
# ✅ Good - safety first
reed config:export backup.json
reed config:sync --force
```

**Validate after editing:**
```bash
# ✅ Good - catch errors early
vim Reed.toml
reed config:validate
reed config:sync --force
```

**Document changes:**
```toml
# ✅ Good - explain reasoning
[server]
# Increased for high-traffic period (2025-01-15)
workers = 16
```

---

**See also:**
- [Common Patterns](common-patterns.md) - Global flags, validation
- [Data Commands](data-commands.md) - Runtime CSV management
- [Server Commands](server-commands.md) - Server control
