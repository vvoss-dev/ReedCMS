# Frame-System Implementation Plan

**Purpose**: Systematic implementation of the Frame-System across ReedBase tickets to ensure coordinated batch operations with consistent Unix timestamps.

**Status**: Planning
**Created**: 2025-10-15
**Version**: 1.0

---

## Overview

The **Frame-System** provides coordinated batch operations where multiple changes share a single Unix timestamp, enabling:

- ✅ Atomic schema migrations with data updates
- ✅ Consistent point-in-time recovery across all tables
- ✅ Fast rollback via pre-computed snapshots (O(1) instead of O(n))
- ✅ Crash recovery with automatic rollback of incomplete frames
- ✅ Audit trail of all coordinated operations

---

## Architecture Principles

### 1. KISS (Keep It Simple, Stupid)
- Frame = ONE timestamp shared by multiple operations
- Snapshot = HashMap of `table → current_timestamp`
- Index = Sorted list of all frames (binary search)
- No nested frames, no 2-phase commits, no complex locking

### 2. DRY (Don't Repeat Yourself)
- Frame-Manager is singleton (one implementation, used everywhere)
- All tickets use SAME Frame API (`Frame::begin()`, `commit()`, `rollback()`)
- Snapshot creation is identical for all use cases

### 3. Consistency First
- All frames follow identical structure
- All snapshots use identical CSV format
- All metrics use identical naming conventions

### 4. Zero Data Loss
- Rollback is versionised (forward recovery, not deletion)
- Frames are never deleted, only marked as "rolled_back"
- Complete audit trail in frame.log and snapshots

---

## Core Concepts

### Frame
A coordinated set of operations sharing ONE Unix timestamp.

```rust
pub struct Frame {
    id: Uuid,              // Unique identifier
    timestamp: i64,        // THE shared Unix timestamp
    name: String,          // Human-readable name
    started_at: Instant,   // For monitoring
    operations: Vec<Op>,   // Log of operations
}
```

### Snapshot
State of ALL tables at frame commit time.

```csv
# .reed/frames/{timestamp}.snapshot.csv
table|timestamp|hash|frame_id
text|1736860800|abc123|uuid002
routes|1736860700|def456|uuid002
meta|1736860750|ghi789|uuid002
```

### Frame-Index
Sorted list of all frames for O(log n) lookup.

```csv
# .reed/frames/index.csv
timestamp|frame_id|name|status|tables_affected|committed_at
1736860800|uuid002|schema_migration_1_2|committed|4|1736860815
```

---

## Implementation Checklist

### Phase 1: Foundation (REED-19-01)

**File**: `src/reedcms/reedstream.rs`

**Task**: Add core Frame types to ReedStream

- [ ] Add `FrameInfo` struct
  ```rust
  pub struct FrameInfo {
      pub id: Uuid,
      pub timestamp: i64,
      pub name: String,
      pub status: FrameStatus,
      pub tables_affected: usize,
      pub started_at: i64,
      pub committed_at: Option<i64>,
  }
  ```

- [ ] Add `FrameStatus` enum
  ```rust
  pub enum FrameStatus {
      Active,       // Frame is currently being built
      Committed,    // Frame successfully committed
      RolledBack,   // Frame was rolled back
      Crashed,      // Frame was active when system crashed
      Archived,     // Frame is older than retention period (snapshot deleted)
  }
  impl FrameStatus {
      pub fn as_str(&self) -> &str {
          match self {
              FrameStatus::Active => "active",
              FrameStatus::Committed => "committed",
              FrameStatus::RolledBack => "rolled_back",
              FrameStatus::Crashed => "crashed",
              FrameStatus::Archived => "archived",
          }
      }
  }
  ```

- [ ] Add `FrameSnapshot` type alias
  ```rust
  pub type FrameSnapshot = HashMap<String, SnapshotEntry>;
  
  pub struct SnapshotEntry {
      pub timestamp: i64,
      pub hash: String,
  }
  ```

- [ ] Add `ReedError` variants
  ```rust
  pub enum ReedError {
      // Existing variants...
      
      // Frame-specific errors
      FrameAlreadyActive { frame_id: Uuid },
      FrameNotFound { frame_id: Uuid },
      NoFrameBeforeTimestamp { target: i64 },
      FrameSnapshotCorrupted { path: String },
  }
  ```

**Acceptance Criteria**:
- ✅ All types compile without errors
- ✅ All types have proper documentation
- ✅ Error variants have helpful context messages
- ✅ Types follow ReedBase naming conventions

**Estimated Time**: 30 minutes

---

### Phase 2: Metrics (REED-19-01A)

**File**: `src/reedcms/reedbase/metrics.rs`

**Task**: Add Frame-specific metrics

