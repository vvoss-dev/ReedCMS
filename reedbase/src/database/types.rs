// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Database API Core Types
//!
//! Defines types used throughout the Database API.

use crate::error::{ReedError, ReedResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Auto-indexing configuration.
///
/// Controls when and how indices are automatically created based on query patterns.
#[derive(Debug, Clone)]
pub struct AutoIndexConfig {
    /// Enable auto-indexing (default: true)
    pub enabled: bool,

    /// Number of repeated queries before creating index (default: 10)
    pub threshold: usize,

    /// Enable foreign key pattern detection (default: true)
    /// Auto-creates index for columns ending with `_id` after 3 queries
    pub foreign_key_detection: bool,

    /// Enable ReedCMS pattern optimization (default: true)
    /// Auto-creates indices for language (`%.@de`) and namespace (`page.%`) patterns
    pub reedcms_patterns: bool,
}

impl Default for AutoIndexConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 10,
            foreign_key_detection: true,
            reedcms_patterns: true,
        }
    }
}

impl AutoIndexConfig {
    /// Creates new configuration with all features enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Disables auto-indexing completely.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            threshold: 0,
            foreign_key_detection: false,
            reedcms_patterns: false,
        }
    }

    /// Creates configuration optimized for ReedCMS.
    pub fn reedcms_optimized() -> Self {
        Self {
            enabled: true,
            threshold: 5, // Faster indexing for CMS
            foreign_key_detection: true,
            reedcms_patterns: true,
        }
    }
}

/// Index backend type.
///
/// Determines storage and performance characteristics of an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndexBackend {
    /// HashMap (in-memory, O(1) exact match, no persistence)
    Hash,

    /// B+-Tree (on-disk, O(log n) lookups, persistent, supports range queries)
    BTree,
}

impl IndexBackend {
    /// Returns optimal backend for query pattern operation.
    ///
    /// ## Rules
    /// - Exact match (=) → Hash (O(1) faster)
    /// - Range (<, >, <=, >=) → BTree (only backend supporting ranges)
    /// - Prefix (LIKE 'foo%') → BTree (efficient range scan)
    /// - Pattern (LIKE '%foo%') → Neither (full scan required)
    ///
    /// ## Input
    /// - `operation`: Query operation type ("equals", "range", "prefix", etc.)
    ///
    /// ## Output
    /// - Optimal backend for the operation
    pub fn for_operation(operation: &str) -> Self {
        match operation {
            "equals" => Self::Hash,
            "range"
            | "prefix"
            | "less_than"
            | "greater_than"
            | "less_than_or_equal"
            | "greater_than_or_equal" => Self::BTree,
            _ => Self::Hash, // Default to Hash
        }
    }

    /// Returns human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Hash => "hash",
            Self::BTree => "btree",
        }
    }
}

/// Index metadata stored in .reed/indices/metadata.json
///
/// Tracks index creation, usage, and backend information for persistent
/// index loading and management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// Table name
    pub table: String,

    /// Column name
    pub column: String,

    /// Backend type (hash or btree)
    pub backend: IndexBackend,

    /// Creation timestamp (Unix seconds)
    pub created_at: u64,

    /// Query pattern that triggered creation ("exact", "range", "prefix")
    pub query_pattern: String,

    /// Whether index was auto-created by pattern detection
    pub auto_created: bool,

    /// Number of times index was used in queries
    pub usage_count: usize,

    /// Last usage timestamp (Unix seconds)
    pub last_used: u64,
}

