# REED-19-23: Version-Log Index (Timestamp B+-Tree)

**Layer**: REED-19 (ReedBase Layer)  
**Phase**: 5 (Distributed + P2P)  
**Dependencies**: REED-19-03 (Binary Delta Versioning), REED-19-20 (B+-Tree), REED-19-21 (Index Migration)  
**Estimated Effort**: 1 day  
**Priority**: Medium  
**Status**: Planned

---

## Executive Summary

Add **timestamp-indexed B+-Tree for version history** to accelerate:
- **Point-in-time recovery** (100x faster: 10s → 100ms for 10k versions)
- **Frame lookups** (instant retrieval of coordinated batches)
- **Audit queries** ("show all changes between T1 and T2")
- **Snapshot browsing** (time-travel queries for debugging)

Currently, finding versions requires linear scan through `.reed/flow/versions.log` (O(n)). This ticket adds a secondary index on timestamp column for O(log n) range queries.

---

## Problem Statement

### Current State (REED-19-03)

**Version Log Format** (`.reed/flow/versions.log`):
```csv
version_id|timestamp|frame_id|key|value_hash|delta_bytes|metadata
1|2025-10-28T08:15:23.001Z|F001|page.title@de|abc123|142|user=alice
2|2025-10-28T08:15:23.001Z|F001|page.desc@de|def456|89|user=alice
3|2025-10-28T08:16:45.500Z|F002|page.title@de|ghi789|67|user=bob
...
10000|2025-10-28T12:30:00.000Z|F500|landing.hero@en|xyz999|234|user=carol
```

**Point-in-Time Recovery Query:**
```rust
// Find all versions between T1 and T2
let versions = read_csv(".reed/flow/versions.log")?;
let filtered: Vec<_> = versions.into_iter()
    .filter(|v| v.timestamp >= t1 && v.timestamp <= t2)
    .collect();
// O(n) scan through all 10k versions → ~10s
```

**Frame Lookup Query:**
```rust
// Find all versions in frame F042
let versions = read_csv(".reed/flow/versions.log")?;
let filtered: Vec<_> = versions.into_iter()
    .filter(|v| v.frame_id == "F042")
    .collect();
// O(n) scan → ~5s
```

### Desired State (REED-19-23)

**Timestamp Index** (`.reed/indices/versions_timestamp.btree`):
```
B+-Tree<RFC3339Timestamp, Vec<VersionId>>
  "2025-10-28T08:15:23.001Z" → [1, 2]
  "2025-10-28T08:16:45.500Z" → [3]
  ...
```

**Frame Index** (`.reed/indices/versions_frame.btree`):
```
B+-Tree<FrameId, Vec<VersionId>>
  "F001" → [1, 2]
  "F002" → [3, 4, 5]
  ...
```

**Optimized Point-in-Time Recovery:**
```rust
// Range scan on timestamp index
let version_ids = timestamp_index.range(t1, t2)?;
let versions = fetch_versions_by_id(version_ids)?;
// O(log n + k) where k = result size → ~100ms
```

**Performance Improvement:**
- Point-in-time recovery: **100x faster** (10s → 100ms for 10k versions)
- Frame lookup: **50x faster** (5s → 100ms for 200-version frame)
- Memory overhead: +10MB for 100k versions (acceptable)

---

## Architecture

### Dual-Index Design

