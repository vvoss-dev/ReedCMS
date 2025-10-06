# User Commands

> Manage user accounts with Argon2 password hashing

**Implementation:** REED-04-04  
**Status:** ✅ Complete  
**File:** `src/reedcms/cli/user_commands.rs`

---

## Overview

User commands manage accounts with secure Argon2id password hashing (RFC 9106). All data stored in `.reed/users.matrix.csv`.

**Global patterns:** See [Common Patterns](common-patterns.md) for flags, output formats, error codes, and validation rules.

---

## Commands

### `reed user:create`

Create a new user account.

```bash
reed user:create <username> --email <email> --password <password> [options]
```

**Required:**
- `username` - 3-32 chars, alphanumeric + underscore
- `--email` - Valid email address
- `--password` - Min 8 characters

**Optional Profile Fields:**
- `--firstname`, `--lastname` - Name
- `--role` - Initial role (default: viewer)
- `--street`, `--city`, `--postcode`, `--region`, `--country` - Address
- `--mobile` - Phone number
- `--twitter`, `--facebook`, `--tiktok`, `--insta`, `--youtube`, `--whatsapp` - Social
- `--desc` - User bio

**Example:**
```bash
reed user:create jdoe \
    --email john@example.com \
    --password secure456 \
    --firstname "John" \
    --lastname "Doe" \
    --role editor
```

**Password:** Argon2id (m=19456, t=2, p=1), ~100ms hashing time  
**Performance:** ~100ms (intentionally slow for security)

---

### `reed user:list`

List all users.

```bash
reed user:list [pattern] [--format <format>]
```

**Flags:**
- `--format` - Output: `table` (default), `json`, `csv`
- `--active-only` - Show only active users
- `--inactive-only` - Show only inactive users

**Examples:**
```bash
reed user:list                # Table format
reed user:list "admin*"       # Filter by pattern
reed user:list --format json  # JSON output
```

**Output (table):**
```
Username  Email                 Role     Active  Created
admin     admin@example.com     admin    true    2025-01-01
jdoe      john@example.com      editor   true    2025-01-02
```

**Performance:** < 10ms for 1000 users

---

### `reed user:get`

Get detailed user information.

```bash
reed user:get <username> [--format <format>]
```

**Flags:**
- `--format` - Output: `text` (default), `json`

**Example:**
```bash
reed user:get admin
```

**Output:**
```
Username:     admin
Email:        admin@example.com
Role:         admin
Active:       true
Created:      2025-01-01T00:00:00Z
```

**Performance:** < 5ms

---

### `reed user:update`

Update user information.

```bash
reed user:update <username> [options]
```

**Flags:** Same as `user:create`, plus:
- `--active <true|false>` - Activate/deactivate account

**Examples:**
```bash
reed user:update admin --email newemail@example.com
reed user:update jdoe --city "Munich" --mobile "+49987654321"
reed user:update alice --active false
```

**Performance:** < 50ms

---

### `reed user:delete`

Delete a user account.

```bash
reed user:delete <username> [--force]
```

**Flags:** See [Common Patterns → Confirmation Prompts](common-patterns.md#confirmation-prompts)

**Example:**
```bash
reed user:delete alice        # With confirmation
reed user:delete alice --force  # Skip prompt
```

**Safety:** Cannot delete own account, automatic backup created  
**Performance:** < 50ms

---

### `reed user:reset-password`

Reset user password.

```bash
reed user:reset-password <username> <new_password>
```

**Example:**
```bash
NEW_PASS=$(openssl rand -base64 12)
reed user:reset-password admin "$NEW_PASS"
```

**Validation:** Min 8 chars, Argon2id hashing  
**Performance:** ~100ms

---

## CSV File Format

**File:** `.reed/users.matrix.csv` (Matrix Type 2 - Lists)

**Structure:**
```
username|password|roles|firstname|lastname|street|city|postcode|region|country|email|mobile|twitter|facebook|tiktok|insta|youtube|whatsapp|desc|created_at|updated_at|last_login|is_active
```

**Key Columns:**
- `username` - Unique identifier
- `password` - Argon2id PHC string: `$argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>`
- `roles` - Comma-separated list
- `email` - Unique email
- `is_active` - `true`/`false`

**Complete format:** See [Common Patterns → CSV File Locations](common-patterns.md#csv-file-locations)

---

## Password Security

### Argon2id Configuration

**Algorithm:** Argon2id (RFC 9106)  
**Parameters:** Memory 19456 KiB, Iterations 2, Parallelism 1  
**Salt:** 16 bytes random  
**Output:** 32 bytes

**PHC Format:**
```
$argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>
```

**Security:**
- GPU-resistant
- Timing-safe verification
- Rainbow table resistant
- ~100ms intentional slowdown

**Requirements:** Min 8 chars (12+ recommended)

---

## User Roles

Roles managed via [Role Commands](role-commands.md).

**Default Roles:** `admin`, `editor`, `viewer`

**Assign Role:**
```bash
reed role:assign username editor
```

**Multiple Roles:** Comma-separated in CSV:
```
admin|$argon2id...|admin,editor,viewer|...
```

---

## Common Workflows

### Initial Admin Setup

```bash
reed user:create admin \
    --email admin@example.com \
    --password SecurePassword123 \
    --desc "System administrator"

reed role:assign admin admin
```

### Bulk Import

```bash
while IFS=',' read -r username email firstname lastname; do
    PASS=$(openssl rand -base64 12)
    reed user:create "$username" \
        --email "$email" \
        --password "$PASS" \
        --firstname "$firstname" \
        --lastname "$lastname"
    echo "$username: $PASS" >> passwords.txt
done < users.csv
```

### User Audit

```bash
reed user:list --inactive-only              # Find inactive
reed user:list --format json > backup.json  # Export
```

---

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| `create` | ~100ms | Argon2 hashing |
| `list` | < 10ms | 1000 users |
| `get` | < 5ms | Cached |
| `update` | < 50ms | CSV write |
| `delete` | < 50ms | CSV write |
| `reset-password` | ~100ms | Argon2 hashing |

---

## Best Practices

**Strong passwords:**
```bash
# ✅ Generate random
PASS=$(openssl rand -base64 16)
reed user:create admin --email admin@example.com --password "$PASS"
```

**Complete profiles:**
```bash
# ✅ Include role and description
reed user:create jdoe \
    --email john@example.com \
    --password "$PASS" \
    --role editor \
    --desc "Content team lead"
```

**Regular audits:**
```bash
# Monthly export
reed user:list --format json > "users_$(date +%Y%m%d).json"
```

---

**See also:**
- [Common Patterns](common-patterns.md) - Global flags, errors, validation
- [Role Commands](role-commands.md) - RBAC system
- [Server Layer - Authentication](../06-server-layer/authentication.md) - Login flow
