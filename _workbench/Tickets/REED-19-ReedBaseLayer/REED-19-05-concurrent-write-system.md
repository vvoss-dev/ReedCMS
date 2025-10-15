# REED-19-06: Concurrent Write System

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-19-06
- **Title**: Concurrent Write System
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-19-02 (Universal Table API), REED-19-03 (Binary Delta Versioning)
- **Estimated Time**: 1 week

## Objective

Implement concurrent write handling with file locking, write queue, and automatic merge for non-conflicting changes. Enable multiple processes to write simultaneously without data loss.

## Requirements

### Concurrency Model

```
Process A writes row 1 → Lock acquired → Write → Delta generated → Unlock
Process B writes row 2 → Queue (wait) → Lock acquired → Merge with A → Write → Unlock
```

### Write Queue

```
.reed/tables/{table_name}/
├── current.csv          # Active version
├── write.lock           # Advisory lock file
└── queue/               # Write queue directory
    ├── {uuid1}.pending  # Pending write (process A)
    └── {uuid2}.pending  # Pending write (process B)
```

### Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Acquire lock | < 10ms | File-based advisory lock |
| Queue write | < 5ms | Write to queue file |
| Process queue | < 50ms | Merge + write + delta |
| Lock timeout | 30s | Fail if lock held > 30s |
| Max queue size | 100 | Reject writes if queue full |

## Implementation Files

### Primary Implementation

**`reedbase/src/concurrent/lock.rs`**

One file = File locking only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! File locking for concurrent write coordination.
//!
//! Uses advisory file locks for cross-process synchronisation.

use std::path::Path;
use std::fs::{File, OpenOptions};
use std::time::{Duration, Instant};
use fs2::FileExt;
use crate::types::{ReedResult, ReedError};

/// Acquire exclusive lock on table.
///
/// ## Input
/// - `table_name`: Table name
/// - `timeout`: Maximum time to wait for lock
///
/// ## Output
/// - `ReedResult<TableLock>`: Lock handle (RAII - auto-releases on drop)
///
/// ## Performance
/// - < 10ms if lock available immediately
/// - Up to `timeout` if lock held by another process
///
/// ## Error Conditions
/// - LockTimeout: Could not acquire lock within timeout
/// - IoError: Cannot create lock file
///
/// ## Example Usage
/// ```rust
/// let lock = acquire_lock("users", Duration::from_secs(30))?;
/// // Lock held - perform write
/// // Lock automatically released when `lock` drops
/// ```
pub fn acquire_lock(table_name: &str, timeout: Duration) -> ReedResult<TableLock> {
    let lock_path = format!(".reed/tables/{}/write.lock", table_name);
    let lock_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&lock_path)
        .map_err(|e| ReedError::IoError {
            path: lock_path.clone(),
            source: e,
        })?;
    
    let start = Instant::now();
    
    loop {
        match lock_file.try_lock_exclusive() {
            Ok(()) => {
                return Ok(TableLock {
                    file: lock_file,
                    path: lock_path,
                });
            }
            Err(_) if start.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(_) => {
                return Err(ReedError::LockTimeout {
                    table: table_name.to_string(),
                    timeout_secs: timeout.as_secs(),
                });
            }
        }
    }
}

/// Table lock handle (RAII).
pub struct TableLock {
    file: File,
    path: String,
}