```
┌──────────────────────────────────────────────────────────────┐
│ Version Log Indexing Architecture                            │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  PRIMARY STORAGE                                             │
│  ┌────────────────────────────────────────┐                  │
│  │ .reed/flow/versions.log (append-only)  │                  │
│  │ version_id | timestamp | frame | ...   │                  │
│  │ 1 | 2025-10-28T08:15:23.001Z | F001    │                  │
│  │ 2 | 2025-10-28T08:15:23.001Z | F001    │                  │
│  │ 3 | 2025-10-28T08:16:45.500Z | F002    │                  │
│  └───────────────┬────────────────────────┘                  │
│                  │                                           │
│                  ├── (indexed on write) ──────┐              │
│                  │                            │              │
│                  ▼                            ▼              │
│  SECONDARY INDICES                                           │
│  ┌──────────────────────────┐  ┌───────────────────────┐    │
│  │ Timestamp Index (B+-Tree)│  │ Frame Index (B+-Tree) │    │
│  │                          │  │                       │    │
│  │ 2025-10-28T08:15:23.001Z │  │ F001 → [1, 2]         │    │
│  │   → [1, 2]               │  │ F002 → [3]            │    │
│  │ 2025-10-28T08:16:45.500Z │  │ ...                   │    │
│  │   → [3]                  │  │                       │    │
│  └──────────────────────────┘  └───────────────────────┘    │
│         │                               │                   │
│         │ (range query)                 │ (point lookup)    │
│         ▼                               ▼                   │
│  ┌────────────────────────────────────────────┐             │
│  │ Query Results: [1, 2, 3]                   │             │
│  └────────────┬───────────────────────────────┘             │
│               │                                             │
│               ▼                                             │
│  ┌────────────────────────────────────────────┐             │
│  │ Fetch Full Versions from Log               │             │
│  └────────────────────────────────────────────┘             │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Index Lifecycle

```rust
// 1. WRITE PATH (append to log + update indices)
fn commit_version(version: Version) -> ReedResult<()> {
    // 1a. Append to primary log
    append_to_log(".reed/flow/versions.log", &version)?;
    
    // 1b. Update timestamp index
    timestamp_index.insert(
        version.timestamp.clone(),
        vec![version.version_id]
    )?;
    
    // 1c. Update frame index
    frame_index.insert(
        version.frame_id.clone(),
        vec![version.version_id]
    )?;
    
    Ok(())
}

