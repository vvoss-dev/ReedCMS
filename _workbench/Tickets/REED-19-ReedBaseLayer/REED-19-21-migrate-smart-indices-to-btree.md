# REED-19-21: Migrate Smart Indices to B+-Tree

**Layer**: REED-19 (ReedBase Layer)  
**Phase**: 5 (Distributed + P2P)  
**Dependencies**: REED-19-11 (Smart Indices), REED-19-20 (B+-Tree Index Engine)  
**Estimated Effort**: 2-3 days  
**Priority**: High  
**Status**: Planned

---

## Executive Summary

Replace in-memory HashMap-based indices from REED-19-11 with persistent B+-Tree indices from REED-19-20 to achieve:
- **100x faster cold starts** (100ms vs 10s for 10M keys)
- **30x lower memory usage** (50MB vs 1.5GB for 10M keys)
- **Persistent across restarts** (no rebuild needed)
- **Backward compatible** (configuration flag to select backend)
- **Gradual migration** (per-index choice: namespace=BTree, language=HashMap, etc.)

This ticket bridges REED-19-11 (in-memory fast lookups) with REED-19-20 (on-disk persistence) to create production-ready indices that survive server restarts and scale to 100M+ keys.

---

## Problem Statement

### Current State (REED-19-11)

Smart Indices use in-memory `HashMap<String, Vec<RowId>>` with O(1) lookups:

```rust
// reedbase/src/indices/smart.rs (existing)
pub struct IndexEngine {
    namespace: HashMap<String, Vec<RowId>>,  // "page" → [101, 102, 103]
    language: HashMap<String, Vec<RowId>>,   // "de" → [201, 202]
    hierarchy: Trie<Vec<RowId>>,             // "page.header" → [301]
}
```

**Limitations:**
1. **Cold Start Penalty**: 10s to rebuild 10M keys from CSV on every server restart
2. **Memory Consumption**: 1.5GB RAM for 10M keys (unacceptable for production)
3. **No Persistence**: Indices lost on crash/restart (rebuild required)
4. **Scalability Limit**: Cannot handle 100M+ keys (RAM exhaustion)

### Desired State (REED-19-21)

Hybrid approach with pluggable backends:

```rust
// reedbase/src/indices/hybrid.rs (new)
pub struct IndexEngine {
    namespace: Box<dyn Index<String, Vec<RowId>>>,  // BTreeIndex by default
    language: Box<dyn Index<String, Vec<RowId>>>,   // BTreeIndex by default
    hierarchy: Box<dyn Index<String, Vec<RowId>>>,  // TrieIndex (custom B+-Tree variant)
}
```

**Benefits:**
- **Instant Cold Start**: 100ms to mmap existing B+-Tree (no rebuild)
- **Low Memory**: 50MB for 10M keys (page cache managed by OS)
- **Persistence**: Indices survive crashes (WAL ensures consistency)
- **Scalability**: Tested to 100M keys (5GB on disk, 200MB RAM)

---

## Architecture

### Backend Selection Strategy

```toml
# .reed/config.toml (new configuration)
[indices]
default_backend = "btree"  # "btree" | "hashmap"

[indices.namespace]
backend = "btree"          # Per-index override
path = ".reed/indices/namespace.btree"

[indices.language]
backend = "btree"
path = ".reed/indices/language.btree"

[indices.hierarchy]
backend = "trie_btree"     # Custom B+-Tree variant for trie structure
path = ".reed/indices/hierarchy.btree"
```

**Decision Matrix:**

| Index Type | Production | Development | Reasoning |
|-----------|-----------|-------------|-----------|
| **namespace** | B+-Tree | B+-Tree | Large cardinality (100k+ namespaces), range queries needed |
| **language** | HashMap | HashMap | Small cardinality (50-200 languages), O(1) sufficient |
| **hierarchy** | TrieBTree | TrieBTree | Prefix queries essential, persistence valuable |

### Trait-Based Abstraction

```rust
// reedbase/src/indices/trait.rs (new)
pub trait Index<K, V>: Send + Sync {
    fn get(&self, key: &K) -> ReedResult<Option<V>>;
    fn range(&self, start: &K, end: &K) -> ReedResult<Vec<(K, V)>>;
    fn insert(&mut self, key: K, value: V) -> ReedResult<()>;
    fn delete(&mut self, key: &K) -> ReedResult<()>;
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)>>;
    
    // Metadata
    fn backend_type(&self) -> &'static str;  // "btree" | "hashmap" | "trie_btree"
    fn memory_usage(&self) -> usize;         // Bytes
    fn disk_usage(&self) -> usize;           // Bytes
}
```

### Implementation Wrappers

```rust
// reedbase/src/indices/hashmap_index.rs (new)
pub struct HashMapIndex<K, V> {
    map: HashMap<K, V>,
}

impl<K, V> Index<K, V> for HashMapIndex<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> ReedResult<Option<V>> {
        Ok(self.map.get(key).cloned())
    }
    
    fn insert(&mut self, key: K, value: V) -> ReedResult<()> {
        self.map.insert(key, value);
        Ok(())
    }
    
    fn backend_type(&self) -> &'static str { "hashmap" }
    fn memory_usage(&self) -> usize { 
        std::mem::size_of_val(&self.map) + 
        self.map.capacity() * (size_of::<K>() + size_of::<V>())
    }
    fn disk_usage(&self) -> usize { 0 }  // In-memory only
}
```