impl IndexMetadata {
    /// Creates new index metadata.
    pub fn new(table: String, column: String, backend: IndexBackend) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            table,
            column,
            backend,
            created_at: now,
            query_pattern: "unknown".to_string(),
            auto_created: false,
            usage_count: 0,
            last_used: now,
        }
    }

    /// Returns index key (table.column format).
    pub fn index_key(&self) -> String {
        format!("{}.{}", self.table, self.column)
    }

    /// Records index usage (increments count, updates last_used).
    pub fn record_usage(&mut self) {
        self.usage_count += 1;
        self.last_used = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Information about a database index.
#[derive(Debug, Clone)]
pub struct IndexInfo {
    /// Table name
    pub table: String,

    /// Column name
    pub column: String,

    /// Index type ("hash" or "btree")
    pub index_type: String,

    /// Backend type
    pub backend: IndexBackend,

    /// Number of entries in index
    pub entry_count: usize,

    /// Memory usage in bytes
    pub memory_bytes: usize,

    /// Disk usage in bytes (0 for in-memory indices)
    pub disk_bytes: usize,

    /// Times this index was used
    pub usage_count: usize,

    /// Whether index was auto-created
    pub auto_created: bool,
}

impl IndexInfo {
    /// Creates new index information.
    pub fn new(table: String, column: String, index_type: String, backend: IndexBackend) -> Self {
        Self {
            table,
            column,
            index_type,
            backend,
            entry_count: 0,
            memory_bytes: 0,
            disk_bytes: 0,
            usage_count: 0,
            auto_created: false,
        }
    }

    /// Returns total storage usage (memory + disk).
    pub fn total_bytes(&self) -> usize {
        self.memory_bytes + self.disk_bytes
    }

    /// Returns index efficiency score (usage_count / memory_kb).
    pub fn efficiency_score(&self) -> f64 {
        if self.memory_bytes == 0 {
            return 0.0;
        }
        let memory_kb = self.memory_bytes as f64 / 1024.0;
        self.usage_count as f64 / memory_kb
    }
}

/// Query execution metrics.
#[derive(Debug, Clone)]
pub struct QueryMetrics {
    /// Parse time in microseconds
    pub parse_time_us: u64,

    /// Execution time in microseconds
    pub execution_time_us: u64,

    /// Number of rows scanned
    pub rows_scanned: usize,

    /// Number of rows returned
    pub rows_returned: usize,

    /// Index used (if any)
    pub index_used: Option<String>,

    /// Whether query used fast path
    pub used_fast_path: bool,
}

impl QueryMetrics {
    /// Creates new metrics.
    pub fn new() -> Self {
        Self {
            parse_time_us: 0,
            execution_time_us: 0,
            rows_scanned: 0,
            rows_returned: 0,
            index_used: None,
            used_fast_path: false,
        }
    }

    /// Returns total time in microseconds.
    pub fn total_time_us(&self) -> u64 {
        self.parse_time_us + self.execution_time_us
    }

    /// Returns total time as Duration.
    pub fn total_duration(&self) -> Duration {
        Duration::from_micros(self.total_time_us())
    }

    /// Returns scan efficiency (rows_returned / rows_scanned).
    pub fn scan_efficiency(&self) -> f64 {
        if self.rows_scanned == 0 {
            return 1.0;
        }
        self.rows_returned as f64 / self.rows_scanned as f64
    }
}

impl Default for QueryMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Database statistics.
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// Number of tables
    pub table_count: usize,

    /// Total number of rows across all tables
    pub total_rows: usize,

    /// Number of indices
    pub index_count: usize,

    /// Total memory used by indices (bytes)
    pub index_memory_bytes: usize,

    /// Total disk used by indices (bytes)
    pub index_disk_bytes: usize,

    /// Total queries executed
    pub query_count: usize,

    /// Total inserts executed
    pub insert_count: usize,

    /// Total updates executed
    pub update_count: usize,

    /// Total deletes executed
    pub delete_count: usize,

    /// Number of auto-created indices
    pub auto_index_count: usize,

    /// Average query time (microseconds)
    pub avg_query_time_us: u64,
}

impl DatabaseStats {
    /// Creates new empty statistics.
    pub fn new() -> Self {
        Self {
            table_count: 0,
            total_rows: 0,
            index_count: 0,
            index_memory_bytes: 0,
            index_disk_bytes: 0,
            query_count: 0,
            insert_count: 0,
            update_count: 0,
            delete_count: 0,
            auto_index_count: 0,
            avg_query_time_us: 0,
        }
    }

    /// Returns total storage used by indices (memory + disk).
    pub fn total_index_bytes(&self) -> usize {
        self.index_memory_bytes + self.index_disk_bytes
    }

    /// Returns total operations (queries + inserts + updates + deletes).
    pub fn total_operations(&self) -> usize {
        self.query_count + self.insert_count + self.update_count + self.delete_count
    }

    /// Returns average query time as Duration.
    pub fn avg_query_duration(&self) -> Duration {
        Duration::from_micros(self.avg_query_time_us)
    }
}

