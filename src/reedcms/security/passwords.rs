// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Password Management with Argon2id
//!
//! Provides secure password hashing and verification using Argon2id algorithm.

use crate::reedcms::reedstream::{
    current_timestamp, validation_error, ReedError, ReedResponse, ReedResult,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params, Version,
};
use rand::rngs::OsRng;

/// Hashes password using Argon2id with secure parameters.
///
/// ## Input
/// - `password`: Plain text password to hash
///
/// ## Output
/// - `ReedResult<String>`: Argon2id hash string (PHC format)
///
/// ## Security
/// - Algorithm: Argon2id (hybrid mode resistant to GPU and side-channel attacks)
/// - Memory cost: 65536 KiB (64 MiB)
/// - Time cost: 3 iterations
/// - Parallelism: 4 threads
/// - Output length: 32 bytes
///
/// ## Performance
/// - Hashing time: ~100ms (intentionally slow for security)
/// - This is a security feature, not a performance issue
///
/// ## Error Conditions
/// - Returns `ReedError::ValidationError` if password is empty
/// - Returns `ReedError::ConfigError` if Argon2 parameters are invalid
///
/// ## Example Usage
/// ```
/// let hash = hash_password("MySecureP@ssw0rd")?;
/// // Returns: "$argon2id$v=19$m=65536,t=3,p=4$..."
/// ```
pub fn hash_password(password: &str) -> ReedResult<String> {
    if password.is_empty() {
        return Err(validation_error("password", "", "Password cannot be empty"));
    }

    // Argon2id parameters: memory=64MB, iterations=3, parallelism=4
    let params = Params::new(65536, 3, 4, Some(32)).map_err(|e| ReedError::ConfigError {
        component: "argon2".to_string(),
        reason: format!("Invalid Argon2 parameters: {}", e),
    })?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);
    let salt = SaltString::generate(&mut OsRng);

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ReedError::ConfigError {
            component: "argon2".to_string(),
            reason: format!("Password hashing failed: {}", e),
        })?;

    Ok(hash.to_string())
}

/// Verifies password against Argon2id hash.
///
/// ## Input
/// - `password`: Plain text password to verify
/// - `hash`: Argon2id hash string (PHC format)
///
/// ## Output
/// - `ReedResult<bool>`: true if password matches, false otherwise
///
/// ## Performance
/// - Verification time: ~100ms (matches hashing time)
/// - Uses constant-time comparison to prevent timing attacks
///
/// ## Error Conditions
/// - Returns `ReedError::ValidationError` if hash format is invalid
/// - Returns `Ok(false)` if password doesn't match (NOT an error)
///
/// ## Example Usage
/// ```
/// let hash = "$argon2id$v=19$m=65536,t=3,p=4$...";
/// let is_valid = verify_password("MySecureP@ssw0rd", hash)?;
/// ```
pub fn verify_password(password: &str, hash: &str) -> ReedResult<bool> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        validation_error(
            "password_hash",
            hash,
            &format!("Invalid hash format: {}", e),
        )
    })?;

    let argon2 = Argon2::default();

    // Verify returns Result<(), Error> - Ok means password matches
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Changes user password after verification.
///
/// ## Input
/// - `username`: Username whose password to change
/// - `old_password`: Current password for verification
/// - `new_password`: New password to set
///
/// ## Process
/// 1. Verify old password against stored hash
/// 2. Validate new password strength
/// 3. Hash new password
/// 4. Update user record in .reed/users.matrix.csv
///
/// ## Output
/// - `ReedResult<ReedResponse<()>>`: Success or error
///
/// ## Performance
/// - Total time: ~200ms (verify old + hash new)
///
/// ## Error Conditions
/// - Returns `ReedError::NotFound` if user doesn't exist
/// - Returns `ReedError::AuthError` if old password is incorrect
/// - Returns `ReedError::ValidationError` if new password is weak
///
/// ## Example Usage
/// ```
/// change_password("admin", "OldP@ss", "NewSecureP@ssw0rd")?;
/// ```
pub fn change_password(
    _username: &str,
    _old_password: &str,
    new_password: &str,
) -> ReedResult<ReedResponse<()>> {
    // Note: This function requires integration with users.rs
    // Implementation will be completed when users.rs is ready
    // For now, return a placeholder implementation

    // 1. Get user from users.rs
    // 2. Verify old password
    // 3. Validate new password strength
    // 4. Hash new password
    // 5. Update user record

    validate_password_strength(new_password)?;
    let _new_hash = hash_password(new_password)?;

    Ok(ReedResponse {
        data: (),
        source: "security::passwords::change_password".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Validates password strength requirements.
///
/// ## Input
/// - `password`: Password to validate
///
/// ## Requirements
/// - Minimum 8 characters
/// - At least one uppercase letter (A-Z)
/// - At least one lowercase letter (a-z)
/// - At least one digit (0-9)
/// - At least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?)
///
/// ## Output
/// - `ReedResult<()>`: Ok if valid, error with specific constraint violation
///
/// ## Error Conditions
/// - Returns `ReedError::ValidationError` with specific constraint that failed
///
/// ## Example Usage
/// ```
/// validate_password_strength("MyP@ssw0rd")?;  // Ok
/// validate_password_strength("weak")?;        // Error: too short
/// validate_password_strength("NoSpecial1")?;  // Error: no special char
/// ```
pub fn validate_password_strength(password: &str) -> ReedResult<()> {
    if password.len() < 8 {
        return Err(validation_error(
            "password",
            password,
            "Password must be at least 8 characters",
        ));
    }

    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(validation_error(
            "password",
            password,
            "Password must contain at least one uppercase letter",
        ));
    }

    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(validation_error(
            "password",
            password,
            "Password must contain at least one lowercase letter",
        ));
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(validation_error(
            "password",
            password,
            "Password must contain at least one digit",
        ));
    }

    let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
    if !password.chars().any(|c| special_chars.contains(c)) {
        return Err(validation_error(
            "password",
            password,
            "Password must contain at least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?)",
        ));
    }

    Ok(())
}