```rust
// reedbase/src/indices/btree_index.rs (new)
pub struct BTreeIndex<K, V> {
    tree: BPlusTree<K, V>,  // From REED-19-20
    path: PathBuf,
}

impl<K, V> Index<K, V> for BTreeIndex<K, V>
where
    K: Ord + Clone + Serialize + DeserializeOwned,
    V: Clone + Serialize + DeserializeOwned,
{
    fn get(&self, key: &K) -> ReedResult<Option<V>> {
        self.tree.get(key)
    }
    
    fn range(&self, start: &K, end: &K) -> ReedResult<Vec<(K, V)>> {
        self.tree.range(start..=end)
    }
    
    fn insert(&mut self, key: K, value: V) -> ReedResult<()> {
        self.tree.insert(key, value)?;
        self.tree.flush()  // WAL persistence
    }
    
    fn backend_type(&self) -> &'static str { "btree" }
    fn memory_usage(&self) -> usize { self.tree.memory_usage() }
    fn disk_usage(&self) -> usize { 
        fs::metadata(&self.path).map(|m| m.len() as usize).unwrap_or(0)
    }
}
```

---

## Migration Strategy

### Phase 1: Add Trait Abstraction (No Breaking Changes)

1. **Extract trait interface** from existing `smart.rs`
2. **Create `HashMapIndex` wrapper** around existing HashMap code
3. **Update `IndexEngine`** to use trait objects
4. **Verify 100% backward compatibility** (all 17 tests pass unchanged)

**File Changes:**
```
reedbase/src/indices/
├── mod.rs              # Export trait + implementations
├── trait.rs            # Index<K, V> trait definition (NEW)
├── hashmap_index.rs    # HashMapIndex wrapper (NEW)
└── smart.rs            # IndexEngine using Box<dyn Index> (MODIFIED)
```

**Example Migration:**

```rust
// Before (REED-19-11)
pub struct IndexEngine {
    namespace: HashMap<String, Vec<RowId>>,
}

impl IndexEngine {
    pub fn query_namespace(&self, ns: &str) -> Vec<RowId> {
        self.namespace.get(ns).cloned().unwrap_or_default()
    }
}

// After (REED-19-21 Phase 1)
pub struct IndexEngine {
    namespace: Box<dyn Index<String, Vec<RowId>>>,
}

impl IndexEngine {
    pub fn query_namespace(&self, ns: &str) -> ReedResult<Vec<RowId>> {
        self.namespace.get(ns).map(|opt| opt.unwrap_or_default())
    }
}
```

**Success Criteria:**
- [ ] All 17 smart_test.rs tests pass without modification
- [ ] Performance within 5% of baseline (trait overhead negligible)
- [ ] Memory usage unchanged

---

### Phase 2: Add B+-Tree Backend (Opt-In)

1. **Create `BTreeIndex` wrapper** around REED-19-20 B+-Tree
2. **Add configuration support** (.reed/config.toml indices section)
3. **Implement index builder factory** (selects backend based on config)
4. **Add migration CLI command** (`reed index:migrate --backend btree`)

**File Changes:**
```
reedbase/src/indices/
├── btree_index.rs      # BTreeIndex wrapper (NEW)
├── builder.rs          # Backend selection logic (NEW)
└── migrate.rs          # Migration tool (NEW)

.reed/
├── config.toml         # Index backend configuration (NEW)
└── indices/            # B+-Tree storage directory (NEW)
    ├── namespace.btree
    ├── namespace.wal
    ├── language.btree
    └── hierarchy.btree
```

**Builder Factory:**

```rust
// reedbase/src/indices/builder.rs (new)
pub struct IndexBuilder {
    config: IndexConfig,
}

impl IndexBuilder {
    pub fn build_namespace_index(&self) -> ReedResult<Box<dyn Index<String, Vec<RowId>>>> {
        match self.config.namespace.backend.as_str() {
            "btree" => {
                let path = PathBuf::from(&self.config.namespace.path);
                let tree = BPlusTree::open_or_create(path, 512)?;
                Ok(Box::new(BTreeIndex::new(tree, path)))
            }
            "hashmap" => {
                Ok(Box::new(HashMapIndex::new()))
            }
            unknown => Err(ReedError::IndexBackendUnknown {
                backend: unknown.to_string(),
            }),
        }
    }
}
```

**Migration Tool:**

```bash
# Rebuild indices from CSV using new backend
reed index:migrate --backend btree

# Output:
# [1/3] Migrating namespace index... 100% (1.2M keys, 2.3s)
# [2/3] Migrating language index... 100% (87 keys, 0.1s)
# [3/3] Migrating hierarchy index... 100% (450k keys, 1.8s)
# Total: 1.65M keys migrated in 4.2s
# Memory: 1.5GB → 50MB (30x reduction)
# Disk: 0 → 320MB
```

**Success Criteria:**
- [ ] B+-Tree backend produces identical query results as HashMap
- [ ] Migration tool completes for 10M keys in <30s
- [ ] Cold start time <200ms (vs 10s for HashMap rebuild)
- [ ] Memory usage <100MB for 10M keys

---

### Phase 3: Make B+-Tree Default (Production)

1. **Change default_backend = "btree"** in config template
2. **Update documentation** to recommend B+-Tree for production
3. **Add performance benchmarks** (cold start, memory, query latency)
4. **Deprecate HashMap backend** (mark for removal in REED-20)

**Configuration Template:**

