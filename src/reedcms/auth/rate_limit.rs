// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0
//
// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One function = One job
// MANDATORY: BBC English for all documentation and comments
// MANDATORY: Thread-safe in-memory storage with RwLock
// MANDATORY: Progressive lockout to prevent brute-force attacks
// MANDATORY: O(1) HashMap operations for performance
//
// == FILE PURPOSE ==
// This file: Progressive rate limiting for failed login attempts
// Architecture: Authentication security layer - brute-force prevention
// Performance: < 1μs per operation (HashMap with RwLock)
// Dependencies: std collections and sync for thread-safe storage
// Data Flow: Failed login → increment counter → check lockout duration

//! Rate Limiting
//!
//! Provides progressive rate limiting for failed login attempts.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, SystemTime};

/// Login attempt tracking structure.
#[derive(Debug, Clone)]
struct LoginAttempt {
    count: u32,
    last_attempt: SystemTime,
}

/// Gets rate limit store (singleton).
///
/// ## Implementation
/// - In-memory HashMap with username → attempt count
/// - Thread-safe with RwLock
/// - Automatic cleanup on successful login
///
/// ## Performance
/// - Lookup: < 1μs (HashMap)
fn get_rate_limit_store() -> &'static RwLock<HashMap<String, LoginAttempt>> {
    static STORE: OnceLock<RwLock<HashMap<String, LoginAttempt>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Checks if user is rate limited.
///
/// ## Input
/// - `username`: Username to check
///
/// ## Output
/// - `bool`: true if locked out, false if allowed
///
/// ## Rate Limit Rules
/// - 5 failed attempts: 1 minute lockout
/// - 10 failed attempts: 5 minute lockout
/// - 20+ failed attempts: 30 minute lockout
///
/// ## Performance
/// - Check: < 1μs (HashMap lookup)
///
/// ## Example Usage
/// ```
/// if is_rate_limited("admin") {
///     return Err(ReedError::AuthError { /* rate limited */ });
/// }
/// ```
pub fn is_rate_limited(username: &str) -> bool {
    let limits = get_rate_limit_store();
    let store = limits.read().unwrap();

    if let Some(attempts) = store.get(username) {
        let lockout_duration = calculate_lockout_time(attempts.count);
        if attempts.last_attempt + lockout_duration > SystemTime::now() {
            return true;
        }
    }

    false
}

/// Records failed login attempt.
///
/// ## Input
/// - `username`: Username that failed authentication
///
/// ## Process
/// 1. Increment attempt counter
/// 2. Update last attempt timestamp
/// 3. Apply progressive lockout on next check
///
/// ## Performance
/// - Record: < 1μs (HashMap insert)
///
/// ## Example Usage
/// ```
/// if !verify_password(&password, &hash)? {
///     record_failed_login(&username);
///     return Err(ReedError::AuthError { /* invalid */ });
/// }
/// ```
pub fn record_failed_login(username: &str) {
    let limits = get_rate_limit_store();
    let mut store = limits.write().unwrap();

    store
        .entry(username.to_string())
        .and_modify(|a| {
            a.count += 1;
            a.last_attempt = SystemTime::now();
        })
        .or_insert(LoginAttempt {
            count: 1,
            last_attempt: SystemTime::now(),
        });
}

/// Clears failed login counter for user.
///
/// ## Input
/// - `username`: Username to clear
///
/// ## Process
/// 1. Remove entry from rate limit store
/// 2. Reset attempt counter to 0
///
/// ## Performance
/// - Clear: < 1μs (HashMap remove)
///
/// ## Example Usage
/// ```
/// if verify_password(&password, &hash)? {
///     clear_failed_logins(&username);
///     // Proceed with authentication
/// }
/// ```
pub fn clear_failed_logins(username: &str) {
    let limits = get_rate_limit_store();
    let mut store = limits.write().unwrap();
    store.remove(username);
}

/// Calculates lockout duration based on attempt count.
///
/// ## Input
/// - `attempts`: Number of failed attempts
///
/// ## Output
/// - `Duration`: Lockout duration
///
/// ## Progressive Lockout Rules
/// - 0-4 attempts: No lockout
/// - 5-9 attempts: 1 minute lockout
/// - 10-19 attempts: 5 minutes lockout
/// - 20+ attempts: 30 minutes lockout
///
/// ## Security Rationale
/// - Progressive lockout prevents brute-force attacks
/// - Short initial lockout reduces user frustration
/// - Long final lockout prevents automated attacks
fn calculate_lockout_time(attempts: u32) -> Duration {
    match attempts {
        0..=4 => Duration::from_secs(0),     // No lockout
        5..=9 => Duration::from_secs(60),    // 1 minute
        10..=19 => Duration::from_secs(300), // 5 minutes
        _ => Duration::from_secs(1800),      // 30 minutes
    }
}