- [ ] Define Frame metrics
  ```rust
  // In metrics collection points
  
  // Frame lifecycle
  "frame_started_total"              // Counter
  "frame_committed_total"            // Counter
  "frame_rolled_back_total"          // Counter
  "frame_crashed_total"              // Counter
  
  // Frame performance
  "frame_commit_duration_seconds"    // Histogram
  "frame_operations_count"           // Histogram
  "frame_tables_affected_count"      // Histogram
  "frame_snapshot_size_bytes"        // Histogram
  
  // Frame recovery
  "frame_recovery_duration_seconds"  // Histogram
  "frame_rollback_duration_seconds"  // Histogram
  ```

- [ ] Add collection points in Frame implementation
  ```rust
  impl Frame {
      pub fn begin(name: &str) -> ReedResult<Self> {
          metrics().record(Metric {
              name: "frame_started_total".to_string(),
              value: 1.0,
              unit: MetricUnit::Count,
              tags: hashmap!{ "name" => name },
          });
          // ...
      }
      
      pub fn commit(self) -> ReedResult<FrameReport> {
          let start = Instant::now();
          // ... commit logic ...
          
          metrics().record(Metric {
              name: "frame_commit_duration_seconds".to_string(),
              value: start.elapsed().as_secs_f64(),
              unit: MetricUnit::Seconds,
              tags: hashmap!{
                  "name" => &self.name,
                  "tables_affected" => &self.tables.len().to_string(),
              },
          });
      }
  }
  ```

- [ ] Add alert rules to metrics documentation
  ```markdown
  **CRITICAL Alerts:**
  - `frame_crashed_total > 0` for 1m → "Frame crashed - data may be inconsistent"
  - `frame_commit_duration_seconds > 30` → "Frame commit taking too long"
  
  **WARNING Alerts:**
  - `frame_operations_count > 100` → "Frame has too many operations - consider splitting"
  - `frame_active_duration_seconds > 300` → "Frame active for >5min - possible leak"
  ```

**Acceptance Criteria**:
- ✅ All metrics follow naming convention `frame_{metric}_{unit}`
- ✅ All metrics have appropriate tags (name, status, etc.)
- ✅ Alert rules are actionable and specific
- ✅ Metrics are documented in REED-19-01A

**Estimated Time**: 20 minutes

---

### Phase 3: Version Log Extension (REED-19-04)

**File**: `src/reedcms/reedbase/version.rs`

**Task**: Add frame_id column to version.log format

- [ ] Update `VersionLogEntry` struct
  ```rust
  pub struct VersionLogEntry {
      pub timestamp: i64,
      pub action: String,
      pub user: String,
      pub base: i64,
      pub size: usize,
      pub rows: usize,
      pub hash: String,
      pub crc32: String,
      pub frame_id: Option<Uuid>,  // ← NEW
  }
  ```

- [ ] Update version.log CSV format
  ```csv
  # Old format (8 columns):
  timestamp|action|user|base|size|rows|hash|crc32
  
  # New format (9 columns):
  timestamp|action|user|base|size|rows|hash|crc32|frame_id
  1736860800|schema_migration|admin|1736860700|2048|120|abc|def|uuid002
  1736860900|rollback|system|1736860700|1024|100|abc|def|uuid003
  ```

- [ ] Update `write_version_log()` function
  ```rust
  pub fn write_version_log(
      table: &str,
      timestamp: i64,
      action: &str,
      user: &str,
      frame_id: Option<Uuid>,  // ← NEW parameter
  ) -> ReedResult<()> {
      let entry = format!(
          "{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
          timestamp,
          action,
          user,
          base,
          size,
          rows,
          hash,
          crc32,
          frame_id.map(|id| id.to_string()).unwrap_or_else(|| "n/a".to_string()),
      );
      // ...
  }
  ```

- [ ] Update `parse_version_log()` function
  ```rust
  pub fn parse_version_log_entry(line: &str) -> ReedResult<VersionLogEntry> {
      let parts: Vec<&str> = line.split('|').collect();
      
      // Support both old (8 cols) and new (9 cols) format
      let frame_id = if parts.len() >= 9 {
          match parts[8] {
              "n/a" | "" => None,
              id_str => Some(Uuid::parse_str(id_str)?),
          }
      } else {
          None  // Old format
      };
      
      Ok(VersionLogEntry {
          // ... existing fields ...
          frame_id,
      })
  }
  ```

- [ ] Add migration function for existing logs
  ```rust
  /// Migrates old 8-column version.log to new 9-column format
  pub fn migrate_version_log_format(table: &str) -> ReedResult<()> {
      let log_path = format!(".reed/{}/version.log", table);
      let entries = read_version_log(table)?;
      
      // Re-write with new format (frame_id = n/a for old entries)
      let mut new_log = String::from("timestamp|action|user|base|size|rows|hash|crc32|frame_id\n");
      for entry in entries {
          new_log.push_str(&format_version_log_entry(&entry)?);
      }
      
      fs::write(log_path, new_log)?;
      Ok(())
  }
  ```