```toml
# .reed/config.toml.example
[indices]
# Backend: "btree" (persistent, low memory) or "hashmap" (fast, high memory)
default_backend = "btree"  # Recommended for production

# Per-index overrides
[indices.namespace]
backend = "btree"          # 100k+ keys → use B+-Tree
path = ".reed/indices/namespace.btree"

[indices.language]
backend = "hashmap"        # <200 keys → HashMap is faster
# No path needed (in-memory)

[indices.hierarchy]
backend = "trie_btree"     # Custom implementation for prefix queries
path = ".reed/indices/hierarchy.btree"
```

**Success Criteria:**
- [ ] Default installation uses B+-Tree
- [ ] Documentation updated (README, tickets, man pages)
- [ ] Benchmark suite added (see Performance Tests below)

---

## Implementation Files

### 1. `reedbase/src/indices/trait.rs` (NEW)

**Purpose:** Define common interface for all index backends

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Index trait abstraction for pluggable backends.
//!
//! Allows ReedBase to switch between HashMap, B+-Tree, or custom implementations
//! without changing query logic.

use crate::error::{ReedError, ReedResult};
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
/// - `TrieBTreeIndex`: Custom implementation for prefix queries
///
/// ## Thread Safety
/// - Implementations must be `Send + Sync` for concurrent access
/// - Write operations require `&mut self` (exclusive access)
pub trait Index<K, V>: Send + Sync + Debug {
    /// Get value for exact key match.
    ///
    /// ## Performance
    /// - HashMap: O(1) average, worst O(n) on hash collision
    /// - B+-Tree: O(log n), <1ms for 10M keys
    fn get(&self, key: &K) -> ReedResult<Option<V>>;
    
    /// Get all key-value pairs in range [start, end].
    ///
    /// ## Performance
    /// - HashMap: Not supported (returns error)
    /// - B+-Tree: O(log n + k) where k = result size, <5ms for 100 keys
    fn range(&self, start: &K, end: &K) -> ReedResult<Vec<(K, V)>>;
    
    /// Insert or update key-value pair.
    ///
    /// ## Performance
    /// - HashMap: O(1) average
    /// - B+-Tree: O(log n) + WAL write, <2ms for 10M keys
    fn insert(&mut self, key: K, value: V) -> ReedResult<()>;
    
    /// Delete key-value pair.
    ///
    /// ## Performance
    /// - HashMap: O(1) average
    /// - B+-Tree: O(log n) + WAL write, <2ms for 10M keys
    fn delete(&mut self, key: &K) -> ReedResult<()>;
    
    /// Iterate all key-value pairs (unordered for HashMap, sorted for B+-Tree).
    ///
    /// ## Performance
    /// - HashMap: O(n), random order
    /// - B+-Tree: O(n), sorted order
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)> + '_>;
    
    // Metadata
    
    /// Backend type identifier.
    ///
    /// ## Returns
    /// - "hashmap" | "btree" | "trie_btree"
    fn backend_type(&self) -> &'static str;
    
    /// Estimated memory usage in bytes.
    ///
    /// ## Calculation
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
```

**Tests:** `reedbase/src/indices/trait_test.rs` (trait contract tests using mock implementation)

---

### 2. `reedbase/src/indices/hashmap_index.rs` (NEW)

**Purpose:** Wrap existing HashMap code in `Index` trait

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! In-memory HashMap-based index implementation.
//!
//! ## Use Cases
//! - Small datasets (<100k keys)
//! - Development/testing (no persistence needed)
//! - Low-cardinality indices (language codes, environment names)
//!
//! ## Performance
//! - Point lookup: O(1) average, <100ns
//! - Range scan: Not supported (returns error)
//! - Memory: 16 bytes/key + key/value size
//! - Cold start: 0ms (empty), 5-10s (rebuild from CSV)

use crate::error::{ReedError, ReedResult};
use crate::indices::Index;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct HashMapIndex<K, V> {
    map: HashMap<K, V>,
}

impl<K, V> HashMapIndex<K, V> {
    /// Create empty index.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    
    /// Create index with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }
    
    /// Number of key-value pairs.
    pub fn len(&self) -> usize {
        self.map.len()
    }
    
    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl<K, V> Index<K, V> for HashMapIndex<K, V>
where
    K: Eq + Hash + Clone + Debug,
    V: Clone + Debug,
{
    fn get(&self, key: &K) -> ReedResult<Option<V>> {
        Ok(self.map.get(key).cloned())
    }
    
    fn range(&self, _start: &K, _end: &K) -> ReedResult<Vec<(K, V)>> {
        Err(ReedError::IndexOperationUnsupported {
            backend: "hashmap",
            operation: "range",
            reason: "HashMap does not support range queries, use B+-Tree backend",
        })
    }
    
    fn insert(&mut self, key: K, value: V) -> ReedResult<()> {
        self.map.insert(key, value);
        Ok(())
    }
    
    fn delete(&mut self, key: &K) -> ReedResult<()> {
        self.map.remove(key);
        Ok(())
    }
    
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)> + '_> {
        Box::new(self.map.iter().map(|(k, v)| (k.clone(), v.clone())))
    }
    
    fn backend_type(&self) -> &'static str {
        "hashmap"
    }
    
    fn memory_usage(&self) -> usize {
        use std::mem::size_of;
        
        let map_overhead = size_of::<HashMap<K, V>>();
        let capacity_bytes = self.map.capacity() * (size_of::<K>() + size_of::<V>() + 16);
        
        map_overhead + capacity_bytes
    }
    
    fn disk_usage(&self) -> usize {
        0  // In-memory only
    }
}

impl<K, V> Default for HashMapIndex<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
```

