// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! HashMap-based index implementation.
//!
//! Wraps `HashMap<K, V>` to implement the `Index<K, V>` trait for in-memory
//! O(1) lookups with no persistence.
//!
//! ## Performance
//!
//! - **Point lookup**: O(1) average, O(n) worst-case hash collision
//! - **Range scan**: Not supported (returns error)
//! - **Insert**: O(1) average
//! - **Delete**: O(1) average
//! - **Memory usage**: ~32 bytes per key-value pair + capacity overhead
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::indices::hashmap_index::HashMapIndex;
//! use reedbase::indices::Index;
//!
//! let mut index = HashMapIndex::<String, Vec<usize>>::new();
//!
//! index.insert("page".to_string(), vec![1, 2, 3])?;
//! let value = index.get(&"page".to_string())?;
//! assert_eq!(value, Some(vec![1, 2, 3]));
//!
//! # Ok::<(), reedbase::ReedError>(())
//! ```

use crate::error::{ReedError, ReedResult};
use crate::indices::Index;
use std::collections::HashMap;
use std::hash::Hash;

/// HashMap-based index for in-memory storage.
///
/// ## Type Parameters
/// - `K`: Key type (must be Clone + Eq + Hash + Ord)
/// - `V`: Value type (must be Clone)
///
/// ## Thread Safety
/// - Not thread-safe (caller must synchronise)
/// - Use external locking for concurrent access
#[derive(Debug)]
pub struct HashMapIndex<K, V>
where
    K: Clone + Eq + Hash + Ord,
    V: Clone,
{
    /// Internal HashMap storage.
    map: HashMap<K, V>,
}

