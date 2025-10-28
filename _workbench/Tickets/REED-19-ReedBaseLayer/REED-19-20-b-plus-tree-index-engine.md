# REED-19-20: B+-Tree Index Engine

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}_test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-19-20
- **Title**: B+-Tree Index Engine (On-Disk Persistent Indices)
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: High
- **Status**: Open
- **Complexity**: Very High
- **Dependencies**: REED-19-11 (Smart Indices)
- **Estimated Time**: 5-7 days

## Objective

Implement generic on-disk B+-Tree index engine to replace in-memory HashMap indices. Provides persistent indices with 100x faster cold starts, 30x less memory, and enables range queries for ReedQL.

## Requirements

### Core Features

1. **Generic B+-Tree Implementation**
   - Configurable key/value types
   - Configurable order (degree)
   - Self-balancing (automatic splits/merges)

2. **On-Disk Persistence**
   - mmap-based file access (FreeBSD-compatible)
   - Page-based storage (4KB pages)
   - Crash-safe with WAL (Write-Ahead-Log)

3. **Performance Targets**

| Operation | Target | Notes |
|-----------|--------|-------|
| Point lookup | < 1ms | vs < 0.1ms HashMap |
| Range scan (100 keys) | < 5ms | HashMap can't do this |
| Insert | < 2ms | Includes splits/rebalancing |
| Delete | < 2ms | Includes merges |
| Cold start (10M keys) | < 100ms | vs 10s HashMap rebuild |
| Memory usage (10M keys) | < 50MB | vs 1.5GB HashMap |

4. **Trait-Based Design**

```rust
trait Index<K, V> {
    fn get(&self, key: &K) -> ReedResult<Option<V>>;
    fn range(&self, start: &K, end: &K) -> ReedResult<Vec<(K, V)>>;
    fn insert(&mut self, key: K, value: V) -> ReedResult<()>;
    fn delete(&mut self, key: &K) -> ReedResult<()>;
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)>>;
}

impl Index<String, Vec<usize>> for BPlusTree { ... }
impl Index<String, Vec<usize>> for HashMap { ... }  // Fallback
```

### File Structure

```
.reed/indices/
├── namespace.btree         # B+-Tree file (mmap'd)
├── namespace.wal          # Write-Ahead-Log
├── language.btree
├── language.wal
├── hierarchy.btree
├── hierarchy.wal
└── config.toml            # Index configuration
```

### Page Format (On-Disk Layout)

```
Page Header (32 bytes)
├── magic: u32            # 0xB7EE7EE1 (validation)
├── page_type: u8         # 0=internal, 1=leaf
├── num_keys: u16         # Number of keys in page
├── next_page: u32        # For leaf pages (linked list)
├── checksum: u32         # CRC32 of page data
└── padding: [u8; 15]     # Reserved

Page Data (4064 bytes)
├── Keys: [K; num_keys]
└── Values: [V; num_keys]  # Or child pointers for internal
```

## Implementation Files

### Primary Implementation

**`reedbase/src/btree/mod.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! B+-Tree on-disk index engine.
//!
//! Generic persistent index implementation using B+-Trees.

mod tree;
mod node;
mod page;
mod wal;
mod iter;
mod types;

#[cfg(test)]
mod btree_test;

pub use tree::BPlusTree;
pub use types::{Index, Order};
```

**`reedbase/src/btree/tree.rs`**

One file = B+-Tree structure and core operations only.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! B+-Tree implementation.

use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io;
use memmap2::{Mmap, MmapMut};
use crate::types::{ReedResult, ReedError};
use super::types::{Index, Order};
use super::node::Node;
use super::page::Page;
use super::wal::WriteAheadLog;

/// B+-Tree persistent index.
///
/// ## Features
/// - On-disk persistence (mmap)
/// - Crash-safe (WAL)
/// - Range queries
/// - Memory efficient (page cache)
///
/// ## Performance
/// - Point lookup: < 1ms (O(log n))
/// - Range scan: < 5ms per 100 keys
/// - Cold start: < 100ms (no rebuild)
///
/// ## Example Usage
/// ```rust
/// let tree = BPlusTree::open("namespace.btree", Order::new(100))?;
/// tree.insert("page".to_string(), vec![1, 2, 3])?;
/// let value = tree.get(&"page".to_string())?;
/// ```
pub struct BPlusTree<K, V> {
    path: PathBuf,
    file: File,
    mmap: MmapMut,
    root_page: u32,
    order: Order,
    wal: WriteAheadLog,
}