**Acceptance Criteria**:
- ✅ Parser handles both old (8-col) and new (9-col) format
- ✅ Migration function successfully updates existing logs
- ✅ All new writes include frame_id (or "n/a")
- ✅ Documentation updated with new format

**Estimated Time**: 30 minutes

---

### Phase 4: Frame Manager Core (REED-19-09)

**File**: `src/reedcms/reedbase/frame.rs` (NEW)

**Task**: Implement Frame-Manager singleton and Frame API

- [ ] Create Frame struct
  ```rust
  pub struct Frame {
      id: Uuid,
      timestamp: i64,
      name: String,
      started_at: Instant,
      operations: Vec<OperationLog>,
      committed: bool,
  }
  
  struct OperationLog {
      step: usize,
      action: String,
      table: Option<String>,
      result: String,
  }
  ```

- [ ] Implement Frame API
  ```rust
  impl Frame {
      pub fn begin(name: &str) -> ReedResult<Self>;
      pub fn timestamp(&self) -> i64;
      pub fn log_operation(&mut self, action: &str, table: Option<&str>);
      pub fn commit(self) -> ReedResult<FrameReport>;
      pub fn rollback(self) -> ReedResult<()>;
  }
  
  impl Drop for Frame {
      fn drop(&mut self) {
          if !self.committed {
              warn!("Frame {} not committed - auto-rollback", self.id);
              let _ = FRAME_MANAGER.rollback(self.id);
          }
      }
  }
  ```

- [ ] Create FrameManager singleton
  ```rust
  pub struct FrameManager {
      active: RwLock<HashMap<Uuid, FrameState>>,
      log_path: PathBuf,  // .reed/frames/frame.log
  }
  
  static FRAME_MANAGER: OnceLock<FrameManager> = OnceLock::new();
  
  pub fn global() -> &'static FrameManager {
      FRAME_MANAGER.get_or_init(|| FrameManager::new())
  }
  ```

- [ ] Implement frame operations
  ```rust
  impl FrameManager {
      pub fn register(&self, frame: Frame) -> ReedResult<()>;
      pub fn commit(&self, frame_id: Uuid) -> ReedResult<FrameReport>;
      pub fn rollback(&self, frame_id: Uuid) -> ReedResult<()>;
      pub fn list_active(&self) -> Vec<FrameInfo>;
      pub fn get(&self, frame_id: Uuid) -> ReedResult<FrameInfo>;
  }
  ```

- [ ] Implement snapshot creation
  ```rust
  fn create_snapshot(&self) -> ReedResult<FrameSnapshot> {
      let mut snapshot = HashMap::new();
      
      for table in ALL_TABLES {
          // Find latest bsdiff file
          let pattern = format!(".reed/{}/*.bsdiff", table);
          let mut files: Vec<_> = glob(&pattern)?
              .filter_map(Result::ok)
              .collect();
          
          files.sort_by_key(|f| {
              extract_timestamp_from_path(f).unwrap_or(0)
          });
          
          let latest = files.last()
              .ok_or(ReedError::NoVersionsFound { table })?;
          
          let ts = extract_timestamp_from_path(latest)?;
          let hash = calculate_hash(latest)?;
          
          snapshot.insert(table.to_string(), SnapshotEntry {
              timestamp: ts,
              hash,
          });
      }
      
      Ok(snapshot)
  }
  ```

- [ ] Implement snapshot persistence
  ```rust
  fn write_snapshot(
      timestamp: i64,
      snapshot: &FrameSnapshot,
      frame_id: Uuid,
  ) -> ReedResult<()> {
      let path = format!(".reed/frames/{}.snapshot.csv", timestamp);
      let mut csv = String::from("table|timestamp|hash|frame_id\n");
      
      for (table, entry) in snapshot {
          csv.push_str(&format!(
              "{}|{}|{}|{}\n",
              table, entry.timestamp, entry.hash, frame_id
          ));
      }
      
      fs::write(path, csv)?;
      Ok(())
  }
  
  fn read_snapshot(timestamp: i64) -> ReedResult<FrameSnapshot> {
      let path = format!(".reed/frames/{}.snapshot.csv", timestamp);
      let content = fs::read_to_string(&path)
          .map_err(|e| ReedError::FrameSnapshotCorrupted { path: path.clone() })?;
      
      let mut snapshot = HashMap::new();
      
      for line in content.lines().skip(1) {  // Skip header
          let parts: Vec<&str> = line.split('|').collect();
          if parts.len() != 4 {
              return Err(ReedError::FrameSnapshotCorrupted { path });
          }
          
          snapshot.insert(
              parts[0].to_string(),
              SnapshotEntry {
                  timestamp: parts[1].parse()?,
                  hash: parts[2].to_string(),
              }
          );
      }
      
      Ok(snapshot)
  }
  ```

