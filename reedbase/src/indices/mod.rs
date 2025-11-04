// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Smart Indices for 100-1000x faster queries.
//!
//! Provides O(1) lookups for filtered queries using specialized indices:
//!
//! ## Index Types
//!
//! - **NamespaceIndex**: O(1) prefix lookups (e.g., `page.*`)
//! - **ModifierIndex**: O(1) modifier lookups (language, environment, season, variant)
//! - **HierarchyTrie**: O(d) hierarchical wildcard queries (e.g., `page.header.*`)
//! - **IndexManager**: Coordinates all indices with set intersection for combined queries
//!
//! ## Performance
//!
//! - **Single index lookup**: < 1μs (O(1) HashMap)
//! - **Hierarchy query**: < 10μs (O(d) trie walk, d typically 2-4)
//! - **Combined query (3 filters)**: < 50μs (3x O(1) + set intersection)
//! - **Index build**: < 50ms for 10,000 keys
//! - **Memory**: ~150 bytes/key (~1.5MB for 10k keys)
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::indices::{IndexManager, QueryFilter};
//! use std::path::Path;
//!
//! // Build indices
//! let mut manager = IndexManager::new();
//! manager.build(Path::new(".reed"), "text")?;
//!
//! // Single filter query
//! let filter = QueryFilter::new()
//!     .with_language("de");
//! let rows = manager.query(&filter)?;
//!
//! // Combined filter query
//! let filter = QueryFilter::new()
//!     .with_namespace("page")
//!     .with_language("de")
//!     .with_environment("prod");
//! let rows = manager.query(&filter)?; // Intersection of all 3 filters
//!
//! // Hierarchical wildcard query
//! let filter = QueryFilter::new()
//!     .with_hierarchy(vec!["page".into(), "header".into(), "*".into()]);
//! let rows = manager.query(&filter)?; // All descendants of page.header
//!
//! # Ok::<(), reedbase::ReedError>(())
//! ```

pub mod btree_index;
pub mod builder;
pub mod hashmap_index;
pub mod hierarchy;
pub mod index_trait;
pub mod manager;
pub mod modifier;
pub mod namespace;
pub mod types;

#[cfg(test)]
mod builder_tests;
#[cfg(test)]
mod indices_test;

// Re-export public API
pub use btree_index::BTreeIndex;
pub use builder::{IndexBackend, IndexBuilder, IndexConfig};
pub use hashmap_index::HashMapIndex;
pub use hierarchy::HierarchyTrie;
pub use index_trait::Index;
pub use manager::{IndexManager, IndexStats};
pub use modifier::ModifierIndex;
pub use namespace::NamespaceIndex;
pub use types::{KeyIndex, Modifiers, QueryFilter};