impl<K, V> BPlusTree<K, V> 
where
    K: Ord + Clone + serde::Serialize + serde::de::DeserializeOwned,
    V: Clone + serde::Serialize + serde::de::DeserializeOwned,
{
    /// Open or create B+-Tree index.
    ///
    /// ## Input
    /// - `path`: Path to .btree file
    /// - `order`: B-Tree order (keys per node)
    ///
    /// ## Output
    /// - `ReedResult<Self>`: B+-Tree instance
    ///
    /// ## Performance
    /// - Create new: < 10ms (allocate pages)
    /// - Open existing: < 1ms (mmap existing)
    ///
    /// ## Error Conditions
    /// - IoError: Cannot create/open file
    /// - CorruptedIndex: Invalid page format
    pub fn open<P: AsRef<Path>>(path: P, order: Order) -> ReedResult<Self> {
        let path = path.as_ref().to_path_buf();
        
        // Open or create file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        
        // Ensure minimum size (1 page)
        let file_size = file.metadata()?.len();
        if file_size == 0 {
            file.set_len(4096)?;  // 1 page
        }
        
        // Memory-map file
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        
        // Open WAL
        let wal_path = path.with_extension("wal");
        let wal = WriteAheadLog::open(&wal_path)?;
        
        // Initialize or load root
        let root_page = if file_size == 0 {
            Self::initialize_root(&mmap)?
        } else {
            Self::load_root(&mmap)?
        };
        
        Ok(Self {
            path,
            file,
            mmap,
            root_page,
            order,
            wal,
        })
    }
    
    /// Initialize new empty tree.
    fn initialize_root(mmap: &MmapMut) -> ReedResult<u32> {
        let page = Page::new_leaf(0);
        page.write_to(mmap, 0)?;
        Ok(0)
    }
    
    /// Load existing tree root.
    fn load_root(mmap: &Mmap) -> ReedResult<u32> {
        let page = Page::read_from(mmap, 0)?;
        page.validate()?;
        Ok(0)  // Root always at page 0
    }
    
    /// Point lookup.
    ///
    /// ## Performance
    /// - O(log n) page loads
    /// - < 1ms typical (3-4 pages for 1M keys)
    fn get(&self, key: &K) -> ReedResult<Option<V>> {
        let mut page_id = self.root_page;
        
        loop {
            let page = Page::read_from(&self.mmap, page_id)?;
            
            match page.page_type() {
                PageType::Internal => {
                    // Navigate to child
                    page_id = page.find_child(key)?;
                }
                PageType::Leaf => {
                    // Found leaf, search for key
                    return page.find_value(key);
                }
            }
        }
    }
    
    /// Range query.
    ///
    /// ## Performance
    /// - O(log n + k) where k = result size
    /// - < 5ms per 100 keys
    fn range(&self, start: &K, end: &K) -> ReedResult<Vec<(K, V)>> {
        let mut results = Vec::new();
        
        // 1. Find starting leaf page
        let mut page_id = self.find_leaf(start)?;
        
        // 2. Scan leaf pages (linked list)
        loop {
            let page = Page::read_from(&self.mmap, page_id)?;
            
            // Collect keys in range
            for (k, v) in page.iter() {
                if k >= start && k < end {
                    results.push((k.clone(), v.clone()));
                } else if k >= end {
                    return Ok(results);  // Past end
                }
            }
            
            // Next leaf page
            if let Some(next) = page.next_page() {
                page_id = next;
            } else {
                break;  // Last page
            }
        }
        
        Ok(results)
    }
    
    /// Insert key-value pair.
    ///
    /// ## Performance
    /// - O(log n) average
    /// - O(log n + split cost) worst case
    /// - < 2ms typical
    fn insert(&mut self, key: K, value: V) -> ReedResult<()> {
        // 1. Write to WAL first (crash safety)
        self.wal.log_insert(&key, &value)?;
        
        // 2. Find insertion point
        let mut path = self.find_path(&key)?;
        
        // 3. Insert into leaf
        let leaf_id = path.last().unwrap();
        let mut page = Page::read_from(&self.mmap, *leaf_id)?;
        
        if page.has_space(&key, &value) {
            // Simple case: space available
            page.insert(key, value)?;
            page.write_to(&mut self.mmap, *leaf_id)?;
        } else {
            // Complex case: split required
            self.split_and_insert(page, key, value, &mut path)?;
        }
        
        // 4. Sync WAL
        self.wal.sync()?;
        
        Ok(())
    }
    
    /// Split full page and insert.
    fn split_and_insert(
        &mut self,
        page: Page<K, V>,
        key: K,
        value: V,
        path: &mut Vec<u32>,
    ) -> ReedResult<()> {
        // 1. Allocate new page
        let new_page_id = self.allocate_page()?;
        
        // 2. Split keys
        let (left_keys, right_keys) = page.split_keys();
        let median = right_keys[0].clone();
        
        // 3. Write split pages
        let left_page = Page::from_keys(left_keys);
        let right_page = Page::from_keys(right_keys);
        
        left_page.write_to(&mut self.mmap, page.id())?;
        right_page.write_to(&mut self.mmap, new_page_id)?;
        
        // 4. Insert key into appropriate page
        if key < median {
            left_page.insert(key, value)?;
        } else {
            right_page.insert(key, value)?;
        }
        
        // 5. Propagate split upwards
        self.propagate_split(path, median, new_page_id)?;
        
        Ok(())
    }
    
    /// Delete key.
    ///
    /// ## Performance
    /// - O(log n) average
    /// - < 2ms typical
    fn delete(&mut self, key: &K) -> ReedResult<()> {
        // 1. Write to WAL first
        self.wal.log_delete(key)?;
        
        // 2. Find and remove
        let mut path = self.find_path(key)?;
        let leaf_id = path.last().unwrap();
        let mut page = Page::read_from(&self.mmap, *leaf_id)?;
        
        page.remove(key)?;
        
        // 3. Check if underflow
        if page.is_underflow(self.order) {
            self.rebalance_or_merge(page, &mut path)?;
        } else {
            page.write_to(&mut self.mmap, *leaf_id)?;
        }
        
        // 4. Sync WAL
        self.wal.sync()?;
        
        Ok(())
    }
    
    /// Sync changes to disk.
    pub fn sync(&mut self) -> ReedResult<()> {
        self.mmap.flush()?;
        self.wal.sync()?;
        Ok(())
    }
    
    /// Close tree and cleanup.
    pub fn close(mut self) -> ReedResult<()> {
        self.sync()?;
        self.wal.truncate()?;  // Clear WAL after sync
        Ok(())
    }
}