- [ ] Implement frame index
  ```rust
  fn append_to_frame_index(entry: FrameIndexEntry) -> ReedResult<()> {
      let path = ".reed/frames/index.csv";
      
      let line = format!(
          "{}|{}|{}|{}|{}|{}\n",
          entry.timestamp,
          entry.frame_id,
          entry.name,
          entry.status.as_str(),
          entry.tables_affected,
          entry.committed_at.unwrap_or(0),
      );
      
      let mut file = OpenOptions::new()
          .create(true)
          .append(true)
          .open(path)?;
      
      file.write_all(line.as_bytes())?;
      Ok(())
  }
  
  fn read_frame_index() -> ReedResult<Vec<FrameIndexEntry>> {
      let path = ".reed/frames/index.csv";
      let content = fs::read_to_string(path)?;
      
      content.lines()
          .skip(1)  // Skip header
          .map(parse_frame_index_line)
          .collect()
  }
  
  pub fn find_nearest_frame(target_ts: i64) -> ReedResult<FrameInfo> {
      let index = read_frame_index()?;
      
      let pos = index.binary_search_by(|frame| {
          frame.timestamp.cmp(&target_ts)
      });
      
      let best_frame = match pos {
          Ok(exact) => &index[exact],
          Err(insert_pos) => {
              if insert_pos == 0 {
                  return Err(ReedError::NoFrameBeforeTimestamp { target: target_ts });
              }
              &index[insert_pos - 1]
          }
      };
      
      Ok(FrameInfo::from(best_frame))
  }
  ```

- [ ] Implement crash recovery
  ```rust
  pub fn recover_crashed_frames() -> ReedResult<RecoveryReport> {
      let log_path = ".reed/frames/frame.log";
      let active_frames = read_frame_log()?
          .into_iter()
          .filter(|f| f.status == FrameStatus::Active);
      
      let mut recovered = Vec::new();
      
      for frame in active_frames {
          warn!("Found crashed frame: {} ({})", frame.name, frame.id);
          
          // Rollback using versionised recovery
          rollback_frame(frame.id)?;
          
          recovered.push(frame);
      }
      
      Ok(RecoveryReport {
          frames_recovered: recovered.len(),
          frames: recovered,
      })
  }
  ```

- [ ] Implement frame cleanup (TTL)
  ```rust
  /// Cleanup old frames based on retention policy
  /// Default: 365 days (configurable via frame.retention.days)
  pub fn cleanup_old_frames() -> ReedResult<CleanupReport> {
      let retention_days = get_project_config("frame.retention.days")
          .unwrap_or("365".to_string())
          .parse::<i64>()
          .unwrap_or(365);
      
      let cutoff = unix_now() - (retention_days * 86400);  // days to seconds
      
      let index = read_frame_index()?;
      let old_frames: Vec<_> = index.iter()
          .filter(|f| f.timestamp < cutoff)
          .filter(|f| f.status == FrameStatus::Committed)  // Only cleanup committed
          .collect();
      
      let mut cleaned = 0;
      
      for frame in old_frames {
          // Delete snapshot file
          let snapshot_path = format!(".reed/frames/{}.snapshot.csv", frame.timestamp);
          if Path::new(&snapshot_path).exists() {
              fs::remove_file(&snapshot_path)?;
          }
          
          // Mark in index as "archived" (don't delete from index for audit trail)
          update_frame_status(frame.frame_id, FrameStatus::Archived)?;
          
          cleaned += 1;
      }
      
      Ok(CleanupReport {
          frames_cleaned: cleaned,
          retention_days,
          cutoff_timestamp: cutoff,
      })
  }
  ```

