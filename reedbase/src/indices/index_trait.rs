// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Index trait abstraction for pluggable backends.
//!
//! Allows ReedBase to switch between HashMap, B+-Tree, or custom implementations
//! without changing query logic.

use crate::error::ReedResult;
use std::fmt::Debug;

/// Common interface for all index implementations.
///
/// ## Type Parameters
/// - `K`: Key type (must be Clone for return values)
/// - `V`: Value type (must be Clone for return values)
///
/// ## Implementations
/// - `HashMapIndex<K, V>`: In-memory O(1) lookups, no persistence
/// - `BTreeIndex<K, V>`: On-disk B+-Tree, persistent, low memory
/// - Custom implementations as needed
///
/// ## Thread Safety
/// - Implementations must be `Send + Sync` for concurrent access
/// - Write operations require `&mut self` (exclusive access)
pub trait Index<K, V>: Send + Sync + Debug {
    /// Get value for exact key match.
    ///
    /// ## Input
    /// - `key`: Key to look up
    ///
    /// ## Output
    /// - `Some(V)` if key exists
    /// - `None` if key not found
    ///
    /// ## Performance
    /// - HashMap: O(1) average, worst O(n) on hash collision
    /// - B+-Tree: O(log n), <1ms for 10M keys
    fn get(&self, key: &K) -> ReedResult<Option<V>>;

    /// Get all key-value pairs in range [start, end] (inclusive).
    ///
    /// ## Input
    /// - `start`: Start of range (inclusive)
    /// - `end`: End of range (inclusive)
    ///
    /// ## Output
    /// - Vector of (key, value) pairs in sorted order
    ///
    /// ## Performance
    /// - HashMap: Not supported (returns error)
    /// - B+-Tree: O(log n + k) where k = result size, <5ms for 100 keys
    ///
    /// ## Error Conditions
    /// - `IndexOperationUnsupported`: Backend doesn't support range queries
    fn range(&self, start: &K, end: &K) -> ReedResult<Vec<(K, V)>>;

    /// Insert or update key-value pair.
    ///
    /// ## Input
    /// - `key`: Key to insert/update
    /// - `value`: Value to store
    ///
    /// ## Performance
    /// - HashMap: O(1) average
    /// - B+-Tree: O(log n) + WAL write, <2ms for 10M keys
    fn insert(&mut self, key: K, value: V) -> ReedResult<()>;

    /// Delete key-value pair.
    ///
    /// ## Input
    /// - `key`: Key to delete
    ///
    /// ## Performance
    /// - HashMap: O(1) average
    /// - B+-Tree: O(log n) + WAL write, <2ms for 10M keys
    fn delete(&mut self, key: &K) -> ReedResult<()>;

    /// Iterate all key-value pairs (unordered for HashMap, sorted for B+-Tree).
    ///
    /// ## Output
    /// - Iterator yielding (key, value) pairs
    ///
    /// ## Performance
    /// - HashMap: O(n), random order
    /// - B+-Tree: O(n), sorted order
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)> + '_>;

    // Metadata methods

    /// Backend type identifier.
    ///
    /// ## Returns
    /// - "hashmap" for in-memory HashMap
    /// - "btree" for on-disk B+-Tree
    /// - Custom identifiers for other implementations
    fn backend_type(&self) -> &'static str;

    /// Estimated memory usage in bytes.
    ///
    /// ## Returns
    /// - HashMap: map size + allocated capacity
    /// - B+-Tree: page cache size (not full file size)
    fn memory_usage(&self) -> usize;

    /// Disk usage in bytes (0 for in-memory backends).
    ///
    /// ## Returns
    /// - HashMap: 0 (no persistence)
    /// - B+-Tree: file size + WAL size
    fn disk_usage(&self) -> usize;
}
