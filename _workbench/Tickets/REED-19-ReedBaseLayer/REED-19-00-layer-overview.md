# REED-19: ReedBase Layer

## Layer Overview

**Status**: Design Phase  
**Priority**: Critical - Foundation Layer  
**Complexity**: Very High  
**Breaking Changes**: YES - Complete ReedBase rewrite

## Summary

Complete reimplementation of ReedBase as a versioned, concurrent-write capable database system with binary delta compression, row-level conflict resolution, function memoization, and distributed P2P synchronisation.

## Motivation

Current ReedBase (REED-02) has limitations:
- ❌ No versioning (only XZ backups)
- ❌ No concurrent writes (last-write-wins)
- ❌ No conflict resolution
- ❌ Inefficient full-file backups
- ❌ No row-level operations
- ❌ No function caching
- ❌ No distributed deployment
- ❌ No multi-location sync

**New ReedBase** provides:
- ✅ Git-like versioning with bsdiff deltas
- ✅ Concurrent writes with auto-merge
- ✅ Row-level conflict resolution
- ✅ Binary delta compression (95%+ savings)
- ✅ Function result caching
- ✅ Encoded metadata (efficient logs)
- ✅ Multi-location deployment (P2P)
- ✅ Automatic synchronisation (rsync)
- ✅ Load-based query routing

## Architecture

```
.reed/
├── registry/              # Global lookups
│   ├── actions.dict       # Action codes
│   └── users.dict         # User codes
│
├── tables/                # All data tables
│   ├── text/
│   │   ├── current.csv    # Current version
│   │   ├── {ts}.bsdiff    # Binary deltas
│   │   └── version.log    # Encoded metadata
│   │
│   ├── routes/
│   ├── meta/
│   ├── users/
│   └── ...
│
├── schema/                # Type definitions
│   └── *.schema.toml
│
├── cache/
│   ├── tables/
│   │   └── *.hashmap      # Table caches
│   └── functions/
│       └── *.cache        # Function result caches
│
└── config.toml

~/.reedbase/               # Global registry
├── registry.toml          # Database registry
├── routing/
│   └── {db_name}/
│       ├── latency.csv    # P2P latency measurements
│       └── load.csv       # System load history
└── sync/
    └── {db_name}.log      # Sync daemon logs
```

## Key Concepts

### 1. Universal Table Structure

Every table follows same pattern:
- `current.csv` - Active version
- `{timestamp}.bsdiff` - Binary deltas (XZ compressed)
- `version.log` - Encoded metadata (pipe-delimited integers)

### 2. Concurrent Writes

Multiple users can write simultaneously:
- Each creates own bsdiff delta
- System auto-merges at row level
- Conflicts detected and presented for resolution
- 90%+ of concurrent writes auto-merge (different rows)

### 3. Binary Delta Compression

- bsdiff creates small deltas (50-500 bytes typical)
- XZ compression reduces further
- Init = delta from empty file
- Rollback = apply delta chain (Git-like)
- 95%+ disk savings vs full snapshots

### 4. Encoded Metadata

version.log format: `timestamp|action|user|base|size|rows|hash`
- All integers (except hash)
- Lookup tables resolve to human-readable
- 5x faster parsing, 50% smaller files

### 5. Function Caching

Functions can cache results:
- Input hash → cached result
- TTL-based expiration
- LRU eviction
- 100-500x speedup for expensive operations

### 6. P2P Distribution

Fully decentralised multi-location deployment:
- **NO master node** - all peers equal
- Each node measures latency independently
- Local-first with load-based forwarding
- Automatic sync via rsync over SSH
- Configurable topologies (Hub-Spoke, Mesh, Custom)

### 7. Name-Based Registry

Global database registry for easy access:
- `~/.reedbase/registry.toml` - Single source of truth
- Name-based resolution: `rdb db:query users_prod`
- Tracks all locations per database
- Explicit registration (no auto-discovery)

## Cross-Cutting Concerns

### Metrics & Observability

**Philosophy**: Monitoring is foundational, not an afterthought. Every module must be observable from day one.

**Reusable Module**: `src/reedcms/reedbase/metrics/`
- ONE implementation of MetricsCollector (singleton, thread-safe)
- Modules use `metrics::global()` for instrumentation
- NO copy-paste metric collection code

**Standard Framework**:

