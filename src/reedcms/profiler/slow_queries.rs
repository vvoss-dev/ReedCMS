// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Slow operation tracker.
//!
//! ## Features
//! - Configurable threshold (default: 100ms)
//! - Ring buffer (last 100 operations)
//! - Thread-safe recording
//! - Query by operation type

use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

/// Tracks slow operations for analysis.
///
/// ## Configuration
/// - Default threshold: 100ms
/// - Environment: REED_SLOW_THRESHOLD (milliseconds)
/// - Storage: Last 100 operations in memory
///
/// ## Example
/// ```rust
/// use reedcms::profiler::global_slow_tracker;
/// use std::time::Duration;
///
/// let tracker = global_slow_tracker();
/// tracker.record("database_query", Duration::from_millis(150), "SELECT * FROM users".to_string());
/// ```
pub struct SlowQueryTracker {
    threshold: Duration,
    queries: Mutex<VecDeque<SlowQuery>>,
}

impl SlowQueryTracker {
    /// Creates new slow query tracker.
    ///
    /// Reads threshold from REED_SLOW_THRESHOLD environment variable.
    pub fn new() -> Self {
        let threshold_ms = std::env::var("REED_SLOW_THRESHOLD")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        Self {
            threshold: Duration::from_millis(threshold_ms),
            queries: Mutex::new(VecDeque::new()),
        }
    }

    /// Records operation if it exceeds threshold.
    ///
    /// ## Arguments
    /// - `operation`: Operation type/name
    /// - `duration`: Operation duration
    /// - `context`: Additional context (query text, parameters, etc.)
    ///
    /// ## Example
    /// ```rust
    /// tracker.record("reedbase_lookup", Duration::from_millis(120), "knowledge.title@de".to_string());
    /// ```
    pub fn record(&self, operation: &str, duration: Duration, context: String) {
        if duration >= self.threshold {
            let query = SlowQuery {
                operation: operation.to_string(),
                duration,
                context,
                timestamp: chrono::Utc::now(),
            };

            if let Ok(mut queries) = self.queries.lock() {
                queries.push_back(query);

                // Keep only last 100
                if queries.len() > 100 {
                    queries.pop_front();
                }
            }
        }
    }

    /// Gets all slow queries.
    ///
    /// ## Returns
    /// Vector of all recorded slow queries (up to 100)
    pub fn get_slow_queries(&self) -> Vec<SlowQuery> {
        self.queries.lock().unwrap().iter().cloned().collect()
    }

    /// Gets slow queries for specific operation.
    ///
    /// ## Arguments
    /// - `operation`: Operation type to filter by
    ///
    /// ## Returns
    /// Vector of slow queries matching the operation type
    pub fn get_by_operation(&self, operation: &str) -> Vec<SlowQuery> {
        self.queries
            .lock()
            .unwrap()
            .iter()
            .filter(|q| q.operation == operation)
            .cloned()
            .collect()
    }

    /// Gets query count.
    pub fn count(&self) -> usize {
        self.queries.lock().unwrap().len()
    }

    /// Clears all slow queries (for testing).
    #[cfg(test)]
    pub fn clear(&self) {
        self.queries.lock().unwrap().clear();
    }
}

impl Default for SlowQueryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Slow query record.
#[derive(Debug, Clone, Serialize)]
pub struct SlowQuery {
    pub operation: String,
    #[serde(serialize_with = "serialize_duration")]
    pub duration: Duration,
    pub context: String,
    #[serde(serialize_with = "serialize_datetime")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u64(duration.as_millis() as u64)
}

fn serialize_datetime<S>(
    dt: &chrono::DateTime<chrono::Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&dt.to_rfc3339())
}

/// Gets global slow query tracker.
///
/// ## Usage
/// ```rust
/// use reedcms::profiler::global_slow_tracker;
///
/// let tracker = global_slow_tracker();
/// let slow_queries = tracker.get_slow_queries();
/// ```
pub fn global_slow_tracker() -> &'static SlowQueryTracker {
    use std::sync::OnceLock;
    static TRACKER: OnceLock<SlowQueryTracker> = OnceLock::new();
    TRACKER.get_or_init(SlowQueryTracker::new)
}
