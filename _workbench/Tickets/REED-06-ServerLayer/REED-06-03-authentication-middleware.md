# REED-06-03: Authentication Middleware

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
- **ID**: REED-06-03
- **Title**: HTTP Authentication Middleware
- **Layer**: Server Layer (REED-06)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-06-01, REED-03-01

## Summary Reference
- **Section**: Authentication Middleware
- **Lines**: 1004-1007, 1445-1467 in project_summary.md
- **Key Concepts**: HTTP Basic Auth, token-based auth, role verification, secure credential handling

## Objective
Implement authentication middleware for Actix-Web with HTTP Basic Auth, token-based authentication, user verification via ReedBase, and role-based access control with secure session management.

## Requirements

### Authentication Methods

#### HTTP Basic Auth
```
Authorization: Basic base64(username:password)
```

#### Token-Based Auth
```
Authorization: Bearer {token}
```

### Implementation (`src/reedcms/auth/middleware.rs`)

```rust
/// Authentication middleware for Actix-Web.
///
/// ## Supported Authentication Methods
/// - HTTP Basic Auth (username:password)
/// - Bearer Token Auth
///
/// ## Process
/// 1. Extract Authorization header
/// 2. Parse authentication type
/// 3. Verify credentials against .reed/users.matrix.csv
/// 4. Validate password with Argon2
/// 5. Load user roles and permissions
/// 6. Inject authenticated user into request extensions
///
/// ## Performance
/// - Auth verification: < 100ms (Argon2 intentional slowdown)
/// - Role lookup: < 1ms (cached)
/// - Unauthorized rejection: < 5ms
///
/// ## Security
/// - Constant-time password comparison
/// - Failed login rate limiting
/// - Automatic session invalidation
pub struct AuthMiddleware {
    required_role: Option<String>,
    required_permission: Option<String>,
}

impl AuthMiddleware {
    /// Creates new authentication middleware.
    ///
    /// ## Arguments
    /// - required_role: Optional role requirement
    /// - required_permission: Optional permission requirement
    pub fn new(required_role: Option<String>, required_permission: Option<String>) -> Self {
        Self {
            required_role,
            required_permission,
        }
    }

    /// Creates middleware requiring no authentication.
    pub fn public() -> Self {
        Self {
            required_role: None,
            required_permission: None,
        }
    }

    /// Creates middleware requiring authentication only.
    pub fn authenticated() -> Self {
        Self {
            required_role: Some("user".to_string()),
            required_permission: None,
        }
    }

    /// Creates middleware requiring admin role.
    pub fn admin_only() -> Self {
        Self {
            required_role: Some("admin".to_string()),
            required_permission: None,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service,
            required_role: self.required_role.clone(),
            required_permission: self.required_permission.clone(),
        }))
    }
}

/// Authentication middleware service.
pub struct AuthMiddlewareService<S> {
    service: S,
    required_role: Option<String>,
    required_permission: Option<String>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let required_role = self.required_role.clone();
        let required_permission = self.required_permission.clone();

        // Extract authorization header
        let auth_result = extract_auth_credentials(&req);

        let fut = self.service.call(req);

        Box::pin(async move {
            match auth_result {
                Ok(credentials) => {
                    // Verify credentials
                    match verify_credentials(&credentials).await {
                        Ok(user) => {
                            // Check role requirement
                            if let Some(required) = required_role {
                                if !user.has_role(&required) {
                                    return Err(create_forbidden_error());
                                }
                            }

                            // Check permission requirement
                            if let Some(required) = required_permission {
                                if !user.has_permission(&required) {
                                    return Err(create_forbidden_error());
                                }
                            }

                            // Inject authenticated user into request
                            req.extensions_mut().insert(user);

                            fut.await
                        }
                        Err(_) => Err(create_unauthorized_error()),
                    }
                }
                Err(_) => {
                    // No authentication required
                    if required_role.is_none() && required_permission.is_none() {
                        fut.await
                    } else {
                        Err(create_unauthorized_error())
                    }
                }
            }
        })
    }
}
```

### Credential Extraction (`src/reedcms/auth/credentials.rs`)