```rust
// Reusable types (defined once in metrics/types.rs)
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: MetricUnit,
    pub tags: HashMap<String, String>,
    pub timestamp: i64,
}

pub enum MetricType {
    Counter,      // Monotonically increasing (total operations)
    Gauge,        // Point-in-time value (queue depth, memory usage)
    Histogram,    // Distribution (latency percentiles)
    Timer,        // Duration measurement
}

pub enum MetricUnit {
    Nanoseconds, Microseconds, Milliseconds, Seconds,
    Bytes, Kilobytes, Megabytes,
    Count, Percent
}

// Global collector (singleton)
use crate::reedbase::metrics::global as metrics;

pub fn operation(&self) -> ReedResult<T> {
    let start = Instant::now();
    let result = self.operation_inner()?;
    
    metrics().record(Metric {
        name: "module_operation_latency".to_string(),
        value: start.elapsed().as_micros() as f64,
        unit: MetricUnit::Microseconds,
        tags: hashmap!{ "module" => "name" },
    });
    
    Ok(result)
}
```

**Every Ticket MUST Include**:

1. **Performance Metrics Table**
   - Minimum 5, maximum 12 metrics
   - Format: `{module}_{operation}_{type}` (e.g., `table_read_latency`)
   - Collection points (file:function)

2. **Alert Rules**
   - Minimum 2 CRITICAL, 2 WARNING alerts
   - Duration-based (avoid false positives)
   - Clear alert messages

3. **Implementation Example**
   - Shows ONE line `metrics().record()` calls
   - NO complex instrumentation code
   - Clean, minimal example

4. **Collection Strategy** (MANDATORY - all 4 fields):
   - **Sampling**: All operations | 10% sample | Per request
   - **Aggregation**: 1-minute | 5-minute rolling window
   - **Storage**: `.reedbase/metrics/{module}.csv`
   - **Retention**: 7 days raw, 90 days aggregated

5. **Why These Metrics Matter**
   - 3-5 key explanations
   - Business/technical impact
   - Thresholds reasoning

**Standard Section Template**:

```markdown
## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| {module}_{op}_latency | Histogram | μs | <100 | >200 | file.rs:function() |

### Alert Rules

**CRITICAL Alerts:**
- `metric > threshold` for duration → "Message"

**WARNING Alerts:**
- `metric > threshold` for duration → "Message"

### Implementation

\`\`\`rust
use crate::reedbase::metrics::global as metrics;

pub fn operation(&self) -> ReedResult<T> {
    let start = Instant::now();
    let result = self.operation_inner()?;
    
    metrics().record(Metric {
        name: "{module}_{operation}_latency".to_string(),
        value: start.elapsed().as_micros() as f64,
        unit: MetricUnit::Microseconds,
        tags: hashmap!{ "module" => "{name}" },
    });
    
    Ok(result)
}
\`\`\`

### Collection Strategy

- **Sampling**: All operations
- **Aggregation**: 1-minute rolling window
- **Storage**: \`.reedbase/metrics/{module}.csv\`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**Primary Metric**: Explanation
- Key point 1
- Key point 2
- Impact on system
```

**Design Principles**:
- **KISS**: One `metrics().record()` call per metric - nothing complex
- **DRY**: MetricsCollector implemented once, reused everywhere
- **Consistency**: Every ticket follows IDENTICAL section structure

See `_workbench/Tickets/templates/metrics-module-design.md` for complete implementation details.

---

### Frame System (Coordinated Batch Operations)

**Philosophy**: Batch operations must share a single timestamp for consistency. Frame-System provides coordinated operations with crash recovery.

**Purpose**: Enable atomic multi-table operations where all changes share ONE Unix timestamp:
- Schema migrations with data updates
- Multi-table backups
- Bulk imports across tables
- Index rebuilds with validation

**Reusable Module**: `src/reedcms/reedbase/frame/`
- ONE implementation of FrameManager (singleton, thread-safe)
- All batch operations use `Frame::begin()` → `commit()` pattern
- NO manual timestamp coordination

**Core Types** (in `src/reedcms/reedstream.rs`):