// 2. READ PATH (range query on index + fetch from log)
fn point_in_time_recovery(t1: Timestamp, t2: Timestamp) -> ReedResult<Vec<Version>> {
    // 2a. Range scan on timestamp index
    let results = timestamp_index.range(&t1, &t2)?;
    
    // 2b. Flatten version IDs
    let version_ids: Vec<usize> = results.into_iter()
        .flat_map(|(_, ids)| ids)
        .collect();
    
    // 2c. Fetch full versions from log (by ID)
    fetch_versions_by_id(".reed/flow/versions.log", &version_ids)
}
```

---

## Implementation Files

### 1. `reedbase/src/versioning/index.rs` (NEW)

**Purpose:** Manage timestamp and frame indices for version log

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Version log indexing for fast timestamp and frame queries.
//!
//! ## Use Cases
//! - Point-in-time recovery: "All versions between T1 and T2"
//! - Frame lookups: "All versions in frame F042"
//! - Audit queries: "What changed in the last hour?"
//! - Snapshot browsing: "View state at 2025-10-28 10:00:00"

use crate::btree::BPlusTree;
use crate::error::{ReedError, ReedResult};
use crate::indices::Index;
use std::path::Path;

/// RFC3339 timestamp string (e.g., "2025-10-28T08:15:23.001Z").
pub type Timestamp = String;

/// Frame ID (e.g., "F001").
pub type FrameId = String;

/// Version ID (1-based sequential counter).
pub type VersionId = usize;

/// Version log indices (timestamp + frame).
pub struct VersionIndices {
    /// Index: Timestamp → Vec<VersionId>
    timestamp_index: Box<dyn Index<Timestamp, Vec<VersionId>>>,
    
    /// Index: FrameId → Vec<VersionId>
    frame_index: Box<dyn Index<FrameId, Vec<VersionId>>>,
}

impl VersionIndices {
    /// Open or create version indices.
    ///
    /// ## Arguments
    /// - `timestamp_path`: Path to timestamp B+-Tree (e.g., ".reed/indices/versions_timestamp.btree")
    /// - `frame_path`: Path to frame B+-Tree (e.g., ".reed/indices/versions_frame.btree")
    ///
    /// ## Performance
    /// - Cold start: <100ms (mmap existing files)
    /// - Memory: ~10MB for 100k versions
    pub fn open_or_create<P: AsRef<Path>>(
        timestamp_path: P,
        frame_path: P,
    ) -> ReedResult<Self> {
        use crate::indices::BTreeIndex;
        
        let timestamp_index = Box::new(BTreeIndex::open_or_create(
            timestamp_path,
            512,  // B+-Tree order
        )?);
        
        let frame_index = Box::new(BTreeIndex::open_or_create(
            frame_path,
            512,
        )?);
        
        Ok(Self {
            timestamp_index,
            frame_index,
        })
    }
    
    /// Add version to indices.
    ///
    /// ## Arguments
    /// - `version_id`: Sequential ID from versions.log
    /// - `timestamp`: RFC3339 timestamp
    /// - `frame_id`: Frame ID (coordinated batch)
    ///
    /// ## Performance
    /// - O(log n) + WAL write
    /// - <2ms for 100k versions
    pub fn insert(
        &mut self,
        version_id: VersionId,
        timestamp: Timestamp,
        frame_id: FrameId,
    ) -> ReedResult<()> {
        // Insert into timestamp index
        let mut ts_versions = self.timestamp_index.get(&timestamp)?
            .unwrap_or_default();
        ts_versions.push(version_id);
        self.timestamp_index.insert(timestamp, ts_versions)?;
        
        // Insert into frame index
        let mut frame_versions = self.frame_index.get(&frame_id)?
            .unwrap_or_default();
        frame_versions.push(version_id);
        self.frame_index.insert(frame_id, frame_versions)?;
        
        Ok(())
    }
    
    /// Query versions by timestamp range (inclusive).
    ///
    /// ## Arguments
    /// - `start`: Start timestamp (e.g., "2025-10-28T08:00:00.000Z")
    /// - `end`: End timestamp (e.g., "2025-10-28T09:00:00.000Z")
    ///
    /// ## Returns
    /// - Sorted list of version IDs in range
    ///
    /// ## Performance
    /// - O(log n + k) where k = result size
    /// - <100ms for 10k versions in range
    pub fn query_timestamp_range(
        &self,
        start: &Timestamp,
        end: &Timestamp,
    ) -> ReedResult<Vec<VersionId>> {
        let results = self.timestamp_index.range(start, end)?;
        
        // Flatten and deduplicate
        let mut version_ids: Vec<VersionId> = results.into_iter()
            .flat_map(|(_, ids)| ids)
            .collect();
        
        version_ids.sort_unstable();
        version_ids.dedup();
        
        Ok(version_ids)
    }
    
    /// Query versions by frame ID.
    ///
    /// ## Arguments
    /// - `frame_id`: Frame ID (e.g., "F042")
    ///
    /// ## Returns
    /// - List of version IDs in frame
    ///
    /// ## Performance
    /// - O(log n) point lookup
    /// - <1ms for 100k frames
    pub fn query_frame(
        &self,
        frame_id: &FrameId,
    ) -> ReedResult<Vec<VersionId>> {
        Ok(self.frame_index.get(frame_id)?
            .unwrap_or_default())
    }
    
    /// Get all unique timestamps (for snapshot browsing).
    ///
    /// ## Returns
    /// - Sorted list of all timestamps in index
    ///
    /// ## Performance
    /// - O(n) where n = unique timestamps
    /// - ~1s for 100k unique timestamps
    pub fn get_all_timestamps(&self) -> ReedResult<Vec<Timestamp>> {
        let mut timestamps: Vec<Timestamp> = self.timestamp_index.iter()
            .map(|(ts, _)| ts)
            .collect();
        
        timestamps.sort();
        
        Ok(timestamps)
    }
    
    /// Get metadata about indices.
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            timestamp_memory: self.timestamp_index.memory_usage(),
            timestamp_disk: self.timestamp_index.disk_usage(),
            frame_memory: self.frame_index.memory_usage(),
            frame_disk: self.frame_index.disk_usage(),
        }
    }
}

/// Index statistics.
#[derive(Debug)]
pub struct IndexStats {
    pub timestamp_memory: usize,
    pub timestamp_disk: usize,
    pub frame_memory: usize,
    pub frame_disk: usize,
}
```

**Tests:** `reedbase/src/versioning/index_test.rs` (20 tests)

---

### 2. `reedbase/src/versioning/recovery.rs` (Modify)

**Purpose:** Integrate indices into point-in-time recovery

**Add to existing recovery module:**

