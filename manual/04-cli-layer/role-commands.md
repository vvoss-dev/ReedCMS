# Role Commands

> Role-based access control and permission management

**Implementation:** REED-04-05  
**Status:** ✅ Complete  
**File:** `src/reedcms/cli/role_commands.rs`

---

## Overview

Role commands provide RBAC with fine-grained permissions using format `resource[rwx]` (r=read, w=write, x=execute).

**Global patterns:** See [Common Patterns](common-patterns.md) for flags, output formats, error codes. See [Common Patterns → Permission Syntax](common-patterns.md#permission-syntax) for permission details.

---

## Commands

### `reed role:create`

Create a new role with permissions.

```bash
reed role:create <rolename> --permissions <permissions> [--desc <description>]
```

**Required:**
- `rolename` - 3-32 chars, alphanumeric + underscore
- `--permissions` - Comma-separated list: `resource[rwx]`

**Examples:**
```bash
# Editor role
reed role:create editor \
    --permissions "text[rwx],content[rwx],route[rw-]" \
    --desc "Content editor"

# Viewer (read-only)
reed role:create viewer \
    --permissions "text[r--],content[r--]" \
    --desc "Read-only"

# Admin (full access)
reed role:create admin \
    --permissions "*[rwx]" \
    --desc "Full administrator"
```

**Available Resources:**
`text`, `content`, `route`, `meta`, `user`, `role`, `template`, `asset`, `system`, `*` (all)

**Permission format:** See [Common Patterns → Permission Syntax](common-patterns.md#permission-syntax)  
**File:** `.reed/roles.matrix.csv`  
**Performance:** < 50ms

---

### `reed role:list`

List all roles.

```bash
reed role:list [pattern] [--format <format>]
```

**Flags:**
- `--format` - Output: `table` (default), `json`, `csv`

**Examples:**
```bash
reed role:list                # Table format
reed role:list "admin*"       # Filter
reed role:list --format json  # JSON output
```

**Output (table):**
```
Role      Permissions                         Description
admin     *[rwx]                              Full administrator
editor    text[rwx],content[rwx],route[rw-]  Content editor
viewer    text[r--],content[r--]             Read-only
```

**Performance:** < 10ms

---

### `reed role:get`

Get detailed role information.

```bash
reed role:get <rolename> [--format <format>]
```

**Flags:**
- `--format` - Output: `text` (default), `json`

**Example:**
```bash
reed role:get editor
```

**Output:**
```
Role:         editor
Description:  Content editor
Permissions:
  - text[rwx]     (read, write, execute)
  - content[rwx]  (read, write, execute)
  - route[rw-]    (read, write)
Users:        3 (jdoe, alice, bob)
```

**Performance:** < 5ms

---

### `reed role:update`

Update role permissions or description.

```bash
reed role:update <rolename> [--permissions <permissions>] [--desc <description>]
```

**Examples:**
```bash
# Update permissions
reed role:update editor --permissions "text[rwx],content[rwx],route[rwx]"

# Update description only
reed role:update editor --desc "Senior content editor"

# Update both
reed role:update editor \
    --permissions "text[rwx],content[rwx]" \
    --desc "Content writer"
```

**Performance:** < 50ms

---

### `reed role:delete`

Delete a role.

```bash
reed role:delete <rolename> [--force]
```

**Flags:** See [Common Patterns → Confirmation Prompts](common-patterns.md#confirmation-prompts)

**Example:**
```bash
reed role:delete old-role        # With confirmation
reed role:delete old-role --force  # Skip prompt
```

**Safety:** Users with this role will lose it (roles list updated automatically)  
**Performance:** < 50ms

---

### `reed role:assign`

Assign role to user.

```bash
reed role:assign <username> <rolename>
```

**Example:**
```bash
reed role:assign jdoe editor
reed role:assign alice admin
```

**Notes:**
- User can have multiple roles (stored as comma-separated list)
- Role must exist before assignment
- User must exist before assignment

**Files Modified:**
- `.reed/user_roles.csv` - Assignment record
- `.reed/users.matrix.csv` - User's roles column updated

**Performance:** < 50ms

---

## CSV File Formats

### roles.matrix.csv

**Structure:**
```
rolename|permissions|description|created_at|updated_at
```

**Example:**
```
editor|text[rwx],content[rwx],route[rw-]|Content editor|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
admin|*[rwx]|Full administrator|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
```

**Type:** Matrix CSV (Type 2 - Lists)  
**Permissions:** Comma-separated list

### user_roles.csv

**Structure:**
```
user_id|role|assigned_at
```

**Example:**
```
jdoe|editor|2025-01-01T00:00:00Z
jdoe|viewer|2025-01-02T00:00:00Z
alice|admin|2025-01-01T00:00:00Z
```

**Complete format:** See [Common Patterns → CSV File Locations](common-patterns.md#csv-file-locations)

---

## Permission Model

### Resource Types

| Resource | Read (r) | Write (w) | Execute (x) |
|----------|----------|-----------|-------------|
| `text` | View text | Create/edit | Delete text |
| `content` | View content | Create/edit | Delete content |
| `route` | View routes | Create/edit | Delete routes |
| `user` | List users | Create/edit | Delete users |
| `role` | List roles | Create/edit | Delete roles |
| `system` | View config | Edit config | System admin |
| `*` | All read | All write | All execute |

### Permission Patterns

**Read-only:**
```
text[r--],content[r--],route[r--]
```

**Read + Write (no delete):**
```
text[rw-],content[rw-]
```

**Full access to specific resources:**
```
text[rwx],content[rwx]
```

**Full system access:**
```
*[rwx]
```

**Permission syntax:** See [Common Patterns → Permission Syntax](common-patterns.md#permission-syntax)

---

## Common Workflows

### Initial Role Setup

```bash
# Create basic roles
reed role:create admin --permissions "*[rwx]" --desc "Full admin"
reed role:create editor --permissions "text[rwx],content[rwx]" --desc "Content editor"
reed role:create viewer --permissions "text[r--],content[r--]" --desc "Read-only"

# Assign to first admin
reed role:assign admin admin
```

### Multi-Role User

```bash
# Create user
reed user:create jdoe --email john@example.com --password secure123

# Assign multiple roles
reed role:assign jdoe editor
reed role:assign jdoe viewer

# Verify
reed user:get jdoe
# Roles: editor, viewer
```

### Permission Audit

```bash
# List all roles
reed role:list

# Check specific role
reed role:get editor

# Find users with role
reed user:list --format json | jq '.[] | select(.roles | contains("editor"))'
```

### Update Permissions

```bash
# Grant delete permission to editors
reed role:update editor --permissions "text[rwx],content[rwx],route[rwx]"

# Restrict to read-write only
reed role:update editor --permissions "text[rw-],content[rw-]"
```

---

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| `create` | < 50ms | CSV write |
| `list` | < 10ms | All roles |
| `get` | < 5ms | Cached |
| `update` | < 50ms | CSV write |
| `delete` | < 50ms | CSV write |
| `assign` | < 50ms | Updates 2 files |

---

## Best Practices

**Descriptive roles:**
```bash
# ✅ Good - clear purpose
reed role:create content-writer --permissions "text[rw-],content[rw-]" --desc "Can write but not delete"
```

**Principle of least privilege:**
```bash
# ✅ Good - only necessary permissions
reed role:create blog-author --permissions "content[rw-],text[r--]"

# ❌ Bad - excessive permissions
reed role:create blog-author --permissions "*[rwx]"
```

**Regular permission audits:**
```bash
# Export roles monthly
reed role:list --format json > "roles_$(date +%Y%m%d).json"
```

---

**See also:**
- [Common Patterns](common-patterns.md) - Global flags, permission syntax
- [User Commands](user-commands.md) - User management
- [Server Layer - Authorization](../06-server-layer/authorization.md) - Runtime checks