**Tests:** `reedbase/src/indices/hashmap_index_test.rs` (17 tests covering all trait methods)

---

### 3. `reedbase/src/indices/btree_index.rs` (NEW)

**Purpose:** Wrap REED-19-20 B+-Tree in `Index` trait

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! On-disk B+-Tree index implementation.
//!
//! ## Use Cases
//! - Large datasets (1M+ keys)
//! - Production deployments (persistence required)
//! - Range queries (hierarchy lookups, date ranges)
//!
//! ## Performance
//! - Point lookup: O(log n), <1ms for 10M keys
//! - Range scan: O(log n + k), <5ms for 100 keys
//! - Memory: <50MB for 10M keys (page cache)
//! - Cold start: <100ms (mmap existing file)
//! - Write: O(log n) + WAL fsync, <2ms

use crate::btree::BPlusTree;  // From REED-19-20
use crate::error::{ReedError, ReedResult};
use crate::indices::Index;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct BTreeIndex<K, V> {
    tree: BPlusTree<K, V>,
    path: PathBuf,
}

impl<K, V> BTreeIndex<K, V>
where
    K: Ord + Clone + Serialize + for<'de> Deserialize<'de> + Debug,
    V: Clone + Serialize + for<'de> Deserialize<'de> + Debug,
{
    /// Open existing B+-Tree or create new one.
    ///
    /// ## Arguments
    /// - `path`: File path (e.g., ".reed/indices/namespace.btree")
    /// - `order`: B+-Tree order (512 recommended for 4KB pages)
    ///
    /// ## Performance
    /// - Existing file: <100ms (mmap, no rebuild)
    /// - New file: <10ms (create header page)
    pub fn open_or_create<P: AsRef<Path>>(path: P, order: usize) -> ReedResult<Self> {
        let path_buf = path.as_ref().to_path_buf();
        
        // Ensure directory exists
        if let Some(parent) = path_buf.parent() {
            fs::create_dir_all(parent).map_err(|e| ReedError::IndexFileCreate {
                path: path_buf.clone(),
                source: e,
            })?;
        }
        
        let tree = BPlusTree::open_or_create(&path_buf, order)?;
        
        Ok(Self {
            tree,
            path: path_buf,
        })
    }
    
    /// Force flush WAL to disk.
    ///
    /// ## Performance
    /// - <5ms (fsync call)
    pub fn flush(&mut self) -> ReedResult<()> {
        self.tree.flush()
    }
}

impl<K, V> Index<K, V> for BTreeIndex<K, V>
where
    K: Ord + Clone + Serialize + for<'de> Deserialize<'de> + Debug,
    V: Clone + Serialize + for<'de> Deserialize<'de> + Debug,
{
    fn get(&self, key: &K) -> ReedResult<Option<V>> {
        self.tree.get(key)
    }
    
    fn range(&self, start: &K, end: &K) -> ReedResult<Vec<(K, V)>> {
        self.tree.range(start..=end)
    }
    
    fn insert(&mut self, key: K, value: V) -> ReedResult<()> {
        self.tree.insert(key, value)?;
        self.tree.flush()  // Persist to WAL immediately
    }
    
    fn delete(&mut self, key: &K) -> ReedResult<()> {
        self.tree.delete(key)?;
        self.tree.flush()
    }
    
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)> + '_> {
        Box::new(self.tree.iter())
    }
    
    fn backend_type(&self) -> &'static str {
        "btree"
    }
    
    fn memory_usage(&self) -> usize {
        self.tree.memory_usage()
    }
    
    fn disk_usage(&self) -> usize {
        let tree_size = fs::metadata(&self.path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);
        
        // Add WAL size
        let mut wal_path = self.path.clone();
        wal_path.set_extension("wal");
        let wal_size = fs::metadata(&wal_path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);
        
        tree_size + wal_size
    }
}
```

**Tests:** `reedbase/src/indices/btree_index_test.rs` (25 tests including persistence, crash recovery)

---

### 4. `reedbase/src/indices/builder.rs` (NEW)

**Purpose:** Factory for creating indices based on configuration

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Index builder factory for backend selection.
//!
//! Reads configuration from `.reed/config.toml` and creates appropriate
//! index implementation (HashMap vs B+-Tree).

use crate::error::{ReedError, ReedResult};
use crate::indices::{BTreeIndex, HashMapIndex, Index};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Index backend configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct IndexConfig {
    #[serde(default = "default_backend")]
    pub default_backend: String,
    
    #[serde(default)]
    pub namespace: BackendConfig,
    
    #[serde(default)]
    pub language: BackendConfig,
    
    #[serde(default)]
    pub hierarchy: BackendConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfig {
    #[serde(default = "default_backend")]
    pub backend: String,
    
    #[serde(default)]
    pub path: String,
}

fn default_backend() -> String {
    "btree".to_string()
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            path: String::new(),
        }
    }
}

/// Index builder factory.
pub struct IndexBuilder {
    config: IndexConfig,
}

impl IndexBuilder {
    /// Load configuration from file.
    pub fn from_config_file<P: AsRef<std::path::Path>>(path: P) -> ReedResult<Self> {
        let content = fs::read_to_string(path).map_err(|e| ReedError::ConfigFileRead {
            path: PathBuf::from(path.as_ref()),
            source: e,
        })?;
        
        let config: IndexConfig = toml::from_str(&content).map_err(|e| ReedError::ConfigParse {
            source: e.to_string(),
        })?;
        
        Ok(Self { config })
    }
    
    /// Use default configuration (B+-Tree for all).
    pub fn default() -> Self {
        Self {
            config: IndexConfig {
                default_backend: "btree".to_string(),
                namespace: BackendConfig {
                    backend: "btree".to_string(),
                    path: ".reed/indices/namespace.btree".to_string(),
                },
                language: BackendConfig {
                    backend: "hashmap".to_string(),  // Small cardinality
                    path: String::new(),
                },
                hierarchy: BackendConfig {
                    backend: "btree".to_string(),
                    path: ".reed/indices/hierarchy.btree".to_string(),
                },
            },
        }
    }
    
    /// Build namespace index.
    pub fn build_namespace_index(&self) -> ReedResult<Box<dyn Index<String, Vec<usize>>>> {
        self.build_index(&self.config.namespace, "namespace")
    }
    
    /// Build language index.
    pub fn build_language_index(&self) -> ReedResult<Box<dyn Index<String, Vec<usize>>>> {
        self.build_index(&self.config.language, "language")
    }
    
    /// Build hierarchy index.
    pub fn build_hierarchy_index(&self) -> ReedResult<Box<dyn Index<String, Vec<usize>>>> {
        self.build_index(&self.config.hierarchy, "hierarchy")
    }
    
    fn build_index(
        &self,
        backend_config: &BackendConfig,
        index_name: &str,
    ) -> ReedResult<Box<dyn Index<String, Vec<usize>>>> {
        match backend_config.backend.as_str() {
            "btree" => {
                if backend_config.path.is_empty() {
                    return Err(ReedError::IndexConfigInvalid {
                        index: index_name.to_string(),
                        reason: "B+-Tree backend requires 'path' configuration",
                    });
                }
                
                let path = PathBuf::from(&backend_config.path);
                let tree = BTreeIndex::open_or_create(path, 512)?;
                Ok(Box::new(tree))
            }
            "hashmap" => {
                Ok(Box::new(HashMapIndex::new()))
            }
            unknown => Err(ReedError::IndexBackendUnknown {
                backend: unknown.to_string(),
                available: vec!["btree".to_string(), "hashmap".to_string()],
            }),
        }
    }
}
```

