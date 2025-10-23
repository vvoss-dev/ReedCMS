// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Namespace index for O(1) prefix lookups.
//!
//! Maps namespace strings to row numbers for instant filtered queries.
//!
//! ## Performance
//! - Build: O(n) where n = number of keys
//! - Query: O(1) HashMap lookup
//! - Memory: ~20 bytes per key
//!
//! ## Example Usage
//! ```rust
//! use reedbase::indices::namespace::NamespaceIndex;
//! use reedbase::indices::types::KeyIndex;
//!
//! let mut index = NamespaceIndex::new();
//!
//! let keys = vec![
//!     KeyIndex { namespace: "page".into(), row: 0, ... },
//!     KeyIndex { namespace: "page".into(), row: 5, ... },
//!     KeyIndex { namespace: "api".into(), row: 10, ... },
//! ];
//!
//! index.build(&keys)?;
//!
//! // O(1) lookup
//! let rows = index.query("page")?; // Returns [0, 5]
//! # Ok::<(), reedbase::ReedError>(())
//! ```

use crate::error::ReedResult;
use crate::indices::types::KeyIndex;
use std::collections::HashMap;

/// Namespace index for O(1) prefix lookups.
///
/// Maps namespace → Vec<row_numbers>.
pub struct NamespaceIndex {
    /// namespace → Vec<row_numbers>
    map: HashMap<String, Vec<usize>>,
}

impl NamespaceIndex {
    /// Create a new empty namespace index.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Build index from key indices.
    ///
    /// ## Input
    /// - `keys` - Array of parsed key indices
    ///
    /// ## Performance
    /// - O(n) where n = number of keys
    /// - < 20ms for 10,000 keys
    pub fn build(&mut self, keys: &[KeyIndex]) -> ReedResult<()> {
        self.map.clear();

        for key_index in keys {
            self.map
                .entry(key_index.namespace.clone())
                .or_insert_with(Vec::new)
                .push(key_index.row);
        }

        Ok(())
    }

    /// Query index for a namespace.
    ///
    /// ## Input
    /// - `namespace` - Namespace to query (e.g., "page", "api")
    ///
    /// ## Output
    /// - `Some(&[row_numbers])` if namespace exists
    /// - `None` if namespace not found
    ///
    /// ## Performance
    /// - O(1) HashMap lookup
    /// - < 1μs typical
    pub fn query(&self, namespace: &str) -> Option<&[usize]> {
        self.map.get(namespace).map(|v| v.as_slice())
    }

    /// Insert a new key into the index.
    ///
    /// ## Input
    /// - `key_index` - Parsed key index to insert
    ///
    /// ## Performance
    /// - O(1) HashMap insert
    pub fn insert(&mut self, key_index: &KeyIndex) -> ReedResult<()> {
        self.map
            .entry(key_index.namespace.clone())
            .or_insert_with(Vec::new)
            .push(key_index.row);

        Ok(())
    }

    /// Remove a key at specific row from the index.
    ///
    /// ## Input
    /// - `row` - Row number to remove
    ///
    /// ## Performance
    /// - O(n) where n = keys in namespace (requires scan to find row)
    pub fn remove(&mut self, row: usize) -> ReedResult<()> {
        for rows in self.map.values_mut() {
            rows.retain(|&r| r != row);
        }

        Ok(())
    }

    /// Get number of unique namespaces.
    pub fn namespace_count(&self) -> usize {
        self.map.len()
    }

    /// Get total number of indexed keys.
    pub fn key_count(&self) -> usize {
        self.map.values().map(|v| v.len()).sum()
    }

    /// Calculate approximate memory usage in bytes.
    ///
    /// ## Returns
    /// - Estimated memory usage (keys + values + overhead)
    pub fn memory_usage(&self) -> usize {
        self.map
            .iter()
            .map(|(key, values)| {
                key.len() // Key string
                    + 24 // String overhead
                    + (values.len() * 8) // usize values
                    + 24 // Vec overhead
            })
            .sum()
    }

    /// Clear the index.
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl Default for NamespaceIndex {
    fn default() -> Self {
        Self::new()
    }
}