impl Drop for TableLock {
    /// Release lock on drop.
    ///
    /// ## Performance
    /// - < 1ms typical
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

/// Check if table is locked.
///
/// ## Input
/// - `table_name`: Table name
///
/// ## Output
/// - `ReedResult<bool>`: True if locked
///
/// ## Performance
/// - < 5ms typical
///
/// ## Error Conditions
/// - IoError: Cannot access lock file
///
/// ## Example Usage
/// ```rust
/// if is_locked("users")? {
///     println!("Table is currently locked");
/// }
/// ```
pub fn is_locked(table_name: &str) -> ReedResult<bool> {
    let lock_path = format!(".reed/tables/{}/write.lock", table_name);
    
    let lock_file = match OpenOptions::new()
        .read(true)
        .open(&lock_path)
    {
        Ok(f) => f,
        Err(_) => return Ok(false), // Lock file doesn't exist
    };
    
    match lock_file.try_lock_exclusive() {
        Ok(()) => {
            let _ = lock_file.unlock();
            Ok(false)
        }
        Err(_) => Ok(true),
    }
}

/// Wait for lock to be released.
///
/// ## Input
/// - `table_name`: Table name
/// - `timeout`: Maximum time to wait
///
/// ## Output
/// - `ReedResult<()>`: Ok when lock released
///
/// ## Performance
/// - Variable (depends on lock holder)
/// - Up to `timeout`
///
/// ## Error Conditions
/// - LockTimeout: Lock not released within timeout
///
/// ## Example Usage
/// ```rust
/// wait_for_unlock("users", Duration::from_secs(60))?;
/// // Lock now available
/// ```
pub fn wait_for_unlock(table_name: &str, timeout: Duration) -> ReedResult<()> {
    let start = Instant::now();
    
    while is_locked(table_name)? {
        if start.elapsed() >= timeout {
            return Err(ReedError::LockTimeout {
                table: table_name.to_string(),
                timeout_secs: timeout.as_secs(),
            });
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    
    Ok(())
}
```

**`reedbase/src/concurrent/queue.rs`**

One file = Write queue management only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Write queue for concurrent write coordination.
//!
//! Queues pending writes when table is locked.

use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::types::{ReedResult, ReedError};

/// Queue a write operation.
///
/// ## Input
/// - `table_name`: Table name
/// - `operation`: Write operation to queue
///
/// ## Output
/// - `ReedResult<String>`: Queue ID (UUID)
///
/// ## Performance
/// - < 5ms typical (write small file)
///
/// ## Error Conditions
/// - QueueFull: Queue has reached maximum size (100 pending)
/// - IoError: Cannot write queue file
///
/// ## Example Usage
/// ```rust
/// let write = PendingWrite {
///     rows: vec![row1, row2],
///     timestamp: 1736860900,
/// };
/// let queue_id = queue_write("users", write)?;
/// println!("Queued with ID: {}", queue_id);
/// ```
pub fn queue_write(table_name: &str, operation: PendingWrite) -> ReedResult<String> {
    let queue_dir = get_queue_dir(table_name);
    fs::create_dir_all(&queue_dir)
        .map_err(|e| ReedError::IoError {
            path: queue_dir.to_string_lossy().to_string(),
            source: e,
        })?;
    
    // Check queue size
    let queue_size = count_pending(table_name)?;
    if queue_size >= 100 {
        return Err(ReedError::QueueFull {
            table: table_name.to_string(),
            size: queue_size,
        });
    }
    
    let queue_id = Uuid::new_v4().to_string();
    let queue_path = queue_dir.join(format!("{}.pending", queue_id));
    
    let json = serde_json::to_string(&operation)
        .map_err(|e| ReedError::SerializationError {
            reason: format!("Failed to serialize write: {}", e),
        })?;
    
    fs::write(&queue_path, json)
        .map_err(|e| ReedError::IoError {
            path: queue_path.to_string_lossy().to_string(),
            source: e,
        })?;
    
    Ok(queue_id)
}

/// Get next pending write from queue.
///
/// ## Input
/// - `table_name`: Table name
///
/// ## Output
/// - `ReedResult<Option<(String, PendingWrite)>>`: (queue_id, write) or None if empty
///
/// ## Performance
/// - < 10ms typical
///
/// ## Error Conditions
/// - IoError: Cannot read queue directory
/// - DeserializationError: Corrupted queue file
///
/// ## Example Usage
/// ```rust
/// while let Some((id, write)) = get_next_pending("users")? {
///     process_write(write)?;
///     remove_from_queue("users", &id)?;
/// }
/// ```
pub fn get_next_pending(table_name: &str) -> ReedResult<Option<(String, PendingWrite)>> {
    let queue_dir = get_queue_dir(table_name);
    
    if !queue_dir.exists() {
        return Ok(None);
    }
    
    let mut entries: Vec<_> = fs::read_dir(&queue_dir)
        .map_err(|e| ReedError::IoError {
            path: queue_dir.to_string_lossy().to_string(),
            source: e,
        })?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pending"))
        .collect();
    
    if entries.is_empty() {
        return Ok(None);
    }
    
    // Sort by creation time (oldest first)
    entries.sort_by_key(|e| e.metadata().ok().and_then(|m| m.created().ok()));
    
    let entry = &entries[0];
    let queue_id = entry.path()
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ReedError::InvalidQueueFile {
            path: entry.path().to_string_lossy().to_string(),
        })?
        .to_string();
    
    let json = fs::read_to_string(entry.path())
        .map_err(|e| ReedError::IoError {
            path: entry.path().to_string_lossy().to_string(),
            source: e,
        })?;
    
    let write: PendingWrite = serde_json::from_str(&json)
        .map_err(|e| ReedError::DeserializationError {
            reason: format!("Failed to deserialize write: {}", e),
        })?;
    
    Ok(Some((queue_id, write)))
}

/// Remove write from queue.
///
/// ## Input
/// - `table_name`: Table name
/// - `queue_id`: Queue ID (UUID)
///
/// ## Output
/// - `ReedResult<()>`: Success or error
///
/// ## Performance
/// - < 5ms typical
///
/// ## Error Conditions
/// - IoError: Cannot delete queue file
///
/// ## Example Usage
/// ```rust
/// remove_from_queue("users", "550e8400-e29b-41d4-a716-446655440000")?;
/// ```
pub fn remove_from_queue(table_name: &str, queue_id: &str) -> ReedResult<()> {
    let queue_path = get_queue_dir(table_name).join(format!("{}.pending", queue_id));
    
    fs::remove_file(&queue_path)
        .map_err(|e| ReedError::IoError {
            path: queue_path.to_string_lossy().to_string(),
            source: e,
        })?;
    
    Ok(())
}

/// Count pending writes in queue.
///
/// ## Input
/// - `table_name`: Table name
///
/// ## Output
/// - `ReedResult<usize>`: Number of pending writes
///
/// ## Performance
/// - < 5ms typical
///
/// ## Error Conditions
/// - IoError: Cannot read queue directory
///
/// ## Example Usage
/// ```rust
/// let pending = count_pending("users")?;
/// println!("{} writes pending", pending);
/// ```
pub fn count_pending(table_name: &str) -> ReedResult<usize> {
    let queue_dir = get_queue_dir(table_name);
    
    if !queue_dir.exists() {
        return Ok(0);
    }
    
    let count = fs::read_dir(&queue_dir)
        .map_err(|e| ReedError::IoError {
            path: queue_dir.to_string_lossy().to_string(),
            source: e,
        })?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pending"))
        .count();
    
    Ok(count)
}

/// Get queue directory path.
///
/// ## Input
/// - `table_name`: Table name
///
/// ## Output
/// - `PathBuf`: Queue directory path
///
/// ## Performance
/// - O(1) operation
/// - < 1μs
fn get_queue_dir(table_name: &str) -> PathBuf {
    PathBuf::from(format!(".reed/tables/{}/queue", table_name))
}

/// Pending write operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingWrite {
    pub rows: Vec<CsvRow>,
    pub timestamp: u64,
    pub operation: WriteOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WriteOperation {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvRow {
    pub key: String,
    pub values: Vec<String>,
}
```

**`reedbase/src/types.rs`** (additions)

```rust
/// Additional ReedBase errors.
#[derive(Error, Debug)]
pub enum ReedError {
    // ... (existing errors)
    
    #[error("Lock timeout for table '{table}' after {timeout_secs}s")]
    LockTimeout {
        table: String,
        timeout_secs: u64,
    },
    
    #[error("Queue full for table '{table}' ({size} pending)")]
    QueueFull {
        table: String,
        size: usize,
    },
    
    #[error("Invalid queue file: {path}")]
    InvalidQueueFile {
        path: String,
    },
    
    #[error("Serialization error: {reason}")]
    SerializationError {
        reason: String,
    },
    
    #[error("Deserialization error: {reason}")]
    DeserializationError {
        reason: String,
    },
}
```

### Test Files

**`reedbase/src/concurrent/lock.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;
    
    #[test]
    fn test_acquire_lock_success() {
        let _temp_dir = TempDir::new().unwrap();
        
        let lock = acquire_lock("test_table", Duration::from_secs(5)).unwrap();
        assert!(is_locked("test_table").unwrap());
        
        drop(lock);
        assert!(!is_locked("test_table").unwrap());
    }
    
    #[test]
    fn test_acquire_lock_timeout() {
        let _temp_dir = TempDir::new().unwrap();
        
        let _lock1 = acquire_lock("test_table", Duration::from_secs(5)).unwrap();
        
        let result = acquire_lock("test_table", Duration::from_millis(500));
        assert!(matches!(result, Err(ReedError::LockTimeout { .. })));
    }
    
    #[test]
    fn test_is_locked() {
        let _temp_dir = TempDir::new().unwrap();
        
        assert!(!is_locked("test_table").unwrap());
        
        let _lock = acquire_lock("test_table", Duration::from_secs(5)).unwrap();
        assert!(is_locked("test_table").unwrap());
    }
    
    #[test]
    fn test_wait_for_unlock() {
        let _temp_dir = TempDir::new().unwrap();
        
        let lock = acquire_lock("test_table", Duration::from_secs(5)).unwrap();
        
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            drop(lock);
        });
        
        wait_for_unlock("test_table", Duration::from_secs(5)).unwrap();
        assert!(!is_locked("test_table").unwrap());
    }
    