```rust
// reedbase/src/versioning/recovery.rs (modifications)

use crate::versioning::index::VersionIndices;

/// Point-in-time recovery with index optimization.
pub struct OptimizedRecovery {
    indices: VersionIndices,
    log_path: PathBuf,
}

impl OptimizedRecovery {
    /// Create recovery engine with indices.
    pub fn new(
        indices: VersionIndices,
        log_path: PathBuf,
    ) -> Self {
        Self { indices, log_path }
    }
    
    /// Recover state at specific timestamp.
    ///
    /// ## Algorithm (Optimized)
    /// 1. Query timestamp index: versions <= target_time
    /// 2. Fetch versions from log (by ID)
    /// 3. Apply deltas in order
    ///
    /// ## Performance
    /// - Old: O(n) scan through all versions (~10s for 10k versions)
    /// - New: O(log n + k) index lookup (~100ms for 10k versions)
    /// - **100x speedup**
    pub fn recover_at_time(
        &self,
        target_time: &Timestamp,
    ) -> ReedResult<HashMap<String, String>> {
        // 1. Query index for versions before target_time
        let epoch = "1970-01-01T00:00:00.000Z".to_string();
        let version_ids = self.indices.query_timestamp_range(&epoch, target_time)?;
        
        // 2. Fetch versions from log (sequential read, ID-based)
        let versions = self.fetch_versions_by_id(&version_ids)?;
        
        // 3. Apply deltas (same as before)
        let mut state = HashMap::new();
        for version in versions {
            let value = apply_delta(&version.delta_bytes)?;
            state.insert(version.key.clone(), value);
        }
        
        Ok(state)
    }
    
    /// Recover entire frame (coordinated batch).
    ///
    /// ## Use Case
    /// - Rollback: "Undo frame F042"
    /// - Audit: "What changed in this transaction?"
    ///
    /// ## Performance
    /// - O(log n) point lookup
    /// - <100ms for 200-version frame
    pub fn recover_frame(
        &self,
        frame_id: &FrameId,
    ) -> ReedResult<Vec<Version>> {
        // 1. Query frame index
        let version_ids = self.indices.query_frame(frame_id)?;
        
        // 2. Fetch versions
        self.fetch_versions_by_id(&version_ids)
    }
    
    fn fetch_versions_by_id(
        &self,
        version_ids: &[VersionId],
    ) -> ReedResult<Vec<Version>> {
        // Read log file
        let content = fs::read_to_string(&self.log_path).map_err(|e| {
            ReedError::VersionLogRead {
                path: self.log_path.clone(),
                source: e,
            }
        })?;
        
        // Parse CSV
        let all_versions = parse_version_log(&content)?;
        
        // Filter by ID (version_id is 1-based, vec index is 0-based)
        let versions: Vec<Version> = version_ids.iter()
            .filter_map(|&id| all_versions.get(id - 1).cloned())
            .collect();
        
        Ok(versions)
    }
}
```

**Benchmark Comparison:**

```rust
// reedbase/benches/recovery.rs (new)
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_recovery(c: &mut Criterion) {
    // Setup: 10k versions over 1 hour
    let (indices, log_path) = setup_test_data(10_000);
    
    let mut group = c.benchmark_group("point_in_time_recovery");
    
    // Baseline: Linear scan (REED-19-03)
    group.bench_function("linear_scan", |b| {
        b.iter(|| {
            let target = "2025-10-28T08:30:00.000Z";
            black_box(recovery_linear_scan(&log_path, target))
        });
    });
    
    // Optimized: Index lookup (REED-19-23)
    group.bench_function("index_lookup", |b| {
        let recovery = OptimizedRecovery::new(indices.clone(), log_path.clone());
        b.iter(|| {
            let target = "2025-10-28T08:30:00.000Z";
            black_box(recovery.recover_at_time(target))
        });
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_recovery);
criterion_main!(benches);
```

**Expected Results:**
- Linear scan: 9.8s ± 0.5s
- Index lookup: 95ms ± 10ms
- **Speedup: 103x**

---

### 3. `reedbase/src/versioning/mod.rs` (Modify)

**Purpose:** Export new index module

```rust
// reedbase/src/versioning/mod.rs (add)
pub mod index;
pub mod recovery;  // Modified

pub use index::{VersionIndices, IndexStats, Timestamp, FrameId, VersionId};
pub use recovery::OptimizedRecovery;
```

---

### 4. CLI Integration: `reed version:query`

**Purpose:** User-facing command for timestamp queries