**Tests:** `reedbase/src/indices/builder_test.rs` (12 tests covering config parsing, backend selection)

---

### 5. `reedbase/src/indices/migrate.rs` (NEW)

**Purpose:** CLI tool to rebuild indices with new backend

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Index migration tool for switching backends.
//!
//! ## Usage
//! ```bash
//! reed index:migrate --backend btree
//! reed index:migrate --backend hashmap --index namespace
//! ```

use crate::error::{ReedError, ReedResult};
use crate::indices::{Index, IndexBuilder};
use crate::parser::Parser;
use std::time::Instant;

/// Migration statistics.
#[derive(Debug)]
pub struct MigrationStats {
    pub index_name: String,
    pub keys_migrated: usize,
    pub duration_ms: u128,
    pub old_backend: String,
    pub new_backend: String,
    pub old_memory: usize,
    pub new_memory: usize,
    pub disk_usage: usize,
}

/// Index migrator.
pub struct IndexMigrator {
    builder: IndexBuilder,
}

impl IndexMigrator {
    pub fn new(builder: IndexBuilder) -> Self {
        Self { builder }
    }
    
    /// Migrate all indices to new backend.
    ///
    /// ## Algorithm
    /// 1. Read CSV data (`.reed/text.csv`)
    /// 2. Parse keys with RBKS v2
    /// 3. Create new indices with target backend
    /// 4. Populate indices from CSV
    /// 5. Replace old index files
    ///
    /// ## Performance
    /// - 10M keys: <30s total
    /// - Progress updates every 100k keys
    pub fn migrate_all(&self, csv_path: &str) -> ReedResult<Vec<MigrationStats>> {
        let mut stats = Vec::new();
        
        // Read CSV data
        let data = std::fs::read_to_string(csv_path).map_err(|e| ReedError::CsvFileRead {
            path: csv_path.to_string(),
            source: e,
        })?;
        
        let parser = Parser::new();
        let rows = parser.parse_csv(&data)?;
        
        println!("Migrating {} keys from CSV...\n", rows.len());
        
        // Migrate namespace index
        stats.push(self.migrate_namespace_index(&rows)?);
        
        // Migrate language index
        stats.push(self.migrate_language_index(&rows)?);
        
        // Migrate hierarchy index
        stats.push(self.migrate_hierarchy_index(&rows)?);
        
        Ok(stats)
    }
    
