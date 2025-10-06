# Authentication

> Login, session management, and request authentication

---

## Overview

Authentication in ReedCMS validates user credentials using Argon2id password verification and manages user sessions for server and API access.

---

## Authentication Flow

```
┌──────────────────────────────────────────────────┐
│         Client Request (Login)                   │
│   POST /login                                    │
│   { username, password }                         │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│      1. Get User from Database                   │
│         get_user(username)                       │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│      2. Verify Password (Argon2id)               │
│         verify_password(password, hash)          │
│         ~100ms timing-safe verification          │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│      3. Create Session                           │
│         session_id = generate_secure_token()     │
│         sessions.insert(session_id, user)        │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│      4. Return Session Token                     │
│         { token: session_id }                    │
└──────────────────────────────────────────────────┘
```

---

## Password Verification

### Login Process

```rust
pub async fn login(
    username: String,
    password: String
) -> ReedResult<ReedResponse<Session>> {
    // 1. Get user
    let user = get_user(&username)
        .map_err(|_| ReedError::AuthenticationFailed {
            reason: "Invalid credentials".to_string(),
        })?;
    
    // 2. Check if active
    if !user.is_active {
        return Err(ReedError::AuthenticationFailed {
            reason: "Account inactive".to_string(),
        });
    }
    
    // 3. Verify password (~100ms)
    let is_valid = verify_password(&password, &user.password_hash)?;
    if !is_valid {
        return Err(ReedError::AuthenticationFailed {
            reason: "Invalid credentials".to_string(),
        });
    }
    
    // 4. Update last login
    update_last_login(&username)?;
    
    // 5. Create session
    let session = create_session(user)?;
    
    Ok(ReedResponse::new(session, "auth::login"))
}
```

**Performance:** ~100ms (Argon2 verification)

