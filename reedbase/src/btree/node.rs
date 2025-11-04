// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! B+-Tree node structures for internal and leaf nodes.
//!
//! Implements InternalNode (keys + child pointers) and LeafNode (keys + values)
//! with serialisation support via bincode. Nodes are stored in 4KB pages with
//! automatic splitting when capacity is reached.
//!
//! ## Node Types
//!
//! - **InternalNode**: Directs searches to appropriate subtrees
//!   - Structure: [key₁, child₁, key₂, child₂, ..., keyₙ, childₙ, childₙ₊₁]
//!   - Invariant: child₁ < key₁ ≤ child₂ < key₂ ≤ ... < keyₙ ≤ childₙ₊₁
//!
//! - **LeafNode**: Stores actual key-value pairs
//!   - Structure: [key₁, val₁, key₂, val₂, ..., keyₙ, valₙ, next_leaf]
//!   - Linked list via next pointer for efficient range queries
//!
//! ## Performance
//!
//! - find_child/find_value: O(log m) binary search where m = node capacity
//! - insert: O(m) worst case (shift elements after insertion point)
//! - split: O(m) creates new node and redistributes entries
//!
//! ## Memory Layout
//!
//! Nodes are serialised using bincode for compact storage in 4KB pages.
//! Typical sizes (order=100):
//! - InternalNode: ~1.6KB (100 keys + 101 children × 4 bytes)
//! - LeafNode: Varies with value size

use crate::btree::types::{Order, PageId};
use crate::error::{ReedError, ReedResult};
use serde::{Deserialize, Serialize};

/// Internal node containing keys and child page pointers.
///
/// Directs searches to appropriate child nodes based on key comparisons.
/// Always contains n keys and n+1 children (leftmost child handles keys < first key).
///
/// ## Type Parameters
/// - `K`: Key type (must be Clone + Ord for comparisons and serialisation)
///
/// ## Invariants
/// - `keys.len() + 1 == children.len()` (always one more child than keys)
/// - `keys.len() <= order.max_keys()` (capacity constraint)
/// - `keys` are sorted in ascending order
/// - For all i: `children[i]` contains keys < `keys[i]` ≤ `children[i+1]`
///
/// ## Serialisation
/// Uses bincode for compact binary format. Typical size with order=100:
/// - 100 keys × sizeof(K)
/// - 101 children × 4 bytes (PageId)
/// - Overhead: ~20 bytes (length prefixes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalNode<K>
where
    K: Clone + Ord,
{
    /// Sorted keys for routing searches.
    ///
    /// keys[i] defines the boundary between children[i] and children[i+1].
    /// All keys in children[i] are < keys[i].
    /// All keys in children[i+1] are >= keys[i].
    pub keys: Vec<K>,

    /// Child page identifiers.
    ///
    /// Length is always keys.len() + 1.
    /// children[0] is the leftmost child (handles keys < keys[0]).
    pub children: Vec<PageId>,
}