```rust
/// Frame: Coordinated set of operations sharing ONE timestamp
pub struct FrameInfo {
    pub id: Uuid,
    pub timestamp: i64,           // THE shared timestamp
    pub name: String,              // Human-readable name
    pub status: FrameStatus,
    pub tables_affected: usize,
    pub started_at: i64,
    pub committed_at: Option<i64>,
}

/// Frame status lifecycle
pub enum FrameStatus {
    Active,       // Frame is currently being built
    Committed,    // Frame successfully committed with snapshot
    RolledBack,   // Frame was rolled back (versionised)
    Crashed,      // Frame was active when system crashed
    Archived,     // Frame older than retention (snapshot deleted)
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

/// Frame snapshot: State of all tables at frame commit
pub type FrameSnapshot = HashMap<String, SnapshotEntry>;

pub struct SnapshotEntry {
    pub timestamp: i64,    // Actual bsdiff timestamp for this table
    pub hash: String,      // Content hash for integrity
}

/// Frame-specific errors
pub enum ReedError {
    // ... existing variants ...
    
    FrameAlreadyActive { frame_id: Uuid },
    FrameNotFound { frame_id: Uuid },
    NoFrameBeforeTimestamp { target: i64 },
    FrameSnapshotCorrupted { path: String },
    ServerStartAborted,
    InvalidChoice,
}
```

**Standard Usage Pattern**:

```rust
use crate::reedbase::frame::Frame;

pub fn migrate_schema(from: u32, to: u32) -> ReedResult<MigrationReport> {
    // 1. Begin frame - get ONE timestamp
    let mut frame = Frame::begin(&format!("schema_migration_{}_{}", from, to))?;
    let ts = frame.timestamp();  // All operations use THIS timestamp
    
    // 2. Perform operations
    write_schema_file(ts)?;
    frame.log_operation("write_schema", None);
    
    for table in affected_tables {
        migrate_table_data(table, ts)?;  // SAME timestamp
        frame.log_operation("migrate_data", Some(table));
    }
    
    rebuild_indices(ts)?;  // SAME timestamp
    frame.log_operation("rebuild_indices", None);
    
    // 3. Commit (creates snapshot automatically)
    let report = frame.commit()?;
    
    Ok(report)
}
```

**Key Features**:

1. **Snapshot Creation**: Frame commit automatically creates `.reed/frames/{timestamp}.snapshot.csv` with current state of ALL tables
2. **Frame Index**: Sorted list in `.reed/frames/index.csv` for O(log n) binary search lookup
3. **Frame Index Cache**: Memory-cached for sub-millisecond lookups (60s TTL)
4. **Crash Recovery**: Automatic rollback of incomplete frames on server start
5. **Versionised Rollback**: Rollback creates NEW version (forward recovery), no data loss
6. **TTL Cleanup**: Configurable retention (default 365 days) via `frame.retention.days` in `.reed/project.csv`

**File Structure**:

```
.reed/
└── frames/
    ├── index.csv                    # Sorted list of all frames (O(log n) lookup)
    ├── frame.log                    # Frame lifecycle events
    ├── {timestamp}.snapshot.csv     # Table states at commit time
    └── ...
```

**Frame Snapshot Format** (`.reed/frames/{timestamp}.snapshot.csv`):

```csv
table|timestamp|hash|frame_id
text|1736860800|abc123|uuid002
routes|1736860700|def456|uuid002
meta|1736860750|ghi789|uuid002
```

**Frame Index Format** (`.reed/frames/index.csv`):

```csv
timestamp|frame_id|name|status|tables_affected|committed_at
1736860800|uuid002|schema_migration_1_2|committed|4|1736860815
```

**CLI Commands**:

```bash
# List all frames
reed frame:list

# List only crashed frames
reed frame:list --crashed

# Show frame details
reed frame:status <frame-id>

# Rollback frame (with confirmation)
reed frame:rollback <frame-id> --confirm

# Cleanup old frames (respects retention policy)
reed frame:cleanup
```

**Performance Guarantees**:

| Operation | Without Frames | With Frames | Speedup |
|-----------|----------------|-------------|---------|
| Point-in-Time Recovery | O(Tables × Versions) | O(log Frames + Tables) | 100-500× |
| Rollback | O(Tables × Versions) | O(Tables) | 100-500× |
| Frame Lookup | N/A | O(log n) < 1ms | N/A |

**Design Principles**:
- **KISS**: Frame = ONE timestamp + snapshot at commit
- **DRY**: FrameManager singleton, reused everywhere
- **Consistency**: All batch operations use Frame-System
- **Zero Data Loss**: Rollback is versionised (forward recovery)

**Configuration**:

