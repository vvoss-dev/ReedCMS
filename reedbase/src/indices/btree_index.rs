// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! B+-Tree-based index implementation.
//!
//! Wraps `BPlusTree<K, V>` to implement the `Index<K, V>` trait for persistent
//! disk-based storage with O(log n) lookups and efficient range queries.
//!
//! ## Performance
//!
//! - **Point lookup**: O(log n), < 1ms for 10M keys
//! - **Range scan**: O(log n + k), < 5ms per 100 keys
//! - **Insert**: O(log n), < 2ms including WAL
//! - **Delete**: O(log n), < 2ms including WAL
//! - **Memory usage**: ~50MB for 10M keys (demand-paged mmap)
//! - **Disk usage**: ~200MB for 10M keys (4KB pages)
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::indices::btree_index::BTreeIndex;
//! use reedbase::indices::Index;
//! use reedbase::btree::Order;
//!
//! let order = Order::new(100)?;
//! let mut index = BTreeIndex::<String, Vec<usize>>::open("namespace.btree", order)?;
//!
//! index.insert("page".to_string(), vec![1, 2, 3])?;
//! let value = index.get(&"page".to_string())?;
//! assert_eq!(value, Some(vec![1, 2, 3]));
//!
//! // Range queries
//! let results = index.range(&"page.a".to_string(), &"page.z".to_string())?;
//!
//! # Ok::<(), reedbase::ReedError>(())
//! ```

use crate::btree::{BPlusTree, Order};
use crate::error::ReedResult;
use crate::indices::Index;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// B+-Tree-based index for persistent storage.
///
/// ## Type Parameters
/// - `K`: Key type (must be Clone + Ord + Serialize + Deserialize)
/// - `V`: Value type (must be Clone + Serialize + Deserialize)
///
/// ## Thread Safety
/// - Not thread-safe (caller must synchronise)
/// - Use external locking for concurrent access
///
/// ## Persistence
/// - All operations are written to disk via mmap
/// - WAL ensures crash recovery
/// - File format: `.btree` (main file) + `.wal` (write-ahead log)
#[derive(Debug)]
pub struct BTreeIndex<K, V>
where
    K: Clone + Ord + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    V: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    /// Internal B+-Tree storage.
    tree: BPlusTree<K, V>,
}

impl<K, V> BTreeIndex<K, V>
where
    K: Clone + Ord + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    V: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    /// Open or create B+-Tree index.
    ///
    /// ## Input
    /// - `path`: Path to B+-Tree file (creates `.btree` and `.wal` files)
    /// - `order`: Tree order defining node capacity
    ///
    /// ## Output
    /// - `Ok(BTreeIndex)`: Successfully opened/created index
    /// - `Err(ReedError)`: I/O error or corruption
    ///
    /// ## Performance
    /// - New file: ~10ms (allocate 1MB)
    /// - Existing file: ~5ms (mmap)
    /// - WAL replay: ~50ms per 1000 entries
    ///
    /// ## Error Conditions
    /// - Parent directory does not exist
    /// - Insufficient permissions
    /// - Disk full
    /// - Corrupted file (invalid magic bytes)
    ///
    /// ## Side Effects
    /// - Creates `.btree` file if doesn't exist
    /// - Creates `.wal` file if doesn't exist
    /// - Replays WAL if found (applies uncommitted changes)
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::btree_index::BTreeIndex;
    /// use reedbase::btree::Order;
    ///
    /// let order = Order::new(100)?;
    /// let index = BTreeIndex::<String, Vec<usize>>::open("index.btree", order)?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn open<P: AsRef<Path>>(path: P, order: Order) -> ReedResult<Self> {
        let tree = BPlusTree::open(path, order)?;
        Ok(Self { tree })
    }

    /// Get reference to underlying B+-Tree.
    ///
    /// ## Output
    /// - Reference to internal BPlusTree
    ///
    /// ## Performance
    /// - O(1) constant time
    pub fn tree(&self) -> &BPlusTree<K, V> {
        &self.tree
    }

    /// Get mutable reference to underlying B+-Tree.
    ///
    /// ## Output
    /// - Mutable reference to internal BPlusTree
    ///
    /// ## Performance
    /// - O(1) constant time
    pub fn tree_mut(&mut self) -> &mut BPlusTree<K, V> {
        &mut self.tree
    }
}

