// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Range scan iterator for B+-Tree leaf traversal.
//!
//! Implements efficient range queries by walking the linked list of leaf nodes.
//! Avoids tree traversal after finding start position by following next pointers.
//!
//! ## Algorithm
//!
//! 1. Search tree for starting leaf (O(log n))
//! 2. Scan keys in current leaf
//! 3. Follow next pointer to next leaf
//! 4. Repeat until end key reached or last leaf
//!
//! ## Performance
//!
//! - Initialisation: O(log n) to find start leaf
//! - Per-item: O(1) amortised (sequential page reads)
//! - Total: O(log n + k) where k = result count
//!
//! ## Memory Usage
//!
//! - Current leaf loaded in memory (~4KB)
//! - No buffering of results (lazy evaluation)
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::btree::{BPlusTree, Order, Index};
//!
//! let tree = BPlusTree::<String, Vec<u8>>::open("index.btree", Order::new(100)?)?;
//!
//! // Range query via Index trait
//! let results = tree.range(&"page.a".to_string(), &"page.z".to_string())?;
//!
//! // Or use iterator directly (via iter() method)
//! for (key, value) in tree.iter() {
//!     println!("{:?}: {:?}", key, value);
//! }
//! # Ok::<(), reedbase::ReedError>(())
//! ```

use crate::btree::node::LeafNode;
use crate::btree::page::Page;
use crate::btree::types::{NodeType, PageId};
use crate::error::ReedResult;
use memmap2::Mmap;
use serde::Deserialize;
use std::marker::PhantomData;

/// Iterator for range scans over B+-Tree leaves.
///
/// Walks linked list of leaf nodes, yielding key-value pairs within range.
/// Stops when end bound reached or last leaf encountered.
///
/// ## Type Parameters
/// - `'a`: Lifetime of mmap reference
/// - `K`: Key type (must be Clone + Ord + Deserialize)
/// - `V`: Value type (must be Clone + Deserialize)
///
/// ## Lifetime
/// Iterator borrows mmap for entire traversal. Cannot modify tree during iteration.
///
/// ## Error Handling
/// Errors (I/O, corruption) cause iterator to terminate early.
/// Check result count if expecting specific number of items.
pub struct RangeScanIterator<'a, K, V>
where
    K: Clone + Ord + for<'de> Deserialize<'de>,
    V: Clone + for<'de> Deserialize<'de>,
{
    /// Memory-mapped file reference (readonly).
    mmap: &'a Mmap,

    /// Current leaf page being scanned.
    current_page: Option<PageId>,

    /// Current leaf node (loaded from current_page).
    current_leaf: Option<LeafNode<K, V>>,

    /// Current position within leaf keys.
    key_index: usize,

    /// Inclusive start bound (filter keys >= start).
    start: K,

    /// Exclusive end bound (stop when key >= end).
    end: K,

    /// Whether iterator has been exhausted.
    done: bool,

    /// Phantom data for type parameters.
    _phantom: PhantomData<&'a (K, V)>,
}

impl<'a, K, V> RangeScanIterator<'a, K, V>
where
    K: Clone + Ord + for<'de> Deserialize<'de>,
    V: Clone + for<'de> Deserialize<'de>,
{
    /// Create new range scan iterator.
    ///
    /// ## Input
    /// - `mmap`: Memory-mapped file reference
    /// - `start_page`: Page ID of leftmost leaf in range
    /// - `start`: Inclusive start bound
    /// - `end`: Exclusive end bound
    ///
    /// ## Output
    /// - Iterator ready to yield first key-value pair
    ///
    /// ## Performance
    /// - O(1) setup (start_page already found by caller)
    ///
    /// ## Example
    /// ```rust
    /// // Internal usage by BPlusTree::range()
    /// let iter = RangeScanIterator::new(&mmap, start_page, start_key, end_key);
    /// for (key, value) in iter {
    ///     // Process results
    /// }
    /// ```
    pub fn new(mmap: &'a Mmap, start_page: PageId, start: K, end: K) -> Self {
        Self {
            mmap,
            current_page: Some(start_page),
            current_leaf: None,
            key_index: 0,
            start,
            end,
            done: false,
            _phantom: PhantomData,
        }
    }

    /// Load next leaf page into memory.
    ///
    /// ## Output
    /// - `Ok(true)`: Leaf loaded successfully
    /// - `Ok(false)`: No more leaves (end of chain)
    /// - `Err(ReedError)`: I/O or corruption error
    ///
    /// ## Performance
    /// - O(1) page read (~10μs from cache, ~1ms from disk)
    ///
    /// ## Side Effects
    /// - Updates `current_leaf` with deserialised node
    /// - Resets `key_index` to 0
    fn load_next_leaf(&mut self) -> ReedResult<bool> {
        let page_id = match self.current_page {
            Some(id) => id,
            None => return Ok(false), // No more pages
        };

        // Read page from mmap
        let page = Page::read_from(self.mmap, page_id)?;

        // Validate page type
        if page.header.page_type != NodeType::Leaf as u8 {
            self.done = true;
            return Ok(false);
        }

        // Deserialise leaf node
        let leaf: LeafNode<K, V> = bincode::deserialize(page.get_data()).map_err(|e| {
            crate::error::ReedError::DeserializationError {
                reason: e.to_string(),
            }
        })?;

        // Update state
        self.current_page = leaf.next;
        self.current_leaf = Some(leaf);
        self.key_index = 0;

        Ok(true)
    }
}