impl<K, V> HashMapIndex<K, V>
where
    K: Clone + Eq + Hash + Ord,
    V: Clone,
{
    /// Create new empty HashMap index.
    ///
    /// ## Output
    /// - Empty index ready for insertions
    ///
    /// ## Performance
    /// - O(1) constant time
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::hashmap_index::HashMapIndex;
    ///
    /// let index = HashMapIndex::<String, Vec<usize>>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Create new HashMap index with specified capacity.
    ///
    /// ## Input
    /// - `capacity`: Initial capacity (number of elements)
    ///
    /// ## Output
    /// - Empty index with pre-allocated space
    ///
    /// ## Performance
    /// - O(1) constant time
    /// - Avoids reallocation during initial insertions
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::hashmap_index::HashMapIndex;
    ///
    /// let index = HashMapIndex::<String, Vec<usize>>::with_capacity(1000);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }

    /// Get number of entries in index.
    ///
    /// ## Output
    /// - Number of key-value pairs stored
    ///
    /// ## Performance
    /// - O(1) constant time
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if index is empty.
    ///
    /// ## Output
    /// - `true` if no entries exist
    /// - `false` otherwise
    ///
    /// ## Performance
    /// - O(1) constant time
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear all entries from index.
    ///
    /// ## Performance
    /// - O(n) where n = number of entries
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl<K, V> Default for HashMapIndex<K, V>
where
    K: Clone + Eq + Hash + Ord,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Index<K, V> for HashMapIndex<K, V>
where
    K: Clone + Eq + Hash + Ord + Send + Sync + std::fmt::Debug,
    V: Clone + Send + Sync + std::fmt::Debug,
{
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
    /// - O(1) average
    /// - O(n) worst-case on hash collision
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::{hashmap_index::HashMapIndex, Index};
    ///
    /// let mut index = HashMapIndex::new();
    /// index.insert("page".to_string(), vec![1, 2])?;
    ///
    /// let value = index.get(&"page".to_string())?;
    /// assert_eq!(value, Some(vec![1, 2]));
    ///
    /// let missing = index.get(&"missing".to_string())?;
    /// assert_eq!(missing, None);
    ///
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    fn get(&self, key: &K) -> ReedResult<Option<V>> {
        Ok(self.map.get(key).cloned())
    }

    /// Get all key-value pairs in range [start, end] (inclusive).
    ///
    /// ## Input
    /// - `start`: Start of range (inclusive)
    /// - `end`: End of range (inclusive)
    ///
    /// ## Output
    /// - `Err(IndexOperationUnsupported)`: HashMap does not support range queries
    ///
    /// ## Note
    /// HashMap stores entries in arbitrary order and cannot efficiently
    /// provide range queries. Use B+-Tree backend for range queries.
    fn range(&self, _start: &K, _end: &K) -> ReedResult<Vec<(K, V)>> {
        Err(ReedError::IndexOperationUnsupported {
            operation: "range".to_string(),
            backend: "hashmap".to_string(),
            reason: "HashMap does not support ordered range queries".to_string(),
        })
    }

    /// Insert or update key-value pair.
    ///
    /// ## Input
    /// - `key`: Key to insert/update
    /// - `value`: Value to store
    ///
    /// ## Performance
    /// - O(1) average
    /// - May trigger resize if capacity exceeded
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::{hashmap_index::HashMapIndex, Index};
    ///
    /// let mut index = HashMapIndex::new();
    /// index.insert("page".to_string(), vec![1, 2])?;
    /// index.insert("page".to_string(), vec![3, 4])?; // Update
    ///
    /// let value = index.get(&"page".to_string())?;
    /// assert_eq!(value, Some(vec![3, 4]));
    ///
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    fn insert(&mut self, key: K, value: V) -> ReedResult<()> {
        self.map.insert(key, value);
        Ok(())
    }

    /// Delete key-value pair.
    ///
    /// ## Input
    /// - `key`: Key to delete
    ///
    /// ## Performance
    /// - O(1) average
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::{hashmap_index::HashMapIndex, Index};
    ///
    /// let mut index = HashMapIndex::new();
    /// index.insert("page".to_string(), vec![1, 2])?;
    /// index.delete(&"page".to_string())?;
    ///
    /// let value = index.get(&"page".to_string())?;
    /// assert_eq!(value, None);
    ///
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    fn delete(&mut self, key: &K) -> ReedResult<()> {
        self.map.remove(key);
        Ok(())
    }

    /// Iterate all key-value pairs (unordered).
    ///
    /// ## Output
    /// - Iterator yielding (key, value) pairs in arbitrary order
    ///
    /// ## Performance
    /// - O(n) where n = number of entries
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::{hashmap_index::HashMapIndex, Index};
    ///
    /// let mut index = HashMapIndex::new();
    /// index.insert("page".to_string(), vec![1, 2])?;
    /// index.insert("api".to_string(), vec![3, 4])?;
    ///
    /// let entries: Vec<_> = index.iter().collect();
    /// assert_eq!(entries.len(), 2);
    ///
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)> + '_> {
        Box::new(
            self.map
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }

    /// Backend type identifier.
    ///
    /// ## Returns
    /// - `"hashmap"` for in-memory HashMap
    ///
    /// ## Performance
    /// - O(1) constant time
    fn backend_type(&self) -> &'static str {
        "hashmap"
    }

    /// Estimated memory usage in bytes.
    ///
    /// ## Returns
    /// - HashMap size + allocated capacity overhead
    ///
    /// ## Performance
    /// - O(1) constant time
    ///
    /// ## Note
    /// This is an approximation based on HashMap capacity.
    /// Actual memory usage depends on key/value sizes.
    fn memory_usage(&self) -> usize {
        // Estimate: HashMap overhead + capacity * entry size
        let capacity = self.map.capacity();
        let entry_size = std::mem::size_of::<(K, V)>();
        std::mem::size_of::<HashMap<K, V>>() + (capacity * entry_size)
    }

    /// Disk usage in bytes (0 for in-memory backends).
    ///
    /// ## Returns
    /// - `0` (HashMap has no persistence)
    ///
    /// ## Performance
    /// - O(1) constant time
    fn disk_usage(&self) -> usize {
        0
    }
}
