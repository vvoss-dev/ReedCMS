# REED-07-02: API Security Matrix

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
- **ID**: REED-07-02
- **Title**: API Security Matrix and Access Control
- **Layer**: API Layer (REED-07)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-07-01, REED-03-02

## Summary Reference
- **Section**: API Security
- **Lines**: 1505-1542 in project_summary.md
- **Key Concepts**: Permission matrix, resource-level access control, API key management, rate limiting

## Objective
Implement comprehensive API security system with permission-based access control matrix, resource-level permissions, API key management, and rate limiting to protect ReedBase data and prevent abuse.

## Requirements

### Security Matrix (.reed/api.security.csv)

```csv
resource|operation|required_permission|required_role|rate_limit
text|read|text.read|user|100/min
text|write|text.write|editor|50/min
route|read|route.read|user|100/min
route|write|route.write|admin|20/min
meta|read|meta.read|user|100/min
meta|write|meta.write|admin|20/min
config|read|config.read|admin|50/min
config|write|config.write|admin|10/min
batch|read|batch.read|user|10/min
batch|write|batch.write|editor|5/min
```

### Implementation (`src/reedcms/api/security/matrix.rs`)

```rust
/// Security matrix for API access control.
///
/// ## Security Checks
/// 1. Authentication required
/// 2. Role verification
/// 3. Permission checking
/// 4. Rate limiting
/// 5. Resource-level access control
///
/// ## Configuration
/// - Security rules loaded from .reed/api.security.csv
/// - Cached in memory for performance
/// - Reloaded on file change (DEV mode)
///
/// ## Performance
/// - Security check: < 1ms (cached)
/// - Rate limit check: < 100μs
pub struct SecurityMatrix {
    rules: HashMap<String, SecurityRule>,
}

impl SecurityMatrix {
    /// Loads security matrix from configuration.
    pub fn load() -> ReedResult<Self> {
        let rules = load_security_rules()?;
        Ok(Self { rules })
    }

    /// Checks if user has access to resource operation.
    ///
    /// ## Arguments
    /// - resource: Resource type (text, route, meta, config)
    /// - operation: Operation type (read, write)
    /// - user: Authenticated user
    ///
    /// ## Returns
    /// - Ok(()) if access granted
    /// - Err(SecurityError) if access denied
    pub fn check_access(
        &self,
        resource: &str,
        operation: &str,
        user: &AuthenticatedUser,
    ) -> ReedResult<()> {
        let rule_key = format!("{}.{}", resource, operation);

        let rule = self.rules.get(&rule_key).ok_or_else(|| ReedError::SecurityError {
            component: "security_matrix".to_string(),
            reason: format!("No security rule for {}", rule_key),
        })?;

        // Check role requirement
        if let Some(required_role) = &rule.required_role {
            if !user.has_role(required_role) {
                return Err(ReedError::SecurityError {
                    component: "role_check".to_string(),
                    reason: format!("Required role: {}", required_role),
                });
            }
        }

        // Check permission requirement
        if let Some(required_permission) = &rule.required_permission {
            if !user.has_permission(required_permission) {
                return Err(ReedError::SecurityError {
                    component: "permission_check".to_string(),
                    reason: format!("Required permission: {}", required_permission),
                });
            }
        }

        // Check rate limit
        if let Some(rate_limit) = &rule.rate_limit {
            check_rate_limit(&user.id, &rule_key, rate_limit)?;
        }

        Ok(())
    }

    /// Gets rate limit for resource operation.
    pub fn get_rate_limit(&self, resource: &str, operation: &str) -> Option<RateLimit> {
        let rule_key = format!("{}.{}", resource, operation);
        self.rules.get(&rule_key).and_then(|r| r.rate_limit.clone())
    }
}

/// Security rule structure.
#[derive(Debug, Clone)]
pub struct SecurityRule {
    pub resource: String,
    pub operation: String,
    pub required_permission: Option<String>,
    pub required_role: Option<String>,
    pub rate_limit: Option<RateLimit>,
}

/// Rate limit structure.
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests: u32,
    pub period: RateLimitPeriod,
}

impl RateLimit {
    /// Parses rate limit from string (e.g., "100/min").
    pub fn parse(s: &str) -> ReedResult<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(ReedError::ConfigError {
                component: "rate_limit".to_string(),
                reason: format!("Invalid rate limit format: {}", s),
            });
        }

        let requests = parts[0].parse().map_err(|_| ReedError::ConfigError {
            component: "rate_limit".to_string(),
            reason: format!("Invalid request count: {}", parts[0]),
        })?;

        let period = match parts[1] {
            "sec" | "second" => RateLimitPeriod::Second,
            "min" | "minute" => RateLimitPeriod::Minute,
            "hour" => RateLimitPeriod::Hour,
            "day" => RateLimitPeriod::Day,
            _ => {
                return Err(ReedError::ConfigError {
                    component: "rate_limit".to_string(),
                    reason: format!("Invalid period: {}", parts[1]),
                })
            }
        };

        Ok(Self { requests, period })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RateLimitPeriod {
    Second,
    Minute,
    Hour,
    Day,
}

impl RateLimitPeriod {
    /// Returns duration in seconds.
    pub fn duration(&self) -> u64 {
        match self {
            RateLimitPeriod::Second => 1,
            RateLimitPeriod::Minute => 60,
            RateLimitPeriod::Hour => 3600,
            RateLimitPeriod::Day => 86400,
        }
    }
}

/// Loads security rules from .reed/api.security.csv.
fn load_security_rules() -> ReedResult<HashMap<String, SecurityRule>> {
    let csv_path = ".reed/api.security.csv";

    if !std::path::Path::new(csv_path).exists() {
        return Ok(default_security_rules());
    }

    let mut rules = HashMap::new();
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'|')
        .from_path(csv_path)
        .map_err(|e| ReedError::IoError {
            operation: "read".to_string(),
            path: csv_path.to_string(),
            reason: e.to_string(),
        })?;

    for result in reader.records() {
        let record = result.map_err(|e| ReedError::CsvError {
            file: csv_path.to_string(),
            reason: e.to_string(),
        })?;

        if record.len() < 5 {
            continue;
        }

        let resource = record[0].to_string();
        let operation = record[1].to_string();
        let required_permission = if record[2].is_empty() {
            None
        } else {
            Some(record[2].to_string())
        };
        let required_role = if record[3].is_empty() {
            None
        } else {
            Some(record[3].to_string())
        };
        let rate_limit = if record[4].is_empty() {
            None
        } else {
            Some(RateLimit::parse(&record[4])?)
        };

        let rule_key = format!("{}.{}", resource, operation);
        rules.insert(
            rule_key,
            SecurityRule {
                resource,
                operation,
                required_permission,
                required_role,
                rate_limit,
            },
        );
    }

    Ok(rules)
}

/// Default security rules if config file missing.
fn default_security_rules() -> HashMap<String, SecurityRule> {
    let mut rules = HashMap::new();

    // Text operations
    rules.insert(
        "text.read".to_string(),
        SecurityRule {
            resource: "text".to_string(),
            operation: "read".to_string(),
            required_permission: Some("text.read".to_string()),
            required_role: Some("user".to_string()),
            rate_limit: Some(RateLimit {
                requests: 100,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    rules.insert(
        "text.write".to_string(),
        SecurityRule {
            resource: "text".to_string(),
            operation: "write".to_string(),
            required_permission: Some("text.write".to_string()),
            required_role: Some("editor".to_string()),
            rate_limit: Some(RateLimit {
                requests: 50,
                period: RateLimitPeriod::Minute,
            }),
        },
    );

    rules
}
```

