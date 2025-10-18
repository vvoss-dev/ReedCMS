// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Type definitions for log system.

use uuid::Uuid;

/// Log entry for version history.
///
/// Represents a single operation in the version log.
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    /// Unix timestamp in nanoseconds.
    pub timestamp: u64,

    /// Action name (e.g., "init", "update", "rollback").
    pub action: String,

    /// Username who performed the action.
    pub user: String,

    /// Previous version timestamp (0 for init).
    pub base_version: u64,

    /// Delta size in bytes.
    pub size: usize,

    /// Number of rows affected.
    pub rows: usize,

    /// SHA-256 hash of delta content.
    pub hash: String,

    /// Frame UUID if part of coordinated batch, None otherwise.
    pub frame_id: Option<Uuid>,
}

impl LogEntry {
    /// Creates a new log entry.
    ///
    /// ## Input
    /// - `timestamp`: Unix timestamp in nanoseconds
    /// - `action`: Action name
    /// - `user`: Username
    /// - `base_version`: Previous version timestamp
    /// - `size`: Delta size in bytes
    /// - `rows`: Number of rows affected
    /// - `hash`: SHA-256 hash of delta
    /// - `frame_id`: Optional frame UUID
    ///
    /// ## Output
    /// - `LogEntry`: New log entry
    ///
    /// ## Example Usage
    /// ```
    /// use reedbase::log::LogEntry;
    ///
    /// let entry = LogEntry::new(
    ///     1736860900,
    ///     "update".to_string(),
    ///     "admin".to_string(),
    ///     1736860800,
    ///     2500,
    ///     15,
    ///     "sha256:abc123".to_string(),
    ///     None,
    /// );
    /// ```
    pub fn new(
        timestamp: u64,
        action: String,
        user: String,
        base_version: u64,
        size: usize,
        rows: usize,
        hash: String,
        frame_id: Option<Uuid>,
    ) -> Self {
        Self {
            timestamp,
            action,
            user,
            base_version,
            size,
            rows,
            hash,
            frame_id,
        }
    }
}

/// Validation report from log validation.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationReport {
    /// Total number of entries in log.
    pub total_entries: usize,

    /// Number of valid entries.
    pub valid_entries: usize,

    /// Number of corrupted entries found.
    pub corrupted_count: usize,

    /// Line numbers of corrupted entries.
    pub corrupted_lines: Vec<usize>,

    /// Whether log was truncated to remove corruption.
    pub truncated: bool,
}

impl ValidationReport {
    /// Creates a new validation report.
    pub fn new() -> Self {
        Self {
            total_entries: 0,
            valid_entries: 0,
            corrupted_count: 0,
            corrupted_lines: Vec::new(),
            truncated: false,
        }
    }

    /// Checks if log is healthy (no corruption).
    pub fn is_healthy(&self) -> bool {
        self.corrupted_count == 0
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}