    fn migrate_namespace_index(
        &self,
        rows: &[(String, String)],
    ) -> ReedResult<MigrationStats> {
        let start = Instant::now();
        
        // Build new index
        let mut index = self.builder.build_namespace_index()?;
        let old_memory = index.memory_usage();
        let old_backend = index.backend_type().to_string();
        
        println!("[1/3] Migrating namespace index...");
        
        // Extract namespace from each key
        let mut count = 0;
        for (key, _) in rows {
            if let Some(namespace) = key.split('.').next() {
                let row_ids = vec![count];  // Simplified for migration
                index.insert(namespace.to_string(), row_ids)?;
                count += 1;
                
                if count % 100_000 == 0 {
                    println!("  Progress: {}/1.0M keys ({:.1}%)", count / 1000, count as f64 / 10_000.0);
                }
            }
        }
        
        let duration = start.elapsed();
        let new_memory = index.memory_usage();
        let disk_usage = index.disk_usage();
        let new_backend = index.backend_type().to_string();
        
        println!("  ✓ Migrated {} keys in {:.2}s", count, duration.as_secs_f64());
        println!("    Memory: {} → {} ({:.1}x)", 
            format_bytes(old_memory), 
            format_bytes(new_memory),
            old_memory as f64 / new_memory.max(1) as f64
        );
        println!("    Disk: {}\n", format_bytes(disk_usage));
        
        Ok(MigrationStats {
            index_name: "namespace".to_string(),
            keys_migrated: count,
            duration_ms: duration.as_millis(),
            old_backend,
            new_backend,
            old_memory,
            new_memory,
            disk_usage,
        })
    }
    
    fn migrate_language_index(
        &self,
        rows: &[(String, String)],
    ) -> ReedResult<MigrationStats> {
        // Similar to namespace, extract @lang suffix
        todo!("Implement language index migration")
    }
    
    fn migrate_hierarchy_index(
        &self,
        rows: &[(String, String)],
    ) -> ReedResult<MigrationStats> {
        // Similar to namespace, extract full hierarchy path
        todo!("Implement hierarchy index migration")
    }
}

fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
```

**CLI Integration:**

```rust
// src/reedcms/cli/index.rs (modified)
pub fn handle_index_command(args: &IndexArgs) -> ReedResult<()> {
    match &args.subcommand {
        IndexSubcommand::Migrate(migrate_args) => {
            let builder = if let Some(config_path) = &migrate_args.config {
                IndexBuilder::from_config_file(config_path)?
            } else {
                IndexBuilder::default()
            };
            
            let migrator = IndexMigrator::new(builder);
            let stats = migrator.migrate_all(&migrate_args.csv_path)?;
            
            // Print summary
            let total_keys: usize = stats.iter().map(|s| s.keys_migrated).sum();
            let total_time: u128 = stats.iter().map(|s| s.duration_ms).sum();
            
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Migration Summary:");
            println!("  Total keys: {}", total_keys);
            println!("  Total time: {:.2}s", total_time as f64 / 1000.0);
            println!("  Throughput: {:.0} keys/s", total_keys as f64 / (total_time as f64 / 1000.0));
            
            Ok(())
        }
        // ... other subcommands
    }
}
```

**Tests:** `reedbase/src/indices/migrate_test.rs` (8 tests covering migration scenarios)

---

### 6. Modify `reedbase/src/indices/smart.rs`

**Purpose:** Update `IndexEngine` to use trait objects

**Changes:**

```rust
// Before (REED-19-11)
pub struct IndexEngine {
    namespace: HashMap<String, Vec<RowId>>,
    language: HashMap<String, Vec<RowId>>,
    hierarchy: Trie<Vec<RowId>>,
}

// After (REED-19-21)
pub struct IndexEngine {
    namespace: Box<dyn Index<String, Vec<RowId>>>,
    language: Box<dyn Index<String, Vec<RowId>>>,
    hierarchy: Box<dyn Index<String, Vec<RowId>>>,
}

impl IndexEngine {
    /// Create engine with specified backends.
    pub fn new(builder: IndexBuilder) -> ReedResult<Self> {
        Ok(Self {
            namespace: builder.build_namespace_index()?,
            language: builder.build_language_index()?,
            hierarchy: builder.build_hierarchy_index()?,
        })
    }
    
    /// Query namespace index.
    pub fn query_namespace(&self, ns: &str) -> ReedResult<Vec<RowId>> {
        self.namespace.get(ns).map(|opt| opt.unwrap_or_default())
    }
    
    /// Query language index.
    pub fn query_language(&self, lang: &str) -> ReedResult<Vec<RowId>> {
        self.language.get(lang).map(|opt| opt.unwrap_or_default())
    }
    
    /// Query hierarchy index (prefix match).
    pub fn query_hierarchy(&self, prefix: &str) -> ReedResult<Vec<RowId>> {
        // Use range query for B+-Tree backend
        let end = format!("{}~", prefix);  // ASCII '~' > all alphanumeric
        let results = self.hierarchy.range(prefix, &end)?;
        
        Ok(results.into_iter()
            .flat_map(|(_, row_ids)| row_ids)
            .collect())
    }
}
```

**Backward Compatibility Test:**

```rust
// reedbase/src/indices/smart_test.rs (modified)
#[test]
fn test_backward_compatibility_with_hashmap() {
    // Use HashMapIndex backend (REED-19-11 behavior)
    let builder = IndexBuilder {
        config: IndexConfig {
            default_backend: "hashmap".to_string(),
            namespace: BackendConfig { backend: "hashmap".to_string(), path: String::new() },
            language: BackendConfig { backend: "hashmap".to_string(), path: String::new() },
            hierarchy: BackendConfig { backend: "hashmap".to_string(), path: String::new() },
        },
    };
    
    let mut engine = IndexEngine::new(builder).unwrap();
    
    // All 17 existing tests should pass unchanged
    // ... existing test logic
}

#[test]
fn test_btree_backend_migration() {
    // Use B+-Tree backend (REED-19-21 new behavior)
    let builder = IndexBuilder::default();  // Uses B+-Tree
    let mut engine = IndexEngine::new(builder).unwrap();
    
    // Same test logic, different backend
    // ... identical test logic to above
}
```

---

## Error Handling

### New Error Variants

```rust
// reedbase/src/error.rs (add these variants)
pub enum ReedError {
    // ... existing variants
    
