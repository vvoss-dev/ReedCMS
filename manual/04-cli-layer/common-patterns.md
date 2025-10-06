# Common CLI Patterns

This reference documents patterns shared across all ReedCMS CLI commands to avoid repetition in command-specific documentation.

---

## Global Flags

Available on **all** `reed` commands:

| Flag | Short | Description |
|------|-------|-------------|
| `--help` | `-h` | Show command help |
| `--verbose` | `-v` | Detailed output with debug info |
| `--json` | `-j` | JSON output format |
| `--dry-run` | `-n` | Preview changes without executing |
| `--force` | `-f` | Skip confirmation prompts |

**Example:**
```bash
reed user:create alice --json --dry-run
```

---

## Output Formats

### Table Format (Default)

```
┌─────────┬──────────┬────────┐
│ Key     │ Value    │ Status │
├─────────┼──────────┼────────┤
│ foo.bar │ example  │ ✓      │
└─────────┴──────────┴────────┘
```

### JSON Format (`--json`)

```json
{
  "status": "success",
  "data": {
    "key": "foo.bar",
    "value": "example"
  },
  "cached": false,
  "timestamp": "2025-01-15T10:30:00Z"
}
```

### CSV Format (Import/Export)

Pipe-delimited: `key|value|description`

---

## Error Codes

| Code | Meaning | Common Cause |
|------|---------|--------------|
| `1` | Invalid input | Missing required field, wrong format |
| `2` | Not found | Key/user/role doesn't exist |
| `3` | Already exists | Duplicate key/username/role |
| `4` | Permission denied | RBAC check failed |
| `5` | I/O error | CSV file read/write failed |
| `6` | Validation error | Invalid format (email, permission syntax) |

---

## CSV File Locations

| Type | File | Format |
|------|------|--------|
| Text | `.reed/text.csv` | `key\|value\|description` |
| Routes | `.reed/routes.csv` | `route\|layout\|description` |
| Meta | `.reed/meta.csv` | `key\|value\|description` |
| Users | `.reed/users.csv` | `id\|username\|email\|hash\|created` |
| Roles | `.reed/roles.csv` | `role\|permissions\|description` |
| User-Roles | `.reed/user_roles.csv` | `user_id\|role\|assigned` |

---

## Key Naming Convention

**Format:** `lowercase.with.dots@lang`

**Rules:**
- Lowercase only
- Dots as separators (NOT underscores)
- Language suffix: `@de`, `@en` (lowercase)
- Namespace for components: `component.element.property`
- Maximum depth: 8 levels (optimal: 4)

**Examples:**
```
page-header.logo.title@de
navigation.main.home@en
footer.copyright.text@de
```

---

## Permission Syntax

**Format:** `resource[rwx]`

| Flag | Permission | Meaning |
|------|------------|---------|
| `r` | Read | View/list data |
| `w` | Write | Create/update data |
| `x` | Execute | Delete/administrative actions |

**Examples:**
```
text[r]         # Read text entries
text[rw]        # Read and write text
text[rwx]       # Full access to text
users[rx]       # Read users and delete (admin)
*[rwx]          # Full system access
```

---

## Common Validation Rules

### Username
- 3-32 characters
- Alphanumeric + underscores
- No spaces

### Email
- RFC 5322 compliant
- Must contain `@` and domain

### Password (create only, never shown)
- Hashed with Argon2id
- ~100ms verification time (intentional)
- Never logged or displayed

### Role Name
- 3-32 characters
- Lowercase, hyphens allowed
- Examples: `admin`, `editor`, `content-writer`

---

## Confirmation Prompts

Commands that modify data show confirmation unless `--force` is used:

```bash
$ reed text:delete page.title@de
⚠️  Delete key 'page.title@de'? [y/N]: y
✓ Deleted: page.title@de
```

Skip with:
```bash
reed text:delete page.title@de --force
```

---

## Dry Run Preview

Use `--dry-run` to preview changes:

```bash
$ reed text:set page.title@de "Beispiel" --dry-run
[DRY RUN] Would set:
  Key: page.title@de
  Value: Beispiel
  File: .reed/text.csv
```

---

## Verbose Output

Add `--verbose` for debugging:

```bash
$ reed text:get page.title@de --verbose
[DEBUG] Reading .reed/text.csv
[DEBUG] Cache: MISS
[DEBUG] Lookup: page.title@de
[DEBUG] Found at line 42
[DEBUG] Response time: 87μs

page.title@de = "Startseite"
```

---

## Standard Workflows

### Import CSV Data
```bash
reed text:import data.csv
reed route:import routes.csv
reed meta:import metadata.csv
```

### Export for Backup
```bash
reed text:export backup-text.csv
reed user:export backup-users.csv --json  # JSON format
```

### Bulk Operations
```bash
# List all, filter with grep, delete matches
reed text:list | grep "old-prefix" | xargs -I {} reed text:delete {} --force
```

### Verify Before Production
```bash
reed text:set key@prod "value" --dry-run  # Check first
reed text:set key@prod "value" --force    # Execute
```

---

**See command-specific documentation for unique flags and examples:**
- [Data Commands](data-commands.md) - text, route, meta
- [User Commands](user-commands.md) - user management
- [Role Commands](role-commands.md) - RBAC system
- [Layout Commands](layout-commands.md) - template management
- [Config Commands](config-commands.md) - Reed.toml
- [Server Commands](server-commands.md) - server control