```rust
/// Extracts authentication credentials from request.
///
/// ## Supported Headers
/// - Authorization: Basic {base64}
/// - Authorization: Bearer {token}
///
/// ## Process
/// 1. Get Authorization header
/// 2. Parse authentication type
/// 3. Extract credentials
/// 4. Decode if necessary
///
/// ## Output
/// - AuthCredentials enum (Basic or Bearer)
pub fn extract_auth_credentials(req: &ServiceRequest) -> ReedResult<AuthCredentials> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| ReedError::AuthError {
            component: "credentials".to_string(),
            reason: "Missing Authorization header".to_string(),
        })?;

    let auth_str = auth_header.to_str().map_err(|_| ReedError::AuthError {
        component: "credentials".to_string(),
        reason: "Invalid Authorization header format".to_string(),
    })?;

    // Parse authentication type
    if auth_str.starts_with("Basic ") {
        parse_basic_auth(auth_str)
    } else if auth_str.starts_with("Bearer ") {
        parse_bearer_auth(auth_str)
    } else {
        Err(ReedError::AuthError {
            component: "credentials".to_string(),
            reason: "Unsupported authentication type".to_string(),
        })
    }
}

/// Parses HTTP Basic Auth credentials.
///
/// ## Format
/// Authorization: Basic base64(username:password)
///
/// ## Process
/// 1. Extract base64 part
/// 2. Decode base64
/// 3. Split by colon
/// 4. Return username and password
///
/// ## Example
/// - Input: "Basic dXNlcjpwYXNz"
/// - Output: ("user", "pass")
fn parse_basic_auth(auth_str: &str) -> ReedResult<AuthCredentials> {
    let encoded = auth_str
        .strip_prefix("Basic ")
        .ok_or_else(|| ReedError::AuthError {
            component: "basic_auth".to_string(),
            reason: "Invalid Basic auth format".to_string(),
        })?;

    let decoded = base64::decode(encoded).map_err(|_| ReedError::AuthError {
        component: "basic_auth".to_string(),
        reason: "Invalid base64 encoding".to_string(),
    })?;

    let decoded_str = String::from_utf8(decoded).map_err(|_| ReedError::AuthError {
        component: "basic_auth".to_string(),
        reason: "Invalid UTF-8 in credentials".to_string(),
    })?;

    let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(ReedError::AuthError {
            component: "basic_auth".to_string(),
            reason: "Invalid credentials format (expected username:password)".to_string(),
        });
    }

    Ok(AuthCredentials::Basic {
        username: parts[0].to_string(),
        password: parts[1].to_string(),
    })
}

/// Parses Bearer token authentication.
///
/// ## Format
/// Authorization: Bearer {token}
///
/// ## Process
/// 1. Extract token part
/// 2. Validate token format
/// 3. Return token
fn parse_bearer_auth(auth_str: &str) -> ReedResult<AuthCredentials> {
    let token = auth_str
        .strip_prefix("Bearer ")
        .ok_or_else(|| ReedError::AuthError {
            component: "bearer_auth".to_string(),
            reason: "Invalid Bearer auth format".to_string(),
        })?;

    if token.is_empty() {
        return Err(ReedError::AuthError {
            component: "bearer_auth".to_string(),
            reason: "Empty token".to_string(),
        });
    }

    Ok(AuthCredentials::Bearer {
        token: token.to_string(),
    })
}

/// Authentication credentials enum.
#[derive(Debug, Clone)]
pub enum AuthCredentials {
    Basic {
        username: String,
        password: String,
    },
    Bearer {
        token: String,
    },
}
```

### Credential Verification (`src/reedcms/auth/verification.rs`)