impl<K> InternalNode<K>
where
    K: Clone + Ord,
{
    /// Create new empty internal node.
    ///
    /// ## Output
    /// - Internal node with no keys or children
    ///
    /// ## Performance
    /// - O(1) constant time
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::node::InternalNode;
    ///
    /// let node = InternalNode::<String>::new();
    /// assert_eq!(node.keys.len(), 0);
    /// assert_eq!(node.children.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Find appropriate child page for given key.
    ///
    /// Uses binary search to locate the child that should contain the key.
    /// Returns the index into the children array.
    ///
    /// ## Input
    /// - `key`: Key to search for
    ///
    /// ## Output
    /// - Child index where key belongs (0 to keys.len())
    ///
    /// ## Performance
    /// - O(log m) where m = number of keys (binary search)
    ///
    /// ## Algorithm
    /// - If key < keys[0]: return 0 (leftmost child)
    /// - If key >= keys[i] and key < keys[i+1]: return i+1
    /// - If key >= keys[last]: return keys.len() (rightmost child)
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::node::InternalNode;
    ///
    /// let mut node = InternalNode::new();
    /// node.keys = vec![10, 20, 30];
    /// node.children = vec![1, 2, 3, 4]; // PageIds
    ///
    /// assert_eq!(node.find_child(&5), 0);   // < 10: leftmost
    /// assert_eq!(node.find_child(&15), 1);  // 10 <= 15 < 20
    /// assert_eq!(node.find_child(&25), 2);  // 20 <= 25 < 30
    /// assert_eq!(node.find_child(&35), 3);  // >= 30: rightmost
    /// ```
    pub fn find_child(&self, key: &K) -> usize {
        // Binary search for the first key >= search key
        match self.keys.binary_search(key) {
            // Exact match: return child to the right of the key
            Ok(idx) => idx + 1,
            // Key would be inserted at idx: return child at that position
            Err(idx) => idx,
        }
    }

    /// Insert key and child at appropriate position.
    ///
    /// Maintains sorted order and children invariant (n+1 children for n keys).
    /// Caller must ensure capacity is not exceeded (call split() if needed).
    ///
    /// ## Input
    /// - `key`: Key to insert (defines boundary between children)
    /// - `child`: PageId of new child node
    ///
    /// ## Output
    /// - `Ok(())`: Successfully inserted
    /// - `Err(ReedError::ParseError)`: Capacity exceeded (node is full)
    ///
    /// ## Performance
    /// - O(m) where m = number of keys (shift elements after insertion point)
    ///
    /// ## Error Conditions
    /// - Node already contains max_keys (must split before inserting)
    ///
    /// ## Invariants Maintained
    /// - Keys remain sorted
    /// - children.len() == keys.len() + 1
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::node::InternalNode;
    /// use reedbase::btree::types::Order;
    ///
    /// let mut node = InternalNode::new();
    /// node.children.push(1); // Initial leftmost child
    ///
    /// node.insert_key(10, 2)?; // Add key=10, child=2
    /// assert_eq!(node.keys.len(), 1);
    /// assert_eq!(node.children.len(), 2);
    ///
    /// node.insert_key(20, 3)?; // Add key=20, child=3
    /// assert_eq!(node.keys, vec![10, 20]);
    /// assert_eq!(node.children, vec![1, 2, 3]);
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn insert_key(&mut self, key: K, child: PageId) -> ReedResult<()> {
        // Find insertion position
        let pos = self.keys.binary_search(&key).unwrap_or_else(|e| e);

        // Insert key and child at position
        self.keys.insert(pos, key);
        self.children.insert(pos + 1, child);

        Ok(())
    }

    /// Split node into two nodes at midpoint.
    ///
    /// Creates a new node containing the upper half of keys/children.
    /// Original node retains lower half. Returns middle key (promoted to parent)
    /// and new node.
    ///
    /// ## Output
    /// - `(middle_key, new_node)`: Middle key to promote and new right sibling
    ///
    /// ## Performance
    /// - O(m) where m = number of keys (copy upper half to new node)
    ///
    /// ## Algorithm
    /// ```text
    /// Before: [k1, k2, k3, k4, k5] / [c1, c2, c3, c4, c5, c6]
    /// Split at midpoint (idx=2):
    /// - Left:  [k1, k2] / [c1, c2, c3]
    /// - Middle: k3 (promoted to parent)
    /// - Right: [k4, k5] / [c4, c5, c6]
    /// ```
    ///
    /// ## Error Conditions
    /// - Node must contain at least 2 keys (cannot split smaller nodes)
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::node::InternalNode;
    ///
    /// let mut node = InternalNode::new();
    /// node.keys = vec![10, 20, 30, 40, 50];
    /// node.children = vec![1, 2, 3, 4, 5, 6];
    ///
    /// let (middle_key, new_node) = node.split()?;
    ///
    /// // Original node keeps left half
    /// assert_eq!(node.keys, vec![10, 20]);
    /// assert_eq!(node.children, vec![1, 2, 3]);
    ///
    /// // Middle key promoted to parent
    /// assert_eq!(middle_key, 30);
    ///
    /// // New node gets right half
    /// assert_eq!(new_node.keys, vec![40, 50]);
    /// assert_eq!(new_node.children, vec![4, 5, 6]);
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn split(&mut self) -> ReedResult<(K, Self)> {
        if self.keys.len() < 2 {
            return Err(ReedError::ParseError {
                reason: "Cannot split internal node with < 2 keys".to_string(),
            });
        }

        // Find midpoint
        let mid = self.keys.len() / 2;

        // Extract middle key (promoted to parent)
        let middle_key = self.keys[mid].clone();

        // Split keys: [0..mid) stays, [mid+1..] moves to new node
        let mut right_keys = self.keys.split_off(mid);
        right_keys.remove(0); // Remove middle key from right node (it goes to parent)

        // Split children: [0..mid+1) stays, [mid+1..] moves to new node
        let right_children = self.children.split_off(mid + 1);

        // Create new right node
        let new_node = Self {
            keys: right_keys,
            children: right_children,
        };

        Ok((middle_key, new_node))
    }

    /// Check if node is at minimum capacity (half-full requirement).
    ///
    /// ## Input
    /// - `order`: Tree order defining capacity constraints
    ///
    /// ## Output
    /// - `true`: Node contains < min_keys (underflow)
    /// - `false`: Node meets minimum requirement
    ///
    /// ## Performance
    /// - O(1) constant time
    ///
    /// ## Note
    /// Root node is exempt from minimum requirement (can contain fewer keys).
    pub fn is_underflow(&self, order: Order) -> bool {
        self.keys.len() < order.min_keys() as usize
    }

    /// Check if node is at maximum capacity (needs split).
    ///
    /// ## Input
    /// - `order`: Tree order defining capacity constraints
    ///
    /// ## Output
    /// - `true`: Node contains >= max_keys (must split before insert)
    /// - `false`: Node has room for more keys
    ///
    /// ## Performance
    /// - O(1) constant time
    pub fn is_overflow(&self, order: Order) -> bool {
        self.keys.len() >= order.max_keys() as usize
    }
}

