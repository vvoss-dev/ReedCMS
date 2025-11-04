// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Index builder factory for creating indices based on configuration.
//!
//! Provides a factory pattern for constructing index backends (HashMap vs B+-Tree)
//! based on TOML configuration or programmatic settings.
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::indices::builder::{IndexBuilder, IndexConfig, IndexBackend};
//! use std::path::Path;
//!
//! // Create from configuration
//! let config = IndexConfig {
//!     backend: IndexBackend::BTree,
//!     btree_order: Some(100),
//!     persist_path: Some("/tmp/reedbase/indices".to_string()),
//! };
//!
//! let builder = IndexBuilder::new(config);
//! let namespace_index = builder.build_namespace_index()?;
//!
//! # Ok::<(), reedbase::ReedError>(())
//! ```

use crate::btree::Order;
use crate::error::{ReedError, ReedResult};
use crate::indices::btree_index::BTreeIndex;
use crate::indices::hashmap_index::HashMapIndex;
use crate::indices::Index;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Index backend type.
///
/// ## Variants
/// - `HashMap`: In-memory O(1) lookups, no persistence
/// - `BTree`: On-disk B+-Tree, persistent, O(log n) lookups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndexBackend {
    /// In-memory HashMap backend.
    ///
    /// **Performance**: O(1) lookups, no persistence
    /// **Memory**: ~32 bytes per entry
    /// **Disk**: 0 bytes
    HashMap,

    /// On-disk B+-Tree backend.
    ///
    /// **Performance**: O(log n) lookups, persistent
    /// **Memory**: ~50MB for 10M keys (demand-paged)
    /// **Disk**: ~200MB for 10M keys
    BTree,
}

impl Default for IndexBackend {
    fn default() -> Self {
        Self::HashMap
    }
}

/// Index configuration.
///
/// Defines backend type and parameters for index creation.
///
/// ## Example
/// ```rust
/// use reedbase::indices::builder::{IndexConfig, IndexBackend};
///
/// let config = IndexConfig {
///     backend: IndexBackend::BTree,
///     btree_order: Some(100),
///     persist_path: Some("/tmp/reedbase/indices".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Backend type (hashmap or btree).
    #[serde(default)]
    pub backend: IndexBackend,

    /// B+-Tree order (keys per node).
    ///
    /// Only used when `backend = BTree`.
    /// Typical values: 50-200 for String keys.
    #[serde(default)]
    pub btree_order: Option<u16>,

    /// Path for persistent indices.
    ///
    /// Only used when `backend = BTree`.
    /// Creates files like `{persist_path}/namespace.btree`.
    #[serde(default)]
    pub persist_path: Option<String>,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            backend: IndexBackend::HashMap,
            btree_order: None,
            persist_path: None,
        }
    }
}

/// Index builder factory.
///
/// Constructs index instances based on configuration.
///
/// ## Type Parameters
/// - `K`: Key type (must be Clone + Ord + Eq + Hash + Serialize + Deserialize)
/// - `V`: Value type (must be Clone + Serialize + Deserialize)
pub struct IndexBuilder {
    config: IndexConfig,
}

impl IndexBuilder {
    /// Create new index builder with configuration.
    ///
    /// ## Input
    /// - `config`: Index configuration specifying backend and parameters
    ///
    /// ## Output
    /// - IndexBuilder instance
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::builder::{IndexBuilder, IndexConfig};
    ///
    /// let config = IndexConfig::default();
    /// let builder = IndexBuilder::new(config);
    /// ```
    pub fn new(config: IndexConfig) -> Self {
        Self { config }
    }

    /// Create index builder from TOML string.
    ///
    /// ## Input
    /// - `toml_str`: TOML configuration string
    ///
    /// ## Output
    /// - `Ok(IndexBuilder)`: Successfully parsed configuration
    /// - `Err(ReedError::ParseError)`: Invalid TOML syntax
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::builder::IndexBuilder;
    ///
    /// let toml = r#"
    /// backend = "btree"
    /// btree_order = 100
    /// persist_path = "/tmp/reedbase/indices"
    /// "#;
    ///
    /// let builder = IndexBuilder::from_toml(toml)?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn from_toml(toml_str: &str) -> ReedResult<Self> {
        let config: IndexConfig = toml::from_str(toml_str).map_err(|e| ReedError::ParseError {
            reason: format!("Failed to parse index config TOML: {}", e),
        })?;