```rust
/// Verifies authentication credentials.
///
/// ## Process (Basic Auth)
/// 1. Lookup user in .reed/users.matrix.csv
/// 2. Verify password with Argon2
/// 3. Load user roles
/// 4. Create authenticated user object
///
/// ## Process (Bearer Token)
/// 1. Lookup token in .reed/sessions.csv
/// 2. Verify token expiration
/// 3. Load associated user
/// 4. Create authenticated user object
///
/// ## Performance
/// - Basic auth: ~100ms (Argon2 verification)
/// - Bearer token: < 10ms (session lookup)
///
/// ## Security
/// - Rate limiting on failed attempts
/// - Constant-time password comparison via Argon2
/// - Automatic session invalidation on logout
pub async fn verify_credentials(credentials: &AuthCredentials) -> ReedResult<AuthenticatedUser> {
    match credentials {
        AuthCredentials::Basic { username, password } => {
            verify_basic_credentials(username, password).await
        }
        AuthCredentials::Bearer { token } => {
            verify_bearer_token(token).await
        }
    }
}

/// Verifies HTTP Basic Auth credentials.
async fn verify_basic_credentials(username: &str, password: &str) -> ReedResult<AuthenticatedUser> {
    // Check rate limiting
    if is_rate_limited(username) {
        return Err(ReedError::AuthError {
            component: "rate_limit".to_string(),
            reason: "Too many failed login attempts".to_string(),
        });
    }

    // Load user from ReedBase
    let user_data = load_user_data(username)?;

    // Verify password with Argon2
    if !verify_password_hash(&user_data.password_hash, password)? {
        record_failed_login(username);
        return Err(ReedError::AuthError {
            component: "password".to_string(),
            reason: "Invalid credentials".to_string(),
        });
    }

    // Clear failed login counter
    clear_failed_logins(username);

    // Load user roles
    let roles = load_user_roles(&user_data.id)?;

    Ok(AuthenticatedUser {
        id: user_data.id,
        username: user_data.username,
        email: user_data.email,
        roles,
    })
}

/// Verifies bearer token.
async fn verify_bearer_token(token: &str) -> ReedResult<AuthenticatedUser> {
    // Lookup session in .reed/sessions.csv
    let session = load_session(token)?;

    // Check expiration
    if session.is_expired() {
        return Err(ReedError::AuthError {
            component: "session".to_string(),
            reason: "Session expired".to_string(),
        });
    }

    // Load user
    let user_data = load_user_by_id(&session.user_id)?;
    let roles = load_user_roles(&session.user_id)?;

    Ok(AuthenticatedUser {
        id: user_data.id,
        username: user_data.username,
        email: user_data.email,
        roles,
    })
}

/// Verifies password hash using Argon2.
fn verify_password_hash(hash: &str, password: &str) -> ReedResult<bool> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let parsed_hash = PasswordHash::new(hash).map_err(|e| ReedError::AuthError {
        component: "argon2".to_string(),
        reason: format!("Invalid password hash: {}", e),
    })?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Authenticated user structure.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

impl AuthenticatedUser {
    /// Checks if user has specific role.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Checks if user has specific permission.
    pub fn has_permission(&self, permission: &str) -> bool {
        // Load role permissions and check
        for role in &self.roles {
            if role_has_permission(role, permission) {
                return true;
            }
        }
        false
    }
}
```

### Rate Limiting (`src/reedcms/auth/rate_limit.rs`)

```rust
/// Checks if user is rate limited.
///
/// ## Rate Limit Rules
/// - 5 failed attempts: 1 minute lockout
/// - 10 failed attempts: 5 minute lockout
/// - 20 failed attempts: 30 minute lockout
///
/// ## Implementation
/// - In-memory HashMap with username → attempt count
/// - Cleanup thread removes expired entries
///
/// ## Performance
/// - Check: < 1μs (HashMap lookup)
pub fn is_rate_limited(username: &str) -> bool {
    let limits = get_rate_limit_store();
    let mut store = limits.write().unwrap();

    if let Some(attempts) = store.get(username) {
        let lockout_until = calculate_lockout_time(attempts.count);
        if attempts.last_attempt + lockout_until > std::time::SystemTime::now() {
            return true;
        }
    }

    false
}

/// Records failed login attempt.
pub fn record_failed_login(username: &str) {
    let limits = get_rate_limit_store();
    let mut store = limits.write().unwrap();

    store
        .entry(username.to_string())
        .and_modify(|a| {
            a.count += 1;
            a.last_attempt = std::time::SystemTime::now();
        })
        .or_insert(LoginAttempt {
            count: 1,
            last_attempt: std::time::SystemTime::now(),
        });
}

/// Clears failed login counter for user.
pub fn clear_failed_logins(username: &str) {
    let limits = get_rate_limit_store();
    let mut store = limits.write().unwrap();
    store.remove(username);
}

/// Calculates lockout duration based on attempt count.
fn calculate_lockout_time(attempts: u32) -> std::time::Duration {
    match attempts {
        0..=4 => std::time::Duration::from_secs(0),
        5..=9 => std::time::Duration::from_secs(60),       // 1 minute
        10..=19 => std::time::Duration::from_secs(300),    // 5 minutes
        _ => std::time::Duration::from_secs(1800),         // 30 minutes
    }
}

/// Login attempt tracking structure.
#[derive(Debug, Clone)]
struct LoginAttempt {
    count: u32,
    last_attempt: std::time::SystemTime,
}

/// Gets rate limit store (singleton).
fn get_rate_limit_store() -> &'static RwLock<HashMap<String, LoginAttempt>> {
    use std::sync::OnceLock;
    static STORE: OnceLock<RwLock<HashMap<String, LoginAttempt>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(HashMap::new()))
}
```