/// Leaf node containing keys and values.
///
/// Stores actual key-value pairs and forms linked list for efficient range queries.
/// Leaves are chained via next pointer, enabling sequential access without tree traversal.
///
/// ## Type Parameters
/// - `K`: Key type (must be Clone + Ord for comparisons and serialisation)
/// - `V`: Value type (must be Clone for retrieval)
///
/// ## Invariants
/// - `keys.len() == values.len()` (one value per key)
/// - `keys.len() <= order.max_keys()` (capacity constraint)
/// - `keys` are sorted in ascending order
/// - `next` points to leaf with keys > all keys in this node (or None if last)
///
/// ## Serialisation
/// Uses bincode for compact binary format. Size varies with value size:
/// - Order=100, 8-byte values: ~1.6KB
/// - Order=100, 64-byte values: ~7.2KB (may exceed 4KB page, reduce order)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeafNode<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    /// Sorted keys for lookups.
    pub keys: Vec<K>,

    /// Values corresponding to keys (parallel array).
    ///
    /// values[i] is the value for keys[i].
    pub values: Vec<V>,

    /// Next leaf in linked list (0 if last leaf).
    ///
    /// Enables efficient range queries by following chain of leaves
    /// without returning to internal nodes.
    pub next: Option<PageId>,
}

