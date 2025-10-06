# Password Hashing with Argon2id

> RFC 9106 compliant password hashing with intentional slowdown

---

## Overview

ReedCMS uses Argon2id for password hashing, providing memory-hard, GPU-resistant protection against brute force and rainbow table attacks.

---

## Argon2id Configuration

### Parameters

```rust
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, Params,
};

let params = Params::new(
    19456,  // Memory cost: 19456 KiB (~19 MB)
    2,      // Time cost: 2 iterations
    1,      // Parallelism: 1 thread
    None    // Output length: 32 bytes (default)
)?;

let argon2 = Argon2::new(
    argon2::Algorithm::Argon2id,  // Hybrid mode
    argon2::Version::V0x13,       // Version 19
    params
);
```

### Why These Parameters?

**Memory: 19456 KiB**
- Fits in L3 cache of modern CPUs
- Too large for GPU parallel attacks
- Balances security and performance

**Iterations: 2**
- Sufficient with high memory cost
- Keeps hashing time ~100ms
- More iterations = longer time

**Parallelism: 1**
- Single-threaded hashing
- Predictable performance
- Easier to benchmark

**Result:** ~100ms per hash on average CPU

---

## Hash Generation

### Function

```rust
pub fn hash_password(password: &str) -> ReedResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(/* params */);
    
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ReedError::HashError {
            reason: format!("Failed to hash password: {}", e),
        })?
        .to_string();
    
    Ok(hash)
}
```

### PHC String Format

**Output:**
```
$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQxMjM0NTY=$Xm3kP7vQ8rL...
│        │     │            │                   │
│        │     │            │                   └─ Hash (Base64)
│        │     │            └───────────────────── Salt (Base64)
│        │     └────────────────────────────────── Parameters
│        └──────────────────────────────────────── Version
└───────────────────────────────────────────────── Algorithm
```

**Components:**
- Algorithm: `argon2id` (hybrid mode)
- Version: `v=19` (version 0x13)
- Parameters: `m=19456,t=2,p=1`
- Salt: 16 bytes, Base64-encoded
- Hash: 32 bytes, Base64-encoded

### Example

```rust
let password = "secure_password_123";
let hash = hash_password(password)?;

println!("{}", hash);
// $argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQxMjM0NTY=$Xm3kP7vQ8rL...
```

**Performance:** ~100ms

---

## Password Verification

### Function

```rust
pub fn verify_password(password: &str, hash: &str) -> ReedResult<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| ReedError::HashError {
            reason: format!("Invalid hash format: {}", e),
        })?;
    
    let argon2 = Argon2::new(/* params */);
    
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(password_hash::Error::Password) => Ok(false),
        Err(e) => Err(ReedError::HashError {
            reason: format!("Verification failed: {}", e),
        }),
    }
}
```

### Timing-Safe Comparison

**Built-in:** Argon2 uses constant-time comparison

**Why important:**
```rust
// ❌ BAD - Timing attack vulnerable
if stored_hash == computed_hash {
    return true;
}

// ✅ GOOD - Constant-time comparison (built into Argon2)
argon2.verify_password(password, &parsed_hash)
```

**Timing attack protection:**
- Comparison takes same time regardless of match
- Prevents attackers from learning partial matches

### Example

```rust
let password = "secure_password_123";
let hash = "$argon2id$v=19$m=19456,t=2,p=1$...";

let is_valid = verify_password(password, hash)?;

if is_valid {
    println!("Password correct!");
} else {
    println!("Password incorrect!");
}
```

**Performance:** ~100ms

---

## Security Properties

### Memory-Hard Function

**Resists GPU attacks:**
- GPUs have limited memory per core
- Argon2 requires 19 MB per hash
- GPU parallelism ineffective

**Comparison:**
```
CPU: 8 cores × 19 MB = 152 MB total
GPU: 1000 cores × 19 MB = 19 GB total (impractical)
```

### Rainbow Table Resistance

**Random salt per password:**
```rust
let salt1 = SaltString::generate(&mut OsRng); // Different
let salt2 = SaltString::generate(&mut OsRng); // Different
```

**Result:**
- Same password → Different hashes
- Pre-computed tables useless

**Example:**
```
Password: "password123"
Hash 1:   $argon2id$...$salt1$hash1
Hash 2:   $argon2id$...$salt2$hash2
```

### Intentional Slowdown

**~100ms per hash:**
- User login: Acceptable delay
- Brute force: 10 attempts/second maximum
- Dictionary attack (1M passwords): ~27 hours

