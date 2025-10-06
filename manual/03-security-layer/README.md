# Security Layer (Layer 03)

> User management, RBAC, and Argon2 password hashing

**Status:** ✅ Complete  
**Implementation:** REED-03-01 to REED-03-03

---

## Overview

The Security Layer provides user authentication, role-based access control (RBAC), and secure password management using Argon2id hashing (RFC 9106 compliant).

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│           Application Layer                      │
│  (CLI, Server, API)                              │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│        Authentication Middleware                 │
│  - Password verification (Argon2id)              │
│  - Session management                            │
│  - Permission checks                             │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│            User Management                       │
│  CRUD operations for users                       │
│  .reed/users.matrix.csv                          │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│            Role Management                       │
│  CRUD operations for roles                       │
│  Permission assignments                          │
│  .reed/roles.matrix.csv                          │
│  .reed/user_roles.csv                            │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│         Password Hashing (Argon2id)              │
│  - Hash generation (~100ms intentional)          │
│  - Timing-safe verification                      │
│  - PHC string format                             │
└──────────────────────────────────────────────────┘
```

---

## Core Concepts

### Argon2id Password Hashing

**Algorithm:** Argon2id (hybrid mode, recommended by RFC 9106)

**Security properties:**
- Memory-hard (resists GPU attacks)
- Timing-safe comparison
- Salted (resists rainbow tables)
- Configurable difficulty

**Parameters:**
```
Memory: 19456 KiB (~19 MB)
Iterations: 2
Parallelism: 1 thread
Salt: 16 bytes (random)
Output: 32 bytes
```

**Performance:** ~100ms per hash (intentional slowdown)

**PHC format:**
```
$argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>
```

### Role-Based Access Control (RBAC)

**Permission format:** `resource[rwx]`

| Flag | Permission | Meaning |
|------|------------|---------|
| `r` | Read | View/list data |
| `w` | Write | Create/update data |
| `x` | Execute | Delete/admin actions |

**Examples:**
```
text[rwx]     # Full access to text
text[rw-]     # Read and write, no delete
text[r--]     # Read-only
*[rwx]        # Full system access
```

**Roles:**
- Stored in `.reed/roles.matrix.csv`
- Comma-separated permission lists
- Assigned to users in `.reed/user_roles.csv`

### User Management

**User accounts:**
- Unique username (3-32 chars)
- Hashed password (Argon2id)
- Email (unique, RFC 5322 compliant)
- Multiple roles (comma-separated)
- Profile fields (name, address, social media)
- Active/inactive status

**Storage:** `.reed/users.matrix.csv` (Matrix CSV Type 2)

---

## Core Components

### Password Service

**File:** `src/reedcms/security/passwords.rs`

**Functions:**
```rust
pub fn hash_password(password: &str) -> ReedResult<String>
pub fn verify_password(password: &str, hash: &str) -> ReedResult<bool>
```

**Features:**
- Argon2id hashing
- Random salt generation
- Timing-safe verification
- PHC string format

**Performance:** ~100ms per operation (security feature)

### User Service

**File:** `src/reedcms/security/users.rs`

**Functions:**
```rust
pub fn create_user(user: UserRequest) -> ReedResult<ReedResponse<User>>
pub fn get_user(username: &str) -> ReedResult<ReedResponse<User>>
pub fn update_user(username: &str, updates: UserRequest) -> ReedResult<ReedResponse<User>>
pub fn delete_user(username: &str) -> ReedResult<ReedResponse<()>>
pub fn list_users(filter: Option<String>) -> ReedResult<ReedResponse<Vec<User>>>
```

**Features:**
- CRUD operations
- Validation (username, email, password)
- Password hashing on create
- Matrix CSV integration

### Role Service

**File:** `src/reedcms/security/roles.rs`

**Functions:**
```rust
pub fn create_role(role: RoleRequest) -> ReedResult<ReedResponse<Role>>
pub fn get_role(rolename: &str) -> ReedResult<ReedResponse<Role>>
pub fn update_role(rolename: &str, updates: RoleRequest) -> ReedResult<ReedResponse<Role>>
pub fn delete_role(rolename: &str) -> ReedResult<ReedResponse<()>>
pub fn assign_role(username: &str, rolename: &str) -> ReedResult<ReedResponse<()>>
```

**Features:**
- CRUD operations
- Permission parsing and validation
- User-role assignment
- Matrix CSV integration

### Authentication Middleware

**File:** `src/reedcms/auth/middleware.rs`

**Features:**
- Password verification
- Session management
- Permission checks
- Request authentication

**Integration:** Used by Server Layer (06) for HTTP auth

---

## CSV File Formats

### users.matrix.csv

```csv
username|password|roles|firstname|lastname|street|city|postcode|region|country|email|mobile|twitter|facebook|tiktok|insta|youtube|whatsapp|desc|created_at|updated_at|last_login|is_active
admin|$argon2id$v=19$m=19456,t=2,p=1$...|admin,editor|John|Doe|...|admin@example.com|...|...|...|Admin user|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z||true
```

**Type:** Matrix CSV (comma-separated roles column)

### roles.matrix.csv

```csv
rolename|permissions|description|created_at|updated_at
admin|text[rwx],user[rwx],role[rwx],system[rwx]|Full administrator|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
editor|text[rwx],content[rwx],route[rw-]|Content editor|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
viewer|text[r--],content[r--],route[r--]|Read-only access|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
```

**Type:** Matrix CSV (comma-separated permissions column)

### user_roles.csv

```csv
user_id|role|assigned_at
admin|admin|2025-01-01T00:00:00Z
jdoe|editor|2025-01-02T00:00:00Z
jdoe|viewer|2025-01-02T00:00:00Z
```

**Type:** Simple CSV (user-role assignments)

---

## Security Features

### Password Requirements

**Minimum length:** 8 characters  
**Recommended:** 12+ characters with mixed types

**Hashing time:** ~100ms (intentional slowdown against brute force)

**Storage:** Never store plain passwords, only Argon2id hashes

### Permission Checking

**Runtime validation:**
```rust
// Check if user has permission
if has_permission(user, "text[rw-]")? {
    // Allow operation
}
```

**Granular control:**
- Resource-level permissions
- Operation-level permissions (read/write/execute)
- Wildcard support (`*[rwx]` = full access)

### Account Security

**Active/inactive status:**
- Inactive accounts cannot log in
- Useful for temporary suspension

**Email verification:** (Optional, implement as needed)

**Password reset:** Via `reed user:reset-password`

---

## Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| Hash password | ~100ms | Argon2id intentional slowdown |
| Verify password | ~100ms | Argon2id verification |
| Create user | ~150ms | Includes hashing + CSV write |
| Get user | < 10ms | CSV read |
| List users | < 20ms | Full CSV scan |
| Assign role | < 10ms | CSV write |
| Check permission | < 1ms | String comparison |

**Security vs Performance:**
- Slow hashing = better security (brute force protection)
- Acceptable for authentication (not performance-critical)
- Cache user data to minimize lookups

---

## Integration

### CLI Layer

```bash
# User management
reed user:create admin --email admin@example.com --password secure123
reed user:list
reed user:get admin
reed user:update admin --email newemail@example.com
reed user:delete alice
reed user:reset-password admin newpass123