```rust
// src/reedcms/cli/version.rs (new subcommand)

/// Query versions by timestamp or frame.
#[derive(Parser)]
pub struct VersionQueryArgs {
    /// Query type
    #[clap(subcommand)]
    query_type: QueryType,
}

#[derive(Parser)]
pub enum QueryType {
    /// Query by timestamp range
    TimeRange {
        /// Start timestamp (RFC3339)
        #[clap(long)]
        start: String,
        
        /// End timestamp (RFC3339)
        #[clap(long)]
        end: String,
    },
    
    /// Query by frame ID
    Frame {
        /// Frame ID (e.g., F042)
        #[clap(long)]
        id: String,
    },
    
    /// List all snapshots
    Snapshots,
}

pub fn handle_version_query(args: VersionQueryArgs) -> ReedResult<()> {
    let indices = VersionIndices::open_or_create(
        ".reed/indices/versions_timestamp.btree",
        ".reed/indices/versions_frame.btree",
    )?;
    
    match args.query_type {
        QueryType::TimeRange { start, end } => {
            let version_ids = indices.query_timestamp_range(&start, &end)?;
            
            println!("Found {} versions between {} and {}", 
                version_ids.len(), start, end);
            
            for id in version_ids {
                println!("  Version {}", id);
            }
        }
        
        QueryType::Frame { id } => {
            let version_ids = indices.query_frame(&id)?;
            
            println!("Frame {} contains {} versions:", id, version_ids.len());
            
            for version_id in version_ids {
                println!("  Version {}", version_id);
            }
        }
        
        QueryType::Snapshots => {
            let timestamps = indices.get_all_timestamps()?;
            
            println!("Available snapshots ({} total):", timestamps.len());
            
            for (i, ts) in timestamps.iter().enumerate().take(20) {
                println!("  [{}] {}", i + 1, ts);
            }
            
            if timestamps.len() > 20 {
                println!("  ... and {} more", timestamps.len() - 20);
            }
        }
    }
    
    Ok(())
}
```

**Usage Examples:**

```bash
# Query versions in time range
reed version:query time-range \
  --start "2025-10-28T08:00:00.000Z" \
  --end "2025-10-28T09:00:00.000Z"

# Output:
# Found 156 versions between 2025-10-28T08:00:00.000Z and 2025-10-28T09:00:00.000Z
#   Version 1
#   Version 2
#   ...

# Query versions in frame
reed version:query frame --id F042

# Output:
# Frame F042 contains 23 versions:
#   Version 42
#   Version 43
#   ...

# List all snapshots
reed version:query snapshots

# Output:
# Available snapshots (1,247 total):
#   [1] 2025-10-28T08:15:23.001Z
#   [2] 2025-10-28T08:16:45.500Z
#   ...
```

---

## Index Maintenance

### Rebuild Index from Log

```rust
// reedbase/src/versioning/rebuild.rs (new)

/// Rebuild indices from version log (for corruption recovery).
pub fn rebuild_indices(
    log_path: &Path,
    timestamp_index_path: &Path,
    frame_index_path: &Path,
) -> ReedResult<()> {
    println!("Rebuilding version indices from log...");
    
    // 1. Delete old index files
    let _ = fs::remove_file(timestamp_index_path);
    let _ = fs::remove_file(frame_index_path);
    
    // 2. Create new indices
    let mut indices = VersionIndices::open_or_create(
        timestamp_index_path,
        frame_index_path,
    )?;
    
    // 3. Read all versions from log
    let content = fs::read_to_string(log_path).map_err(|e| {
        ReedError::VersionLogRead {
            path: log_path.to_path_buf(),
            source: e,
        }
    })?;
    
    let versions = parse_version_log(&content)?;
    
    println!("Indexing {} versions...", versions.len());
    
    // 4. Insert all versions into indices
    for (i, version) in versions.iter().enumerate() {
        indices.insert(
            version.version_id,
            version.timestamp.clone(),
            version.frame_id.clone(),
        )?;
        
        if (i + 1) % 1000 == 0 {
            println!("  Progress: {}/{} ({:.1}%)", 
                i + 1, 
                versions.len(),
                (i + 1) as f64 / versions.len() as f64 * 100.0
            );
        }
    }
    
    println!("✓ Rebuild complete");
    
    // 5. Show statistics
    let stats = indices.stats();
    println!("  Timestamp index: {} (disk: {})", 
        format_bytes(stats.timestamp_memory),
        format_bytes(stats.timestamp_disk)
    );
    println!("  Frame index: {} (disk: {})",
        format_bytes(stats.frame_memory),
        format_bytes(stats.frame_disk)
    );
    
    Ok(())
}
```

**CLI Command:**

```bash
reed version:rebuild-indices

# Output:
# Rebuilding version indices from log...
# Indexing 10,247 versions...
#   Progress: 1000/10247 (9.8%)
#   Progress: 2000/10247 (19.5%)
#   ...
# ✓ Rebuild complete
#   Timestamp index: 8.2 MB (disk: 12.4 MB)
#   Frame index: 2.1 MB (disk: 3.8 MB)
```

---

## Performance Characteristics

### Memory Usage

