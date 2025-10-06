# User Management

> CRUD operations for user accounts with extended profiles

**See CLI reference:** [User Commands](../04-cli-layer/user-commands.md) for command usage.

---

## User Model

### Structure

```rust
pub struct User {
    pub username: String,        // Unique, 3-32 chars
    pub password_hash: String,   // Argon2id hash
    pub roles: Vec<String>,      // Comma-separated in CSV
    pub email: String,           // Unique, RFC 5322
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub postcode: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>, // 2-char ISO code
    pub mobile: Option<String>,
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub tiktok: Option<String>,
    pub insta: Option<String>,
    pub youtube: Option<String>,
    pub whatsapp: Option<String>,
    pub desc: Option<String>,
    pub created_at: String,      // ISO 8601
    pub updated_at: String,      // ISO 8601
    pub last_login: Option<String>, // ISO 8601
    pub is_active: bool,
}
```

### Validation Rules

**Username:**
- 3-32 characters
- Alphanumeric + underscores
- Unique
- Regex: `^[a-z0-9_]{3,32}$`

**Email:**
- RFC 5322 compliant
- Unique
- Must contain `@` and domain

**Password (plain, before hashing):**
- Minimum 8 characters
- Recommended 12+ characters

---

## CSV Storage

**File:** `.reed/users.matrix.csv`

**Format:**
```csv
username|password|roles|firstname|lastname|street|city|postcode|region|country|email|mobile|twitter|facebook|tiktok|insta|youtube|whatsapp|desc|created_at|updated_at|last_login|is_active
admin|$argon2id$...|admin,editor,viewer|John|Doe|Main St 1|Berlin|10115|BE|DE|admin@example.com|+49123|@admin|||admin_official||+49123|Admin user|2025-01-01T00:00:00Z|2025-01-06T10:00:00Z|2025-01-06T10:30:00Z|true
```

**Matrix type:** Roles column contains comma-separated list

**See:** [Data Layer - CSV Architecture](../02-data-layer/csv-architecture.md#type-2-matrix-lists)

---

## Operations

### Create User

```rust
pub fn create_user(req: UserRequest) -> ReedResult<ReedResponse<User>> {
    // 1. Validate
    validate_username(&req.username)?;
    validate_email(&req.email)?;
    check_unique_username(&req.username)?;
    check_unique_email(&req.email)?;
    
    // 2. Hash password
    let password_hash = hash_password(&req.password)?;
    
    // 3. Create user struct
    let user = User {
        username: req.username,
        password_hash,
        email: req.email,
        created_at: current_timestamp(),
        updated_at: current_timestamp(),
        is_active: true,
        // ... other fields
    };
    
    // 4. Write to CSV
    append_to_csv(".reed/users.matrix.csv", &user)?;
    
    Ok(ReedResponse::new(user, "users::create"))
}
```

**CLI:**
```bash
reed user:create jdoe \
    --email john@example.com \
    --password secure123 \
    --firstname "John" \
    --lastname "Doe" \
    --role editor
```

**Performance:** ~150ms (includes Argon2 hashing)

### Get User

```rust
pub fn get_user(username: &str) -> ReedResult<ReedResponse<User>> {
    // Read CSV
    let users = read_csv(".reed/users.matrix.csv")?;
    
    // Find user
    let user = users.iter()
        .find(|u| u.username == username)
        .ok_or(ReedError::NotFound {
            key: username.to_string(),
            context: "users".to_string(),
        })?;
    
    Ok(ReedResponse::new(user.clone(), "users::get"))
}
```

**CLI:**
```bash
reed user:get jdoe
reed user:get jdoe --format json
```

**Performance:** < 10ms

### List Users

```rust
pub fn list_users(pattern: Option<String>) -> ReedResult<ReedResponse<Vec<User>>> {
    // Read CSV
    let mut users = read_csv(".reed/users.matrix.csv")?;
    
    // Apply filter
    if let Some(pat) = pattern {
        users.retain(|u| glob_match(&pat, &u.username));
    }
    
    Ok(ReedResponse::new(users, "users::list"))
}
```

**CLI:**
```bash
reed user:list
reed user:list "admin*"
reed user:list --format json
reed user:list --active-only
```

**Performance:** < 20ms for 1,000 users

### Update User

```rust
pub fn update_user(username: &str, updates: UserRequest) -> ReedResult<ReedResponse<User>> {
    // 1. Get existing user
    let mut user = get_user(username)?.data;
    
    // 2. Apply updates
    if let Some(email) = updates.email {
        validate_email(&email)?;
        user.email = email;
    }
    if let Some(firstname) = updates.firstname {
        user.firstname = Some(firstname);
    }
    // ... other fields
    
    user.updated_at = current_timestamp();
    
    // 3. Write to CSV (full rewrite)
    update_csv_record(".reed/users.matrix.csv", &user)?;
    
    Ok(ReedResponse::new(user, "users::update"))
}
```

**CLI:**
```bash
reed user:update jdoe --email newemail@example.com
reed user:update jdoe --city "Munich" --mobile "+49987654321"
reed user:update jdoe --active false
```

**Performance:** < 50ms

### Delete User

```rust
pub fn delete_user(username: &str) -> ReedResult<ReedResponse<()>> {
    // 1. Check user exists
    get_user(username)?;
    
    // 2. Remove from CSV
    remove_csv_record(".reed/users.matrix.csv", username)?;
    
    // 3. Remove role assignments
    remove_user_roles(username)?;
    
    Ok(ReedResponse::new((), "users::delete"))
}
```

**CLI:**
```bash
reed user:delete jdoe          # With confirmation
reed user:delete jdoe --force  # Skip confirmation
reed user:delete jdoe --dry-run  # Preview only
```

**Performance:** < 50ms

### Reset Password

```rust
pub fn reset_password(username: &str, new_password: &str) -> ReedResult<ReedResponse<()>> {
    // 1. Validate new password
    if new_password.len() < 8 {
        return Err(ReedError::ValidationError { /* ... */ });
    }
    
    // 2. Hash new password
    let new_hash = hash_password(new_password)?;
    
    // 3. Update user
    update_user(username, UserRequest {
        password_hash: Some(new_hash),
        ..Default::default()
    })?;
    
    Ok(ReedResponse::new((), "users::reset_password"))
}
```

**CLI:**
```bash
reed user:reset-password jdoe newpassword123

# Generate random password
NEW_PASS=$(openssl rand -base64 12)
reed user:reset-password jdoe "$NEW_PASS"
```

**Performance:** ~100ms (Argon2 hashing)

---

## Common Workflows

### Initial Admin Setup

```bash
# Create admin user
reed user:create admin \
    --email admin@example.com \
    --password SecureAdminPassword123 \
    --desc "System administrator"

# Assign admin role
reed role:assign admin admin

# Verify
reed user:get admin
```

### Bulk User Import

```bash
# From CSV
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
# List all users
reed user:list --format json > users_backup.json

# Find inactive users
reed user:list --inactive-only

# Export user data
reed user:export users-$(date +%Y%m%d).json
```

---

**See also:**
- [Password Hashing](password-hashing.md) - Argon2id details
- [Role System](role-system.md) - RBAC and permissions
- [CLI User Commands](../04-cli-layer/user-commands.md) - Complete CLI reference
