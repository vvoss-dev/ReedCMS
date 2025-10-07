# CLI Layer

> Command-line interface for all ReedCMS operations

## Status

**Completed Commands:**
- ✅ Data Commands (REED-04-02): `set:text`, `get:text`, `list:text`, `set:route`, `get:route`, `set:meta`, `get:meta`, `set:server`, `set:project`
- ✅ Layout Commands (REED-04-03): `init:layout`, `list:layouts`, `validate:layout`, `remove:layout`
- ✅ User Commands (REED-04-04): `create:user`, `list:users`, `get:user`, `update:user`, `delete:user`, `reset:password`
- ✅ Role Commands (REED-04-05): `create:role`, `list:roles`, `get:role`, `update:role`, `delete:role`, `assign:role`
- ✅ Config Commands (REED-04-12): `config:sync`, `config:export`, `config:show`, `config:validate`, `config:init`
- ✅ Server Commands (partial): `server:io`

**In Progress:**
- 🔄 Taxonomy Commands (REED-04-06)
- 🔄 Migration Commands (REED-04-07)
- 🔄 Build Commands (REED-04-08)
- 🔄 Other Server Commands (REED-04-09)
- 🔄 Agent Commands (REED-04-10)

## Overview

The CLI Layer provides the primary interface for interacting with ReedCMS. All operations use the `reed` command with a consistent `namespace:action` format.

## Command Structure

```
reed <namespace>:<action> [args] [flags]
```

**Components:**
- `namespace` - Command category (data, user, role, layout, config, server)
- `action` - Operation to perform (get, set, create, list, etc.)
- `args` - Positional arguments (required parameters)
- `flags` - Optional flags (start with `--`)

**Examples:**
```bash
reed data:get page.title@en
reed user:create admin --email admin@example.com
reed server:io --port 8333
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│ User Input: reed data:get page.title@en                │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│ CLI Parser (src/reedcms/cli/parser.rs)                  │
│  - Parse command structure                              │
│  - Extract namespace:action                             │
│  - Parse arguments and flags                            │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│ CLI Router (src/reedcms/cli/router.rs)                  │
│  - Route to appropriate command handler                 │
│  - Match "data:get" → data_commands::get_text()        │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│ Command Handler (src/reedcms/cli/data_commands.rs)     │
│  - Execute business logic                               │
│  - Call ReedBase services                               │
│  - Return ReedResult<ReedResponse<T>>                   │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│ Output Formatter                                        │
│  - Format response for terminal                         │
│  - Handle errors with user-friendly messages            │
│  - Print to stdout/stderr                               │
└─────────────────────────────────────────────────────────┘
```

## Quick Reference

| Command | Purpose | Status |
|---------|---------|--------|
| **Data Management** | | |
| `data:get` | Retrieve text/route/meta content | ✅ Complete |
| `data:set` | Store text/route/meta content | ✅ Complete |
| `data:list` | List matching keys | ✅ Complete |
| **Layout Management** | | |
| `layout:init` | Create new layout with scaffolding | ✅ Complete |
| `layout:list` | List all layouts | ✅ Complete |
| `layout:validate` | Validate layout structure | ✅ Complete |
| `layout:remove` | Remove layout | ✅ Complete |
| **User Management** | | |
| `user:create` | Create new user | ✅ Complete |
| `user:list` | List users | ✅ Complete |
| `user:get` | Get user details | ✅ Complete |
| `user:update` | Update user | ✅ Complete |
| `user:delete` | Delete user | ✅ Complete |
| `user:reset-password` | Reset password | ✅ Complete |
| **Role Management** | | |
| `role:create` | Create new role | ✅ Complete |
| `role:list` | List roles | ✅ Complete |
| `role:get` | Get role details | ✅ Complete |
| `role:update` | Update role permissions | ✅ Complete |
| `role:delete` | Delete role | ✅ Complete |
| `role:assign` | Assign role to user | ✅ Complete |
| **Configuration** | | |
| `config:sync` | Sync TOML → CSV | ✅ Complete |
| `config:export` | Export CSV → TOML | ✅ Complete |
| `config:show` | Show current config | ✅ Complete |
| `config:validate` | Validate TOML | ✅ Complete |
| `config:init` | Initialise new config | ✅ Complete |
| **Server** | | |
| `server:io` | Start HTTP/socket server | ✅ Complete |
| **Taxonomy** | | 🔄 In Progress |
| **Migration** | | 🔄 In Progress |
| **Build** | | 🔄 In Progress |

## Detailed Command Documentation

See individual command reference pages:

- [Data Commands](data-commands.md) - Text, route, meta operations ✅
- [Layout Commands](layout-commands.md) - Layout scaffolding ✅
- [User Commands](user-commands.md) - User management ✅
- [Role Commands](role-commands.md) - Role and permissions ✅
- [Config Commands](config-commands.md) - Configuration management ✅
- [Server Commands](server-commands.md) - Server operations (partial) ✅
- [Taxonomy Commands](taxonomy-commands.md) - Coming soon 🔄
- [Migration Commands](migration-commands.md) - Coming soon 🔄
- [Build Commands](build-commands.md) - Coming soon 🔄

