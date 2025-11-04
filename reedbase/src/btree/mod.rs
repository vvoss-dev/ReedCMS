// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! B+-Tree on-disk index engine.
//!
//! Generic persistent index implementation using B+-Trees with mmap-based
//! file access for production use.
//!
//! ## Features
//!
//! - **On-Disk Persistence**: mmap-based file I/O (FreeBSD-compatible)
//! - **Crash Safety**: Write-Ahead-Log (WAL) for recovery
//! - **Range Queries**: Efficient range scans via linked leaf pages
//! - **Memory Efficient**: ~50MB for 10M keys (vs 1.5GB HashMap)
//! - **Fast Cold Start**: <100ms to load (vs 10s HashMap rebuild)
//!
//! ## Performance
//!
//! - Point lookup: < 1ms (O(log n))
//! - Range scan: < 5ms per 100 keys (O(log n + k))
//! - Insert: < 2ms (including splits)
//! - Delete: < 2ms (including merges)
//! - Cold start (10M keys): < 100ms
//! - Memory usage (10M keys): < 50MB
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::btree::{BPlusTree, Order};
//!
//! // Create or open B+-Tree
//! let order = Order::new(512)?; // 512 keys per node
//! let mut tree = BPlusTree::open("namespace.btree", order)?;
//!
//! // Insert key-value pair
//! tree.insert("page".to_string(), vec![1, 2, 3])?;
//!
//! // Point lookup
//! let value = tree.get(&"page".to_string())?;
//! assert_eq!(value, Some(vec![1, 2, 3]));
//!
//! // Range scan
//! let results = tree.range(&"page.a".to_string(), &"page.z".to_string())?;
//!
//! # Ok::<(), reedbase::ReedError>(())
//! ```
//!
//! ## File Format
//!
//! - **Main file**: `.btree` (4KB pages with mmap)
//! - **WAL file**: `.wal` (append-only log)
//!
//! Pages use CRC32 checksums for integrity validation.

mod iter;
mod node;
mod page;
mod tree;
mod types;
mod wal;

#[cfg(test)]
mod btree_test;

// Re-export public API
pub use iter::RangeScanIterator;
pub use tree::BPlusTree;
pub use types::{Order, PageId, BTREE_MAGIC};

// Re-export Index trait from indices module (canonical definition).
pub use crate::indices::Index;