    /// Index backend not recognized.
    IndexBackendUnknown {
        backend: String,
        available: Vec<String>,
    },
    
    /// Index operation not supported by backend.
    IndexOperationUnsupported {
        backend: &'static str,
        operation: &'static str,
        reason: &'static str,
    },
    
    /// Index configuration invalid.
    IndexConfigInvalid {
        index: String,
        reason: &'static str,
    },
    
    /// Index file creation failed.
    IndexFileCreate {
        path: PathBuf,
        source: std::io::Error,
    },
    
    /// Configuration file read failed.
    ConfigFileRead {
        path: PathBuf,
        source: std::io::Error,
    },
    
    /// Configuration parse error.
    ConfigParse {
        source: String,
    },
}
```

### Error Messages

```rust
impl fmt::Display for ReedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReedError::IndexBackendUnknown { backend, available } => {
                write!(f, "Index backend '{}' unknown. Available: {}", 
                    backend, available.join(", "))
            }
            ReedError::IndexOperationUnsupported { backend, operation, reason } => {
                write!(f, "Operation '{}' not supported by {} backend: {}", 
                    operation, backend, reason)
            }
            ReedError::IndexConfigInvalid { index, reason } => {
                write!(f, "Invalid configuration for index '{}': {}", index, reason)
            }
            // ... other variants
        }
    }
}
```

---

## Performance Tests

### Benchmark Suite

```rust
// reedbase/benches/index_backends.rs (new)
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use reedbase::indices::{BTreeIndex, HashMapIndex, Index};

fn benchmark_point_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("point_lookup");
    
    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        // HashMap
        let mut hashmap = HashMapIndex::new();
        for i in 0..*size {
            hashmap.insert(format!("key{}", i), vec![i]).unwrap();
        }
        
        group.bench_with_input(BenchmarkId::new("hashmap", size), size, |b, _| {
            b.iter(|| {
                let key = format!("key{}", size / 2);
                black_box(hashmap.get(&key))
            });
        });
        
        // B+-Tree
        let mut btree = BTreeIndex::open_or_create(
            format!("/tmp/bench_{}.btree", size), 
            512
        ).unwrap();
        for i in 0..*size {
            btree.insert(format!("key{}", i), vec![i]).unwrap();
        }
        
        group.bench_with_input(BenchmarkId::new("btree", size), size, |b, _| {
            b.iter(|| {
                let key = format!("key{}", size / 2);
                black_box(btree.get(&key))
            });
        });
    }
    
    group.finish();
}

fn benchmark_range_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_scan");
    
    // Only B+-Tree supports range queries
    for size in [1_000, 10_000, 100_000].iter() {
        let mut btree = BTreeIndex::open_or_create(
            format!("/tmp/bench_range_{}.btree", size),
            512
        ).unwrap();
        
        for i in 0..*size {
            btree.insert(format!("key{:08}", i), vec![i]).unwrap();
        }
        
        group.bench_with_input(BenchmarkId::new("btree_100_keys", size), size, |b, _| {
            b.iter(|| {
                let start = format!("key{:08}", size / 2);
                let end = format!("key{:08}", size / 2 + 100);
                black_box(btree.range(&start, &end))
            });
        });
    }
    
    group.finish();
}

