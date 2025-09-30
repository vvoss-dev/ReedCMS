# REED-04-05: CLI Role Management Commands

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
- **ID**: REED-04-05
- **Title**: CLI Role Management Commands
- **Layer**: CLI Layer (REED-04)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-04-01, REED-03-02

## Summary Reference
- **Section**: CLI Role Management
- **Lines**: 1091-1098 in project_summary.md
- **Key Concepts**: Role CRUD operations, permission management, inheritance

## Objective
Implement complete CLI interface for role management including creation, listing, updating, deletion, permission management, and inheritance configuration.

## Requirements

### Commands to Implement

```bash
# Role creation
reed role:create editor --permissions "text[rwx],route[rw-]"
reed role:create admin --permissions "*[rwx]" --inherit "editor"

# Role listing
reed role:list
reed role:list --show-permissions
reed role:list --format table

# Role details
reed role:show editor

# Role updates
reed role:update editor --permissions "text[rwx],route[rwx],content[rw-]"
reed role:update admin --inherit "superuser"
reed role:update viewer --desc "Read-only access for all content"

# Role deletion
reed role:delete rolename
reed role:delete rolename --confirm

# Permission management
reed role:permissions editor
reed role:permissions editor --add "cms[rw-]"
reed role:permissions editor --remove "content[rwx]"
reed role:permissions editor --set "text[rwx],route[rw-]"

# Role users
reed role:users editor
```

### Implementation (`src/reedcms/cli/role_commands.rs`)