impl<K, V> Index<K, V> for BPlusTree<K, V>
where
    K: Ord + Clone + serde::Serialize + serde::de::DeserializeOwned,
    V: Clone + serde::Serialize + serde::de::DeserializeOwned,
{
    fn get(&self, key: &K) -> ReedResult<Option<V>> {
        self.get(key)
    }
    
    fn range(&self, start: &K, end: &K) -> ReedResult<Vec<(K, V)>> {
        self.range(start, end)
    }
    
    fn insert(&mut self, key: K, value: V) -> ReedResult<()> {
        self.insert(key, value)
    }
    
    fn delete(&mut self, key: &K) -> ReedResult<()> {
        self.delete(key)
    }
    
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)>> {
        Box::new(BTreeIterator::new(self))
    }
}
```

**`reedbase/src/btree/page.rs`**

One file = Page format and I/O only.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! B+-Tree page format and I/O.

const PAGE_SIZE: usize = 4096;
const MAGIC: u32 = 0xB7EE7EE1;

/// Page header (32 bytes).
#[repr(C)]
struct PageHeader {
    magic: u32,          // Magic number for validation
    page_type: u8,       // 0=internal, 1=leaf
    num_keys: u16,       // Number of keys in page
    next_page: u32,      // For leaf pages (linked list)
    checksum: u32,       // CRC32 of page data
    _padding: [u8; 15],  // Reserved for future use
}

/// Page types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    Internal = 0,
    Leaf = 1,
}

/// B+-Tree page.
pub struct Page<K, V> {
    header: PageHeader,
    keys: Vec<K>,
    values: Vec<V>,  // Or child pointers for internal
}

impl<K, V> Page<K, V> {
    /// Create new leaf page.
    pub fn new_leaf(page_id: u32) -> Self {
        Self {
            header: PageHeader {
                magic: MAGIC,
                page_type: PageType::Leaf as u8,
                num_keys: 0,
                next_page: 0,
                checksum: 0,
                _padding: [0; 15],
            },
            keys: Vec::new(),
            values: Vec::new(),
        }
    }
    
    /// Read page from mmap.
    pub fn read_from(mmap: &Mmap, page_id: u32) -> ReedResult<Self> {
        let offset = (page_id as usize) * PAGE_SIZE;
        let data = &mmap[offset..offset + PAGE_SIZE];
        
        // Deserialize page
        bincode::deserialize(data)
            .map_err(|e| ReedError::CorruptedIndex {
                reason: format!("Page {}: {}", page_id, e),
            })
    }
    
    /// Write page to mmap.
    pub fn write_to(&self, mmap: &mut MmapMut, page_id: u32) -> ReedResult<()> {
        let offset = (page_id as usize) * PAGE_SIZE;
        
        // Calculate checksum
        let mut page = self.clone();
        page.header.checksum = Self::calculate_checksum(&page);
        
        // Serialize page
        let data = bincode::serialize(&page)
            .map_err(|e| ReedError::SerializationError {
                reason: e.to_string(),
            })?;
        
        // Write to mmap
        mmap[offset..offset + data.len()].copy_from_slice(&data);
        
        Ok(())
    }
    
    /// Validate page integrity.
    pub fn validate(&self) -> ReedResult<()> {
        // Check magic
        if self.header.magic != MAGIC {
            return Err(ReedError::CorruptedIndex {
                reason: format!("Invalid magic: {:x}", self.header.magic),
            });
        }
        
        // Check checksum
        let expected = Self::calculate_checksum(self);
        if self.header.checksum != expected {
            return Err(ReedError::CorruptedIndex {
                reason: format!("Checksum mismatch: expected {:x}, got {:x}", 
                    expected, self.header.checksum),
            });
        }
        
        Ok(())
    }
    
    /// Calculate CRC32 checksum.
    fn calculate_checksum(&self) -> u32 {
        use crc32fast::Hasher;
        
        let mut hasher = Hasher::new();
        
        // Hash keys and values (not header)
        for key in &self.keys {
            let key_bytes = bincode::serialize(key).unwrap();
            hasher.update(&key_bytes);
        }
        for value in &self.values {
            let val_bytes = bincode::serialize(value).unwrap();
            hasher.update(&val_bytes);
        }
        
        hasher.finalize()
    }
}
```