```csv
# .reed/project.csv
key|value|description
frame.retention.days|365|Frame snapshot retention (days)
```

```bash
# Change retention
reed set:project frame.retention.days 730  # 2 years
```

See `_workbench/Tickets/FRAME-SYSTEM-IMPLEMENTATION-PLAN.md` for complete implementation details.

---

## Layer Tickets

### Core Infrastructure (01-08)
- **REED-19-01**: Registry & Dictionary System
- **REED-19-02**: Universal Table API
- **REED-19-03**: Binary Delta Versioning (bsdiff)
- **REED-19-04**: Crash Recovery via CRC32 Validation & Delta Reconstruction
- **REED-19-05**: Concurrent Write System
- **REED-19-06**: Row-Level CSV Merge
- **REED-19-07**: Conflict Resolution
- **REED-19-08**: RBKS v2 Key Validation

### Advanced Features (09-11)
- **REED-19-09**: Column Schema Validation
- **REED-19-10**: Smart Indices (100-1000x faster queries)
- **REED-19-11**: Function System & Caching

### Query Layer (12)
- **REED-19-12**: CLI SQL-Like Query Interface (ReedQL)

### Migration & Testing (13-15)
- **REED-19-13**: Migration from REED-02
- **REED-19-14**: Performance Testing & Benchmarks
- **REED-19-15**: Documentation

### Distributed P2P System (16-18)
- **REED-19-16**: Database Registry & Name Resolution
- **REED-19-17**: Multi-Location Sync System (Rsync-based)
- **REED-19-18**: P2P Latency Measurement & Load-Based Query Routing

### Security & Compliance (19)
- **REED-19-19**: YubiKey Encryption System (Pro Feature)

## Impact Analysis

### What Changes

**Core Architecture:**
- `.reed/*.csv` → `.reed/tables/{name}/current.csv`
- Backup system → Versioning system (bsdiff deltas)
- Single-writer → Concurrent writers
- File-level ops → Row-level ops
- Path-based access → Name-based registry
- Single-location → Multi-location P2P

**APIs:**
- `reedbase::get::text()` - Same API, different implementation
- `reedbase::set::text()` - Now returns conflict info
- New: `reedbase::merge::resolve_conflict()`
- New: `reedbase::version::list()`, `rollback()`
- New: `reedbase::registry::*` - Registry management
- New: `reedbase::routing::*` - Query routing
- New: `reedbase::sync::*` - Multi-location sync

**CLI:**
- Existing commands mostly work (API compatible)
- New: `rdb version:*` commands
- New: `rdb conflict:*` commands
- New: `rdb dict:*` commands
- New: `rdb db:init` - Database initialization with locations
- New: `rdb db:register` - Register existing database
- New: `rdb db:list` - Show all registered databases
- New: `rdb db:nodes` - Show P2P node status
- New: `rdb db:sync` - Manual synchronisation
- New: `rdb db:measure:start` - Start latency measurement daemon
- Changed: `reed backup:*` → `rdb version:*`

### What Stays Same

- ✅ Pipe-delimited CSV format
- ✅ Key nomenclature (`lowercase.with.dots@lang`)
- ✅ HashMap caching (O(1) lookups)
- ✅ XZ compression
- ✅ Environment fallback (`@dev`, `@prod`)
- ✅ Multi-language support

### What Gets Better

- ✅ **10x faster writes** (deltas vs full files)
- ✅ **95% less disk usage** (binary deltas)
- ✅ **Concurrent writes** (no more "database locked")
- ✅ **Complete history** (every change tracked)
- ✅ **Conflict resolution** (row-level merge)
- ✅ **Function caching** (100-500x speedup)
- ✅ **Distributed deployment** (multi-location P2P)
- ✅ **Automatic failover** (load-based routing)
- ✅ **Global accessibility** (name-based registry)

## Migration Strategy

### Phase 1: Parallel Implementation

Build new ReedBase alongside old:
- `src/reedcms/reedbase/` - Old (keep working)
- `src/reedcms/reedbase_v2/` - New implementation
- Feature flag: `reedbase_v2`

### Phase 2: Testing

Extensive testing:
- Unit tests (100% coverage target)
- Integration tests (real-world scenarios)
- Performance benchmarks
- Concurrent write stress tests
- P2P sync stress tests
- Migration testing (old → new)

### Phase 3: Migration Command

```bash
rdb migrate:reedbase-v2
```