fn benchmark_cold_start(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start");
    
    for size in [10_000, 100_000, 1_000_000].iter() {
        // Pre-populate B+-Tree file
        {
            let mut btree = BTreeIndex::open_or_create(
                format!("/tmp/coldstart_{}.btree", size),
                512
            ).unwrap();
            
            for i in 0..*size {
                btree.insert(format!("key{}", i), vec![i]).unwrap();
            }
        }  // Close file
        
        group.bench_with_input(BenchmarkId::new("btree_mmap", size), size, |b, _| {
            b.iter(|| {
                // Time to open existing file (mmap)
                black_box(BTreeIndex::<String, Vec<usize>>::open_or_create(
                    format!("/tmp/coldstart_{}.btree", size),
                    512
                ))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("hashmap_rebuild", size), size, |b, _| {
            b.iter(|| {
                // Time to rebuild HashMap from scratch
                let mut hashmap = HashMapIndex::new();
                for i in 0..*size {
                    hashmap.insert(format!("key{}", i), vec![i]).unwrap();
                }
                black_box(hashmap)
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_point_lookup, benchmark_range_scan, benchmark_cold_start);
criterion_main!(benches);
```

### Expected Results

**Point Lookup (1M keys):**
- HashMap: ~100ns (O(1))
- B+-Tree: ~800ns (O(log n), 20 page accesses)

**Range Scan (100 keys from 1M):**
- HashMap: Not supported
- B+-Tree: ~4ms (100 sequential page reads)

**Cold Start (1M keys):**
- HashMap rebuild: ~8s (parse CSV + insert)
- B+-Tree mmap: ~80ms (load header + validate)

---

## Acceptance Criteria

### Phase 1: Trait Abstraction
- [ ] `Index<K, V>` trait defined with 6 core methods + 3 metadata methods
- [ ] `HashMapIndex` wrapper created with 100% backward compatibility
- [ ] All 17 `smart_test.rs` tests pass without modification
- [ ] Performance within 5% of baseline (trait overhead <5ns per call)

### Phase 2: B+-Tree Backend
- [ ] `BTreeIndex` wrapper integrates REED-19-20 B+-Tree
- [ ] `IndexBuilder` factory selects backend from `.reed/config.toml`
- [ ] Migration tool (`reed index:migrate`) completes 10M keys in <30s
- [ ] Cold start time <200ms for 10M keys (vs 10s HashMap rebuild)
- [ ] Memory usage <100MB for 10M keys (vs 1.5GB HashMap)
- [ ] Query results identical between HashMap and B+-Tree backends

### Phase 3: Production Default
- [ ] Default configuration uses B+-Tree for namespace and hierarchy
- [ ] Documentation updated (README, tickets, man pages)
- [ ] Benchmark suite added (point lookup, range scan, cold start)
- [ ] Performance tests pass: point lookup <1ms, range scan <5ms (100 keys)

### Cross-Cutting
- [ ] All new files have license headers (BBC English comments)
- [ ] Separate test files (`{name}_test.rs`) with 100% coverage
- [ ] Error handling uses specific `ReedError` variants
- [ ] Documentation includes Input/Output/Performance/Error Conditions sections

---

## Documentation Updates

### README.md

```markdown
## Index Backends

ReedBase supports two index backends:

- **B+-Tree** (default): Persistent on-disk indices with low memory usage
  - Cold start: <100ms for 10M keys
  - Memory: <50MB for 10M keys
  - Range queries: Supported
  
- **HashMap** (legacy): In-memory indices with fast lookups
  - Cold start: 5-10s rebuild from CSV
  - Memory: 1.5GB for 10M keys
  - Range queries: Not supported

Configure backends in `.reed/config.toml`:

```toml
[indices]
default_backend = "btree"

[indices.namespace]
backend = "btree"
path = ".reed/indices/namespace.btree"

[indices.language]
backend = "hashmap"  # Small cardinality, keep in-memory
```

### Migration Guide

```bash
# Switch from HashMap to B+-Tree
reed index:migrate --backend btree

# Expected output:
# [1/3] Migrating namespace index... ✓ 1.2M keys (2.3s)
# [2/3] Migrating language index... ✓ 87 keys (0.1s)
# [3/3] Migrating hierarchy index... ✓ 450k keys (1.8s)
# Total: 1.65M keys migrated in 4.2s
# Memory: 1.5GB → 50MB (30x reduction)
```

### Man Page Updates

```
REED-INDEX-MIGRATE(1)

NAME
    reed index:migrate - Migrate indices to different backend

SYNOPSIS
    reed index:migrate [OPTIONS]

OPTIONS
    --backend <TYPE>
        Target backend: "btree" or "hashmap"
    
    --index <NAME>
        Migrate specific index (default: all)
        Values: namespace, language, hierarchy
    
    --config <PATH>
        Configuration file (default: .reed/config.toml)

EXAMPLES
    # Migrate all indices to B+-Tree
    reed index:migrate --backend btree
    
    # Migrate only namespace index
    reed index:migrate --backend btree --index namespace
    
    # Use custom configuration
    reed index:migrate --config /etc/reed/indices.toml

PERFORMANCE
    - 10M keys: ~30s total migration time
    - Memory savings: 30x (1.5GB → 50MB)
    - Cold start improvement: 100x (10s → 100ms)
```

---

## Timeline

### Day 1: Trait Abstraction
- [ ] Create `trait.rs` with `Index<K, V>` definition
- [ ] Create `hashmap_index.rs` wrapper
- [ ] Update `smart.rs` to use `Box<dyn Index>`
- [ ] Verify all 17 tests pass

### Day 2: B+-Tree Integration
- [ ] Create `btree_index.rs` wrapper
- [ ] Create `builder.rs` factory
- [ ] Add configuration support (`.reed/config.toml`)
- [ ] Write 25 integration tests

### Day 3: Migration Tool
- [ ] Create `migrate.rs` with CLI integration
- [ ] Test migration with 1M, 10M, 100M keys
- [ ] Benchmark cold start, memory, query performance
- [ ] Update documentation

---

## Risk Mitigation

### Risk 1: Performance Regression
- **Mitigation**: Benchmark suite in CI/CD, fail if >10% regression
- **Rollback**: Configuration flag allows instant switch to HashMap

### Risk 2: Data Loss During Migration
- **Mitigation**: Automatic backup before migration, validate row count after
- **Rollback**: Restore from backup if validation fails

### Risk 3: Incompatible Configuration
- **Mitigation**: `reed config:validate` command checks syntax before applying
- **Fallback**: Use default configuration if file missing/invalid

---

## Related Tickets

- **REED-19-11**: Smart Indices (HashMap baseline)
- **REED-19-20**: B+-Tree Index Engine (on-disk implementation)
- **REED-19-22**: ReedQL Range-Query Optimization (uses B+-Tree range scans)
- **REED-19-23**: Version-Log Index (timestamp-based B+-Tree)

---

## Notes

This ticket is the critical bridge between in-memory prototypes (REED-19-11) and production-grade persistence (REED-19-20). The trait abstraction ensures future backends (LSM-tree, radix tree) can be added without changing query logic.

**Key Design Decisions:**
1. **Trait objects over generics**: Simpler API, runtime backend selection
2. **Configuration-driven**: No code changes needed to switch backends
3. **Gradual migration**: Per-index choice allows testing in production
4. **Backward compatible**: HashMap backend preserved for low-latency use cases