    #[test]
    fn test_wait_for_unlock_timeout() {
        let _temp_dir = TempDir::new().unwrap();
        
        let _lock = acquire_lock("test_table", Duration::from_secs(5)).unwrap();
        
        let result = wait_for_unlock("test_table", Duration::from_millis(200));
        assert!(matches!(result, Err(ReedError::LockTimeout { .. })));
    }
}
```

**`reedbase/src/concurrent/queue.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_queue_and_get_write() {
        let _temp_dir = TempDir::new().unwrap();
        
        let write = PendingWrite {
            rows: vec![CsvRow {
                key: "1".to_string(),
                values: vec!["Alice".to_string()],
            }],
            timestamp: 1736860900,
            operation: WriteOperation::Insert,
        };
        
        let queue_id = queue_write("test_table", write.clone()).unwrap();
        
        let (id, retrieved) = get_next_pending("test_table").unwrap().unwrap();
        assert_eq!(id, queue_id);
        assert_eq!(retrieved.timestamp, write.timestamp);
    }
    
    #[test]
    fn test_remove_from_queue() {
        let _temp_dir = TempDir::new().unwrap();
        
        let write = PendingWrite {
            rows: vec![],
            timestamp: 1736860900,
            operation: WriteOperation::Insert,
        };
        
        let queue_id = queue_write("test_table", write).unwrap();
        assert_eq!(count_pending("test_table").unwrap(), 1);
        
        remove_from_queue("test_table", &queue_id).unwrap();
        assert_eq!(count_pending("test_table").unwrap(), 0);
    }
    
    #[test]
    fn test_count_pending() {
        let _temp_dir = TempDir::new().unwrap();
        
        assert_eq!(count_pending("test_table").unwrap(), 0);
        
        let write = PendingWrite {
            rows: vec![],
            timestamp: 1736860900,
            operation: WriteOperation::Insert,
        };
        
        queue_write("test_table", write.clone()).unwrap();
        assert_eq!(count_pending("test_table").unwrap(), 1);
        
        queue_write("test_table", write.clone()).unwrap();
        assert_eq!(count_pending("test_table").unwrap(), 2);
    }
    
    #[test]
    fn test_queue_full() {
        let _temp_dir = TempDir::new().unwrap();
        
        let write = PendingWrite {
            rows: vec![],
            timestamp: 1736860900,
            operation: WriteOperation::Insert,
        };
        
        // Queue 100 writes (max)
        for _ in 0..100 {
            queue_write("test_table", write.clone()).unwrap();
        }
        
        // 101st write should fail
        let result = queue_write("test_table", write);
        assert!(matches!(result, Err(ReedError::QueueFull { .. })));
    }
    
    #[test]
    fn test_get_next_pending_fifo() {
        let _temp_dir = TempDir::new().unwrap();
        
        let write1 = PendingWrite {
            rows: vec![],
            timestamp: 1736860900,
            operation: WriteOperation::Insert,
        };
        
        let write2 = PendingWrite {
            rows: vec![],
            timestamp: 1736861000,
            operation: WriteOperation::Insert,
        };
        
        queue_write("test_table", write1).unwrap();
        std::thread::sleep(Duration::from_millis(10)); // Ensure different creation times
        queue_write("test_table", write2).unwrap();
        
        let (_, first) = get_next_pending("test_table").unwrap().unwrap();
        assert_eq!(first.timestamp, 1736860900); // Older one first
    }
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Acquire lock (immediate) | < 10ms |
| Acquire lock (contended) | Up to timeout (30s default) |
| Queue write | < 5ms |
| Get next pending | < 10ms |
| Remove from queue | < 5ms |
| Count pending | < 5ms |
| Process queue (10 writes) | < 500ms |

