// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Data structures for table operations.

/// Result of a write operation.
#[derive(Debug, Clone)]
pub struct WriteResult {
    /// Timestamp of the new version.
    pub timestamp: u64,

    /// Size of delta file in bytes.
    pub delta_size: u64,

    /// Size of current.csv in bytes.
    pub current_size: u64,
}

/// Version metadata from version.log.
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// Unix timestamp in nanoseconds.
    pub timestamp: u64,

    /// Action name (create, update, delete, rollback, etc.).
    pub action: String,

    /// Username who made the change.
    pub user: String,

    /// Size of delta file in bytes.
    pub delta_size: u64,

    /// Optional description/message.
    pub message: Option<String>,
}

/// Parsed CSV row.
#[derive(Debug, Clone)]
pub struct CsvRow {
    /// First column (typically a key).
    pub key: String,

    /// Remaining columns.
    pub values: Vec<String>,
}

/// Table statistics.
#[derive(Debug, Clone)]
pub struct TableStats {
    /// Table name.
    pub name: String,

    /// Size of current.csv in bytes.
    pub current_size: u64,

    /// Total size of all deltas in bytes.
    pub deltas_size: u64,

    /// Number of versions.
    pub version_count: usize,

    /// Timestamp of newest version.
    pub latest_version: u64,

    /// Timestamp of oldest version.
    pub oldest_version: u64,
}
