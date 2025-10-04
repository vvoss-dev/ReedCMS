// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! API Rate Limiting
//!
//! Implements sliding window rate limiting for API operations.
//!
//! ## Strategy
//! - Sliding window algorithm (more accurate than fixed windows)
//! - Per-user, per-operation tracking
//! - In-memory storage with automatic cleanup
//! - Configurable limits via SecurityMatrix
//!
//! ## Storage
//! - HashMap: user_id:operation → Vec<timestamp>
//! - RwLock for thread safety
//! - Automatic cleanup every 5 minutes
//!
//! ## Performance
//! - Check: O(n) where n is requests in window (typically < 1000)
//! - < 100μs typical per check
//! - Memory: ~8 bytes per tracked request
//!
//! ## Example Usage
//! ```rust
//! let limit = RateLimit { requests: 100, period: RateLimitPeriod::Minute };
//! check_rate_limit("user123", "text.read", &limit)?;
//! ```

use crate::reedcms::api::security::matrix::RateLimit;
use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::collections::HashMap;
use std::sync::RwLock;

/// Checks rate limit for user and operation.
///
/// ## Arguments
/// - `user_id`: User identifier (username or API key hash)
/// - `operation`: Operation identifier (e.g., "text.read")
/// - `limit`: Rate limit configuration
///
/// ## Returns
/// - `Ok(())` if within limit
/// - `Err(ReedError::AuthError)` if limit exceeded
///
/// ## Performance
/// - O(n) where n is requests in current window
/// - < 100μs typical (n < 1000)
///
/// ## Example Usage
/// ```rust
/// let limit = RateLimit::parse("100/min")?;
/// check_rate_limit("alice", "text.read", &limit)?;
/// ```
pub fn check_rate_limit(user_id: &str, operation: &str, limit: &RateLimit) -> ReedResult<()> {
    let store = get_rate_limit_store();
    let mut store = store.write().unwrap();

    let key = format!("{}:{}", user_id, operation);
    let now = std::time::SystemTime::now();

    // Get or create entry
    let entry = store.entry(key.clone()).or_insert_with(|| RateLimitEntry {
        requests: Vec::new(),
    });

    // Calculate window start time
    let window_start =
        now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() - limit.period.duration();

    // Remove expired requests (outside sliding window)
    entry.requests.retain(|&timestamp| timestamp > window_start);

    // Check if limit exceeded
    if entry.requests.len() >= limit.requests as usize {
        return Err(ReedError::AuthError {
            user: Some(user_id.to_string()),
            action: operation.to_string(),
            reason: format!(
                "Rate limit exceeded: {} requests per {:?}",
                limit.requests, limit.period
            ),
        });
    }

    // Record this request
    entry
        .requests
        .push(now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());

    Ok(())
}

/// Rate limit entry tracking request timestamps.
#[derive(Debug, Clone)]
struct RateLimitEntry {
    requests: Vec<u64>, // Unix timestamps in seconds
}

/// Gets rate limit store singleton.
///
/// ## Output
/// - Static reference to RwLock-protected HashMap
///
/// ## Thread Safety
/// - RwLock allows multiple concurrent reads
/// - Exclusive write access during limit checks
fn get_rate_limit_store() -> &'static RwLock<HashMap<String, RateLimitEntry>> {
    use std::sync::OnceLock;
    static STORE: OnceLock<RwLock<HashMap<String, RateLimitEntry>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Cleans up expired rate limit entries.
///
/// ## Cleanup Strategy
/// - Removes entries with no requests in last 24 hours
/// - Prevents memory leak from inactive users
/// - Called automatically every 5 minutes by cleanup thread
///
/// ## Performance
/// - O(n) where n is number of tracked entries
/// - < 100ms for < 10,000 entries
pub fn cleanup_rate_limits() {
    let store = get_rate_limit_store();
    let mut store = store.write().unwrap();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let cutoff = now - 86400; // 24 hours ago

    // Remove entries with all requests older than cutoff
    store.retain(|_, entry| entry.requests.iter().any(|&timestamp| timestamp > cutoff));
}

/// Starts rate limit cleanup thread.
///
/// ## Background Thread
/// - Runs indefinitely
/// - Sleeps 5 minutes between cleanups
/// - Low priority, minimal CPU usage
///
/// ## Example Usage
/// ```rust
/// start_cleanup_thread(); // Call once at server startup
/// ```
pub fn start_cleanup_thread() {
    std::thread::spawn(|| loop {
        std::thread::sleep(std::time::Duration::from_secs(300)); // 5 minutes
        cleanup_rate_limits();
    });
}

/// Gets current request count for user-operation pair.
///
/// ## Arguments
/// - `user_id`: User identifier
/// - `operation`: Operation identifier
///
/// ## Returns
/// - Number of requests in current window
///
/// ## Use Case
/// - Monitoring and debugging
/// - Display remaining quota to users
///
/// ## Example Usage
/// ```rust
/// let count = get_request_count("alice", "text.read");
/// println!("Requests so far: {}", count);
/// ```
pub fn get_request_count(user_id: &str, operation: &str) -> usize {
    let store = get_rate_limit_store();
    let store = store.read().unwrap();

    let key = format!("{}:{}", user_id, operation);
    store.get(&key).map(|e| e.requests.len()).unwrap_or(0)
}

/// Clears rate limit for specific user-operation pair.
///
/// ## Arguments
/// - `user_id`: User identifier
/// - `operation`: Operation identifier
///
/// ## Use Case
/// - Admin override
/// - Testing
/// - Manual reset after false positives
///
/// ## Example Usage
/// ```rust
/// clear_rate_limit("alice", "text.read");
/// ```
pub fn clear_rate_limit(user_id: &str, operation: &str) {
    let store = get_rate_limit_store();
    let mut store = store.write().unwrap();

    let key = format!("{}:{}", user_id, operation);
    store.remove(&key);
}