## Error Conditions

- **LockTimeout**: Could not acquire lock within timeout period
- **QueueFull**: Queue has reached maximum size (100 pending writes)
- **InvalidQueueFile**: Queue file corrupted or malformed
- **IoError**: File system errors (read/write/delete)
- **SerializationError**: Cannot serialize pending write to JSON
- **DeserializationError**: Cannot deserialize pending write from JSON

## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| lock_acquire_latency | Histogram | ms | <10 | >100 | lock.rs:acquire() |
| lock_wait_time | Histogram | ms | <50 | >5000 | lock.rs:acquire() (contended) |
| queue_depth | Gauge | count | <10 | >50 | queue.rs:queue_write() |
| queue_add_latency | Histogram | ms | <5 | >20 | queue.rs:queue_write() |
| lock_timeouts | Counter | count | <1% | >5% | lock.rs:acquire() |
| queue_full_rejections | Counter | count | 0 | >0 | queue.rs:queue_write() |

### Alert Rules

**CRITICAL Alerts:**
- `queue_full_rejections > 0` for 1 minute → "Write queue full - system overloaded"
- `lock_timeouts > 5%` for 5 minutes → "High lock contention - investigate concurrent writes"

**WARNING Alerts:**
- `queue_depth > 50` for 5 minutes → "Write queue backing up - check processing speed"
- `lock_wait_time p99 > 5s` for 5 minutes → "Long lock waits - possible deadlock or slow writes"

