# Security Matrix

> Permission-based API access control

**Status:** 🔄 Planned

---

## Concept

**Map API endpoints to RBAC permissions.**

```
HTTP Method + Endpoint → Permission Check
```

---

## Permission Mapping

### Text Endpoints

| Endpoint | Method | Permission |
|----------|--------|------------|
| `/api/v1/text/{key}` | GET | `text[r--]` |
| `/api/v1/text` | POST | `text[-w-]` |
| `/api/v1/text/{key}` | PUT | `text[-w-]` |
| `/api/v1/text/{key}` | DELETE | `text[--x]` |

### User Endpoints

| Endpoint | Method | Permission |
|----------|--------|------------|
| `/api/v1/users` | GET | `user[r--]` |
| `/api/v1/users/{username}` | GET | `user[r--]` |
| `/api/v1/users` | POST | `user[-w-]` |
| `/api/v1/users/{username}` | PUT | `user[-w-]` |
| `/api/v1/users/{username}` | DELETE | `user[--x]` |

### Role Endpoints

| Endpoint | Method | Permission |
|----------|--------|------------|
| `/api/v1/roles` | GET | `role[r--]` |
| `/api/v1/roles` | POST | `role[-w-]` |
| `/api/v1/roles/{rolename}` | PUT | `role[-w-]` |
| `/api/v1/roles/{rolename}` | DELETE | `role[--x]` |

---

## Authorization Check

### Process

```
1. Extract Bearer token
2. Validate session → Get user
3. Map endpoint to permission
4. Check: user.has_permission(required)
5. Allow or deny (403 Forbidden)
```

### Implementation

```rust
pub async fn authorize_request(
    req: &HttpRequest,
    required: &str
) -> ReedResult<User> {
    // 1. Get user from token
    let user = authenticate(req).await?;
    
    // 2. Check permission
    if !has_permission(&user, required)? {
        return Err(ReedError::PermissionDenied {
            user: user.username,
            resource: required.to_string(),
        });
    }
    
    Ok(user)
}
```

---

## Example Checks

### Editor Role

**Permissions:** `text[rwx], content[rwx], route[rw-]`

**Request:** `GET /api/v1/text/page.title`  
**Required:** `text[r--]`  
**Result:** ✅ Allowed (editor has `text[rwx]`)

**Request:** `DELETE /api/v1/users/jdoe`  
**Required:** `user[--x]`  
**Result:** ❌ Denied (editor doesn't have `user[--x]`)

### Admin Role

**Permissions:** `*[rwx]` (all resources)

**Any request:** ✅ Allowed

---

## Error Response

### 403 Forbidden

```json
{
  "error": {
    "type": "PermissionDenied",
    "message": "Insufficient permissions",
    "required": "user[--x]",
    "user_permissions": ["text[rwx]", "content[rwx]"]
  }
}
```

---

**See also:**
- [Security Layer - Role System](../03-security-layer/role-system.md) - RBAC details
- [Authentication](authentication.md) - Token validation
