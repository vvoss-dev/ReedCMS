# REED-19-03A: Backup & Point-in-Time Recovery

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
- **ID**: REED-19-03A
- **Title**: Backup & Point-in-Time Recovery
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Low
- **Dependencies**: REED-19-02 (Table API), REED-19-03 (Delta Versioning), REED-19-04 (Version Log)
- **Estimated Time**: 2 days

## Objective

Add simple backup creation and point-in-time recovery to ReedBase. Uses existing version.log timestamps for consistency without complex coordination.

## Requirements

### Core Functions

1. **`reed backup:create`** - Create full installation backup
2. **`reed backup:list`** - List available backups
3. **`reed restore:point-in-time <timestamp>`** - Restore all tables to consistent state

### Design Philosophy

**KISS Approach:**
- ✅ Use standard `tar` + `xz` (no reinventing compression)
- ✅ Use existing `version.log` timestamps (no new snapshot infrastructure)
- ✅ Let admins handle remote backup (rsync), scheduling (cron), monitoring

**What We DON'T Build:**
- ❌ Custom compression algorithms
- ❌ Remote backup replication (admins have rsync)
- ❌ Backup scheduling (admins have cron)
- ❌ Incremental backups (tar handles this well enough)

### File Structure

```
.reed/backups/
├── 1736860800.tar.gz        # Full backup (tar + xz)
├── 1736863400.tar.gz        # Another backup
└── 1736866000.tar.gz        # Latest backup
```

### Point-in-Time Consistency Algorithm

**The KISS Solution:**

```rust
// For target timestamp 1736860800 (e.g., "14:00")
for each table {
    // Read version.log entries
    let versions = read_version_log(table)?;
    
    // Find last entry <= target
    let best_match = versions.iter()
        .filter(|v| v.timestamp <= target)
        .max_by_key(|v| v.timestamp);
    
    match best_match {
        Some(v) => restore_table_to(table, v.timestamp),
        None => skip_table(table),  // Didn't exist yet
    }
}
```

**Example:**
```
Target: 14:00 (1736860800)

users.csv versions:
- 13:45 (1736855100) ✓ <= 14:00 → USE THIS
- 14:05 (1736858700) ✗ > 14:00

orders.csv versions:
- 13:58 (1736856480) ✓ <= 14:00 → USE THIS
- 14:12 (1736859120) ✗ > 14:00

products.csv versions:
- 14:01 (1736857260) ✗ > 14:00
→ Table didn't exist yet, skip

Result: Consistent state "as of 14:00"
```

**Why This Works:**
- Each table's version.log has timestamps
- No coordination needed between tables
- Naturally consistent: "all tables as they were at or before timestamp X"

## Implementation Files

### Primary Implementation

**`src/reedcms/reedbase/backup/mod.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Backup and point-in-time recovery for ReedBase.

mod create;
mod list;
mod restore;
mod types;

pub use create::create_backup;
pub use list::list_backups;
pub use restore::restore_point_in_time;
pub use types::{BackupInfo, RestoreReport};
```

**`src/reedcms/reedbase/backup/create.rs`**

One file = Backup creation only.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Backup creation using tar + xz.

use std::path::Path;
use std::process::Command;
use crate::types::{ReedResult, ReedError};
use super::types::BackupInfo;