impl<K, V> Index<K, V> for BTreeIndex<K, V>
where
    K: Clone + Ord + Serialize + for<'de> Deserialize<'de> + Send + Sync + std::fmt::Debug,
    V: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + std::fmt::Debug,
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
    /// - O(log n) tree traversal
    /// - < 1ms typical (3-4 page reads)
    ///
    /// ## Error Conditions
    /// - I/O error reading pages
    /// - Corrupted page data
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::{btree_index::BTreeIndex, Index};
    /// use reedbase::btree::Order;
    ///
    /// let order = Order::new(100)?;
    /// let mut index = BTreeIndex::<String, Vec<usize>>::open("index.btree", order)?;
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
        self.tree.get(key)
    }

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
    /// - O(log n + k) where k = result size
    /// - < 5ms for 100 keys
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::{btree_index::BTreeIndex, Index};
    /// use reedbase::btree::Order;
    ///
    /// let order = Order::new(100)?;
    /// let mut index = BTreeIndex::<String, Vec<usize>>::open("index.btree", order)?;
    /// index.insert("page.title".to_string(), vec![1])?;
    /// index.insert("page.description".to_string(), vec![2])?;
    /// index.insert("api.endpoint".to_string(), vec![3])?;
    ///
    /// let results = index.range(&"page.a".to_string(), &"page.z".to_string())?;
    /// assert_eq!(results.len(), 2); // page.title and page.description
    ///
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    fn range(&self, start: &K, end: &K) -> ReedResult<Vec<(K, V)>> {
        self.tree.range(start, end)
    }

    /// Insert or update key-value pair.
    ///
    /// ## Input
    /// - `key`: Key to insert/update
    /// - `value`: Value to store
    ///
    /// ## Performance
    /// - O(log n) tree traversal
    /// - < 2ms typical (including WAL write)
    ///
    /// ## Error Conditions
    /// - Disk full (cannot allocate new pages)
    /// - I/O error during write
    /// - Page split failures
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::{btree_index::BTreeIndex, Index};
    /// use reedbase::btree::Order;
    ///
    /// let order = Order::new(100)?;
    /// let mut index = BTreeIndex::<String, Vec<usize>>::open("index.btree", order)?;
    /// index.insert("page".to_string(), vec![1, 2])?;
    /// index.insert("page".to_string(), vec![3, 4])?; // Update
    ///
    /// let value = index.get(&"page".to_string())?;
    /// assert_eq!(value, Some(vec![3, 4]));
    ///
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    fn insert(&mut self, key: K, value: V) -> ReedResult<()> {
        self.tree.insert(key, value)
    }

    /// Delete key-value pair.
    ///
    /// ## Input
    /// - `key`: Key to delete
    ///
    /// ## Performance
    /// - O(log n) tree traversal
    /// - < 2ms typical (including WAL write)
    ///
    /// ## Error Conditions
    /// - I/O error during write
    /// - Page merge failures
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::{btree_index::BTreeIndex, Index};
    /// use reedbase::btree::Order;
    ///
    /// let order = Order::new(100)?;
    /// let mut index = BTreeIndex::<String, Vec<usize>>::open("index.btree", order)?;
    /// index.insert("page".to_string(), vec![1, 2])?;
    /// index.delete(&"page".to_string())?;
    ///
    /// let value = index.get(&"page".to_string())?;
    /// assert_eq!(value, None);
    ///
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    fn delete(&mut self, key: &K) -> ReedResult<()> {
        self.tree.delete(key)
    }

    /// Iterate all key-value pairs in sorted order.
    ///
    /// ## Output
    /// - Iterator yielding (key, value) pairs in ascending key order
    ///
    /// ## Performance
    /// - O(n) where n = number of entries
    /// - Sequential leaf page access
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::{btree_index::BTreeIndex, Index};
    /// use reedbase::btree::Order;
    ///
    /// let order = Order::new(100)?;
    /// let mut index = BTreeIndex::<String, Vec<usize>>::open("index.btree", order)?;
    /// index.insert("page".to_string(), vec![1, 2])?;
    /// index.insert("api".to_string(), vec![3, 4])?;
    ///
    /// let entries: Vec<_> = index.iter().collect();
    /// assert_eq!(entries.len(), 2);
    /// // Entries are in sorted order: api, page
    ///
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)> + '_> {
        self.tree.iter()
    }

    /// Backend type identifier.
    ///
    /// ## Returns
    /// - `"btree"` for on-disk B+-Tree
    ///
    /// ## Performance
    /// - O(1) constant time
    fn backend_type(&self) -> &'static str {
        "btree"
    }

    /// Estimated memory usage in bytes.
    ///
    /// ## Returns
    /// - Page cache size (not full file size)
    ///
    /// ## Performance
    /// - O(1) constant time
    ///
    /// ## Note
    /// B+-Tree uses mmap which is demand-paged by the OS.
    /// This reports only the in-memory structures, not the mmap size.
    fn memory_usage(&self) -> usize {
        self.tree.memory_usage()
    }

    /// Disk usage in bytes.
    ///
    /// ## Returns
    /// - File size + WAL size
    ///
    /// ## Performance
    /// - O(1) constant time
    fn disk_usage(&self) -> usize {
        self.tree.disk_usage()
    }
}
