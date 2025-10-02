# REED-04-04: CLI User Management Commands

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
- **ID**: REED-04-04
- **Title**: CLI User Management Commands
- **Layer**: CLI Layer (REED-04)
- **Priority**: Medium
- **Status**: Complete
- **Complexity**: Medium
- **Dependencies**: REED-04-01, REED-03-01, REED-03-02

## Summary Reference
- **Section**: CLI User Management
- **Lines**: 1082-1090 in project_summary.md
- **Key Concepts**: User CRUD operations, password management, role assignment

## Objective
Implement complete CLI interface for user management including creation, listing, updating, deletion, password changes, and role management.

## Requirements

### Commands to Implement

```bash
# User creation
reed user:create username --roles "editor,author"
reed user:create username --email "user@example.com" --firstname "Jane" --lastname "Doe"

# User listing
reed user:list
reed user:list --format table
reed user:list --format json
reed user:list --role "editor"

# User details
reed user:show username

# User updates
reed user:update username --email "newemail@example.com"
reed user:update username --firstname "John" --lastname "Smith"
reed user:update username --mobile "+44123456789"

# User deletion
reed user:delete username
reed user:delete username --confirm

# Password management
reed user:passwd username
reed user:passwd username --new "newpassword"

# Role management
reed user:roles username
reed user:roles username --add "editor"
reed user:roles username --remove "author"
reed user:roles username --set "admin,editor"

# User search
reed user:search --role "editor"
reed user:search --email "example.com"
reed user:search --active true
```

### Implementation (`src/reedcms/cli/user_commands.rs`)

```rust
/// Creates new user with interactive or flag-based input.
///
/// ## Arguments
/// - args[0]: username
///
/// ## Flags
/// - --roles: Comma-separated role names
/// - --email: Email address
/// - --firstname: First name
/// - --lastname: Last name
/// - --password: Password (prompts if not provided)
/// - --mobile: Mobile number
///
/// ## Interactive Mode
/// If flags not provided, prompts for required fields
///
/// ## Output
/// ✓ User 'username' created successfully
///   Email: user@example.com
///   Roles: editor, author
pub fn create_user(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Lists users with optional filtering.
///
/// ## Flags
/// - --format: Output format (table, json, csv)
/// - --role: Filter by role
/// - --active: Filter by active status
///
/// ## Output Formats
/// - table: ASCII table with columns
/// - json: JSON array of user objects
/// - csv: CSV format
pub fn list_users(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Shows detailed user information.
///
/// ## Arguments
/// - args[0]: username
///
/// ## Output
/// User: admin
/// Email: admin@example.com
/// Name: Admin User
/// Roles: admin, editor
/// Mobile: +44123456789
/// Created: 2024-01-15 10:00:00
/// Last login: 2024-01-20 15:30:00
/// Status: active
pub fn show_user(args: &[String]) -> ReedResult<ReedResponse<String>>

/// Updates user profile data.
///
/// ## Arguments
/// - args[0]: username
///
/// ## Flags
/// - --email: New email
/// - --firstname: New first name
/// - --lastname: New last name
/// - --mobile: New mobile
/// - --active: Active status (true/false)
///
/// ## Output
/// ✓ User 'username' updated successfully
pub fn update_user(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Deletes user with confirmation.
///
/// ## Arguments
/// - args[0]: username
///
/// ## Flags
/// - --confirm: Skip confirmation prompt
///
/// ## Safety
/// - Prompts for confirmation unless --confirm
/// - Cannot delete last admin user
/// - Warns about associated content
///
/// ## Output
/// ⚠ This will permanently delete user 'username' and all associated data.
/// ? Are you sure? (y/N): _
pub fn delete_user(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Changes user password.
///
/// ## Arguments
/// - args[0]: username
///
/// ## Flags
/// - --new: New password (prompts if not provided)
///
/// ## Security
/// - Prompts for password twice for confirmation
/// - Validates password strength
/// - Uses Argon2id hashing
///
/// ## Output
/// ✓ Password changed for user 'username'
pub fn change_password(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Manages user roles.
///
/// ## Arguments
/// - args[0]: username
///
/// ## Flags
/// - --add: Add roles (comma-separated)
/// - --remove: Remove roles (comma-separated)
/// - --set: Set roles (replaces all, comma-separated)
///
/// ## Modes
/// - No flags: List current roles
/// - --add: Add roles to existing
/// - --remove: Remove roles from existing
/// - --set: Replace all roles
///
/// ## Output
/// Current roles for 'username': editor, author
/// ✓ Added roles: admin
/// New roles: admin, editor, author
pub fn manage_roles(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Searches users by criteria.
///
/// ## Flags
/// - --role: Filter by role
/// - --email: Filter by email pattern
/// - --active: Filter by active status
/// - --format: Output format
///
/// ## Output
/// Found 3 users matching criteria:
/// - admin (admin@example.com)
/// - editor (editor@example.com)
/// - author (author@example.com)
pub fn search_users(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>
```

### Output Formatting (`src/reedcms/cli/user_output.rs`)

```rust
/// Formats user list as ASCII table.
///
/// ## Table Format
/// ```
/// +----------+----------------------+------------------+--------+
/// | Username | Email                | Roles            | Status |
/// +----------+----------------------+------------------+--------+
/// | admin    | admin@example.com    | admin            | active |
/// | editor   | editor@example.com   | editor, author   | active |
/// +----------+----------------------+------------------+--------+
/// ```
pub fn format_user_table(users: &[UserInfo]) -> String

/// Formats user list as JSON.
pub fn format_user_json(users: &[UserInfo]) -> String

/// Formats user list as CSV.
pub fn format_user_csv(users: &[UserInfo]) -> String

/// Formats single user details.
pub fn format_user_details(user: &UserInfo) -> String
```

## Implementation Files

### Primary Implementation
- `src/reedcms/cli/user_commands.rs` - User commands
- `src/reedcms/cli/user_output.rs` - Output formatting

### Test Files
- `src/reedcms/cli/user_commands.test.rs`
- `src/reedcms/cli/user_output.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test user creation
- [ ] Test user listing with formats
- [ ] Test user details display
- [ ] Test user updates
- [ ] Test user deletion
- [ ] Test password change
- [ ] Test role management (add/remove/set)

### Integration Tests
- [ ] Test complete user lifecycle
- [ ] Test role validation
- [ ] Test password strength validation
- [ ] Test confirmation prompts
- [ ] Test output formats

### Security Tests
- [ ] Test password masking in prompts
- [ ] Test last admin protection
- [ ] Test role existence validation
- [ ] Test email uniqueness

### Performance Tests
- [ ] User creation: < 200ms (including Argon2)
- [ ] User listing: < 100ms for 1000 users
- [ ] User search: < 50ms

## Acceptance Criteria
- [ ] All user commands implemented
- [ ] Multiple output formats (table/json/csv)
- [ ] Interactive and flag-based modes
- [ ] Password prompts secure (masked)
- [ ] Confirmation prompts for destructive ops
- [ ] Role management working (add/remove/set)
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-04-01 (CLI Foundation), REED-03-01 (User Management), REED-03-02 (Roles)

## Blocks
- None

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1082-1090 in `project_summary.md`

## Notes
User management commands must be secure and user-friendly. Password prompts should mask input. Destructive operations require confirmation. The --confirm flag allows scripting while maintaining safety for manual use. Multiple output formats enable both human use (table) and machine processing (json/csv).