| Versions | Timestamp Index | Frame Index | Total |
|---------|----------------|-------------|-------|
| 10k | 2 MB | 0.5 MB | 2.5 MB |
| 100k | 18 MB | 4 MB | 22 MB |
| 1M | 180 MB | 40 MB | 220 MB |

**Overhead:** ~220 bytes per version (acceptable for audit trail)

### Disk Usage

| Versions | Log File | Timestamp Index | Frame Index | Total |
|---------|---------|----------------|-------------|-------|
| 10k | 1.2 MB | 3 MB | 0.8 MB | 5 MB |
| 100k | 12 MB | 28 MB | 7 MB | 47 MB |
| 1M | 120 MB | 280 MB | 70 MB | 470 MB |

**Overhead:** 2.9x (3.9x total vs log-only)

### Query Performance

| Operation | Without Index | With Index | Speedup |
|----------|--------------|-----------|---------|
| Point-in-time recovery (10k versions) | 9.8s | 95ms | 103x |
| Frame lookup (200 versions) | 5.2s | 105ms | 49x |
| Snapshot list (1k snapshots) | 8.5s | 120ms | 71x |

---

## Acceptance Criteria

### Index Operations
- [ ] `VersionIndices::open_or_create` completes in <100ms
- [ ] `insert()` completes in <2ms per version
- [ ] `query_timestamp_range()` completes in <100ms for 10k results
- [ ] `query_frame()` completes in <1ms

### Recovery Performance
- [ ] Point-in-time recovery 100x faster (10s → 100ms for 10k versions)
- [ ] Frame lookup 50x faster (5s → 100ms for 200 versions)
- [ ] Snapshot browsing 70x faster (8s → 120ms for 1k snapshots)

### Maintenance
- [ ] Index rebuild completes in <5s for 100k versions
- [ ] Corrupted indices auto-rebuild on first query
- [ ] CLI commands for manual rebuild and stats

### Cross-Cutting
- [ ] License headers in all new files
- [ ] Separate test files with 100% coverage
- [ ] Documentation includes performance benchmarks

---

## Documentation Updates

### README.md

```markdown
## Version History Indices

ReedBase maintains secondary indices on version history for fast queries:

### Timestamp Index
- **Purpose**: Point-in-time recovery, audit queries
- **Performance**: 100x faster than linear scan
- **Storage**: `.reed/indices/versions_timestamp.btree`

```bash
# Recover state at specific time (100ms vs 10s)
reed version:query time-range \
  --start "2025-10-28T08:00:00Z" \
  --end "2025-10-28T09:00:00Z"
```

### Frame Index
- **Purpose**: Batch operation lookups, rollback
- **Performance**: 50x faster than linear scan
- **Storage**: `.reed/indices/versions_frame.btree`

```bash
# Find all versions in coordinated frame (100ms vs 5s)
reed version:query frame --id F042
```

### Index Maintenance

Indices are updated automatically on every write. For manual rebuild:

```bash
# Rebuild from version log (useful after corruption)
reed version:rebuild-indices
```

---

## Error Handling

### New Error Variants

```rust
// reedbase/src/error.rs (add)
pub enum ReedError {
    // ... existing
    
    /// Version log read failed.
    VersionLogRead {
        path: PathBuf,
        source: std::io::Error,
    },
    
    /// Version index corrupted.
    VersionIndexCorrupted {
        index_type: String,  // "timestamp" | "frame"
        path: PathBuf,
    },
}
```

---

## Timeline

### Day 1: Implementation
- [ ] Create `versioning/index.rs` with dual B+-Tree indices
- [ ] Modify `versioning/recovery.rs` to use indices
- [ ] Write 20 unit tests
- [ ] Write 8 integration tests
- [ ] Add CLI commands (`version:query`, `version:rebuild-indices`)
- [ ] Benchmark and validate 100x speedup

---

## Related Tickets

- **REED-19-03**: Binary Delta Versioning (version log format)
- **REED-19-04**: Crash Recovery (uses version log)
- **REED-19-20**: B+-Tree Index Engine (underlying data structure)
- **REED-19-21**: Index Migration (trait abstraction)

---

## Notes

This ticket completes the "fast history" capability of ReedBase:
- **REED-19-03**: Binary deltas (95% space savings)
- **REED-19-04**: Crash recovery (CRC32 validation)
- **REED-19-23**: Timestamp indices (100x faster queries) ← **This ticket**

**Result:** Production-ready audit trail with instant time-travel queries.