# Role management
reed role:create editor --permissions "text[rwx],content[rwx]"
reed role:list
reed role:get editor
reed role:update editor --permissions "text[rwx],content[rwx],route[rwx]"
reed role:delete old-role
reed role:assign jdoe editor
```

**See:** [CLI Commands - User Commands](../04-cli-layer/user-commands.md)  
**See:** [CLI Commands - Role Commands](../04-cli-layer/role-commands.md)

### Server Layer

```rust
// Authentication middleware
use crate::reedcms::auth::middleware::authenticate;

async fn protected_route(req: HttpRequest) -> Result<HttpResponse> {
    // Verify credentials
    let user = authenticate(&req).await?;
    
    // Check permissions
    if !user.has_permission("text[rw-]")? {
        return Err(ReedError::PermissionDenied { /* ... */ });
    }
    
    // Process request
    Ok(HttpResponse::Ok().json(/* ... */))
}
```

**See:** [Server Layer - Authentication](../06-server-layer/authentication.md)

---

## Documentation

- [Password Hashing](password-hashing.md) - Argon2id implementation details
- [User Management](user-management.md) - Complete user CRUD reference
- [Role System](role-system.md) - RBAC implementation and permission model
- [Authentication](authentication.md) - Middleware and session management

---

## Related Layers

- **Layer 02 - Data:** Provides CSV storage for users and roles
- **Layer 04 - CLI:** Exposes security operations via commands
- **Layer 06 - Server:** Uses authentication middleware
- **Layer 07 - API:** Implements security matrix for API endpoints

---

## Summary

The Security Layer provides:
- ✅ Argon2id password hashing (RFC 9106)
- ✅ User CRUD operations with extended profiles
- ✅ Role-based access control (RBAC)
- ✅ Fine-grained permissions (`resource[rwx]`)
- ✅ Authentication middleware
- ✅ Matrix CSV storage
- ✅ Timing-safe password verification
- ✅ ~100ms intentional hash slowdown (security feature)

All features production-ready and fully tested.