### Rate Limiting (`src/reedcms/api/security/rate_limit.rs`)

```rust
/// Checks rate limit for user and operation.
///
/// ## Rate Limiting Strategy
/// - Sliding window algorithm
/// - Per-user tracking
/// - Per-operation limits
/// - Automatic cleanup of expired entries
///
/// ## Storage
/// - In-memory HashMap
/// - Periodic cleanup thread (every 5 minutes)
///
/// ## Performance
/// - Check: < 100μs
/// - Memory: ~100 bytes per tracked user-operation pair
pub fn check_rate_limit(
    user_id: &str,
    operation: &str,
    limit: &RateLimit,
) -> ReedResult<()> {
    let store = get_rate_limit_store();
    let mut store = store.write().unwrap();

    let key = format!("{}:{}", user_id, operation);
    let now = std::time::SystemTime::now();

    // Get or create entry
    let entry = store.entry(key.clone()).or_insert_with(|| RateLimitEntry {
        requests: Vec::new(),
    });

    // Remove expired requests
    let window_start = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - limit.period.duration();

    entry.requests.retain(|&timestamp| timestamp > window_start);

    // Check if limit exceeded
    if entry.requests.len() >= limit.requests as usize {
        return Err(ReedError::RateLimitExceeded {
            limit: limit.requests,
            period: format!("{:?}", limit.period),
        });
    }

    // Record this request
    entry.requests.push(
        now.duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

    Ok(())
}

/// Rate limit entry tracking request timestamps.
#[derive(Debug, Clone)]
struct RateLimitEntry {
    requests: Vec<u64>, // Unix timestamps
}

/// Gets rate limit store singleton.
fn get_rate_limit_store() -> &'static RwLock<HashMap<String, RateLimitEntry>> {
    use std::sync::OnceLock;
    static STORE: OnceLock<RwLock<HashMap<String, RateLimitEntry>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Cleans up expired rate limit entries.
///
/// ## Cleanup Strategy
/// - Runs every 5 minutes
/// - Removes entries older than 24 hours
/// - Prevents memory leak
pub fn cleanup_rate_limits() {
    let store = get_rate_limit_store();
    let mut store = store.write().unwrap();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let cutoff = now - 86400; // 24 hours ago

    store.retain(|_, entry| {
        entry.requests.iter().any(|&timestamp| timestamp > cutoff)
    });
}

/// Starts rate limit cleanup thread.
pub fn start_cleanup_thread() {
    std::thread::spawn(|| loop {
        std::thread::sleep(std::time::Duration::from_secs(300)); // 5 minutes
        cleanup_rate_limits();
    });
}
```