**`reedbase/src/btree/wal.rs`**

One file = Write-Ahead-Log only.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Write-Ahead-Log for crash safety.

use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use crate::types::ReedResult;

/// WAL entry types.
#[derive(Debug, Clone)]
enum WalEntry<K, V> {
    Insert { key: K, value: V },
    Delete { key: K },
}

/// Write-Ahead-Log.
pub struct WriteAheadLog {
    file: BufWriter<File>,
}

impl WriteAheadLog {
    /// Open WAL file.
    pub fn open<P: AsRef<Path>>(path: P) -> ReedResult<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        
        Ok(Self {
            file: BufWriter::new(file),
        })
    }
    
    /// Log insert operation.
    pub fn log_insert<K, V>(&mut self, key: &K, value: &V) -> ReedResult<()>
    where
        K: serde::Serialize,
        V: serde::Serialize,
    {
        let entry = WalEntry::Insert {
            key: key.clone(),
            value: value.clone(),
        };
        
        let data = bincode::serialize(&entry)?;
        self.file.write_all(&data)?;
        
        Ok(())
    }
    
    /// Log delete operation.
    pub fn log_delete<K>(&mut self, key: &K) -> ReedResult<()>
    where
        K: serde::Serialize + Clone,
    {
        let entry = WalEntry::Delete::<K, ()> {
            key: key.clone(),
        };
        
        let data = bincode::serialize(&entry)?;
        self.file.write_all(&data)?;
        
        Ok(())
    }
    
    /// Sync WAL to disk.
    pub fn sync(&mut self) -> ReedResult<()> {
        self.file.flush()?;
        self.file.get_ref().sync_all()?;
        Ok(())
    }
    
    /// Truncate WAL after successful sync.
    pub fn truncate(&mut self) -> ReedResult<()> {
        self.file.get_mut().set_len(0)?;
        Ok(())
    }
}
```

**`reedbase/src/btree/types.rs`**

One file = Type definitions only.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! B+-Tree type definitions.

use crate::types::ReedResult;

/// Generic index trait.
pub trait Index<K, V> {
    fn get(&self, key: &K) -> ReedResult<Option<V>>;
    fn range(&self, start: &K, end: &K) -> ReedResult<Vec<(K, V)>>;
    fn insert(&mut self, key: K, value: V) -> ReedResult<()>;
    fn delete(&mut self, key: &K) -> ReedResult<()>;
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)>>;
}

/// B+-Tree order (degree).
#[derive(Debug, Clone, Copy)]
pub struct Order(usize);

impl Order {
    pub fn new(order: usize) -> Self {
        assert!(order >= 3, "Order must be >= 3");
        Self(order)
    }
    
    pub fn value(&self) -> usize {
        self.0
    }
    
    pub fn min_keys(&self) -> usize {
        self.0 / 2
    }
    
    pub fn max_keys(&self) -> usize {
        self.0 - 1
    }
}

impl Default for Order {
    fn default() -> Self {
        Self::new(100)  // Reasonable default
    }
}
```