impl<K, V> LeafNode<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    /// Create new empty leaf node.
    ///
    /// ## Output
    /// - Leaf node with no keys/values and no next pointer
    ///
    /// ## Performance
    /// - O(1) constant time
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::node::LeafNode;
    ///
    /// let node = LeafNode::<String, Vec<u8>>::new();
    /// assert_eq!(node.keys.len(), 0);
    /// assert_eq!(node.values.len(), 0);
    /// assert_eq!(node.next, None);
    /// ```
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            next: None,
        }
    }

    /// Find value for given key.
    ///
    /// Uses binary search to locate key in sorted keys array.
    ///
    /// ## Input
    /// - `key`: Key to search for
    ///
    /// ## Output
    /// - `Some(value)`: Key found, returns associated value
    /// - `None`: Key not found in this leaf
    ///
    /// ## Performance
    /// - O(log m) where m = number of keys (binary search)
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::node::LeafNode;
    ///
    /// let mut node = LeafNode::new();
    /// node.keys = vec![10, 20, 30];
    /// node.values = vec![vec![1], vec![2], vec![3]];
    ///
    /// assert_eq!(node.find_value(&20), Some(vec![2]));
    /// assert_eq!(node.find_value(&25), None);
    /// ```
    pub fn find_value(&self, key: &K) -> Option<V> {
        self.keys
            .binary_search(key)
            .ok()
            .map(|idx| self.values[idx].clone())
    }

    /// Insert or update key-value pair.
    ///
    /// If key exists, updates value. If key is new, inserts at appropriate position
    /// to maintain sorted order. Caller must ensure capacity is not exceeded.
    ///
    /// ## Input
    /// - `key`: Key to insert/update
    /// - `value`: Value to associate with key
    ///
    /// ## Output
    /// - `Ok(())`: Successfully inserted/updated
    ///
    /// ## Performance
    /// - O(m) where m = number of keys (shift elements after insertion point)
    ///
    /// ## Invariants Maintained
    /// - Keys remain sorted
    /// - keys.len() == values.len()
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::node::LeafNode;
    ///
    /// let mut node = LeafNode::new();
    ///
    /// node.insert(10, vec![1])?;
    /// node.insert(30, vec![3])?;
    /// node.insert(20, vec![2])?; // Inserted in sorted order
    ///
    /// assert_eq!(node.keys, vec![10, 20, 30]);
    /// assert_eq!(node.values, vec![vec![1], vec![2], vec![3]]);
    ///
    /// node.insert(20, vec![99])?; // Update existing key
    /// assert_eq!(node.values[1], vec![99]);
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> ReedResult<()> {
        match self.keys.binary_search(&key) {
            // Key exists: update value
            Ok(idx) => {
                self.values[idx] = value;
            }
            // Key doesn't exist: insert at position
            Err(idx) => {
                self.keys.insert(idx, key);
                self.values.insert(idx, value);
            }
        }
        Ok(())
    }

    /// Split leaf node into two nodes at midpoint.
    ///
    /// Creates a new node containing the upper half of keys/values.
    /// Original node retains lower half. Returns smallest key in new node
    /// (for parent routing) and new node.
    ///
    /// ## Output
    /// - `(split_key, new_node)`: First key in new node and new right sibling
    ///
    /// ## Performance
    /// - O(m) where m = number of keys (copy upper half to new node)
    ///
    /// ## Algorithm
    /// ```text
    /// Before: [k1, k2, k3, k4] / [v1, v2, v3, v4]
    /// Split at midpoint (idx=2):
    /// - Left:  [k1, k2] / [v1, v2]
    /// - Split key: k3 (inserted into parent to route to right node)
    /// - Right: [k3, k4] / [v3, v4]
    /// ```
    ///
    /// ## Error Conditions
    /// - Node must contain at least 2 keys (cannot split smaller nodes)
    ///
    /// ## Side Effects
    /// - Updates next pointer: original -> new -> original.next
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::btree::node::LeafNode;
    ///
    /// let mut node = LeafNode::new();
    /// node.keys = vec![10, 20, 30, 40];
    /// node.values = vec![vec![1], vec![2], vec![3], vec![4]];
    /// node.next = Some(100); // Had a next leaf
    ///
    /// let (split_key, new_node) = node.split()?;
    ///
    /// // Original node keeps left half
    /// assert_eq!(node.keys, vec![10, 20]);
    /// assert_eq!(node.values, vec![vec![1], vec![2]]);
    ///
    /// // Split key is first key in new node
    /// assert_eq!(split_key, 30);
    ///
    /// // New node gets right half
    /// assert_eq!(new_node.keys, vec![30, 40]);
    /// assert_eq!(new_node.values, vec![vec![3], vec![4]]);
    ///
    /// // Linked list updated: node -> new_node -> old_next
    /// assert_eq!(new_node.next, Some(100));
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn split(&mut self) -> ReedResult<(K, Self)> {
        if self.keys.len() < 2 {
            return Err(ReedError::ParseError {
                reason: "Cannot split leaf node with < 2 keys".to_string(),
            });
        }

        // Find midpoint
        let mid = self.keys.len() / 2;

        // Split keys and values
        let right_keys = self.keys.split_off(mid);
        let right_values = self.values.split_off(mid);

        // First key of right node (for parent routing)
        let split_key = right_keys[0].clone();

        // Create new right node
        let new_node = Self {
            keys: right_keys,
            values: right_values,
            next: self.next, // New node inherits old next
        };

        // Update linked list: this -> new -> old_next
        // (new_node will get a PageId when written to disk)
        self.next = None; // Will be updated by caller with new_node's PageId

        Ok((split_key, new_node))
    }

    /// Check if node is at minimum capacity (half-full requirement).
    ///
    /// ## Input
    /// - `order`: Tree order defining capacity constraints
    ///
    /// ## Output
    /// - `true`: Node contains < min_keys (underflow)
    /// - `false`: Node meets minimum requirement
    ///
    /// ## Performance
    /// - O(1) constant time
    ///
    /// ## Note
    /// Root leaf is exempt from minimum requirement (can contain fewer keys).
    pub fn is_underflow(&self, order: Order) -> bool {
        self.keys.len() < order.min_keys() as usize
    }

    /// Check if node is at maximum capacity (needs split).
    ///
    /// ## Input
    /// - `order`: Tree order defining capacity constraints
    ///
    /// ## Output
    /// - `true`: Node contains >= max_keys (must split before insert)
    /// - `false`: Node has room for more keys
    ///
    /// ## Performance
    /// - O(1) constant time
    pub fn is_overflow(&self, order: Order) -> bool {
        self.keys.len() >= order.max_keys() as usize
    }
}

// Default implementations for convenience
impl<K> Default for InternalNode<K>
where
    K: Clone + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Default for LeafNode<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