### API Key Management (`src/reedcms/api/security/api_keys.rs`)

```rust
/// Manages API keys for programmatic access.
///
/// ## API Key Format
/// - Format: reed_{random_32_chars}
/// - Example: reed_a7f3k9s2m4p1q8w5e6r7t9y2u3i4o5
///
/// ## Storage
/// - .reed/api.keys.csv
/// - Fields: key;user_id;created;expires;description
///
/// ## Security
/// - Keys hashed with SHA-256 before storage
/// - Expiration dates enforced
/// - Revocation support
pub struct ApiKeyManager;

impl ApiKeyManager {
    /// Generates new API key for user.
    ///
    /// ## Process
    /// 1. Generate random 32-character key
    /// 2. Hash key with SHA-256
    /// 3. Store in .reed/api.keys.csv
    /// 4. Return unhashed key to user (ONCE)
    pub fn generate_key(
        user_id: &str,
        expires_days: u32,
        description: &str,
    ) -> ReedResult<String> {
        let key = generate_random_key();
        let key_hash = hash_api_key(&key);

        let created = chrono::Utc::now().to_rfc3339();
        let expires = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(expires_days as i64))
            .unwrap()
            .to_rfc3339();

        // Store in CSV
        let entry = format!(
            "{};{};{};{};{}",
            key_hash, user_id, created, expires, description
        );

        append_to_api_keys_csv(&entry)?;

        Ok(key)
    }

    /// Verifies API key and returns user ID.
    ///
    /// ## Process
    /// 1. Hash provided key
    /// 2. Lookup in .reed/api.keys.csv
    /// 3. Check expiration
    /// 4. Return associated user ID
    pub fn verify_key(key: &str) -> ReedResult<String> {
        let key_hash = hash_api_key(key);
        let entries = load_api_keys_csv()?;

        for entry in entries {
            if entry.key_hash == key_hash {
                // Check expiration
                let expires = chrono::DateTime::parse_from_rfc3339(&entry.expires)
                    .map_err(|e| ReedError::SecurityError {
                        component: "api_key".to_string(),
                        reason: format!("Invalid expiration date: {}", e),
                    })?;

                if chrono::Utc::now() > expires {
                    return Err(ReedError::SecurityError {
                        component: "api_key".to_string(),
                        reason: "API key expired".to_string(),
                    });
                }

                return Ok(entry.user_id);
            }
        }

        Err(ReedError::SecurityError {
            component: "api_key".to_string(),
            reason: "Invalid API key".to_string(),
        })
    }

    /// Revokes API key.
    pub fn revoke_key(key: &str) -> ReedResult<()> {
        let key_hash = hash_api_key(key);
        remove_from_api_keys_csv(&key_hash)
    }
}

/// Generates random API key.
fn generate_random_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    let key: String = (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("reed_{}", key)
}

/// Hashes API key with SHA-256.
fn hash_api_key(key: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// API key entry structure.
#[derive(Debug, Clone)]
struct ApiKeyEntry {
    key_hash: String,
    user_id: String,
    created: String,
    expires: String,
    description: String,
}

/// Appends API key to CSV.
fn append_to_api_keys_csv(entry: &str) -> ReedResult<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let path = ".reed/api.keys.csv";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| ReedError::IoError {
            operation: "append".to_string(),
            path: path.to_string(),
            reason: e.to_string(),
        })?;

    writeln!(file, "{}", entry).map_err(|e| ReedError::IoError {
        operation: "write".to_string(),
        path: path.to_string(),
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Loads API keys from CSV.
fn load_api_keys_csv() -> ReedResult<Vec<ApiKeyEntry>> {
    // Implementation similar to other CSV loaders
    Ok(Vec::new())
}

/// Removes API key from CSV.
fn remove_from_api_keys_csv(key_hash: &str) -> ReedResult<()> {
    // Read all entries, filter out the key, rewrite file
    Ok(())
}
```