## Global Flags

All commands support these flags:

| Flag | Purpose | Example |
|------|---------|---------|
| `--help`, `-h` | Show command help | `reed data:get --help` |
| `--verbose`, `-v` | Verbose output | `reed data:list -v` |
| `--json` | JSON output format | `reed user:list --json` |
| `--dry-run` | Show what would happen | `reed user:delete admin --dry-run` |
| `--force`, `-f` | Skip confirmations | `reed config:sync --force` |

## Output Formats

### Success Output

```bash
$ reed data:get page.title@en
Welcome
```

### Error Output

```bash
$ reed data:get nonexistent@en
Error: Not found: nonexistent@en
```

### Verbose Output

```bash
$ reed data:get page.title@en --verbose
Key: page.title@en
Value: Welcome
Source: ReedBase cache (text.csv)
Cached: true
Timestamp: 1704067200
Duration: 42μs
```

### JSON Output

```bash
$ reed data:get page.title@en --json
{
  "data": "Welcome",
  "source": "ReedBase cache",
  "cached": true,
  "timestamp": 1704067200,
  "metrics": {
    "duration_us": 42,
    "cache_hit": true
  }
}
```

## Common Patterns

### Check if Key Exists

```bash
if reed data:get page.title@en > /dev/null 2>&1; then
    echo "Key exists"
else
    echo "Key not found"
fi
```

### Batch Operations

```bash
# Set multiple keys
for lang in en de fr; do
    reed data:set page.title@$lang "Title in $lang"
done

# List all keys for a language
reed data:list "*@en"
```

### Scripting

```bash
#!/bin/bash
# Backup and update content

# Get current value
OLD_VALUE=$(reed data:get page.title@en)

# Update value
reed data:set page.title@en "New Title"

# Verify
NEW_VALUE=$(reed data:get page.title@en)
echo "Changed from '$OLD_VALUE' to '$NEW_VALUE'"
```

## Error Handling

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Invalid command |
| `3` | Not found |
| `4` | Validation error |
| `5` | Permission denied |

### Error Messages

All errors are human-readable:

```bash
$ reed data:set invalid.key@toolong "value"
Error: Validation error: Language code must be 2 characters (got: toolong)

$ reed user:create admin
Error: Missing required flag: --email

$ reed config:sync
Error: This command requires --force flag (destructive operation)
```

## Best Practices

### 1. Use Descriptive Keys

```bash
# ✅ Good
reed data:set page.home.hero.title@en "Welcome"

# ❌ Bad
reed data:set t1@en "Welcome"
```

### 2. Always Add Descriptions

```bash
# ✅ Good
reed data:set page.title@en "Welcome" --desc "Homepage title"

# ❌ Bad (no context for other developers)
reed data:set page.title@en "Welcome"
```

### 3. Use Dry-Run for Destructive Operations

```bash
# Check what would be deleted
reed user:delete admin --dry-run

# Then actually delete
reed user:delete admin --force
```

### 4. Validate Before Deployment

```bash
# Validate layout structure
reed layout:validate knowledge

# Validate all routes
reed validate:routes
```

## Performance

| Operation | Typical Time |
|-----------|--------------|
| `data:get` (cached) | < 1ms |
| `data:set` | < 50ms (includes CSV write) |
| `data:list` | < 10ms for 1000 keys |
| `user:create` | < 100ms (includes Argon2) |
| `layout:init` | < 500ms (creates files) |
| `config:sync` | < 200ms (parses TOML + writes CSV) |

## File Reference

```
src/reedcms/cli/
├── mod.rs                     # CLI module definition
├── parser.rs                  # Command parsing
├── router.rs                  # Command routing
├── data_commands.rs           # Data operations ✅
├── layout_commands.rs         # Layout operations ✅
├── user_commands.rs           # User operations ✅
├── role_commands.rs           # Role operations ✅
├── config_commands.rs         # Config operations ✅
├── taxonomy_commands.rs       # Taxonomy operations 🔄
├── migration_commands.rs      # Migration operations 🔄
├── build_commands.rs          # Build operations 🔄
└── server_commands.rs         # Server operations (partial) ✅
```

## Next Steps

- [Data Commands](data-commands.md) - Complete reference for all data operations
- [User Commands](user-commands.md) - User management details
- [Config Commands](config-commands.md) - Configuration workflow

## Summary

The CLI Layer provides:
- ✅ Consistent `namespace:action` command structure
- ✅ 30+ commands (data, layout, user, role, config)
- ✅ Global flags (--help, --verbose, --json, --dry-run, --force)
- ✅ Multiple output formats (text, verbose, JSON)
- ✅ Script-friendly error codes
- ✅ Human-readable error messages
- ✅ High performance (< 1ms for cached reads)

Commands are production-ready and fully tested where marked ✅ Complete.