        Ok(Self { config })
    }

    /// Build namespace index (String → Vec<usize>).
    ///
    /// ## Output
    /// - `Ok(Box<dyn Index>)`: Index instance (HashMap or B+-Tree)
    /// - `Err(ReedError)`: Configuration error or I/O error
    ///
    /// ## Error Conditions
    /// - B+-Tree backend: Invalid order, missing persist_path, I/O error
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::builder::{IndexBuilder, IndexConfig};
    ///
    /// let config = IndexConfig::default();
    /// let builder = IndexBuilder::new(config);
    /// let index = builder.build_namespace_index()?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn build_namespace_index(&self) -> ReedResult<Box<dyn Index<String, Vec<usize>>>> {
        match self.config.backend {
            IndexBackend::HashMap => {
                let index = HashMapIndex::<String, Vec<usize>>::new();
                Ok(Box::new(index))
            }
            IndexBackend::BTree => {
                let order = self.get_btree_order()?;
                let path = self.get_index_path("namespace.btree")?;
                let index = BTreeIndex::<String, Vec<usize>>::open(path, order)?;
                Ok(Box::new(index))
            }
        }
    }

    /// Build language index (String → Vec<usize>).
    ///
    /// ## Output
    /// - `Ok(Box<dyn Index>)`: Index instance (HashMap or B+-Tree)
    /// - `Err(ReedError)`: Configuration error or I/O error
    ///
    /// ## Example
    /// ```rust
    /// use reedbase::indices::builder::{IndexBuilder, IndexConfig};
    ///
    /// let config = IndexConfig::default();
    /// let builder = IndexBuilder::new(config);
    /// let index = builder.build_language_index()?;
    /// # Ok::<(), reedbase::ReedError>(())
    /// ```
    pub fn build_language_index(&self) -> ReedResult<Box<dyn Index<String, Vec<usize>>>> {
        match self.config.backend {
            IndexBackend::HashMap => {
                let index = HashMapIndex::<String, Vec<usize>>::new();
                Ok(Box::new(index))
            }
            IndexBackend::BTree => {
                let order = self.get_btree_order()?;
                let path = self.get_index_path("language.btree")?;
                let index = BTreeIndex::<String, Vec<usize>>::open(path, order)?;
                Ok(Box::new(index))
            }
        }
    }

    /// Build environment index (String → Vec<usize>).
    ///
    /// ## Output
    /// - `Ok(Box<dyn Index>)`: Index instance (HashMap or B+-Tree)
    /// - `Err(ReedError)`: Configuration error or I/O error
    pub fn build_environment_index(&self) -> ReedResult<Box<dyn Index<String, Vec<usize>>>> {
        match self.config.backend {
            IndexBackend::HashMap => {
                let index = HashMapIndex::<String, Vec<usize>>::new();
                Ok(Box::new(index))
            }
            IndexBackend::BTree => {
                let order = self.get_btree_order()?;
                let path = self.get_index_path("environment.btree")?;
                let index = BTreeIndex::<String, Vec<usize>>::open(path, order)?;
                Ok(Box::new(index))
            }
        }
    }

    /// Build season index (String → Vec<usize>).
    ///
    /// ## Output
    /// - `Ok(Box<dyn Index>)`: Index instance (HashMap or B+-Tree)
    /// - `Err(ReedError)`: Configuration error or I/O error
    pub fn build_season_index(&self) -> ReedResult<Box<dyn Index<String, Vec<usize>>>> {
        match self.config.backend {
            IndexBackend::HashMap => {
                let index = HashMapIndex::<String, Vec<usize>>::new();
                Ok(Box::new(index))
            }
            IndexBackend::BTree => {
                let order = self.get_btree_order()?;
                let path = self.get_index_path("season.btree")?;
                let index = BTreeIndex::<String, Vec<usize>>::open(path, order)?;
                Ok(Box::new(index))
            }
        }
    }

    /// Build variant index (String → Vec<usize>).
    ///
    /// ## Output
    /// - `Ok(Box<dyn Index>)`: Index instance (HashMap or B+-Tree)
    /// - `Err(ReedError)`: Configuration error or I/O error
    pub fn build_variant_index(&self) -> ReedResult<Box<dyn Index<String, Vec<usize>>>> {
        match self.config.backend {
            IndexBackend::HashMap => {
                let index = HashMapIndex::<String, Vec<usize>>::new();
                Ok(Box::new(index))
            }
            IndexBackend::BTree => {
                let order = self.get_btree_order()?;
                let path = self.get_index_path("variant.btree")?;
                let index = BTreeIndex::<String, Vec<usize>>::open(path, order)?;
                Ok(Box::new(index))
            }
        }
    }

    /// Build hierarchy index (Vec<String> → Vec<usize>).
    ///
    /// ## Output
    /// - `Ok(Box<dyn Index>)`: Index instance (HashMap or B+-Tree)
    /// - `Err(ReedError)`: Configuration error or I/O error
    ///
    /// ## Note
    /// Hierarchy index stores Vec<String> keys for trie-like hierarchical lookups.
    pub fn build_hierarchy_index(&self) -> ReedResult<Box<dyn Index<Vec<String>, Vec<usize>>>> {
        match self.config.backend {
            IndexBackend::HashMap => {
                let index = HashMapIndex::<Vec<String>, Vec<usize>>::new();
                Ok(Box::new(index))
            }
            IndexBackend::BTree => {
                let order = self.get_btree_order()?;
                let path = self.get_index_path("hierarchy.btree")?;
                let index = BTreeIndex::<Vec<String>, Vec<usize>>::open(path, order)?;
                Ok(Box::new(index))
            }
        }
    }

    /// Get B+-Tree order from configuration.
    ///
    /// ## Output
    /// - `Ok(Order)`: Valid B+-Tree order
    /// - `Err(ReedError)`: Missing or invalid order
    ///
    /// ## Default
    /// - If not specified: Order 100 (typical for String keys)
    fn get_btree_order(&self) -> ReedResult<Order> {
        let order_value = self.config.btree_order.unwrap_or(100);
        Order::new(order_value)
    }

    /// Get index file path from configuration.
    ///
    /// ## Input
    /// - `filename`: Index filename (e.g., "namespace.btree")
    ///
    /// ## Output
    /// - `Ok(PathBuf)`: Full path to index file
    /// - `Err(ReedError)`: Missing persist_path in configuration
    fn get_index_path(&self, filename: &str) -> ReedResult<PathBuf> {
        let persist_path =
            self.config
                .persist_path
                .as_ref()
                .ok_or_else(|| ReedError::ParseError {
                    reason: "B+-Tree backend requires persist_path in configuration".to_string(),
                })?;

        Ok(Path::new(persist_path).join(filename))
    }

    /// Get current configuration.
    ///
    /// ## Output
    /// - Reference to IndexConfig
    pub fn config(&self) -> &IndexConfig {
        &self.config
    }
}

impl Default for IndexBuilder {
    fn default() -> Self {
        Self {
            config: IndexConfig::default(),
        }
    }
}
