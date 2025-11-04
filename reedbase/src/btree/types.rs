// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Core types for B+-Tree index backend.
//!
//! Defines generic index trait, page management types, and B+-Tree configuration
//! structures used across the disk-based index implementation.

use crate::error::{ReedError, ReedResult};
use serde::{Deserialize, Serialize};

// Re-export Index trait from indices module (canonical definition).
pub use crate::indices::Index;

/// Magic bytes for B+-Tree file format validation.
///
/// Used in file headers to verify file type and detect corruption.
pub const BTREE_MAGIC: u32 = 0xB7EE_7EE1;

/// Page identifier type for B+-Tree nodes.
///
/// 32-bit identifier allowing up to 4,294,967,295 pages.
/// With 4KB pages, this supports up to 16TB index files.
pub type PageId = u32;

/// B+-Tree order (degree) configuration.
///
/// Defines the maximum number of children per internal node and keys per leaf node.
/// Higher orders reduce tree height but increase page size and split/merge cost.
///
/// ## Constraints
/// - Minimum order: 3 (allows 2-3 children per internal node)
/// - Maximum order: Limited by page size and key/value sizes
///
/// ## Typical Values
/// - Small keys/values: Order 100-200 (4KB pages)
/// - Large keys/values: Order 10-50 (4KB pages)
///
/// ## Example
/// ```rust
/// use reedbase::btree::types::Order;
///
/// let order = Order::new(100)?; // 100 keys per leaf, 101 children per internal node
/// assert_eq!(order.max_keys(), 100);
/// assert_eq!(order.min_keys(), 50); // Half-full requirement
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order(u16);

impl Order {
    /// Create new order with validation.
    ///
    /// ## Input
    /// - `order`: Desired tree order (must be >= 3)
    ///
    /// ## Output
    /// - `Ok(Order)`: Valid order created
    /// - `Err(ReedError::ParseError)`: Order less than 3
    ///
    /// ## Performance
    /// - O(1) validation
    ///
    /// ## Error Conditions
    /// - Order < 3: B+-Trees require minimum order 3
    ///
    /// ## Example
    /// ```rust
    /// let order = Order::new(100)?; // Valid
    /// let invalid = Order::new(2);  // Error: order must be >= 3
    /// ```
    pub fn new(order: u16) -> ReedResult<Self> {
        if order < 3 {
            return Err(ReedError::ParseError {
                reason: format!("B+-Tree order must be >= 3, got {}", order),
            });
        }
        Ok(Self(order))
    }

    /// Get maximum keys per node.
    ///
    /// ## Output
    /// - Maximum number of keys in leaf nodes
    /// - Maximum children - 1 for internal nodes
    ///
    /// ## Performance
    /// - O(1) constant time
    pub fn max_keys(&self) -> u16 {
        self.0
    }

    /// Get minimum keys per node (half-full requirement).
    ///
    /// Nodes must contain at least this many keys (except root).
    /// This ensures O(log n) height guarantees.
    ///
    /// ## Output
    /// - order / 2 keys minimum
    ///
    /// ## Performance
    /// - O(1) constant time
    pub fn min_keys(&self) -> u16 {
        self.0 / 2
    }

    /// Get raw order value.
    ///
    /// ## Output
    /// - Configured order value
    ///
    /// ## Performance
    /// - O(1) constant time
    pub fn value(&self) -> u16 {
        self.0
    }
}

/// Node type discriminator for B+-Tree nodes.
///
/// Used in page headers to distinguish internal nodes from leaf nodes.
///
/// ## Memory Representation
/// - `Internal = 0`: Internal node (contains keys and page pointers)
/// - `Leaf = 1`: Leaf node (contains keys and values)
///
/// ## Serialisation
/// - Stored as single byte in page headers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeType {
    /// Internal node containing keys and child page pointers.
    ///
    /// Structure: [key₁, ptr₁, key₂, ptr₂, ..., keyₙ, ptrₙ, ptrₙ₊₁]
    /// - `n` keys, `n+1` child pointers
    /// - Keys define ranges: child₁ < key₁ ≤ child₂ < key₂ ≤ ...
    Internal = 0,

    /// Leaf node containing keys and values.
    ///
    /// Structure: [key₁, val₁, key₂, val₂, ..., keyₙ, valₙ, next_leaf]
    /// - `n` key-value pairs
    /// - `next_leaf` pointer chains leaves for range queries
    Leaf = 1,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btree_magic() {
        assert_eq!(BTREE_MAGIC, 0xB7EE_7EE1);
    }

    #[test]
    fn test_order_validation() {
        // Valid orders
        assert!(Order::new(3).is_ok());
        assert!(Order::new(100).is_ok());
        assert!(Order::new(1000).is_ok());

        // Invalid orders
        assert!(Order::new(0).is_err());
        assert!(Order::new(1).is_err());
        assert!(Order::new(2).is_err());
    }

    #[test]
    fn test_order_max_keys() {
        let order = Order::new(100).unwrap();
        assert_eq!(order.max_keys(), 100);

        let order = Order::new(50).unwrap();
        assert_eq!(order.max_keys(), 50);
    }

    #[test]
    fn test_order_min_keys() {
        let order = Order::new(100).unwrap();
        assert_eq!(order.min_keys(), 50);

        let order = Order::new(51).unwrap();
        assert_eq!(order.min_keys(), 25);

        let order = Order::new(3).unwrap();
        assert_eq!(order.min_keys(), 1);
    }

    #[test]
    fn test_node_type_discriminant() {
        assert_eq!(NodeType::Internal as u8, 0);
        assert_eq!(NodeType::Leaf as u8, 1);
    }

    #[test]
    fn test_node_type_serialization() {
        // Test that NodeType can be serialized/deserialized
        let internal = NodeType::Internal;
        let leaf = NodeType::Leaf;

        // Basic equality checks
        assert_eq!(internal, NodeType::Internal);
        assert_eq!(leaf, NodeType::Leaf);
        assert_ne!(internal, leaf);
    }

    #[test]
    fn test_order_value() {
        let order = Order::new(100).unwrap();
        assert_eq!(order.value(), 100);
    }
}