### Error Responses (`src/reedcms/auth/errors.rs`)

```rust
/// Creates 401 Unauthorized error response.
fn create_unauthorized_error() -> Error {
    actix_web::error::ErrorUnauthorized(
        serde_json::json!({
            "error": "Unauthorized",
            "message": "Authentication required",
            "status": 401
        })
        .to_string(),
    )
}

/// Creates 403 Forbidden error response.
fn create_forbidden_error() -> Error {
    actix_web::error::ErrorForbidden(
        serde_json::json!({
            "error": "Forbidden",
            "message": "Insufficient permissions",
            "status": 403
        })
        .to_string(),
    )
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/auth/middleware.rs` - Authentication middleware
- `src/reedcms/auth/credentials.rs` - Credential extraction
- `src/reedcms/auth/verification.rs` - Credential verification
- `src/reedcms/auth/rate_limit.rs` - Rate limiting
- `src/reedcms/auth/errors.rs` - Error responses

### Test Files
- `src/reedcms/auth/middleware.test.rs`
- `src/reedcms/auth/credentials.test.rs`
- `src/reedcms/auth/verification.test.rs`
- `src/reedcms/auth/rate_limit.test.rs`

## File Structure
```
src/reedcms/auth/
├── middleware.rs          # Auth middleware
├── middleware.test.rs     # Middleware tests
├── credentials.rs         # Credential extraction
├── credentials.test.rs    # Credential tests
├── verification.rs        # Verification logic
├── verification.test.rs   # Verification tests
├── rate_limit.rs          # Rate limiting
├── rate_limit.test.rs     # Rate limit tests
├── errors.rs              # Error responses
└── errors.test.rs         # Error tests
```

## Testing Requirements

### Unit Tests
- [ ] Test credential extraction from Authorization header
- [ ] Test Basic auth parsing and decoding
- [ ] Test Bearer token parsing
- [ ] Test password verification with Argon2
- [ ] Test rate limiting logic
- [ ] Test lockout duration calculation
- [ ] Test user role checking
- [ ] Test permission verification

### Integration Tests
- [ ] Test complete authentication flow
- [ ] Test middleware with valid credentials
- [ ] Test middleware with invalid credentials
- [ ] Test role-based access control
- [ ] Test rate limiting with repeated failures
- [ ] Test session token authentication
- [ ] Test 401/403 error responses

### Security Tests
- [ ] Test constant-time password comparison
- [ ] Test SQL injection resistance (N/A for CSV)
- [ ] Test timing attack resistance
- [ ] Test rate limiting effectiveness
- [ ] Test session expiration
- [ ] Test malformed header handling

### Performance Tests
- [ ] Basic auth verification: < 100ms
- [ ] Bearer token verification: < 10ms
- [ ] Rate limit check: < 1μs
- [ ] Middleware overhead: < 5ms

## Acceptance Criteria
- [ ] HTTP Basic Auth working
- [ ] Bearer token auth implemented
- [ ] Argon2 password verification functional
- [ ] Rate limiting active with progressive lockout
- [ ] Role-based access control working
- [ ] Permission checking implemented
- [ ] 401 Unauthorized responses correct
- [ ] 403 Forbidden responses correct
- [ ] Session management functional
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-06-01 (Server Foundation), REED-03-01 (User Management), REED-03-02 (Role System)

## Blocks
- REED-06-04 (Response Builder may need auth context)
- REED-07-02 (API Security Matrix needs auth middleware)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1004-1007, 1445-1467 in `project_summary.md`

## Notes
Authentication middleware is critical for securing ReedCMS. HTTP Basic Auth provides simple username/password authentication suitable for API access. Bearer tokens enable session-based authentication for web interfaces. Rate limiting prevents brute-force attacks by progressively increasing lockout duration. Argon2 verification is intentionally slow (~100ms) to resist password cracking. All authentication logic must use constant-time comparisons to prevent timing attacks.
