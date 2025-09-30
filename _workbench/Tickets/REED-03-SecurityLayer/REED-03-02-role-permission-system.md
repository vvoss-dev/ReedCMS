# REED-03-02: Role Permission System

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function (CONFIG and SYSTEM use identical logic)
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs` - every topic (auth, login,...) has its own scoped rs service file
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-03-02
- **Title**: Role-Based Permission System
- **Layer**: Security Layer (REED-03)
- **Priority**: High
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-03-01

## Summary Reference
- **Section**: Security & Permission System - Roles
- **Lines**: 367-396 in project_summary.md
- **Key Concepts**: Unix-style permissions, role inheritance, sub-millisecond cached lookups

## Objective
Implement Unix-style permission system with role inheritance, sub-millisecond cached lookups, circular dependency detection, and hierarchical resource matching.

## Requirements

### Role Matrix CSV Structure
```csv
rolename;permissions;inherits;desc;created_at;updated_at;is_active
editor;text[rwx],route[rw-],content[rw-];;Standard Content Editor;1640995200;1640995200;true
admin;*[rwx];editor;Full Admin with inheritance;1640995200;1640995200;true
author;text[rw-],content[r--];editor;Content Author;1640995200;1640995200;true
viewer;*[r--];;Read-only access;1640995200;1640995200;true
```

### Permission Syntax
```
Format: resource[permissions]

Permissions:
- r: read
- w: write
- x: execute (admin actions)
- -: no permission