Migrates existing `.reed/` to new structure:
- Moves CSVs to tables/
- Creates initial version.log entries
- Generates schemas
- Preserves all data
- Creates registry entries

### Phase 4: Cutover

- Mark REED-02 as deprecated
- Make reedbase_v2 default
- Remove old code (after deprecation period)

## Risks

### High Risk

1. **Data Loss During Migration**
   - Mitigation: Automatic backup before migration
   - Mitigation: Dry-run mode with validation
   - Mitigation: Rollback capability

2. **Performance Regression**
   - Mitigation: Extensive benchmarks
   - Mitigation: Keep old implementation as fallback
   - Mitigation: Performance tests in CI

3. **Breaking Existing Integrations**
   - Mitigation: API compatibility layer
   - Mitigation: Gradual deprecation warnings
   - Mitigation: Comprehensive docs

4. **Network Failures in P2P System**
   - Mitigation: Local-first architecture (always works offline)
   - Mitigation: Automatic retry with exponential backoff
   - Mitigation: Health monitoring and automatic failover

### Medium Risk

1. **Concurrent Write Bugs**
   - Mitigation: Extensive stress testing
   - Mitigation: Conservative auto-merge (ask on doubt)

2. **Merge Logic Errors**
   - Mitigation: Formal test suite with all cases
   - Mitigation: Manual override always available

3. **Disk Space Increase (Many Versions)**
   - Mitigation: Configurable max_versions
   - Mitigation: Automatic cleanup
   - Mitigation: Deltas are tiny (95% savings)

4. **Sync Conflicts in Multi-Location Setup**
   - Mitigation: Row-level conflict detection
   - Mitigation: Last-write-wins with conflict log
   - Mitigation: Manual resolution tools

## Success Criteria

- [ ] All REED-19 tickets completed (01-18)
- [ ] 100% test coverage for core functionality
- [ ] Performance benchmarks show improvement
- [ ] Migration tested on real data
- [ ] Documentation complete
- [ ] Zero data loss in migration
- [ ] API compatibility maintained
- [ ] Concurrent writes work reliably
- [ ] P2P sync works across 10+ nodes
- [ ] Query routing shows <10ms overhead
- [ ] Registry system handles 1000+ databases

## Timeline Estimate

**Conservative: 5-6 weeks full-time**

- Week 1: Core tables + versioning (REED-19-01 to REED-19-03)
- Week 2: Crash recovery + concurrent writes (REED-19-04 to REED-19-07)
- Week 3: Schemas + indices + functions (REED-19-08 to REED-19-11)
- Week 4: ReedQL + migration (REED-19-12 to REED-19-13)
- Week 5: P2P system (REED-19-16 to REED-19-18)
- Week 6: Testing + docs (REED-19-14 to REED-19-15)

**Aggressive: 3-4 weeks** (if parallel development)

## Open Questions

1. **Function caching TTL defaults?**
   - Proposal: Configurable per function, default 1 hour

2. **Max versions per table?**
   - Proposal: Default 32 (like current backup), configurable

3. **Conflict resolution timeout?**
   - Proposal: Block write until resolved, or queue for async resolution

4. **Schema enforcement level?**
   - Proposal: Warning mode first, strict mode optional

5. **Keep REED-02 as fallback?**
   - Proposal: Yes, feature flag for 6 months, then remove

6. **Default sync topology?**
   - Proposal: Hub-Spoke with local as hub, configurable

7. **Load threshold defaults?**
   - Proposal: CPU 80%, Memory 90%, configurable per database

8. **Measurement interval?**
   - Proposal: 30s default, configurable down to 10s

## References

- Current: REED-02 (Data Layer)
- Inspiration: Git (delta compression, versioning)
- Inspiration: PostgreSQL MVCC (concurrent writes)
- Inspiration: Redis (function memoization)
- Inspiration: Rsync (efficient file sync)
- Inspiration: Consul (distributed health checks)

## Notes

This is a **foundational change** that affects nearly every part of ReedCMS. While risky, it solves fundamental limitations and sets ReedCMS up for enterprise scale with distributed deployment.

The key insights:
- **CSV is structured data**, making row-level operations and intelligent merging possible (unlike Git's line-based text merging)
- **P2P without master** eliminates single point of failure and enables truly distributed deployment
- **Local-first routing** ensures zero-downtime even when all remotes are down
- **Name-based registry** makes database access intuitive and location-independent