impl<'a, K, V> Iterator for RangeScanIterator<'a, K, V>
where
    K: Clone + Ord + for<'de> Deserialize<'de>,
    V: Clone + for<'de> Deserialize<'de>,
{
    type Item = (K, V);

    /// Get next key-value pair in range.
    ///
    /// ## Output
    /// - `Some((K, V))`: Next pair within [start, end)
    /// - `None`: End of range or error encountered
    ///
    /// ## Performance
    /// - O(1) amortised per call
    /// - Page loads amortised across all keys in leaf
    ///
    /// ## Algorithm
    /// ```text
    /// loop:
    ///   if no current_leaf:
    ///     load_next_leaf()
    ///     if failed: return None
    ///
    ///   if key_index < leaf.keys.len():
    ///     key = leaf.keys[key_index]
    ///     if key < start: skip
    ///     if key >= end: return None
    ///     return Some((key, value))
    ///   else:
    ///     load_next_leaf()
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        loop {
            // Load leaf if not already loaded
            if self.current_leaf.is_none() {
                match self.load_next_leaf() {
                    Ok(true) => {} // Leaf loaded
                    Ok(false) => {
                        self.done = true;
                        return None;
                    } // No more leaves
                    Err(_) => {
                        self.done = true;
                        return None;
                    } // Error
                }
            }

            let leaf = self.current_leaf.as_ref().unwrap();

            // Check if we have more keys in current leaf
            if self.key_index < leaf.keys.len() {
                let key = &leaf.keys[self.key_index];
                let value = &leaf.values[self.key_index];
                self.key_index += 1;

                // Filter by range
                if key < &self.start {
                    continue; // Skip keys before start
                }
                if key >= &self.end {
                    self.done = true;
                    return None; // Reached end bound
                }

                return Some((key.clone(), value.clone()));
            } else {
                // Exhausted current leaf, move to next
                self.current_leaf = None;
                match self.load_next_leaf() {
                    Ok(true) => {} // Loaded next leaf
                    Ok(false) => {
                        self.done = true;
                        return None;
                    } // No more leaves
                    Err(_) => {
                        self.done = true;
                        return None;
                    } // Error
                }
            }
        }
    }

    /// Provide size hint for iterator.
    ///
    /// ## Output
    /// - `(0, None)`: Cannot determine size without full scan
    ///
    /// ## Performance
    /// - O(1) constant time
    ///
    /// ## Note
    /// Size hint is (0, None) because we don't know result count upfront.
    /// Determining size would require scanning entire range (defeating lazy evaluation).
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Cannot determine size without scanning
        (0, None)
    }
}

// Additional trait implementations for convenience

/// Debug implementation for iterator.
impl<'a, K, V> std::fmt::Debug for RangeScanIterator<'a, K, V>
where
    K: Clone + Ord + std::fmt::Debug + for<'de> Deserialize<'de>,
    V: Clone + std::fmt::Debug + for<'de> Deserialize<'de>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RangeScanIterator")
            .field("current_page", &self.current_page)
            .field("key_index", &self.key_index)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("done", &self.done)
            .finish()
    }
}