### Implementation

```rust
use crate::reedbase::metrics::global as metrics;
use std::time::Instant;

pub fn acquire(&self, timeout: Duration) -> ReedResult<LockGuard> {
    let start = Instant::now();
    let guard = self.acquire_inner(timeout)?;
    
    let elapsed = start.elapsed();
    
    metrics().record(Metric {
        name: "lock_acquire_latency".to_string(),
        value: elapsed.as_millis() as f64,
        unit: MetricUnit::Milliseconds,
        tags: hashmap!{ "table" => &self.table_name },
    });
    
    if elapsed > Duration::from_millis(10) {
        metrics().record(Metric {
            name: "lock_wait_time".to_string(),
            value: elapsed.as_millis() as f64,
            unit: MetricUnit::Milliseconds,
            tags: hashmap!{ "table" => &self.table_name, "contended" => "true" },
        });
    }
    
    Ok(guard)
}

pub fn queue_write(&self, write: PendingWrite) -> ReedResult<()> {
    let start = Instant::now();
    let result = self.queue_write_inner(write)?;
    
    let depth = self.get_queue_depth()?;
    
    metrics().record(Metric {
        name: "queue_depth".to_string(),
        value: depth as f64,
        unit: MetricUnit::Count,
        tags: hashmap!{ "table" => &self.table_name },
    });
    
    metrics().record(Metric {
        name: "queue_add_latency".to_string(),
        value: start.elapsed().as_millis() as f64,
        unit: MetricUnit::Milliseconds,
        tags: hashmap!{ "table" => &self.table_name },
    });
    
    Ok(result)
}
```