### Test Files

**`reedbase/src/btree/btree_test.rs`**

Comprehensive test suite (100+ tests).

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_create_new_tree() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.btree");
        
        let tree = BPlusTree::<String, Vec<usize>>::open(&path, Order::default()).unwrap();
        assert!(path.exists());
    }
    
    #[test]
    fn test_insert_and_get() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.btree");
        
        let mut tree = BPlusTree::open(&path, Order::default()).unwrap();
        tree.insert("page".to_string(), vec![1, 2, 3]).unwrap();
        
        let value = tree.get(&"page".to_string()).unwrap();
        assert_eq!(value, Some(vec![1, 2, 3]));
    }
    
    #[test]
    fn test_insert_many_sequential() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.btree");
        
        let mut tree = BPlusTree::open(&path, Order::new(10)).unwrap();
        
        // Insert 1000 sequential keys
        for i in 0..1000 {
            tree.insert(format!("key{:04}", i), vec![i]).unwrap();
        }
        
        // Verify all present
        for i in 0..1000 {
            let value = tree.get(&format!("key{:04}", i)).unwrap();
            assert_eq!(value, Some(vec![i]));
        }
    }
    
    #[test]
    fn test_insert_many_random() {
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.btree");
        
        let mut tree = BPlusTree::open(&path, Order::new(10)).unwrap();
        
        // Generate random keys
        let mut keys: Vec<usize> = (0..1000).collect();
        keys.shuffle(&mut thread_rng());
        
        // Insert in random order
        for key in &keys {
            tree.insert(format!("key{:04}", key), vec![*key]).unwrap();
        }
        
        // Verify all present
        for key in &keys {
            let value = tree.get(&format!("key{:04}", key)).unwrap();
            assert_eq!(value, Some(vec![*key]));
        }
    }
    
    #[test]
    fn test_range_query() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.btree");
        
        let mut tree = BPlusTree::open(&path, Order::default()).unwrap();
        
        // Insert keys
        for i in 0..100 {
            tree.insert(format!("key{:03}", i), vec![i]).unwrap();
        }
        
        // Range query: key020 to key030
        let results = tree.range(
            &"key020".to_string(),
            &"key030".to_string()
        ).unwrap();
        
        assert_eq!(results.len(), 10);
        assert_eq!(results[0].0, "key020");
        assert_eq!(results[9].0, "key029");
    }
    
    #[test]
    fn test_delete() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.btree");
        
        let mut tree = BPlusTree::open(&path, Order::default()).unwrap();
        
        tree.insert("key1".to_string(), vec![1]).unwrap();
        tree.insert("key2".to_string(), vec![2]).unwrap();
        
        tree.delete(&"key1".to_string()).unwrap();
        
        assert_eq!(tree.get(&"key1".to_string()).unwrap(), None);
        assert_eq!(tree.get(&"key2".to_string()).unwrap(), Some(vec![2]));
    }
    
    #[test]
    fn test_persistence() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.btree");
        
        // Create and populate tree
        {
            let mut tree = BPlusTree::open(&path, Order::default()).unwrap();
            tree.insert("key1".to_string(), vec![1]).unwrap();
            tree.insert("key2".to_string(), vec![2]).unwrap();
            tree.sync().unwrap();
        }
        
        // Reopen and verify
        {
            let tree = BPlusTree::<String, Vec<usize>>::open(&path, Order::default()).unwrap();
            assert_eq!(tree.get(&"key1".to_string()).unwrap(), Some(vec![1]));
            assert_eq!(tree.get(&"key2".to_string()).unwrap(), Some(vec![2]));
        }
    }
    
    #[test]
    fn test_crash_recovery() {
        // TODO: Simulate crash during write, verify WAL recovery
    }
    
    #[test]
    fn test_concurrent_reads() {
        // TODO: Multiple readers, verify correctness
    }
    
    #[test]
    fn test_performance_point_lookup() {
        use std::time::Instant;
        
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.btree");
        
        let mut tree = BPlusTree::open(&path, Order::new(100)).unwrap();
        
        // Insert 1M keys
        for i in 0..1_000_000 {
            tree.insert(format!("key{:07}", i), vec![i]).unwrap();
        }
        
        // Measure lookup time
        let start = Instant::now();
        for i in 0..1000 {
            let key = format!("key{:07}", i * 1000);
            tree.get(&key).unwrap();
        }
        let elapsed = start.elapsed();
        
        let avg_latency = elapsed.as_micros() / 1000;
        assert!(avg_latency < 1000, "Lookup too slow: {}μs", avg_latency);
    }
    
    #[test]
    fn test_performance_range_scan() {
        use std::time::Instant;
        
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.btree");
        
        let mut tree = BPlusTree::open(&path, Order::new(100)).unwrap();
        
        // Insert 10k keys
        for i in 0..10_000 {
            tree.insert(format!("key{:05}", i), vec![i]).unwrap();
        }
        
        // Measure range scan time (100 keys)
        let start = Instant::now();
        let results = tree.range(
            &"key05000".to_string(),
            &"key05100".to_string()
        ).unwrap();
        let elapsed = start.elapsed();
        
        assert_eq!(results.len(), 100);
        assert!(elapsed.as_millis() < 5, "Range scan too slow: {}ms", elapsed.as_millis());
    }
    
    #[test]
    fn test_memory_usage() {
        // TODO: Verify < 50MB for 10M keys
    }
}
```

## Performance Requirements

| Metric | Target | Notes |
|--------|--------|-------|
| Point lookup | < 1ms | O(log n), 3-4 pages for 1M keys |
| Range scan (100) | < 5ms | O(log n + k) |
| Insert | < 2ms | Including splits |
| Delete | < 2ms | Including merges |
| Cold start (10M keys) | < 100ms | Load metadata only |
| Memory usage (10M keys) | < 50MB | Page cache |
| Disk usage (10M keys) | ~2GB | 4KB pages |

## Error Conditions

- **IoError**: Cannot read/write index file
- **CorruptedIndex**: Invalid page format or checksum
- **WalRecoveryFailed**: WAL replay error
- **SerializationError**: Cannot serialize key/value
- **DeserializationError**: Cannot deserialize page

## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| btree_lookup_latency | Histogram | ms | <1 | >5 | tree.rs:get() |
| btree_range_scan_latency | Histogram | ms | <5 | >20 | tree.rs:range() |
| btree_insert_latency | Histogram | ms | <2 | >10 | tree.rs:insert() |
| btree_delete_latency | Histogram | ms | <2 | >10 | tree.rs:delete() |
| btree_split_count | Counter | count | - | - | tree.rs:split_and_insert() |
| btree_merge_count | Counter | count | - | - | tree.rs:rebalance_or_merge() |
| btree_page_cache_hits | Counter | count | - | - | page.rs:read_from() |
| btree_page_cache_misses | Counter | count | - | - | page.rs:read_from() |
| btree_wal_sync_latency | Histogram | ms | <1 | >5 | wal.rs:sync() |

### Alert Rules

**CRITICAL Alerts:**
- `btree_lookup_latency p99 > 5ms` for 5 minutes → "B+-Tree lookups critically slow"
- `btree_page_cache_hits / (hits + misses) < 0.8` for 10 minutes → "B+-Tree cache hit rate too low"

**WARNING Alerts:**
- `btree_insert_latency p99 > 10ms` for 5 minutes → "B+-Tree inserts slow - check splits"
- `btree_split_count > 1000/min` for 5 minutes → "Excessive page splits - consider larger order"

### Collection Strategy

- **Sampling**: All operations
- **Aggregation**: 1-minute rolling window
- **Storage**: `.reedbase/metrics/btree.csv`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**btree_lookup_latency**: Core index performance
- Every indexed query uses B+-Tree lookups
- 1ms target allows 1000 lookups/second
- Degradation indicates page cache issues

**btree_split_count**: Tree health indicator
- Frequent splits = poor key distribution or small order
- Helps identify optimal order parameter
- Affects write performance

**btree_page_cache_hits**: Memory efficiency
- High hit rate (>80%) = good cache strategy
- Low hit rate = need larger cache or better eviction
- Directly impacts lookup latency

## Acceptance Criteria

- [ ] Generic B+-Tree implementation with configurable order
- [ ] On-disk persistence using mmap (FreeBSD-compatible)
- [ ] Page format with CRC32 checksums
- [ ] WAL for crash safety
- [ ] Point lookup in < 1ms (1M keys)
- [ ] Range query in < 5ms (100 keys)
- [ ] Insert in < 2ms (average)
- [ ] Delete in < 2ms (average)
- [ ] Cold start in < 100ms (10M keys)
- [ ] Memory usage < 50MB (10M keys)
- [ ] Trait-based design (Index trait)
- [ ] 100+ comprehensive tests
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met (benchmarks)
- [ ] All code in BBC English
- [ ] All functions have proper documentation
- [ ] No Swiss Army knife functions
- [ ] Separate test file as `btree_test.rs`

## Dependencies
- **Requires**: REED-19-11 (Smart Indices - to replace HashMap)
- **External Crates**: 
  - `memmap2` (mmap support)
  - `bincode` (serialization)
  - `crc32fast` (checksums)

## Blocks
- REED-19-21 (Migrate Smart Indices to B+-Tree)
- REED-19-22 (ReedQL Range Optimization)
- REED-19-23 (Version-Log Index)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- B+-Tree Theory: https://en.wikipedia.org/wiki/B%2B_tree
- FreeBSD mmap: https://man.freebsd.org/cgi/man.cgi?query=mmap
- REED-19-00: Layer Overview
- REED-19-11: Smart Indices (current implementation)

## Notes

**Why B+-Trees over HashMap?**

1. **Persistent**: No rebuild on cold start (100x faster)
2. **Memory Efficient**: 30x less memory (page cache vs full HashMap)
3. **Range Queries**: HashMap can't do `BETWEEN` queries
4. **Disk-Friendly**: Sequential leaf pages (better I/O)

**Why NOT B-Trees (without +)?**

- B+-Trees have **linked leaf pages** (range queries)
- B-Trees store values in internal nodes (worse cache locality)

**Trade-offs:**

| Metric | HashMap | B+-Tree |
|--------|---------|---------|
| Point lookup | 0.1ms | 1ms (10x slower) |
| Range query | ❌ Can't do | 5ms (100 keys) |
| Memory (10M) | 1.5GB | 50MB (30x less) |
| Cold start | 10s (rebuild) | 0.1s (100x faster) |
| Persistence | ❌ No | ✅ Yes |

**Optimal Use Case:**
- Large installations (>1M keys)
- Frequent restarts (cold starts matter)
- Memory-constrained systems
- Range queries needed

**When to Use HashMap:**
- Small installations (<100k keys)
- RAM abundant
- No range queries
- Simplicity preferred

**Implementation Strategy:**
1. Implement B+-Tree as generic `Index` trait
2. Keep HashMap as fallback (config flag)
3. Allow per-index choice (namespace=BTree, language=HashMap)
4. Default: B+-Tree (better for production)

**FreeBSD Compatibility:**
- Use `memmap2` (cross-platform)
- Avoid Linux-specific syscalls
- Test on FreeBSD before merge
