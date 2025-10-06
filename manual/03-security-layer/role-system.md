# Role System (RBAC)

> Role-based access control with fine-grained permissions

**See CLI reference:** [Role Commands](../04-cli-layer/role-commands.md) for command usage.

---

## RBAC Model

### Permission Format

```
resource[rwx]
```

**Flags:**
- `r` = Read (view, list)
- `w` = Write (create, update)
- `x` = Execute (delete, admin)
- `-` = Denied

**Examples:**
```
text[rwx]     # Full access
text[rw-]     # Read + write, no delete
text[r--]     # Read-only
*[rwx]        # Full system access (wildcard)
```

### Available Resources

| Resource | Description |
|----------|-------------|
| `text` | Text content operations |
| `content` | Content management |
| `route` | URL routing |
| `meta` | Metadata management |
| `user` | User management |
| `role` | Role management |
| `template` | Template editing |
| `asset` | Asset management |
| `system` | System configuration |
| `*` | All resources (wildcard) |

---

## CSV Storage

### roles.matrix.csv

```csv
rolename|permissions|description|created_at|updated_at
admin|*[rwx]|Full administrator|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
editor|text[rwx],content[rwx],route[rw-]|Content editor|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
viewer|text[r--],content[r--],route[r--]|Read-only access|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
```

**Matrix type:** Permissions column contains comma-separated list

### user_roles.csv

```csv
user_id|role|assigned_at
admin|admin|2025-01-01T00:00:00Z
jdoe|editor|2025-01-02T00:00:00Z
jdoe|viewer|2025-01-02T00:00:00Z
```

**Purpose:** Maps users to roles (many-to-many relationship)

---

## Operations

### Create Role

```bash
reed role:create editor \
    --permissions "text[rwx],content[rwx],route[rw-]" \
    --desc "Content editor with full text access"
```

**Performance:** < 50ms

### Assign Role

```bash
reed role:assign jdoe editor
```

**Updates:**
- `.reed/user_roles.csv` - Assignment record
- `.reed/users.matrix.csv` - User's roles column

**Performance:** < 50ms

### Check Permission

```rust
pub fn has_permission(user: &User, required: &str) -> ReedResult<bool> {
    // Parse required permission
    let (resource, flags) = parse_permission(required)?;
    
    // Get user's roles
    for role_name in &user.roles {
        let role = get_role(role_name)?;
        
        // Check each permission in role
        for perm in &role.permissions {
            if permission_matches(perm, resource, flags) {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}
```

**Example:**
```rust
if has_permission(&user, "text[rw-]")? {
    // Allow text read and write
}
```

---

## Default Roles

### Admin

```
Permissions: *[rwx]
Description: Full system access
```

**Use case:** System administrators

### Editor

```
Permissions: text[rwx], content[rwx], route[rw-]
Description: Content management
```

**Use case:** Content creators and editors

### Viewer

```
Permissions: text[r--], content[r--], route[r--]
Description: Read-only access
```

**Use case:** Auditors, read-only users

---

## Permission Patterns

### Read-Only

```
text[r--],content[r--],route[r--]
```

**Can:** View text, content, routes  
**Cannot:** Modify or delete anything

### Content Writer

```
text[rw-],content[rw-]
```

**Can:** View and create/edit content  
**Cannot:** Delete content

### Full Resource Access

```
text[rwx],content[rwx],route[rwx]
```

**Can:** Full access to specific resources  
**Cannot:** Access users, roles, system config

### System Admin

```
*[rwx]
```

**Can:** Everything  
**Use:** System administrators only

---

## Integration

### Server Middleware

```rust
use crate::reedcms::auth::middleware::check_permission;

async fn protected_route(req: HttpRequest) -> Result<HttpResponse> {
    // Get authenticated user
    let user = get_authenticated_user(&req).await?;
    
    // Check permission
    if !has_permission(&user, "text[rw-]")? {
        return Err(ReedError::PermissionDenied {
            user: user.username,
            resource: "text".to_string(),
            action: "write".to_string(),
        });
    }
    
    // Process request
    Ok(HttpResponse::Ok().json(/* ... */))
}
```

### CLI Commands

**Permission checks before operations:**
```rust
// Before deleting text
if !has_permission(&current_user, "text[rwx]")? {
    return Err(ReedError::PermissionDenied { /* ... */ });
}

// Delete text
delete_text(key)?;
```

---

## Best Practices

**Principle of least privilege:**
```bash
# ✅ Good - only necessary permissions
reed role:create blog-author --permissions "content[rw-],text[r--]"

# ❌ Bad - excessive permissions
reed role:create blog-author --permissions "*[rwx]"
```

**Descriptive role names:**
```bash
# ✅ Good
reed role:create content-writer
reed role:create senior-editor
reed role:create read-only-auditor

# ❌ Bad
reed role:create role1
reed role:create temp
```

**Regular permission audits:**
```bash
# Monthly review
reed role:list
reed role:get editor
```

---

**See also:**
- [User Management](user-management.md) - User operations
- [Authentication](authentication.md) - Login and sessions
- [CLI Role Commands](../04-cli-layer/role-commands.md) - Complete CLI reference