### Collection Strategy

- **Sampling**: All operations
- **Aggregation**: 1-minute rolling window
- **Storage**: `.reedbase/metrics/concurrent.csv`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**lock_acquire_latency**: Write performance baseline
- Uncontended locks should be <10ms
- Long acquisition indicates high concurrency or slow operations
- Directly impacts write throughput

**queue_depth**: System load indicator
- Low depth (<10) = healthy system
- Growing depth = writes arriving faster than processing
- Sustained high depth requires scaling or optimization

**lock_timeouts**: Concurrency health
- Timeouts indicate deadlock or extremely slow operations
- Should be rare (<1% of attempts)
- Frequent timeouts require investigation

**queue_full_rejections**: Critical capacity issue
- Queue full = writes being dropped
- Zero tolerance metric - must never happen in production
- Triggers immediate capacity increase or rate limiting

## CLI Commands

```bash
# Lock status (internal use)
reed debug:lock users
# Output: Locked (held by PID 12345)

# Queue status
reed debug:queue users
# Output: 5 pending writes in queue

# Force unlock (dangerous - only use if process crashed)
reed admin:unlock users --force
# Warning: This may cause data corruption if lock holder is still running
```

## Acceptance Criteria

- [ ] Acquire exclusive lock on table
- [ ] Release lock on drop (RAII pattern)
- [ ] Lock timeout if held too long
- [ ] Check if table is locked
- [ ] Wait for lock to be released
- [ ] Queue write when lock held
- [ ] Get next pending write (FIFO order)
- [ ] Remove write from queue
- [ ] Count pending writes
- [ ] Reject writes if queue full (100 max)
- [ ] Serialize/deserialize pending writes
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation (Input/Output/Performance/Error Conditions/Example Usage)
- [ ] No Swiss Army knife functions (each function = one job)
- [ ] Separate test files as `lock.test.rs` and `queue.test.rs`

## Dependencies

**Requires**: 
- REED-19-02 (Universal Table API - integration point)
- REED-19-03 (Binary Delta Versioning - for delta generation during merge)

**Blocks**: 
- REED-19-06 (Row-Level CSV Merge - uses locking)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-19-00: Layer Overview

## Notes

**Concurrency Strategy:**

1. **Process A** writes → Acquires lock → Writes → Generates delta → Releases lock
2. **Process B** writes → Lock held → Queues write → Waits
3. **Process A** finishes → Releases lock
4. **Process B** acquires lock → Reads queue → Merges with current → Writes → Releases lock

**Auto-merge Success Rate:**
- **90%+** of concurrent writes auto-merge successfully (different rows)
- **10%** require conflict resolution (same row modified)

**Trade-offs:**
- **Pro**: Zero data loss from concurrent writes
- **Pro**: High throughput (queue + merge faster than blocking)
- **Pro**: Cross-process synchronisation (advisory locks work across processes)
- **Con**: Lock contention under heavy load (mitigated by fast operations)
- **Con**: Queue grows if lock held too long (limited to 100 pending)

**Future Enhancements:**
- Per-row locking (finer-grained concurrency)
- Optimistic locking (version numbers)
- Distributed locking (for multi-server)