**Security:**
- Generic error messages (don't reveal if user exists)
- Timing-safe password comparison
- Account status check

---

## Session Management

### Session Structure

```rust
pub struct Session {
    pub session_id: String,      // Secure random token
    pub user: User,               // User data
    pub created_at: u64,          // Unix timestamp
    pub expires_at: u64,          // Unix timestamp
    pub ip_address: Option<String>,
}
```

### Session Creation

```rust
pub fn create_session(user: User) -> ReedResult<Session> {
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;
    
    // Generate secure random token (32 bytes)
    let session_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    let now = current_timestamp_seconds();
    
    let session = Session {
        session_id,
        user,
        created_at: now,
        expires_at: now + 3600, // 1 hour
        ip_address: None,
    };
    
    // Store in session cache
    SESSIONS.write().unwrap().insert(session.session_id.clone(), session.clone());
    
    Ok(session)
}
```

**Token format:** 32-character alphanumeric string  
**Expiration:** 1 hour (configurable)

### Session Validation

```rust
pub fn validate_session(session_id: &str) -> ReedResult<User> {
    let sessions = SESSIONS.read().unwrap();
    
    // Get session
    let session = sessions.get(session_id)
        .ok_or(ReedError::AuthenticationFailed {
            reason: "Invalid session".to_string(),
        })?;
    
    // Check expiration
    let now = current_timestamp_seconds();
    if now > session.expires_at {
        return Err(ReedError::AuthenticationFailed {
            reason: "Session expired".to_string(),
        });
    }
    
    Ok(session.user.clone())
}
```

### Session Storage

**In-memory (current):**
```rust
lazy_static! {
    static ref SESSIONS: RwLock<HashMap<String, Session>> = 
        RwLock::new(HashMap::new());
}
```

**Limitations:**
- Lost on server restart
- Not shared across server instances

**Future:** Persistent session storage (Redis, database)

---

## Middleware Integration

### HTTP Authentication

```rust
use actix_web::{HttpRequest, HttpResponse};

pub async fn authenticate(req: &HttpRequest) -> ReedResult<User> {
    // Get Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(ReedError::AuthenticationFailed {
            reason: "Missing Authorization header".to_string(),
        })?;
    
    // Parse Bearer token
    let auth_str = auth_header.to_str()
        .map_err(|_| ReedError::AuthenticationFailed {
            reason: "Invalid Authorization header".to_string(),
        })?;
    
    if !auth_str.starts_with("Bearer ") {
        return Err(ReedError::AuthenticationFailed {
            reason: "Invalid Authorization format".to_string(),
        });
    }
    
    let session_id = &auth_str[7..]; // Remove "Bearer "
    
    // Validate session
    validate_session(session_id)
}
```

### Usage in Routes

```rust
async fn protected_route(req: HttpRequest) -> Result<HttpResponse> {
    // Authenticate
    let user = authenticate(&req).await?;
    
    // Check permission
    if !has_permission(&user, "text[rw-]")? {
        return Err(ReedError::PermissionDenied { /* ... */ });
    }
    
    // Process request
    Ok(HttpResponse::Ok().json(/* ... */))
}
```

---

## API Authentication

### Bearer Token

**Header format:**
```
Authorization: Bearer <session_id>
```

**Example:**
```bash
curl -H "Authorization: Bearer abc123..." \
     https://example.com/api/v1/text/get?key=page.title
```

### API Key (Future)

**For service-to-service:**
```
X-API-Key: <api_key>
```

---

## Security Best Practices

**Use HTTPS in production:**
```
❌ http://example.com/login  # Credentials sent in clear text
✅ https://example.com/login # Encrypted connection
```

**Short session expiration:**
```rust
// ✅ Good - 1 hour
expires_at: now + 3600

// ⚠️  Acceptable - 24 hours (with renewal)
expires_at: now + 86400

// ❌ Bad - never expires
expires_at: u64::MAX
```

**Generic error messages:**
```rust
// ✅ Good
"Invalid credentials"

// ❌ Bad
"User 'admin' not found"
"Password incorrect for user 'jdoe'"
```

**Rate limiting (recommended):**
```rust
// Limit login attempts
if login_attempts > 5 {
    return Err(ReedError::RateLimitExceeded { /* ... */ });
}
```

---

## CLI Authentication

**CLI commands run as system user:**
- No authentication required
- Direct CSV file access
- Suitable for local administration

**Server commands require authentication:**
```bash
# These work without auth (local)
reed user:create admin --email ...
reed text:set page.title "Welcome"

# These require auth (remote API)
curl -H "Authorization: Bearer ..." \
     https://example.com/api/v1/users
```

---

## Common Workflows

### User Login

```rust
// 1. User submits credentials
let username = "jdoe";
let password = "user_password";

// 2. Login
let session = login(username.to_string(), password.to_string()).await?;

// 3. Store session token (client-side)
save_token(&session.session_id);

// 4. Use token for subsequent requests
let user = validate_session(&session.session_id)?;
```

### Session Renewal

```rust
pub fn renew_session(session_id: &str) -> ReedResult<Session> {
    let user = validate_session(session_id)?;
    create_session(user)
}
```

### Logout

```rust
pub fn logout(session_id: &str) -> ReedResult<()> {
    SESSIONS.write().unwrap().remove(session_id);
    Ok(())
}
```

---

## Troubleshooting

**"Invalid credentials" on correct password:**
- Check user exists: `reed user:get username`
- Check account active: `reed user:get username | grep is_active`
- Verify password was set: Check hash starts with `$argon2id$`

**Session expires too quickly:**
- Increase expiration time in `create_session()`
- Implement session renewal

**Sessions lost on restart:**
- Expected (in-memory storage)
- Implement persistent storage (future enhancement)

---

**See also:**
- [Password Hashing](password-hashing.md) - Argon2id details
- [User Management](user-management.md) - User operations
- [Role System](role-system.md) - Permission checks
- [Server Layer](../06-server-layer/) - HTTP server integration