Examples:
- text[rwx] - Full text access
- route[rw-] - Read/write routes, no execute
- *[rwx] - Wildcard full access
- content/blog/* - Hierarchical resource matching
```

### Implementation Files

#### Role CRUD Operations (`src/reedcms/security/roles.rs`)

```rust
/// Creates new role with permission validation.
///
/// ## Input
/// - `req.rolename`: Role name (alphanumeric + underscore)
/// - `req.permissions`: Comma-separated permissions (text[rwx],route[rw-])
/// - `req.inherits`: Optional parent role name
/// - `req.desc`: Role description
///
/// ## Validation
/// - Role name uniqueness
/// - Permission syntax validation
/// - Parent role existence check
/// - Circular inheritance detection
pub fn create_role(req: &ReedRequest) -> ReedResult<ReedResponse<RoleInfo>>

/// Retrieves role with resolved permissions (including inherited).
pub fn get_role(rolename: &str) -> ReedResult<ReedResponse<RoleInfo>>

/// Lists all roles.
pub fn list_roles() -> ReedResult<ReedResponse<Vec<RoleInfo>>>

/// Updates role permissions or inheritance.
pub fn update_role(rolename: &str, updates: RoleUpdate) -> ReedResult<ReedResponse<RoleInfo>>

/// Deletes role with dependency check.
pub fn delete_role(rolename: &str, confirm: bool) -> ReedResult<ReedResponse<()>>

/// Role information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub rolename: String,
    pub permissions: Vec<Permission>,
    pub inherits: Option<String>,
    pub desc: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
}
```

#### Permission Checking (`src/reedcms/security/permissions.rs`)

```rust
/// Checks if user has permission for resource and action.
///
/// ## Permission Resolution Order
/// 1. Check user's direct permissions
/// 2. Check user's role permissions
/// 3. Check inherited role permissions (recursive)
/// 4. Apply hierarchical resource matching
/// 5. Cache result for subsequent lookups
///
/// ## Performance
/// - Cached lookup: < 1ms (sub-millisecond)
/// - Uncached lookup: < 10ms
pub fn check_permission(user: &str, resource: &str, action: &str) -> ReedResult<bool>

/// Parses permission string into structure.
///
/// ## Examples
/// - "text[rwx]" → Permission { resource: "text", read: true, write: true, execute: true }
/// - "route[rw-]" → Permission { resource: "route", read: true, write: true, execute: false }
pub fn parse_permission(perm: &str) -> ReedResult<Permission>

/// Validates permission syntax.
pub fn validate_permission_syntax(perm: &str) -> ReedResult<()>

/// Permission structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}
```

#### Role Inheritance (`src/reedcms/security/inheritance.rs`)

```rust
/// Resolves role inheritance chain.
///
/// ## Process
/// 1. Start with role
/// 2. Follow inherits chain
/// 3. Detect circular dependencies
/// 4. Build complete permission set
///
/// ## Circular Detection
/// - Track visited roles
/// - Return error if cycle detected
///
/// ## Output
/// - Vector of role names from child to parent
/// - Example: ["author", "editor", "viewer"]
pub fn resolve_inheritance(role: &str) -> ReedResult<Vec<String>>

/// Checks for circular inheritance.
pub fn has_circular_inheritance(role: &str) -> ReedResult<bool>

/// Merges permissions from inheritance chain.
///
/// ## Rules
/// - Child permissions override parent
/// - Wildcard (*) applies to all resources
/// - More specific resources override general
pub fn merge_inherited_permissions(roles: &[String]) -> ReedResult<Vec<Permission>>
```

#### Permission Cache (`src/reedcms/security/cache.rs`)

```rust
/// Permission cache for sub-millisecond lookups.
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

pub type PermissionCache = Arc<RwLock<HashMap<String, bool>>>;

lazy_static! {
    pub static ref PERMISSION_CACHE: PermissionCache = Arc::new(RwLock::new(HashMap::new()));
}

/// Caches permission check result.
///
/// ## Cache Key Format
/// - "username:resource:action"
/// - Example: "admin:text:write"
pub fn cache_permission(user: &str, resource: &str, action: &str, result: bool)

/// Retrieves cached permission.
pub fn get_cached_permission(user: &str, resource: &str, action: &str) -> Option<bool>

/// Invalidates cache on permission/role changes.
pub fn invalidate_cache()

/// Invalidates cache for specific user.
pub fn invalidate_user_cache(user: &str)
```

## Implementation Files

### Primary Implementation
- `src/reedcms/security/roles.rs` - Role CRUD operations
- `src/reedcms/security/permissions.rs` - Permission checking
- `src/reedcms/security/inheritance.rs` - Role inheritance
- `src/reedcms/security/cache.rs` - Permission cache

### Test Files
- `src/reedcms/security/roles.test.rs`
- `src/reedcms/security/permissions.test.rs`
- `src/reedcms/security/inheritance.test.rs`
- `src/reedcms/security/cache.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test permission parsing (text[rwx])
- [ ] Test role creation with valid permissions
- [ ] Test inheritance resolution
- [ ] Test circular inheritance detection
- [ ] Test permission merging
- [ ] Test cache operations

### Integration Tests
- [ ] Test complete permission check workflow
- [ ] Test user → role → inherited permissions
- [ ] Test hierarchical resource matching (content/blog/*)
- [ ] Test wildcard permissions (*[rwx])
- [ ] Test cache invalidation

### Security Tests
- [ ] Test permission override (child > parent)
- [ ] Test circular inheritance prevention
- [ ] Test wildcard restriction validation
- [ ] Test privilege escalation prevention

### Performance Tests
- [ ] Cached permission check: < 1ms
- [ ] Uncached permission check: < 10ms
- [ ] Inheritance resolution: < 5ms
- [ ] Cache invalidation: < 1ms

## Acceptance Criteria
- [ ] Unix-style permission syntax parsed correctly
- [ ] Role inheritance with circular detection working
- [ ] Sub-millisecond cached lookups achieved
- [ ] Hierarchical resource matching implemented
- [ ] Cache invalidation on permission changes
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-03-01 (User Management)

## Blocks
- REED-04-05 (CLI role commands need this)
- REED-06-03 (Authentication middleware needs permission checking)
- REED-07-02 (API security matrix needs permission system)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 367-396 in `project_summary.md`

## Notes
The permission cache is critical for performance. Sub-millisecond lookups enable permission checks in every request without performance degradation. Cache invalidation must be automatic on any role/permission change. Circular inheritance detection prevents infinite loops and must be enforced at role creation time.