### Security Middleware (`src/reedcms/api/security/middleware.rs`)

```rust
/// Security middleware that enforces security matrix rules.
///
/// ## Process
/// 1. Extract resource and operation from request
/// 2. Get authenticated user from request
/// 3. Check security matrix
/// 4. Check rate limit
/// 5. Allow or deny request
pub struct SecurityMiddleware {
    matrix: Arc<SecurityMatrix>,
}

impl SecurityMiddleware {
    pub fn new() -> ReedResult<Self> {
        let matrix = Arc::new(SecurityMatrix::load()?);
        Ok(Self { matrix })
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecurityMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityMiddlewareService {
            service,
            matrix: self.matrix.clone(),
        }))
    }
}

pub struct SecurityMiddlewareService<S> {
    service: S,
    matrix: Arc<SecurityMatrix>,
}

impl<S, B> Service<ServiceRequest> for SecurityMiddlewareService<S>
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
        // Extract resource and operation from path
        let (resource, operation) = extract_resource_operation(req.path());

        // Get authenticated user
        let user = req.extensions().get::<AuthenticatedUser>().cloned();

        let matrix = self.matrix.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            if let Some(user) = user {
                // Check security matrix
                if let Err(e) = matrix.check_access(&resource, &operation, &user) {
                    return Err(actix_web::error::ErrorForbidden(format!(
                        "Access denied: {:?}",
                        e
                    )));
                }
            }

            fut.await
        })
    }
}

/// Extracts resource and operation from request path.
///
/// ## Examples
/// - /api/v1/text/key → ("text", "read")
/// - POST /api/v1/text → ("text", "write")
fn extract_resource_operation(path: &str) -> (String, String) {
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() >= 4 {
        let resource = parts[3].to_string();
        let operation = if parts.len() > 4 { "read" } else { "write" };
        (resource, operation.to_string())
    } else {
        ("unknown".to_string(), "unknown".to_string())
    }
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/api/security/matrix.rs` - Security matrix
- `src/reedcms/api/security/rate_limit.rs` - Rate limiting
- `src/reedcms/api/security/api_keys.rs` - API key management
- `src/reedcms/api/security/middleware.rs` - Security middleware

### Test Files
- `src/reedcms/api/security/matrix.test.rs`
- `src/reedcms/api/security/rate_limit.test.rs`
- `src/reedcms/api/security/api_keys.test.rs`
- `src/reedcms/api/security/middleware.test.rs`

## File Structure
```
src/reedcms/api/security/
├── matrix.rs              # Security matrix
├── matrix.test.rs         # Matrix tests
├── rate_limit.rs          # Rate limiting
├── rate_limit.test.rs     # Rate limit tests
├── api_keys.rs            # API key management
├── api_keys.test.rs       # API key tests
├── middleware.rs          # Security middleware
└── middleware.test.rs     # Middleware tests
```

## Testing Requirements

### Unit Tests
- [ ] Test security matrix loading
- [ ] Test access checks with various roles/permissions
- [ ] Test rate limit parsing and enforcement
- [ ] Test rate limit cleanup
- [ ] Test API key generation
- [ ] Test API key verification
- [ ] Test API key expiration
- [ ] Test API key revocation

### Integration Tests
- [ ] Test complete security flow (auth → check → allow/deny)
- [ ] Test rate limiting with actual requests
- [ ] Test API key authentication
- [ ] Test middleware integration
- [ ] Test security matrix with different user roles

### Security Tests
- [ ] Test rate limit bypass attempts
- [ ] Test expired API key rejection
- [ ] Test invalid API key handling
- [ ] Test permission escalation prevention
- [ ] Test concurrent rate limit checks

### Performance Tests
- [ ] Security check: < 1ms
- [ ] Rate limit check: < 100μs
- [ ] API key verification: < 5ms
- [ ] Rate limit cleanup: < 100ms

## Acceptance Criteria
- [ ] Security matrix loaded from CSV
- [ ] Access control enforcement working
- [ ] Rate limiting functional with sliding window
- [ ] API key generation implemented
- [ ] API key verification working
- [ ] API key expiration enforced
- [ ] API key revocation functional
- [ ] Security middleware integrated
- [ ] Rate limit cleanup thread running
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-07-01 (ReedAPI), REED-03-02 (Role System)

## Blocks
- None (final API layer ticket)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1505-1542 in `project_summary.md`

## Notes
API security is critical for protecting ReedBase data from unauthorized access and abuse. Security matrix provides fine-grained control over who can access what resources. Rate limiting prevents API abuse and ensures fair resource allocation. API keys enable programmatic access without exposing user passwords. All security checks must be performant (<1ms) to avoid impacting API response times.