```rust
/// Creates new role with permissions.
///
/// ## Arguments
/// - args[0]: rolename
///
/// ## Flags
/// - --permissions: Unix-style permissions (required)
/// - --inherit: Parent role name (optional)
/// - --desc: Role description (required)
///
/// ## Permission Syntax
/// - resource[rwx]: Full access
/// - resource[rw-]: Read/write only
/// - resource[r--]: Read-only
/// - *[rwx]: Wildcard full access
///
/// ## Output
/// ✓ Role 'editor' created successfully
///   Permissions: text[rwx], route[rw-], content[rw-]
///   Inherits: viewer
pub fn create_role(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Lists all roles.
///
/// ## Flags
/// - --show-permissions: Show full permission list
/// - --format: Output format (table, json, csv)
///
/// ## Output (table format)
/// ```
/// +----------+---------------------------+----------+--------+
/// | Role     | Permissions               | Inherits | Active |
/// +----------+---------------------------+----------+--------+
/// | admin    | *[rwx]                    | editor   | yes    |
/// | editor   | text[rwx],route[rw-]     | viewer   | yes    |
/// | viewer   | *[r--]                    |          | yes    |
/// +----------+---------------------------+----------+--------+
/// ```
pub fn list_roles(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Shows detailed role information.
///
/// ## Arguments
/// - args[0]: rolename
///
/// ## Output
/// Role: editor
/// Description: Content editor with full text access
/// Permissions:
///   - text[rwx]: Full text content access
///   - route[rw-]: Read/write route configuration
///   - content[rw-]: Read/write content management
/// Inherits: viewer
/// Inherited permissions:
///   - *[r--]: Read-only access to all resources
/// Users with this role: 5
/// Created: 2024-01-15 10:00:00
/// Status: active
pub fn show_role(args: &[String]) -> ReedResult<ReedResponse<String>>

/// Updates role properties.
///
/// ## Arguments
/// - args[0]: rolename
///
/// ## Flags
/// - --permissions: New permissions (replaces all)
/// - --inherit: New parent role
/// - --desc: New description
/// - --active: Active status (true/false)
///
/// ## Output
/// ✓ Role 'editor' updated successfully
pub fn update_role(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Deletes role with confirmation.
///
/// ## Arguments
/// - args[0]: rolename
///
/// ## Flags
/// - --confirm: Skip confirmation prompt
///
/// ## Safety Checks
/// - Cannot delete if users assigned
/// - Cannot delete if other roles inherit
/// - Prompts for confirmation unless --confirm
///
/// ## Output
/// ⚠ Role 'editor' is assigned to 3 users: admin, jane, john
/// ⚠ Role 'author' inherits from 'editor'
/// ? Delete anyway? (y/N): _
pub fn delete_role(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Manages role permissions.
///
/// ## Arguments
/// - args[0]: rolename
///
/// ## Flags
/// - --add: Add permissions (preserves existing)
/// - --remove: Remove permissions
/// - --set: Set permissions (replaces all)
///
/// ## Modes
/// - No flags: List current permissions
/// - --add: Add to existing permissions
/// - --remove: Remove from existing permissions
/// - --set: Replace all permissions
///
/// ## Output
/// Current permissions for 'editor':
///   - text[rwx]
///   - route[rw-]
/// ✓ Added: content[rw-]
/// New permissions:
///   - text[rwx]
///   - route[rw-]
///   - content[rw-]
pub fn manage_permissions(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Lists users with specific role.
///
/// ## Arguments
/// - args[0]: rolename
///
/// ## Output
/// Users with role 'editor' (5):
/// - admin (admin@example.com) - direct
/// - jane (jane@example.com) - direct
/// - john (john@example.com) - via author role
pub fn list_role_users(args: &[String]) -> ReedResult<ReedResponse<String>>
```

### Validation (`src/reedcms/cli/role_validation.rs`)

```rust
/// Validates Unix-style permission syntax.
///
/// ## Valid Formats
/// - resource[rwx]
/// - resource[rw-]
/// - resource[r--]
/// - *[rwx] (wildcard)
/// - content/blog/*[rw-] (hierarchical)
///
/// ## Invalid Formats
/// - resource[xyz] (invalid flags)
/// - resource (missing brackets)
/// - [rwx] (missing resource)
pub fn validate_permission_syntax(perm: &str) -> ReedResult<()>

/// Checks for circular inheritance.
///
/// ## Process
/// 1. Build inheritance chain
/// 2. Detect cycles
/// 3. Return error if cycle found
///
/// ## Example
/// - editor inherits viewer ✓
/// - admin inherits editor ✓
/// - editor inherits admin ✗ (cycle)
pub fn check_circular_inheritance(role: &str, parent: &str) -> ReedResult<()>

/// Validates role name format.
///
/// ## Rules
/// - Alphanumeric + underscore
/// - 3-32 characters
/// - No reserved names (system, root)
pub fn validate_role_name(name: &str) -> ReedResult<()>
```

## Implementation Files

### Primary Implementation
- `src/reedcms/cli/role_commands.rs` - Role commands
- `src/reedcms/cli/role_validation.rs` - Validation
- `src/reedcms/cli/role_output.rs` - Output formatting

### Test Files
- `src/reedcms/cli/role_commands.test.rs`
- `src/reedcms/cli/role_validation.test.rs`
- `src/reedcms/cli/role_output.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test role creation
- [ ] Test permission parsing
- [ ] Test inheritance configuration
- [ ] Test role updates
- [ ] Test role deletion checks
- [ ] Test permission management (add/remove/set)

### Integration Tests
- [ ] Test complete role lifecycle
- [ ] Test circular inheritance detection
- [ ] Test role deletion with dependencies
- [ ] Test inherited permissions resolution

### Security Tests
- [ ] Test wildcard permission restrictions
- [ ] Test privilege escalation prevention
- [ ] Test role dependency checks

### Performance Tests
- [ ] Role creation: < 50ms
- [ ] Role listing: < 100ms
- [ ] Permission resolution: < 10ms

## Acceptance Criteria
- [ ] All role commands implemented
- [ ] Unix-style permission syntax parsed
- [ ] Circular inheritance detection working
- [ ] Multiple output formats supported
- [ ] Permission management (add/remove/set) functional
- [ ] Role dependency checks enforced
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-04-01 (CLI Foundation), REED-03-02 (Role System)

## Blocks
- None

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1091-1098 in `project_summary.md`

## Notes
Role management is critical for security. Permission syntax must be validated strictly. Circular inheritance must be prevented at creation time. Role deletion requires careful dependency checking to prevent orphaned user assignments or broken inheritance chains.