/// Create full backup of .reed/ directory.
///
/// ## Input
/// - None (backs up current .reed/ directory)
///
/// ## Output
/// - `ReedResult<BackupInfo>`: Backup metadata
///
/// ## Process
/// 1. Generate timestamp: SystemTime::now()
/// 2. Create tar.gz: `tar czf backups/{timestamp}.tar.gz .reed/`
/// 3. Return backup info
///
/// ## Performance
/// - Depends on installation size
/// - Typical: < 30s for 100MB installation
///
/// ## Error Conditions
/// - IoError: Cannot create backup directory
/// - CommandFailed: tar command failed
/// - InsufficientSpace: Not enough disk space
///
/// ## Example Usage
/// ```rust
/// let backup = create_backup()?;
/// println!("Backup created: {}", backup.path.display());
/// ```
pub fn create_backup() -> ReedResult<BackupInfo> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Generate timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Ensure backup directory exists
    let backup_dir = Path::new(".reed/backups");
    std::fs::create_dir_all(backup_dir)?;
    
    // Backup path
    let backup_path = backup_dir.join(format!("{}.tar.gz", timestamp));
    
    // Execute tar command
    let output = Command::new("tar")
        .arg("czf")
        .arg(&backup_path)
        .arg(".reed/")
        .output()
        .map_err(|e| ReedError::CommandFailed {
            command: "tar".to_string(),
            error: e.to_string(),
        })?;
    
    if !output.status.success() {
        return Err(ReedError::CommandFailed {
            command: "tar".to_string(),
            error: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    
    // Get file size
    let size = std::fs::metadata(&backup_path)?.len();
    
    Ok(BackupInfo {
        timestamp,
        path: backup_path,
        size_bytes: size,
        size_mb: size as f64 / 1_048_576.0,
    })
}
```

**`src/reedcms/reedbase/backup/list.rs`**

One file = List backups only.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! List available backups.

use std::path::Path;
use std::fs;
use crate::types::ReedResult;
use super::types::BackupInfo;

/// List all available backups.
///
/// ## Output
/// - `ReedResult<Vec<BackupInfo>>`: List of backups (newest first)
///
/// ## Performance
/// - < 10ms typical (read directory + stat files)
///
/// ## Error Conditions
/// - IoError: Cannot read backup directory
///
/// ## Example Usage
/// ```rust
/// let backups = list_backups()?;
/// for backup in backups {
///     println!("{} - {} MB", backup.timestamp, backup.size_mb);
/// }
/// ```
pub fn list_backups() -> ReedResult<Vec<BackupInfo>> {
    let backup_dir = Path::new(".reed/backups");
    
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut backups = Vec::new();
    
    for entry in fs::read_dir(backup_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Only .tar.gz files
        if path.extension().and_then(|s| s.to_str()) != Some("gz") {
            continue;
        }
        
        // Parse timestamp from filename
        let filename = path.file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_suffix(".tar"));
        
        if let Some(ts_str) = filename {
            if let Ok(timestamp) = ts_str.parse::<i64>() {
                let size = entry.metadata()?.len();
                
                backups.push(BackupInfo {
                    timestamp,
                    path: path.clone(),
                    size_bytes: size,
                    size_mb: size as f64 / 1_048_576.0,
                });
            }
        }
    }
    
    // Sort by timestamp (newest first)
    backups.sort_by_key(|b| -b.timestamp);
    
    Ok(backups)
}
```

**`src/reedcms/reedbase/backup/restore.rs`**

One file = Point-in-time restore only.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Point-in-time recovery using version.log timestamps.

use std::path::Path;
use crate::types::{ReedResult, ReedError};
use crate::reedbase::tables::Table;
use super::types::RestoreReport;

/// Restore all tables to consistent point-in-time.
///
/// ## Input
/// - `target_timestamp`: Unix timestamp (seconds) to restore to
///
/// ## Output
/// - `ReedResult<RestoreReport>`: What was restored
///
/// ## Process (KISS Algorithm)
/// 1. List all tables in .reed/tables/
/// 2. For each table:
///    - Read version.log
///    - Find last entry where timestamp <= target
///    - Restore table to that version
/// 3. Return report of restored states
///
/// ## Consistency Guarantee
/// All tables will be at their state as of or before target_timestamp.
/// No mixed states (e.g., users@14:05 + orders@13:58 for target 14:00).
///
/// ## Performance
/// - < 1 minute typical (depends on delta chain length)
/// - Each table restored independently
///
/// ## Error Conditions
/// - TableRestoreFailed: One or more tables failed to restore
/// - NoTablesFound: No tables exist
///
/// ## Example Usage
/// ```rust
/// // Restore to 2 hours ago
/// let two_hours_ago = SystemTime::now()
///     .duration_since(UNIX_EPOCH)
///     .unwrap()
///     .as_secs() - 7200;
///
/// let report = restore_point_in_time(two_hours_ago)?;
/// println!("Restored {} tables", report.tables_restored.len());
/// ```
pub fn restore_point_in_time(target_timestamp: i64) -> ReedResult<RestoreReport> {
    let tables_dir = Path::new(".reed/tables");
    
    if !tables_dir.exists() {
        return Err(ReedError::NoTablesFound);
    }
    
    let mut restored = Vec::new();
    let mut skipped = Vec::new();
    let mut errors = Vec::new();
    
    // Iterate all tables
    for entry in std::fs::read_dir(tables_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        
        let table_name = entry.file_name().to_string_lossy().to_string();
        let table = Table::new(Path::new(".reed"), &table_name);
        
        // Read version.log
        match table.list_versions() {
            Ok(versions) => {
                // Find best match: last version <= target
                let best_match = versions.iter()
                    .filter(|v| v.timestamp <= target_timestamp)
                    .max_by_key(|v| v.timestamp);
                
                match best_match {
                    Some(version) => {
                        // Restore to this version
                        match table.rollback(version.timestamp, "system") {
                            Ok(_) => {
                                restored.push((table_name.clone(), version.timestamp));
                            }
                            Err(e) => {
                                errors.push((table_name.clone(), e));
                            }
                        }
                    }
                    None => {
                        // Table didn't exist at target time
                        skipped.push(table_name.clone());
                    }
                }
            }
            Err(e) => {
                errors.push((table_name.clone(), e));
            }
        }
    }
    
    Ok(RestoreReport {
        target_timestamp,
        tables_restored: restored,
        tables_skipped: skipped,
        errors,
    })
}
```

**`src/reedcms/reedbase/backup/types.rs`**

One file = Type definitions only.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Backup and restore types.

use std::path::PathBuf;
use crate::types::ReedError;

/// Backup information.
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub timestamp: i64,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub size_mb: f64,
}

/// Restore report.
#[derive(Debug, Clone)]
pub struct RestoreReport {
    pub target_timestamp: i64,
    pub tables_restored: Vec<(String, i64)>,  // (table_name, actual_timestamp)
    pub tables_skipped: Vec<String>,          // Tables that didn't exist yet
    pub errors: Vec<(String, ReedError)>,     // Tables that failed to restore
}
```

## CLI Commands

```bash
# Create backup
reed backup:create
# Output: Backup created: .reed/backups/1736860800.tar.gz (45.2 MB)

# List backups
reed backup:list
# Output:
# 1736866000 (2025-01-15 16:00) - 45.2 MB
# 1736860800 (2025-01-15 14:00) - 44.8 MB
# 1736855600 (2025-01-15 12:00) - 44.1 MB

# Restore to specific time
reed restore:point-in-time 1736860800
# Output:
# Restoring to 2025-01-15 14:00...
# ✓ users: restored to 13:45 (15 minutes before target)
# ✓ orders: restored to 13:58 (2 minutes before target)
# ⊘ products: skipped (didn't exist yet)
# Restored 2 tables, skipped 1

# Restore to "2 hours ago"
reed restore:point-in-time --hours-ago 2
```

## Performance Requirements

- Backup creation: < 30s (100MB installation)
- List backups: < 10ms
- Point-in-time restore: < 1 minute (typical)

## Error Conditions

- **CommandFailed**: tar command failed
- **InsufficientSpace**: Not enough disk space for backup
- **NoTablesFound**: No tables to restore
- **TableRestoreFailed**: One or more tables failed to restore
- **IoError**: File system errors

## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| backup_duration | Histogram | seconds | <30 | >120 | create.rs:create_backup() |
| backup_size_mb | Gauge | MB | - | - | create.rs (backup file size) |
| restore_duration | Histogram | seconds | <60 | >300 | restore.rs:restore_point_in_time() |
| tables_restored_count | Histogram | count | - | - | restore.rs (tables processed) |
| restore_errors | Counter | count | 0 | >1 | restore.rs (failed tables) |

### Alert Rules

**CRITICAL Alerts:**
- `restore_errors > 1` for any restore → "Table restore failed - data integrity risk"
- `backup_duration > 120s` for 3 consecutive backups → "Backup extremely slow"

**WARNING Alerts:**
- `backup_duration > 60s` for 5 consecutive backups → "Backup taking longer than expected"
- `restore_duration > 120s` for any restore → "Restore taking very long"

### Implementation

```rust
use crate::reedbase::metrics::global as metrics;
use std::time::Instant;

pub fn create_backup() -> ReedResult<BackupInfo> {
    let start = Instant::now();
    
    let result = create_backup_inner()?;
    
    metrics().record(Metric {
        name: "backup_duration".to_string(),
        value: start.elapsed().as_secs() as f64,
        unit: MetricUnit::Seconds,
        tags: hashmap!{ "operation" => "create" },
    });
    
    metrics().gauge("backup_size_mb", result.size_mb, hashmap!{});
    
    Ok(result)
}
```

### Collection Strategy

- **Sampling**: All operations
- **Aggregation**: 5-minute rolling window
- **Storage**: `.reedbase/metrics/backup.csv`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**backup_duration**: Capacity planning
- Increasing duration indicates data growth
- Must fit within backup window
- Alerts prevent backup failures

**restore_errors**: Data integrity
- ANY restore error is critical
- Indicates corrupted deltas or version logs
- Requires immediate investigation

**restore_duration**: Recovery time objective (RTO)
- Affects downtime during disaster recovery
- Long restores indicate complex delta chains
- May require full backup instead of relying on deltas

## Acceptance Criteria

- [ ] `create_backup()` creates tar.gz in .reed/backups/
- [ ] Backup filename is Unix timestamp (seconds)
- [ ] `list_backups()` lists all backups sorted newest first
- [ ] `restore_point_in_time()` restores all tables consistently
- [ ] Point-in-time uses version.log timestamps (no new infrastructure)
- [ ] Tables that didn't exist yet are skipped
- [ ] Restore report shows what was restored and to which timestamp
- [ ] CLI commands work (`backup:create`, `backup:list`, `restore:point-in-time`)
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks meet targets
- [ ] All code in BBC English
- [ ] Each file has one clear responsibility

## Dependencies
- **Requires**: REED-19-02 (Table API), REED-19-03 (Delta Versioning), REED-19-04 (Version Log)

## Blocks
- None (optional enhancement for production)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- REED-19-00: Layer Overview
- REED-19-03: Binary Delta Versioning
- REED-19-04: Encoded Log System

## Notes

This ticket adds simple backup and point-in-time recovery to ReedBase using KISS principles.

**Key Design Decisions:**

1. **Use tar + xz**: Standard, proven, no custom compression
2. **Use existing version.log**: No new snapshot infrastructure needed
3. **Let admins handle advanced features**: Remote backup (rsync), scheduling (cron)

**The KISS Algorithm:**
- For each table, find last version.log entry <= target timestamp
- Restore to that version
- Result: Consistent state without coordination

**Why This Works:**
- version.log already has timestamps for every write
- No synchronization needed between tables
- Naturally consistent: "all tables as of time X"

**Admin Workflow:**
```bash
# Daily backup (cron)
0 2 * * * cd /app && reed backup:create

# Keep last 7 days
0 3 * * * find .reed/backups/ -name "*.tar.gz" -mtime +7 -delete

# Remote replication (admin's choice of tool)
0 4 * * * rsync -av .reed/backups/ backup-server:/backups/reedbase/

# Disaster recovery
reed restore:point-in-time 1736860800
```

---

## Frame-Based Point-in-Time Recovery

**Integration with Frame-System** (100-500× faster recovery):

The Frame-System provides pre-computed snapshots that dramatically accelerate point-in-time recovery.

### Performance Comparison

**Without Frames** (original algorithm):
```rust
pub fn restore_point_in_time(target_ts: i64) -> ReedResult<RestoreReport> {
    for table in ALL_TABLES {
        // Read and parse version.log (I/O + parsing overhead)
        let versions = read_version_log(table)?;  // O(n) per table
        
        // Find best match
        let best = versions.iter()
            .filter(|v| v.timestamp <= target_ts)
            .max_by_key(|v| v.timestamp)?;
        
        restore_table(table, best.timestamp)?;
    }
}
```

**Performance**: O(Tables × Versions)
- 10 tables × 1000 versions = 10,000 log entries to parse
- ~500ms for typical database

**With Frames** (optimised algorithm):
```rust
use crate::reedbase::frame::{find_nearest_frame, read_snapshot};

pub fn restore_point_in_time(target_ts: i64) -> ReedResult<RestoreReport> {
    // 1. Find nearest frame snapshot (binary search on index)
    let frame = find_nearest_frame(target_ts)?;  // O(log n)
    
    // 2. Load pre-computed snapshot (1 CSV file, ~10 lines)
    let snapshot = read_snapshot(frame.timestamp)?;  // O(tables)
    
    // 3. Restore each table to snapshot state
    let restore_ts = unix_now();
    for (table, entry) in snapshot {
        restore_table_to_new_version(
            &table,
            entry.timestamp,
            restore_ts,
        )?;
    }
    
    Ok(RestoreReport {
        frame_id: frame.id,
        frame_timestamp: frame.timestamp,
        tables_restored: snapshot.len(),
        new_version: restore_ts,
    })
}
```

**Performance**: O(log Frames + Tables)
- Binary search: log₂(1000 frames) = ~10 comparisons
- Load snapshot: 1 file read, parse ~10 lines
- Restore tables: 10 operations
- **~5ms total = 100× speedup**

### Frame Snapshot Format

Snapshots are created automatically when Frames commit (see REED-19-00):

```csv
# .reed/frames/1736860800.snapshot.csv
table|timestamp|hash|frame_id
text|1736860800|abc123|uuid002
routes|1736860700|def456|uuid002
meta|1736860750|ghi789|uuid002
users|1736860800|jkl012|uuid002
```

This pre-computed snapshot eliminates the need to search version.log for each table.

### Rollback Using Frames

Rollback is even simpler with Frames:

```rust
/// Rollback a frame (e.g., after failed migration).
///
/// Uses Frame snapshots for fast, atomic rollback.
pub fn rollback_frame(frame_id: Uuid) -> ReedResult<RollbackReport> {
    let frame = FRAME_MANAGER.get(frame_id)?;
    let rollback_ts = unix_now();
    
    // 1. Find previous committed frame
    let index = read_frame_index()?;
    let prev_frame = index.iter()
        .filter(|f| f.timestamp < frame.timestamp && f.status == FrameStatus::Committed)
        .max_by_key(|f| f.timestamp)
        .ok_or(ReedError::NoFrameBeforeTimestamp { target: frame.timestamp })?;
    
    // 2. Load previous snapshot
    let prev_snapshot = read_snapshot(prev_frame.timestamp)?;
    
    // 3. Restore to previous state (as NEW version)
    for (table, entry) in prev_snapshot {
        restore_table_to_new_version(
            &table,
            entry.timestamp,
            rollback_ts,
        )?;
        
        // Write version.log
        write_version_log(
            &table,
            rollback_ts,
            &format!("rollback from frame {}", frame_id),
            "system",
            None,  // Rollback doesn't create new frame
        )?;
    }
    
    // 4. Create new snapshot for rollback state
    let new_snapshot = create_snapshot()?;
    write_snapshot(rollback_ts, &new_snapshot, frame_id)?;
    
    // 5. Mark frame as rolled back
    FRAME_MANAGER.update_status(
        frame_id,
        FrameStatus::RolledBack,
        Some(rollback_ts),
    )?;
    
    Ok(RollbackReport {
        frame_id,
        rolled_back_to: prev_frame.timestamp,
        new_version: rollback_ts,
        tables_affected: prev_snapshot.len(),
    })
}
```

### CLI Integration

```bash
# Point-in-time restore (automatically uses frames if available)
reed restore:point-in-time 1736860800

# List available restore points (frames)
reed frame:list --committed

# Restore to specific frame
reed restore:frame <frame-id>

# Rollback a frame (e.g., failed migration)
reed frame:rollback <frame-id> --confirm
```

### Fallback to Version-Log

If no frame snapshot exists near the target timestamp, the system falls back to the original version-log algorithm:

```rust
pub fn restore_point_in_time(target_ts: i64) -> ReedResult<RestoreReport> {
    // Try frame-based restore first
    match find_nearest_frame(target_ts) {
        Ok(frame) => {
            info!("Using frame snapshot {} for fast restore", frame.id);
            restore_from_frame_snapshot(frame)
        }
        Err(ReedError::NoFrameBeforeTimestamp { .. }) => {
            warn!("No frame found, falling back to version-log search");
            restore_from_version_logs(target_ts)  // Original algorithm
        }
        Err(e) => Err(e),
    }
}
```

### Performance Guarantees

| Operation | Without Frames | With Frames | Speedup |
|-----------|----------------|-------------|---------|
| Point-in-Time Recovery | O(Tables × Versions) ~500ms | O(log Frames + Tables) ~5ms | **100×** |
| Rollback | O(Tables × Versions) ~500ms | O(Tables) ~10ms | **50×** |
| Find Restore Point | Linear scan | Binary search <1ms | **500×** |

### Frame Index Cache

Frame lookups are further accelerated by memory caching (see REED-19-00):

```rust
// Frame index cached in memory (60s TTL)
pub fn find_nearest_frame_cached(target_ts: i64) -> ReedResult<FrameInfo> {
    let cache = FRAME_INDEX_CACHE.get_or_init(|| FrameIndexCache::new());
    let index = cache.get()?;  // From cache, not disk
    
    // Binary search on cached index
    // ...
}
```

**Sub-millisecond lookups** for repeated restore-point queries.

### Migration Path

Existing installations without frames:

1. **Graceful fallback**: Version-log algorithm still works
2. **Gradual adoption**: New frames created as operations occur
3. **Speedup grows**: More frames = faster recovery over time
4. **No data migration**: Old data works with new code

---