- [ ] Implement frame index caching
  ```rust
  /// Memory-cached frame index for sub-millisecond lookups
  pub struct FrameIndexCache {
      index: RwLock<Option<(Vec<FrameIndexEntry>, Instant)>>,
      ttl: Duration,
  }
  
  static FRAME_INDEX_CACHE: OnceLock<FrameIndexCache> = OnceLock::new();
  
  impl FrameIndexCache {
      pub fn new() -> Self {
          Self {
              index: RwLock::new(None),
              ttl: Duration::from_secs(60),  // 1 minute cache
          }
      }
      
      pub fn get(&self) -> ReedResult<Vec<FrameIndexEntry>> {
          let cache = self.index.read().unwrap();
          
          // Check if cache is valid
          if let Some((ref index, cached_at)) = *cache {
              if cached_at.elapsed() < self.ttl {
                  return Ok(index.clone());
              }
          }
          
          drop(cache);  // Release read lock
          
          // Refresh cache
          let fresh_index = read_frame_index_from_disk()?;
          let mut cache = self.index.write().unwrap();
          *cache = Some((fresh_index.clone(), Instant::now()));
          
          Ok(fresh_index)
      }
      
      pub fn invalidate(&self) {
          let mut cache = self.index.write().unwrap();
          *cache = None;
      }
  }
  
  pub fn find_nearest_frame_cached(target_ts: i64) -> ReedResult<FrameInfo> {
      let cache = FRAME_INDEX_CACHE.get_or_init(|| FrameIndexCache::new());
      let index = cache.get()?;
      
      // Binary search on cached index
      let pos = index.binary_search_by(|frame| {
          frame.timestamp.cmp(&target_ts)
      });
      
      let best_frame = match pos {
          Ok(exact) => &index[exact],
          Err(insert_pos) => {
              if insert_pos == 0 {
                  return Err(ReedError::NoFrameBeforeTimestamp { target: target_ts });
              }
              &index[insert_pos - 1]
          }
      };
      
      Ok(FrameInfo::from(best_frame))
  }
  ```

- [ ] Add CLI commands
  ```rust
  // In src/reedcms/cli/frame.rs (NEW)
  
  /// List all frames, optionally filtered by status
  pub fn list_frames(filter: Option<FrameStatus>) -> ReedResult<()> {
      let frames = FRAME_MANAGER.list_all()?;
      
      let filtered: Vec<_> = if let Some(status) = filter {
          frames.into_iter().filter(|f| f.status == status).collect()
      } else {
          frames
      };
      
      println!("Frame ID                             | Timestamp   | Name                    | Status      | Tables");
      println!("-------------------------------------|-------------|-------------------------|-------------|--------");
      
      for frame in filtered {
          println!(
              "{} | {} | {:23} | {:11} | {}",
              frame.id,
              frame.timestamp,
              truncate(&frame.name, 23),
              frame.status.as_str(),
              frame.tables_affected,
          );
      }
      
      Ok(())
  }
  
  /// Show detailed frame status
  pub fn show_frame_status(frame_id: Uuid) -> ReedResult<()> {
      let frame = FRAME_MANAGER.get(frame_id)?;
      
      println!("Frame: {}", frame.id);
      println!("Name: {}", frame.name);
      println!("Timestamp: {} ({})", frame.timestamp, format_timestamp(frame.timestamp));
      println!("Status: {}", frame.status.as_str());
      println!("Tables affected: {}", frame.tables_affected);
      
      if let Some(committed_at) = frame.committed_at {
          println!("Committed at: {} ({})", committed_at, format_timestamp(committed_at));
      }
      
      // Show operations if available
      let snapshot = read_snapshot(frame.timestamp)?;
      println!("\nSnapshot:");
      for (table, entry) in snapshot {
          println!("  {} → {} ({})", table, entry.timestamp, entry.hash);
      }
      
      Ok(())
  }
  
  /// Rollback a frame
  pub fn rollback_frame_cli(frame_id: Uuid, confirm: bool) -> ReedResult<()> {
      let frame = FRAME_MANAGER.get(frame_id)?;
      
      if !confirm {
          println!("⚠️  WARNING: This will rollback frame '{}'", frame.name);
          println!("   Timestamp: {}", frame.timestamp);
          println!("   Tables affected: {}", frame.tables_affected);
          println!("\n   Use --confirm to proceed");
          return Ok(());
      }
      
      println!("Rolling back frame {}...", frame_id);
      let report = rollback_frame(frame_id)?;
      
      println!("✅ Rollback complete:");
      println!("   Rolled back to: {}", report.rolled_back_to);
      println!("   New version: {}", report.new_version);
      println!("   Tables affected: {}", report.tables_affected);
      
      Ok(())
  }
  ```

- [ ] Add server startup recovery check
  ```rust
  // In src/reedcms/reed/reedserver.rs
  
  pub fn start_server() -> ReedResult<()> {
      // Check for crashed frames
      let crashed = find_crashed_frames()?;
      
      if !crashed.is_empty() {
          eprintln!("⚠️  WARNING: Found {} crashed frame(s)", crashed.len());
          eprintln!("");
          
          for frame in &crashed {
              eprintln!("  • {} - {} ({})", frame.id, frame.name, frame.timestamp);
          }
          
          eprintln!("");
          eprintln!("Options:");
          eprintln!("  1. Auto-recover (rollback all crashed frames)");
          eprintln!("  2. Manual recovery (use 'reed frame:list --crashed')");
          eprintln!("  3. Continue anyway (NOT RECOMMENDED)");
          eprintln!("");
          
          print!("Choose [1/2/3]: ");
          io::stdout().flush()?;
          
          let mut choice = String::new();
          io::stdin().read_line(&mut choice)?;
          
          match choice.trim() {
              "1" => {
                  println!("Auto-recovering crashed frames...");
                  let report = recover_crashed_frames()?;
                  println!("✅ Recovered {} frame(s)", report.frames_recovered);
              }
              "2" => {
                  println!("Please run: reed frame:list --crashed");
                  println!("Then use: reed frame:rollback <frame-id> --confirm");
                  return Err(ReedError::ServerStartAborted);
              }
              "3" => {
                  warn!("Starting server with crashed frames - database may be inconsistent!");
              }
              _ => {
                  return Err(ReedError::InvalidChoice);
              }
          }
      }
      
      // Continue with normal server startup
      println!("Starting ReedBase server...");
      // ...
  }
  ```

