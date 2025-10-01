# REED-03-01: User Management System

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
- **ID**: REED-03-01
- **Title**: User Management System
- **Layer**: Security Layer (REED-03)
- **Priority**: High
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-02-01, REED-02-02

## Summary Reference
- **Section**: Security & Permission System - User Management
- **Lines**: 356-378 in project_summary.md
- **Key Concepts**: Argon2 password hashing, extended profile data, account status tracking

## Objective
Implement comprehensive user management with Argon2 password hashing, extended profile data including social media, and complete account status tracking.

## Requirements

### User Matrix CSV Structure
```csv
username|password|roles|firstname|lastname|street|city|postcode|region|country|email|mobile|twitter|facebook|tiktok|insta|youtube|whatsapp|desc|created_at|updated_at|last_login|is_active
admin|$argon2id$hash|admin|Admin|User|Main St 1|London|SW1A 1AA|London|UK|admin@example.com|+44123456789|||||||System Administrator|1640995200|1640995200||true
editor|$argon2id$hash|editor|Jane|Doe|High St 42|Manchester|M1 1AA|Manchester|UK|jane@example.com|+44987654321|@jane_editor|jane.doe||||Content Editor|1640995200|1640995200|1640999800|true
```

### Implementation Files

#### User CRUD Operations (`src/reedcms/security/users.rs`)

```rust
/// Creates new user with validation and password hashing.
///
/// ## Input
/// - `req.username`: Username (alphanumeric + underscore, 3-32 chars)
/// - `req.password`: Plain text password (will be hashed)
/// - `req.email`: Email address (validated)
/// - `req.roles`: Comma-separated role names
/// - Additional profile fields
///
/// ## Validation
/// - Username uniqueness
/// - Email uniqueness and format
/// - Password strength (min 8 chars, uppercase, lowercase, digit, special)
/// - Role existence check
pub fn create_user(req: &ReedRequest) -> ReedResult<ReedResponse<UserInfo>>

/// Retrieves user by username.
pub fn get_user(username: &str) -> ReedResult<ReedResponse<UserInfo>>

/// Lists all users with optional filtering.
pub fn list_users(filter: Option<UserFilter>) -> ReedResult<ReedResponse<Vec<UserInfo>>>

/// Updates user profile data (not password).
pub fn update_user(username: &str, updates: UserUpdate) -> ReedResult<ReedResponse<UserInfo>>

/// Deletes user (requires confirmation).
pub fn delete_user(username: &str, confirm: bool) -> ReedResult<ReedResponse<()>>

/// User information structure (without password hash)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub roles: Vec<String>,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub mobile: Option<String>,
    pub social_media: SocialMedia,
    pub address: Address,
    pub desc: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub last_login: Option<u64>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMedia {
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub tiktok: Option<String>,
    pub instagram: Option<String>,
    pub youtube: Option<String>,
    pub whatsapp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: Option<String>,
    pub city: Option<String>,
    pub postcode: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
}
```

#### Password Management (`src/reedcms/security/passwords.rs`)

```rust
/// Hashes password using Argon2id.
///
/// ## Security
/// - Algorithm: Argon2id
/// - Memory cost: 65536 KiB
/// - Time cost: 3 iterations
/// - Parallelism: 4 threads
///
/// ## Performance
/// - Hashing time: ~100ms (intentionally slow for security)
pub fn hash_password(password: &str) -> ReedResult<String>

/// Verifies password against hash.
///
/// ## Performance
/// - Verification time: ~100ms
pub fn verify_password(password: &str, hash: &str) -> ReedResult<bool>

/// Changes user password.
///
/// ## Process
/// 1. Verify old password
/// 2. Validate new password strength
/// 3. Hash new password
/// 4. Update user record
pub fn change_password(username: &str, old_password: &str, new_password: &str) -> ReedResult<ReedResponse<()>>

/// Validates password strength.
///
/// ## Requirements
/// - Minimum 8 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
/// - At least one special character
pub fn validate_password_strength(password: &str) -> ReedResult<()>
```

#### Validation (`src/reedcms/security/validation.rs`)

```rust
/// Validates email format.
pub fn validate_email(email: &str) -> ReedResult<()>

/// Validates username format and uniqueness.
pub fn validate_username(username: &str) -> ReedResult<()>

/// Checks if username already exists.
pub fn username_exists(username: &str) -> ReedResult<bool>

/// Checks if email already exists.
pub fn email_exists(email: &str) -> ReedResult<bool>
```

## Implementation Files

### Primary Implementation
- `src/reedcms/security/users.rs` - User CRUD operations
- `src/reedcms/security/passwords.rs` - Argon2 password hashing
- `src/reedcms/security/validation.rs` - Email/username validation

### Test Files
- `src/reedcms/security/users.test.rs`
- `src/reedcms/security/passwords.test.rs`
- `src/reedcms/security/validation.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test user creation with valid data
- [ ] Test password hashing (Argon2id)
- [ ] Test password verification
- [ ] Test email validation
- [ ] Test username validation
- [ ] Test uniqueness checks

### Integration Tests
- [ ] Test complete user lifecycle (create → update → delete)
- [ ] Test password change workflow
- [ ] Test duplicate username prevention
- [ ] Test duplicate email prevention
- [ ] Test role assignment

### Security Tests
- [ ] Test password strength validation
- [ ] Test Argon2 hash format
- [ ] Test timing attack resistance
- [ ] Test SQL injection prevention (N/A for CSV)

### Performance Tests
- [ ] Password hashing: ~100ms (security requirement)
- [ ] User creation: < 150ms (including password hashing)
- [ ] User lookup: < 50ms

## Acceptance Criteria
- [ ] Argon2id password hashing implemented
- [ ] Email/username uniqueness enforced
- [ ] Social media profiles supported (6 platforms)
- [ ] Account status tracking (last_login, is_active)
- [ ] Password strength validation working
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-02-01 (ReedBase), REED-02-02 (CSV Handler)

## Blocks
- REED-03-02 (Role system needs user management)
- REED-04-04 (CLI user commands need this)
- REED-06-03 (Authentication middleware needs this)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 356-378 in `project_summary.md`

## Notes
Security is paramount. Argon2id with proper parameters (memory: 65536 KiB, iterations: 3, parallelism: 4) provides strong protection against brute-force attacks. The intentionally slow hashing time (~100ms) is a security feature, not a performance issue. All password-related operations must use constant-time comparison to prevent timing attacks.