impl Default for DatabaseStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_index_config_default() {
        let config = AutoIndexConfig::default();
        assert!(config.enabled);
        assert_eq!(config.threshold, 10);
        assert!(config.foreign_key_detection);
        assert!(config.reedcms_patterns);
    }

    #[test]
    fn test_auto_index_config_disabled() {
        let config = AutoIndexConfig::disabled();
        assert!(!config.enabled);
        assert_eq!(config.threshold, 0);
        assert!(!config.foreign_key_detection);
        assert!(!config.reedcms_patterns);
    }

    #[test]
    fn test_auto_index_config_reedcms_optimized() {
        let config = AutoIndexConfig::reedcms_optimized();
        assert!(config.enabled);
        assert_eq!(config.threshold, 5);
        assert!(config.foreign_key_detection);
        assert!(config.reedcms_patterns);
    }

    #[test]
    fn test_index_info_new() {
        let info = IndexInfo::new(
            "text".to_string(),
            "key".to_string(),
            "btree".to_string(),
            IndexBackend::BTree,
        );
        assert_eq!(info.table, "text");
        assert_eq!(info.column, "key");
        assert_eq!(info.index_type, "btree");
        assert_eq!(info.backend, IndexBackend::BTree);
        assert_eq!(info.entry_count, 0);
        assert!(!info.auto_created);
    }

    #[test]
    fn test_index_info_total_bytes() {
        let mut info = IndexInfo::new(
            "text".to_string(),
            "key".to_string(),
            "btree".to_string(),
            IndexBackend::BTree,
        );
        info.memory_bytes = 1024;
        info.disk_bytes = 2048;
        assert_eq!(info.total_bytes(), 3072);
    }

    #[test]
    fn test_index_info_efficiency_score() {
        let mut info = IndexInfo::new(
            "text".to_string(),
            "key".to_string(),
            "hash".to_string(),
            IndexBackend::Hash,
        );
        info.memory_bytes = 10240; // 10 KB
        info.usage_count = 100;
        assert!((info.efficiency_score() - 10.0).abs() < 0.01); // 100 / 10 = 10
    }

    #[test]
    fn test_index_backend_for_operation() {
        assert_eq!(IndexBackend::for_operation("equals"), IndexBackend::Hash);
        assert_eq!(IndexBackend::for_operation("range"), IndexBackend::BTree);
        assert_eq!(
            IndexBackend::for_operation("less_than"),
            IndexBackend::BTree
        );
        assert_eq!(
            IndexBackend::for_operation("greater_than"),
            IndexBackend::BTree
        );
        assert_eq!(IndexBackend::for_operation("prefix"), IndexBackend::BTree);
        assert_eq!(IndexBackend::for_operation("unknown"), IndexBackend::Hash); // Default
    }

    #[test]
    fn test_index_backend_name() {
        assert_eq!(IndexBackend::Hash.name(), "hash");
        assert_eq!(IndexBackend::BTree.name(), "btree");
    }

    #[test]
    fn test_index_metadata_new() {
        let metadata =
            IndexMetadata::new("text".to_string(), "key".to_string(), IndexBackend::BTree);
        assert_eq!(metadata.table, "text");
        assert_eq!(metadata.column, "key");
        assert_eq!(metadata.backend, IndexBackend::BTree);
        assert_eq!(metadata.query_pattern, "unknown");
        assert!(!metadata.auto_created);
        assert_eq!(metadata.usage_count, 0);
    }

    #[test]
    fn test_index_metadata_index_key() {
        let metadata =
            IndexMetadata::new("text".to_string(), "key".to_string(), IndexBackend::Hash);
        assert_eq!(metadata.index_key(), "text.key");
    }

    #[test]
    fn test_index_metadata_record_usage() {
        let mut metadata =
            IndexMetadata::new("text".to_string(), "key".to_string(), IndexBackend::Hash);
        let initial_usage = metadata.usage_count;
        let initial_last_used = metadata.last_used;

        std::thread::sleep(std::time::Duration::from_millis(10));
        metadata.record_usage();

        assert_eq!(metadata.usage_count, initial_usage + 1);
        assert!(metadata.last_used > initial_last_used);
    }

    #[test]
    fn test_query_metrics_default() {
        let metrics = QueryMetrics::default();
        assert_eq!(metrics.parse_time_us, 0);
        assert_eq!(metrics.execution_time_us, 0);
        assert_eq!(metrics.rows_scanned, 0);
        assert_eq!(metrics.rows_returned, 0);
        assert!(metrics.index_used.is_none());
        assert!(!metrics.used_fast_path);
    }

    #[test]
    fn test_query_metrics_total_time() {
        let mut metrics = QueryMetrics::new();
        metrics.parse_time_us = 10;
        metrics.execution_time_us = 990;
        assert_eq!(metrics.total_time_us(), 1000);
    }

    #[test]
    fn test_query_metrics_scan_efficiency() {
        let mut metrics = QueryMetrics::new();
        metrics.rows_scanned = 1000;
        metrics.rows_returned = 10;
        assert!((metrics.scan_efficiency() - 0.01).abs() < 0.001); // 10/1000 = 0.01

        // With index (high efficiency)
        metrics.rows_scanned = 10;
        metrics.rows_returned = 10;
        assert!((metrics.scan_efficiency() - 1.0).abs() < 0.001); // 10/10 = 1.0
    }

    #[test]
    fn test_database_stats_default() {
        let stats = DatabaseStats::default();
        assert_eq!(stats.table_count, 0);
        assert_eq!(stats.total_rows, 0);
        assert_eq!(stats.query_count, 0);
    }

    #[test]
    fn test_database_stats_total_index_bytes() {
        let mut stats = DatabaseStats::new();
        stats.index_memory_bytes = 1024;
        stats.index_disk_bytes = 2048;
        assert_eq!(stats.total_index_bytes(), 3072);
    }

    #[test]
    fn test_database_stats_total_operations() {
        let mut stats = DatabaseStats::new();
        stats.query_count = 100;
        stats.insert_count = 20;
        stats.update_count = 10;
        stats.delete_count = 5;
        assert_eq!(stats.total_operations(), 135);
    }
}