**Acceptance Criteria**:
- ✅ Frame::begin() creates unique timestamp and ID
- ✅ Frame::commit() creates snapshot + updates index
- ✅ Frame::rollback() performs versionised rollback
- ✅ Drop handler prevents uncommitted frames
- ✅ Crash recovery works on server restart
- ✅ All operations have metrics collection
- ✅ Binary search works correctly for frame lookup
- ✅ Snapshots are atomic (temp file + rename)

**Estimated Time**: 2-3 hours

---

### Phase 5: Point-in-Time Recovery Integration (REED-19-03A)

**File**: Update existing `REED-19-03A-backup-point-in-time-recovery.md`

**Task**: Replace version.log search with frame-snapshot lookup

- [ ] Update `restore_point_in_time()` implementation
  ```rust
  // OLD VERSION (slow):
  pub fn restore_point_in_time(target_ts: i64) -> ReedResult<RestoreReport> {
      for table in ALL_TABLES {
          let versions = read_version_log(table)?;  // O(n)
          let best = versions.iter()
              .filter(|v| v.timestamp <= target_ts)
              .max_by_key(|v| v.timestamp)?;
          
          restore_table(table, best.timestamp)?;
      }
  }
  
  // NEW VERSION (fast):
  pub fn restore_point_in_time(target_ts: i64) -> ReedResult<RestoreReport> {
      // 1. Find nearest frame snapshot
      let frame = find_nearest_frame(target_ts)?;  // O(log n)
      
      // 2. Load snapshot (O(tables))
      let snapshot = read_snapshot(frame.timestamp)?;
      
      // 3. Restore each table
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

- [ ] Update rollback implementation
  ```rust
  pub fn rollback_frame(frame_id: Uuid) -> ReedResult<RollbackReport> {
      let frame = FRAME_MANAGER.get(frame_id)?;
      let rollback_ts = unix_now();
      
      // Find previous frame before this one
      let index = read_frame_index()?;
      let prev_frame = index.iter()
          .filter(|f| f.timestamp < frame.timestamp && f.status == FrameStatus::Committed)
          .max_by_key(|f| f.timestamp)
          .ok_or(ReedError::NoFrameBeforeTimestamp { target: frame.timestamp })?;
      
      // Load previous snapshot
      let prev_snapshot = read_snapshot(prev_frame.timestamp)?;
      
      // Restore each table to previous state (as NEW version)
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
      
      // Create snapshot of rollback state
      let new_snapshot = create_snapshot()?;
      write_snapshot(rollback_ts, &new_snapshot, frame_id)?;
      
      // Update frame status
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

- [ ] Update documentation section "Point-in-Time Recovery Algorithm"
  ```markdown
  ## Point-in-Time Recovery Algorithm (Frame-Optimized)
  
  ### Performance
  - **Without Frames**: O(Tables × Versions) = 10 tables × 1000 versions = 10,000 comparisons
  - **With Frames**: O(log Frames + Tables) = log₂(1000) + 10 = ~20 operations
  - **Speedup**: ~500× faster
  
  ### Algorithm
  1. Binary search frame index for nearest frame ≤ target (O(log n))
  2. Load snapshot from frame (O(1) file read)
  3. Restore tables from snapshot (O(tables))
  4. Write new version-log entries (O(tables))
  5. Create new snapshot for restore point (O(tables))
  ```

**Acceptance Criteria**:
- ✅ Point-in-time restore uses frame snapshots
- ✅ Rollback is versionised (forward recovery)
- ✅ Performance metrics show 100×+ improvement
- ✅ Documentation reflects new algorithm

**Estimated Time**: 45 minutes

---

### Phase 6: Schema Migration Integration (REED-19-09)

**File**: Update existing `REED-19-09-schema-migrations-versioning.md`

**Task**: Add Frame-based schema migration example

- [ ] Add "Frame-Based Migration" section
  ```markdown
  ## Frame-Based Schema Migration
  
  Schema migrations use the Frame-System to guarantee atomicity across:
  - Schema file changes
  - Table data transformations
  - Index rebuilds
  - Validation checks
  
  All operations share ONE timestamp = atomic unit.
  ```

- [ ] Add implementation example
  ```rust
  pub fn migrate_schema(from: u32, to: u32) -> ReedResult<MigrationReport> {
      // 1. Begin frame
      let mut frame = Frame::begin(&format!("schema_migration_{}_{}", from, to))?;
      let ts = frame.timestamp();
      
      // 2. Write new schema file
      let schema = load_migration_schema(to)?;
      let schema_path = format!(".reed/schema/{}.schema.toml", ts);
      fs::write(&schema_path, schema.to_toml()?)?;
      frame.log_operation("write_schema", None);
      
      // 3. Migrate table data (all with SAME timestamp)
      for table in schema.affected_tables() {
          migrate_table_data(table, from, to, ts)?;
          frame.log_operation("migrate_data", Some(table));
          
          // Write version.log with frame_id
          write_version_log(
              table,
              ts,
              &format!("schema_migration_{}_{}", from, to),
              "admin",
              Some(frame.id),
          )?;
      }
      
      // 4. Rebuild affected indices
      for index in schema.affected_indices() {
          rebuild_index(index, ts)?;
          frame.log_operation("rebuild_index", Some(index));
      }
      
      // 5. Validate migration
      for table in schema.affected_tables() {
          validate_table_integrity(table)?;
          frame.log_operation("validate", Some(table));
      }
      
      // 6. Commit frame (creates snapshot automatically)
      let report = frame.commit()?;
      
      Ok(MigrationReport {
          from_version: from,
          to_version: to,
          frame_id: report.frame_id,
          timestamp: ts,
          tables_migrated: schema.affected_tables().len(),
          indices_rebuilt: schema.affected_indices().len(),
          duration: report.duration,
      })
  }
  ```

- [ ] Add rollback example
  ```rust
  pub fn rollback_migration(migration_frame_id: Uuid) -> ReedResult<RollbackReport> {
      // Frame-System handles rollback automatically
      rollback_frame(migration_frame_id)
  }
  ```

- [ ] Update Performance section
  ```markdown
  ## Performance Guarantees
  
  | Operation | Target | Monitoring |
  |-----------|--------|------------|
  | Migration | <10s per table | `frame_commit_duration_seconds` |
  | Rollback | <5s total | `frame_rollback_duration_seconds` |
  | Validation | <1s per table | `schema_validation_duration_seconds` |
  | Recovery | <30s total | `frame_recovery_duration_seconds` |
  ```

**Acceptance Criteria**:
- ✅ Schema migration uses Frame API
- ✅ All operations share same timestamp
- ✅ Rollback is one command
- ✅ Example code compiles and follows REED patterns

**Estimated Time**: 30 minutes

---

## Verification Checklist

### Code Quality
- [ ] All functions have single responsibility (KISS)
- [ ] No code duplication (DRY)
- [ ] All public functions documented with `///` comments
- [ ] Error handling uses specific `ReedError` variants
- [ ] All timestamps are `i64` Unix seconds (consistent)
- [ ] All UUIDs use `uuid::Uuid` type

### Performance
- [ ] Frame creation: O(1)
- [ ] Snapshot creation: O(tables) ≤ 10ms
- [ ] Frame lookup: O(log n) ≤ 1ms
- [ ] Rollback: O(tables) ≤ 100ms
- [ ] Recovery: O(crashed_frames × tables) ≤ 1s

### Testing
- [ ] Unit tests for Frame::begin(), commit(), rollback()
- [ ] Unit tests for snapshot creation/parsing
- [ ] Unit tests for frame index binary search
- [ ] Integration test: Schema migration with rollback
- [ ] Integration test: Crash recovery
- [ ] Integration test: Point-in-time restore with frames

### Documentation
- [ ] All new types documented in REED-19-01
- [ ] All metrics documented in REED-19-01A
- [ ] Frame API documented in REED-19-09
- [ ] Examples provided for common use cases
- [ ] Migration guide for existing deployments

### Consistency
- [ ] All CSV files use pipe `|` delimiter
- [ ] All timestamps are 10-digit Unix seconds
- [ ] All IDs are UUIDs (not sequential)
- [ ] All file names follow `{timestamp}.{type}.csv` pattern
- [ ] All functions use `ReedResult<T>` return type

---

## Implementation Order

**Total Estimated Time**: 4-5 hours

1. **Phase 1: Foundation** (30min)
   - Add types to reedstream.rs
   - Compile check

2. **Phase 2: Metrics** (20min)
   - Define metrics in REED-19-01A
   - Update metrics.rs placeholders

3. **Phase 3: Version Log** (30min)
   - Extend version.log format
   - Add backward compatibility

4. **Phase 4: Frame Core** (2-3h) ⚠️ MAIN WORK
   - Implement Frame struct
   - Implement FrameManager
   - Implement snapshots
   - Implement index
   - Implement recovery

5. **Phase 5: Backup Integration** (45min)
   - Update REED-19-03A
   - Refactor restore functions

6. **Phase 6: Schema Integration** (30min)
   - Update REED-19-09
   - Add migration example

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_lifecycle() {
        let frame = Frame::begin("test").unwrap();
        let ts = frame.timestamp();
        
        frame.log_operation("step1", Some("text"));
        frame.log_operation("step2", Some("routes"));
        
        let report = frame.commit().unwrap();
        assert_eq!(report.operations, 2);
        
        // Snapshot should exist
        let snapshot = read_snapshot(ts).unwrap();
        assert!(snapshot.contains_key("text"));
    }
    
    #[test]
    fn test_frame_rollback() {
        let frame = Frame::begin("rollback_test").unwrap();
        let frame_id = frame.id;
        
        // Make some changes...
        
        let report = rollback_frame(frame_id).unwrap();
        assert!(report.tables_affected > 0);
        
        // Frame should be marked as rolled_back
        let info = FRAME_MANAGER.get(frame_id).unwrap();
        assert_eq!(info.status, FrameStatus::RolledBack);
    }
    
    #[test]
    fn test_frame_index_binary_search() {
        // Create test frames
        for i in 0..100 {
            let frame = Frame::begin(&format!("test_{}", i)).unwrap();
            frame.commit().unwrap();
        }
        
        // Search for frame in middle
        let target = unix_now() - 50;
        let found = find_nearest_frame(target).unwrap();
        
        assert!(found.timestamp <= target);
    }
}
```

### Integration Tests
```rust
#[test]
fn test_schema_migration_with_rollback() {
    // 1. Initial state
    let initial = create_test_database();
    
    // 2. Run migration
    let report = migrate_schema(1, 2).unwrap();
    let migration_frame = report.frame_id;
    
    // 3. Verify migration
    assert_eq!(read_schema_version(), 2);
    
    // 4. Rollback
    rollback_frame(migration_frame).unwrap();
    
    // 5. Verify rollback
    assert_eq!(read_schema_version(), 1);
}
```

---

## Success Criteria

### Functional
- ✅ Frame-based schema migration works end-to-end
- ✅ Point-in-time recovery uses frames
- ✅ Crash recovery restores consistency
- ✅ Rollback is versionised (no data loss)

### Performance
- ✅ Frame commit <100ms (target: <50ms)
- ✅ Snapshot lookup <1ms (binary search)
- ✅ Recovery <30s for typical crash

### Quality
- ✅ 100% test coverage for Frame core
- ✅ All documentation complete
- ✅ No compiler warnings
- ✅ All metrics implemented

### Consistency
- ✅ All tickets use identical Frame API
- ✅ All timestamps are Unix seconds
- ✅ All errors are specific ReedError variants
- ✅ All CSV formats consistent

---

## Rollout Plan

### 1. Development (This plan)
- Implement all phases
- Write tests
- Verify locally

### 2. Documentation
- Update REED-19-01, 01A, 03A, 04, 09
- Add examples to each ticket
- Update main ReedBase overview

### 3. Migration
- Existing deployments: Run version.log migration
- Create initial frame.log
- Create frame index from existing snapshots

### 4. Deployment
- Commit all changes in one batch
- Tag as REED-19-FRAMES-COMPLETE
- Update TICKET-STATUS.md

---

## Notes

**Design Decisions**:
- Frame = coordinated timestamp (not transaction)
- Snapshots = pre-computed state (not computed on-demand)
- Index = sorted list (not database)
- Rollback = versionised (not destructive)

**Trade-offs**:
- Storage: ~500 bytes per frame (negligible)
- CPU: Snapshot creation on commit (minimal, <10ms)
- Complexity: Frame-Manager singleton (unavoidable for crash recovery)

**Alternatives Considered**:
- ❌ No frames, search version.log → Too slow (O(n))
- ❌ Database for frame index → Over-engineering
- ❌ Nested frames → Complexity explosion
- ✅ Current design → KISS, fast, reliable

---

**Questions Before Implementation**: ✅ RESOLVED

1. **Frame TTL/auto-cleanup**: ✅ Default 365 days, configurable via `.reed/project.csv` + `reed set:project frame.retention.days 730`
2. **Frame index caching**: ✅ Yes - memory cache for sub-millisecond lookups
3. **CLI commands**: ✅ Add `reed frame:list --crashed`, `reed frame:status`, `reed frame:rollback`
4. **Recovery mode**: ✅ Manual via CLI, EXCEPT if server fails to start → show recovery menu automatically

**Next Steps**: Implement Phase 1 (Foundation).