**Without slowdown:**
- Modern CPU: ~1 billion SHA256/second
- Dictionary attack: Instant

---

## Integration

### User Creation

```rust
let password = "user_password";
let hash = hash_password(password)?;

let user = User {
    username: "jdoe".to_string(),
    password_hash: hash,
    // ... other fields
};

save_user(user)?;
```

### User Login

```rust
let username = "jdoe";
let password = "user_password";

// Fetch user from database
let user = get_user(username)?;

// Verify password
if verify_password(password, &user.password_hash)? {
    // Login successful
    create_session(user)?;
} else {
    // Login failed
    return Err(ReedError::AuthenticationFailed);
}
```

### Password Reset

```rust
let username = "jdoe";
let new_password = "new_secure_password";

// Hash new password
let new_hash = hash_password(new_password)?;

// Update user
update_user(username, UserUpdate {
    password_hash: Some(new_hash),
    ..Default::default()
})?;
```

---

## Best Practices

**Never store plain passwords:**
```rust
// ❌ NEVER do this
struct User {
    password: String,  // Plain password - NO!
}

// ✅ Always hash
struct User {
    password_hash: String,  // Argon2 hash - YES!
}
```

**Never log passwords:**
```rust
// ❌ BAD
println!("User password: {}", password);

// ✅ GOOD
println!("User authenticated: {}", username);
```

**Use strong passwords:**
```rust
// Minimum 8 characters (enforced)
if password.len() < 8 {
    return Err(ReedError::ValidationError { /* ... */ });
}

// Recommend 12+ characters
if password.len() < 12 {
    warn!("Password shorter than recommended 12 characters");
}
```

**Handle verification errors properly:**
```rust
// ❌ BAD - Reveals information
match verify_password(password, hash) {
    Ok(true) => "Correct password",
    Ok(false) => "Wrong password",
    Err(e) => format!("Error: {}", e),  // Leaks error details
}

// ✅ GOOD - Generic error
match verify_password(password, hash) {
    Ok(true) => "Login successful",
    _ => "Invalid credentials",  // Generic message
}
```

---

## Performance Benchmarks

### Hashing

| CPU | Time | Notes |
|-----|------|-------|
| Intel i7-9750H | ~95ms | Typical laptop |
| AMD Ryzen 9 5950X | ~75ms | High-end desktop |
| Intel Xeon E5-2680 | ~120ms | Server CPU |
| ARM Cortex-A72 | ~200ms | Raspberry Pi 4 |

**Target:** ~100ms average

### Verification

**Same as hashing:** ~100ms

**Why:** Must re-compute hash with same parameters

---

## Comparison with Other Algorithms

| Algorithm | GPU-Resistant | Rainbow-Resistant | Speed | Recommendation |
|-----------|---------------|-------------------|-------|----------------|
| **Argon2id** | ✅ Yes | ✅ Yes | ~100ms | ✅ **Recommended** (RFC 9106) |
| bcrypt | ⚠️ Partial | ✅ Yes | ~50ms | ✅ Acceptable |
| scrypt | ✅ Yes | ✅ Yes | ~80ms | ✅ Acceptable |
| PBKDF2 | ❌ No | ✅ Yes | ~10ms | ⚠️ Use high iterations |
| SHA256 | ❌ No | ❌ No | <1μs | ❌ Never for passwords |
| MD5 | ❌ No | ❌ No | <1μs | ❌ Never (broken) |

**ReedCMS choice:** Argon2id (industry best practice, RFC 9106)

---

## Troubleshooting

### Verification Always Fails

**Check hash format:**
```rust
// Valid PHC format starts with $argon2id$
if !hash.starts_with("$argon2id$") {
    eprintln!("Invalid hash format");
}
```

**Check version compatibility:**
```rust
// Ensure same Argon2 version
let parsed = PasswordHash::new(hash)?;
assert_eq!(parsed.version, Some(argon2::Version::V0x13));
```

### Hashing Too Slow

**Reduce parameters (not recommended):**
```rust
// Faster but less secure
let params = Params::new(
    16384,  // Reduced memory: 16 MB
    2,      // Same iterations
    1,      // Same parallelism
    None
)?;
```

**Better: Accept the slowdown** (security feature, not bug)

### Hashing Too Fast

**Check parameters:**
```rust
// Ensure correct settings
assert_eq!(params.m_cost(), 19456);
assert_eq!(params.t_cost(), 2);
```

---

**See also:**
- [User Management](user-management.md) - User CRUD operations
- [Authentication](authentication.md) - Login and session management
- [CLI User Commands](../04-cli-layer/user-commands.md) - User management CLI
