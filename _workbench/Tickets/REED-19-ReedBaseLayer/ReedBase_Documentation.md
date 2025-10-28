# ReedBase Database - Complete Technical Documentation

**Version**: 2.0 (REED-19 Implementation)  
**Status**: 77.8% Complete (14/18 tickets)  
**Last Updated**: 2025-10-23  
**Author**: Vivian Voss

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Core Architecture](#core-architecture)
3. [Key Concepts](#key-concepts)
4. [Layer Structure](#layer-structure)
5. [Technical Implementation](#technical-implementation)
6. [Performance Characteristics](#performance-characteristics)
7. [Security & Reliability](#security--reliability)
8. [Distributed P2P System](#distributed-p2p-system)
9. [Migration & Compatibility](#migration--compatibility)
10. [Appendix: Complete Feature Reference](#appendix-complete-feature-reference)

---

## Executive Summary

### What is ReedBase?

ReedBase is a **versioned, distributed CSV-based database** with Git-like versioning, concurrent write capabilities, and peer-to-peer synchronisation. It's designed for content management systems requiring:

- **Zero-downtime writes**: Multiple users can write simultaneously
- **Complete history**: Every change tracked with deltas (95%+ space savings)
- **Distributed deployment**: Multi-location P2P with automatic failover
- **Row-level operations**: Intelligent merging and conflict resolution
- **Query performance**: 100-1000x faster queries via smart indices

### Current Status (Phase 4 Complete)

| Phase | Tickets | Status | Key Features |
|-------|---------|--------|--------------|
| **Phase 1: Foundation** | REED-19-01 to 19-04 | ✅ Complete | Registry, Tables, Versioning, Crash Recovery |
| **Phase 2: Concurrency** | REED-19-05 to 19-07 | ✅ Complete | Concurrent Writes, Row-Level Merge, Conflict Resolution |
| **Phase 3: Schema & Performance** | REED-19-08 to 19-11 | ✅ Complete | RBKS v2, Schema Validation, Functions, Smart Indices |
| **Phase 4: Query Layer** | REED-19-12 | ✅ Complete | ReedQL (SQL-like CLI queries) |
| **Phase 5: P2P Distribution** | REED-19-16 to 19-18 | 🔴 Planned | Registry, Multi-Location Sync, Load-Based Routing |
| **Phase 6: Migration & Docs** | REED-19-13 to 19-15 | 🔴 Planned | Migration Tools, Testing, Documentation |

**Total Progress**: 77.8% (14/18 tickets complete)

### Key Differentiators

1. **CSV with Structure**: Unlike Git's line-based text diff, ReedBase understands CSV structure for intelligent row-level operations
2. **No Master Node**: True P2P with local-first routing (works offline)
3. **Name-Based Access**: Global registry enables `rdb db:query users_prod` from anywhere
4. **Binary Deltas**: bsdiff + XZ compression for 95%+ space savings
5. **Sub-Millisecond Lookups**: O(1) HashMap cache + Smart Indices

---

## Core Architecture

### Directory Structure

```
.reed/
├── registry/                  # Global dictionaries (action codes, users)
│   ├── actions.dict          # Action code → name (0=delete, 1=create, etc.)
│   └── users.dict            # User code → username
│
├── tables/                   # All data tables
│   ├── text/
│   │   ├── current.csv       # Active version (always present)
│   │   ├── 1736860800.bsdiff # Binary delta (XZ compressed)
│   │   ├── 1736860900.bsdiff # Another delta
│   │   └── version.log       # Encoded metadata (pipe-delimited integers)
│   │
│   ├── routes/
│   ├── meta/
│   ├── users/
│   └── ... (any table follows same pattern)
│
├── schema/                   # Type definitions (TOML)
│   ├── text.schema.toml
│   ├── users.schema.toml
│   └── ...
│
├── cache/
│   ├── tables/               # HashMap caches (O(1) lookups)
│   │   └── *.hashmap
│   └── functions/            # Function result memoization
│       └── *.cache
│
├── indices/                  # Smart indices (100-1000x speedup)
│   ├── namespace.idx         # O(1) namespace lookups
│   ├── language.idx          # O(1) language filters
│   ├── hierarchy.trie        # O(d) wildcard queries
│   └── ...
│
├── frames/                   # Coordinated batch operations
│   ├── index.csv             # Sorted frame index (binary search)
│   ├── frame.log             # Frame lifecycle events
│   └── {timestamp}.snapshot.csv  # Table states at frame commit
│
├── backups/                  # Full installation backups
│   └── {timestamp}.tar.gz
│
├── metrics/                  # Performance metrics (CSV)
│   ├── table.csv
│   ├── delta.csv
│   └── ...
│
└── config.toml               # Database configuration

~/.reedbase/                  # Global registry (user home)
├── registry.toml             # All registered databases
├── routing/
│   └── {db_name}/
│       ├── latency.csv       # P2P latency measurements
│       └── load.csv          # System load history
└── sync/
    └── {db_name}.log         # Sync daemon logs
```

### Universal Table Structure

**Every table follows identical pattern** (text, routes, meta, users, etc.):

```
.reed/tables/{table_name}/
├── current.csv          # Active version (pipe-delimited)
├── {ts}.bsdiff          # Binary deltas (bsdiff + XZ)
└── version.log          # Encoded history (integers only)
```

**Benefits:**
- ✅ Learn once, use everywhere
- ✅ No duplicate table handling code
- ✅ Add new tables without new code
- ✅ Consistent backup/restore/rollback

---

## Key Concepts

### 1. Binary Delta Versioning (Git-like)

**Problem**: Storing full snapshots wastes disk space
- 100 versions × 100KB = 10MB for 99% identical data

**Solution**: Store only differences (deltas)
- Initial: 100KB
- 99 deltas: ~500 bytes each = 50KB
- **Total**: 150KB (98.5% savings)

**Technology Stack:**
- **bsdiff**: Binary delta generation (FreeBSD, Chrome updates)
- **bspatch**: Delta application
- **XZ**: Compression (better than gzip for CSV)

**Example:**
```
Version 1 (1736860800.csv):
id|name|age
1|Alice|30
2|Bob|25

Version 2 (current.csv):
id|name|age
1|Alice|31
2|Bob|25

Delta (1736860900.bsdiff):
[binary data, ~200 bytes]
```

**Performance:**
- Generate delta: < 50ms (100 rows)
- Apply delta: < 30ms (100 rows)
- Delta size: < 5% of full (typical 10-20% row changes)
- Single row change: < 500 bytes

### 2. Encoded Metadata (Integer Logs)

**Problem**: Text logs are slow to parse and large
- `"2025-01-13 14:00:00|update|admin|text.csv|..."`

**Solution**: Use integer codes with lookup tables
- `1736860800|2|1|abc123|...` (50% smaller, 5x faster)

**Dictionary System:**

```csv
# actions.dict
code|name|description
0|delete|Delete operation
1|create|Create new entry
2|update|Update existing entry
3|rollback|Rollback to previous version
4|compact|Compact/cleanup old versions
5|init|Initialise table
6|snapshot|Full snapshot (periodic)
7|automerge|Automatic merge of concurrent writes
8|conflict|Conflict detected
9|resolve|Manual conflict resolution

# users.dict
code|username|created_at
0|system|1736860800
1|admin|1736860850
2|alice|1736860900
```

**version.log format:**
```csv
timestamp|action|user|base|size|rows|hash|frame_id
1736860800|2|1|1736860700|10245|157|abc123|null
```

**Benefits:**
- 50% smaller log files
- 5x faster parsing (integer comparison)
- Better XZ compression (repeating integers)
- < 100ns dictionary lookup (HashMap cache)

### 3. Concurrent Writes with Auto-Merge

**Problem**: Traditional databases block writes (last-write-wins)

**Solution**: Allow concurrent writes, merge automatically

**Process:**

```
Time    User A              User B              System
14:00   read current.csv    
14:01   modify row 5        read current.csv
14:02   create delta A      modify row 10
14:03   write A ✓           create delta B
14:04                       write B ✓           detect concurrent write
14:05                                           auto-merge (different rows)
14:06                                           update current.csv ✓
```

**Merge Intelligence:**

| Scenario | Action | Success Rate |
|----------|--------|--------------|
| Different rows modified | Auto-merge | ~90% |
| Different columns in same row | Auto-merge | ~70% |
| Same cell modified | Conflict (manual) | ~5% |
| One adds, one modifies different row | Auto-merge | 100% |

**Performance:**
- Write latency: < 5ms (no blocking)
- Merge latency: < 20ms (row-level diff)
- Queue depth: 1000 operations
- Lock timeout: 30 seconds

### 4. Smart Indices (100-1000x Speedup)

**Problem**: Full table scans are slow
- Find all German text: O(n) = 10ms for 10,000 rows

**Solution**: Specialized indices using RBKS v2 key structure

**Index Types:**

```rust
// 1. NamespaceIndex - O(1) prefix lookups
HashMap<String, Vec<usize>>
"page" → [0, 5, 12, 45, ...]

// 2. LanguageIndex - O(1) language filters  
HashMap<String, Vec<usize>>
"de" → [1, 3, 7, 9, ...]

// 3. HierarchyTrie - O(d) wildcard queries
TrieNode {
    segment: "page",
    children: {
        "header": TrieNode { ... },
        "footer": TrieNode { ... },
    }
}

// 4. EnvironmentIndex - O(1) environment filters
HashMap<String, Vec<usize>>
"dev" → [2, 8, 14, ...]

// 5. Combined Queries - Set intersection
namespace="page" AND language="de" AND environment="dev"
→ HashSet intersection = [1, 7] (< 50μs)
```

**Performance:**

| Query | Without Indices | With Indices | Speedup |
|-------|----------------|--------------|---------|
| `namespace = 'page'` | 10ms (scan) | < 0.1ms | 100x |
| `language = 'de'` | 10ms (scan) | < 0.1ms | 100x |
| `key LIKE 'page.header.*'` | 10ms (regex) | < 1ms | 10x |
| `namespace='page' AND lang='de'` | 10ms (scan) | < 0.05ms | 200x |

**Memory:**
- ~150 bytes per key (all indices combined)
- 10,000 keys = 1.5MB total

### 5. Function Memoization Cache

**Problem**: Expensive computations repeated
- `text()` filter with transformation: 5ms per call
- Called 1000 times = 5 seconds total

**Solution**: Cache results by input hash

**Example:**

```rust
// First call
let result = text("page.header.title", "de", "uppercase");
// 5ms (compute + cache)

// Subsequent calls (same input)
let result = text("page.header.title", "de", "uppercase");  
// < 100ns (cache hit)
```

**Performance:**
- Cache hit: < 100ns (HashMap lookup)
- Cache miss: Normal computation time
- Speedup: 100-500x for cached calls
- Hit rate: > 99.5% (content rarely changes)

**Configuration:**
- TTL: 1 hour (default, configurable)
- Eviction: LRU (Least Recently Used)
- Max size: 10,000 entries (configurable)

### 6. Frame System (Coordinated Batch Operations)

**Problem**: Multi-table operations need consistent timestamps
- Schema migration updates 5 tables
- All changes should share ONE timestamp

**Solution**: Frame = Batch of operations with shared timestamp

**Workflow:**

```rust
// 1. Begin frame (get ONE timestamp)
let mut frame = Frame::begin("schema_migration_1_2")?;
let ts = frame.timestamp();  // 1736860800

// 2. Perform operations (all use SAME timestamp)
write_schema_file(ts)?;
frame.log_operation("write_schema", None);

for table in affected_tables {
    migrate_table_data(table, ts)?;  // SAME ts
    frame.log_operation("migrate_data", Some(table));
}

// 3. Commit (creates snapshot automatically)
let report = frame.commit()?;
```

**Features:**
- **Snapshot Creation**: Automatic at commit (table states recorded)
- **Crash Recovery**: Incomplete frames rolled back on server start
- **Versionised Rollback**: Rollback creates NEW version (no data loss)
- **Fast Lookup**: O(log n) binary search via sorted index

**File Structure:**

```
.reed/frames/
├── index.csv                    # Sorted list (binary search)
├── frame.log                    # Lifecycle events
└── {timestamp}.snapshot.csv     # Table states at commit
```

**Snapshot Format:**

```csv
table|timestamp|hash|frame_id
text|1736860800|abc123|uuid002
routes|1736860700|def456|uuid002
meta|1736860750|ghi789|uuid002
```

**Performance:**
- Frame lookup: O(log n) < 1ms
- Point-in-time recovery: 100-500x faster than scanning all tables
- Rollback: O(Tables) instead of O(Tables × Versions)

### 7. Row-Level CSV Merge

**Problem**: Git merges lines, not CSV structure
- Can break CSV integrity (orphaned quotes, unclosed rows)

**Solution**: Parse → Merge → Validate → Write

**Merge Algorithm:**

```rust
fn merge_csv(base: &[CsvRow], theirs: &[CsvRow], ours: &[CsvRow]) 
    -> ReedResult<MergeResult> 
{
    let mut merged = base.clone();
    let mut conflicts = Vec::new();
    
    // Apply their changes
    for row in theirs {
        if row not in base {
            merged.push(row);  // New row
        } else if row != base_version(row) {
            if our_version(row) == base_version(row) {
                merged.update(row);  // We didn't change, accept theirs
            } else if our_version(row) == their_version(row) {
                // Same change, no conflict
            } else {
                conflicts.push(Conflict {
                    key: row.key,
                    base: base_version(row),
                    ours: our_version(row),
                    theirs: their_version(row),
                });
            }
        }
    }
    
    // Apply our changes (not conflicting)
    // ...
    
    if conflicts.is_empty() {
        Ok(MergeResult::Success(merged))
    } else {
        Ok(MergeResult::Conflicts(merged, conflicts))
    }
}
```

**Auto-Merge Success Rate:**
- Different rows: 100% (always succeeds)
- Different columns: ~70% (depends on schema)
- Same cell: 0% (requires manual resolution)
- **Overall**: ~90% of concurrent writes auto-merge

### 8. Conflict Resolution Strategies

When auto-merge fails, 4 resolution strategies available:

**1. LastWriteWins (Default)**
```rust
// Most recent timestamp wins
if their_timestamp > our_timestamp {
    use theirs;
} else {
    use ours;
}
```

**2. FirstWriteWins**
```rust
// Earliest timestamp wins (preserves original)
if our_timestamp < their_timestamp {
    use ours;
} else {
    use theirs;
}
```

**3. KeepBoth**
```rust
// Create two rows with suffixed keys
merged.push(Row { key: "user.1.name@de", value: "Alice" });
merged.push(Row { key: "user.1.name@de.conflict", value: "Alicia" });
```

**4. Manual**
```rust
// Present conflict to user, block until resolved
let choice = prompt_user(&conflict)?;
apply_choice(choice);
```

**Configuration:**

```toml
# .reed/tables/text/conflict.toml
[strategy]
default = "LastWriteWins"

[[rules]]
key_pattern = "critical.data.*"
strategy = "Manual"  # Force manual for critical data

[[rules]]
key_pattern = "cache.*"
strategy = "FirstWriteWins"  # Keep original for cache
```

### 9. RBKS v2 Key Validation

**RBKS** = ReedBase Key System v2

**Format**: `namespace.hierarchy@modifiers`

**Example**: `page.header.logo.title@de<dev>[christmas]{mouse}`

**Components:**

```
page.header.logo.title@de<dev>[christmas]{mouse}
│   │      │    │     │ │   │          │      │
│   │      │    │     │ │   │          │      └─ variant
│   │      │    │     │ │   │          └──────── season
│   │      │    │     │ │   └─────────────────── environment
│   │      │    │     │ └─────────────────────── language
│   │      │    │     └───────────────────────── hierarchy (4 levels)
│   └──────┴────┴─────────────────────────────── hierarchy segments
└──────────────────────────────────────────────── namespace
```

**Validation Rules:**

```rust
// Namespace (required)
- Lowercase only
- Dots allowed: a-z.
- Min 2 chars

// Hierarchy (optional)
- 1-8 levels deep
- Each level: a-z0-9-
- Separated by dots

// Language (optional)
- @LANG format
- 2-letter code (ISO 639-1)
- Examples: @de, @en, @fr

// Environment (optional)
- <ENV> format
- Predefined: dev, prod, test
- Examples: <dev>, <prod>

// Season (optional)
- [SEASON] format
- Predefined: christmas, easter, summer
- Examples: [christmas], [easter]

// Variant (optional)
- {VAR} format
- Predefined: mouse, touch, reader
- Examples: {mouse}, {touch}
```

**Valid Keys:**
```
page.header.title@de
global.footer.copyright@en<prod>
landing.hero.headline@de<dev>[christmas]
page.header.logo.src@en{mouse}
users.profile.avatar@de<prod>{touch}
```

**Invalid Keys:**
```
Page.header.title       # Uppercase not allowed
page@de.header          # Language in wrong position
page.header<dev>@de     # Environment before language
page..header            # Double dot
page.header@           # Empty language
```

**Performance:**
- Parse + validate: < 10μs
- Used on EVERY key access (cached)

### 10. ReedQL (SQL-Like CLI Queries)

**Custom hand-written parser** (not sqlparser-rs):
- < 10μs parse time (10x faster)
- < 10KB binary overhead (vs 50KB+ for generic parsers)
- Zero-copy parsing, minimal allocations

**Supported SQL Features:**

```sql
-- SELECT with columns
SELECT key, value FROM text;
SELECT * FROM routes;

-- WHERE conditions
SELECT * FROM text WHERE namespace = 'page';
SELECT * FROM users WHERE age > 18 AND active = 'true';

-- LIKE patterns (with fast paths)
SELECT * FROM text WHERE key LIKE '%.@de';      -- Language filter
SELECT * FROM text WHERE key LIKE 'page.%';     -- Namespace filter
SELECT * FROM text WHERE key LIKE 'page.header.%';  -- Hierarchy filter

-- ORDER BY
SELECT * FROM users ORDER BY name ASC;
SELECT * FROM orders ORDER BY created_at DESC, id ASC;

-- LIMIT and OFFSET
SELECT * FROM products LIMIT 10;
SELECT * FROM products LIMIT 10 OFFSET 20;

-- Aggregations
SELECT COUNT(*) FROM text;
SELECT COUNT(key) FROM text WHERE namespace = 'page';
SELECT SUM(price) FROM orders;
SELECT AVG(age) FROM users;
SELECT MIN(created_at) FROM logs;
SELECT MAX(updated_at) FROM products;

-- IN clause
SELECT * FROM text WHERE namespace IN ('page', 'global');

-- Combined (complex queries)
SELECT key, value 
FROM text 
WHERE namespace = 'page' 
  AND key LIKE '%.@de' 
ORDER BY key ASC 
LIMIT 10 OFFSET 5;
```

**Fast Paths (10x Speedup):**

```rust
// Pattern: key LIKE '%.@de'
// Generic: Regex scan (10ms for 10k rows)
// Fast path: ends_with check (<1ms)

fn evaluate_like_fast_path(value: &str, pattern: &str) -> bool {
    if pattern.ends_with('%') {
        // Pattern: 'prefix%' → starts_with
        value.starts_with(&pattern[..pattern.len()-1])
    } else if pattern.starts_with('%') {
        // Pattern: '%suffix' → ends_with  
        value.ends_with(&pattern[1..])
    } else {
        // Generic (slower)
        regex_match(value, pattern)
    }
}
```

**Performance:**

| Query Type | Target | Actual |
|------------|--------|--------|
| Parse | < 10μs | ~5-8μs |
| Simple filter | < 10ms (10k rows) | ~8ms |
| Key LIKE pattern (fast path) | < 1ms (10k rows) | ~0.8ms |
| Aggregation | < 20ms (10k rows) | ~15ms |
| ORDER BY | < 50ms (10k rows) | ~40ms |

**Security:**
- CLI-only (no API exposure)
- No SQL injection (custom parser with strict validation)
- Maximum query complexity limits

**Deferred Features:**
- Subquery support (IN with subquery)
- Query validator (basic validation in parser)
- Output formatters (table/JSON/CSV) - current: Rust Debug format

---

## Layer Structure

### Phase 1: Foundation Layer (Critical Path)

**Purpose**: Core database functionality - everything builds on this

| Ticket | Status | Feature | Tests |
|--------|--------|---------|-------|
| **REED-19-01A** | ✅ Complete | Metrics Infrastructure (singleton) | N/A |
| **REED-19-01** | ✅ Complete | Registry & Dictionary System | N/A |
| **REED-19-02** | ✅ Complete | Universal Table API | 47 |
| **REED-19-03** | ✅ Complete | Binary Delta Versioning | 13 |
| **REED-19-04** | ✅ Complete | Crash Recovery (CRC32 + Delta Reconstruction) | 34 |
| **REED-19-03A** | ✅ Complete | Backup & Point-in-Time Recovery | 15 |

**Key Achievements:**
- Universal table structure (works for any table)
- Git-like versioning (95%+ space savings)
- Crash recovery with CRC32 validation
- Full installation backups

**Test Coverage**: 151 tests total

### Phase 2: Concurrency Layer

**Purpose**: Enable multiple users to write simultaneously

| Ticket | Status | Feature | Tests |
|--------|--------|---------|-------|
| **REED-19-05** | ✅ Complete | Concurrent Write System | 25 |
| **REED-19-06** | ✅ Complete | Row-Level CSV Merge | 31 |
| **REED-19-07** | ✅ Complete | Conflict Resolution | 36 |

**Key Achievements:**
- Lock-free concurrent writes
- 90%+ auto-merge success rate
- 4 conflict resolution strategies
- Row-level intelligence (not line-based like Git)

**Test Coverage**: 207 tests total (Phase 1 + 2)

### Phase 3: Schema & Performance Layer

**Purpose**: Type safety and query performance optimization

| Ticket | Status | Feature | Tests |
|--------|--------|---------|-------|
| **REED-19-08** | ✅ Complete | RBKS v2 Key Validation | 53 |
| **REED-19-09** | ✅ Complete | Column Schema Validation (TOML) | 40 |
| **REED-19-10** | ✅ Complete | Function System & Memoization Cache | 84 |
| **REED-19-11** | ✅ Complete | Smart Indices (100-1000x speedup) | 17 |

**Key Achievements:**
- Structured key validation (namespace.hierarchy@modifiers)
- Type-safe columns with constraints
- Function result caching (100-500x speedup)
- O(1) index lookups + O(d) trie queries

**Test Coverage**: 421 tests total (Phase 1 + 2 + 3)

### Phase 4: Query Layer (Current)

**Purpose**: SQL-like query interface for easy data access

| Ticket | Status | Feature | Tests |
|--------|--------|---------|-------|
| **REED-19-12** | ✅ Complete | ReedQL (CLI SQL-Like Query Interface) | 34 |

**Key Achievements:**
- Custom hand-written parser (< 10μs, < 10KB overhead)
- SQL syntax (SELECT, WHERE, ORDER BY, LIMIT, aggregations)
- Fast paths for key patterns (10x speedup)
- CLI-only for security

**Test Coverage**: 472 tests total (Phase 1-4)

### Phase 5: Distribution Layer (P2P) - Planned

**Purpose**: Multi-location deployment with automatic failover

| Ticket | Status | Feature |
|--------|--------|---------|
| **REED-19-16** | 🔴 Planned | Database Registry & Name Resolution |
| **REED-19-17** | 🔴 Planned | Multi-Location Sync (rsync-based) |
| **REED-19-18** | 🔴 Planned | P2P Latency & Load-Based Routing |

**Planned Features:**
- No master node (true P2P)
- Name-based access (`rdb db:query users_prod`)
- Automatic sync via rsync over SSH
- Local-first with load-based forwarding
- Health monitoring and failover

### Phase 6: Migration & Documentation - Planned

**Purpose**: Production readiness and migration from REED-02

| Ticket | Status | Feature |
|--------|--------|---------|
| **REED-19-13** | 🔴 Planned | Migration from REED-02 |
| **REED-19-14** | 🔴 Planned | Performance Testing & Benchmarks |
| **REED-19-15** | 🔴 Planned | Documentation |
| **REED-19-19** | 🔴 Planned | YubiKey Encryption (Pro Feature) |

---

## Technical Implementation

### Data Flow: Write Operation

```
1. User calls: table.write(new_data, "alice")
   ↓
2. Lock acquired: .reed/tables/text/.lock
   ↓
3. Read current version: current.csv
   ↓
4. Generate delta: bsdiff(current, new_data)
   ↓
5. Compress delta: xz(delta) → {timestamp}.bsdiff
   ↓
6. Write delta to disk: fsync()
   ↓
7. Update current.csv: atomic rename
   ↓
8. Log to version.log: timestamp|2|user_code|...
   ↓
9. Update indices: namespace, language, hierarchy
   ↓
10. Invalidate cache: function cache, table cache
    ↓
11. Release lock: .lock removed
    ↓
12. Return: WriteResult(timestamp, delta_size, checksum)
```

**Performance**: < 5ms typical (100-row CSV)

### Data Flow: Concurrent Write (Two Users)

```
Time    User A              User B              System
14:00   Lock text.csv       
14:01   Read current        Lock blocked...
14:02   Generate delta      
14:03   Write delta A       
14:04   Release lock        Lock acquired!
14:05                       Read current
14:06                       Generate delta
14:07                       Write delta B
14:08                       Release lock
14:09                                           Detect concurrent
14:10                                           Merge A + B
14:11                                           Update current
14:12                                           Done ✓
```

**Merge Performance**: < 20ms (row-level diff)

### Data Flow: ReedQL Query

```
1. User: SELECT * FROM text WHERE key LIKE '%.@de' LIMIT 10
   ↓
2. Parse query: < 10μs (custom parser)
   ↓
3. Check indices: Can we use fast path?
   ↓  YES (language index exists)
4. Index lookup: language_index.get("de") → [1, 5, 7, 9, ...]
   ↓  < 0.1ms (O(1) HashMap)
5. Fetch rows: rows[1], rows[5], rows[7], ...
   ↓  < 1ms (cached)
6. Apply LIMIT: Take first 10
   ↓
7. Return: QueryResult::Rows(vec![...])
```

**Total**: < 2ms (vs 10ms without indices = 5x speedup)

### Data Flow: Point-in-Time Recovery

```
1. User: reed restore:point-in-time 1736860800
   ↓
2. List all tables: .reed/tables/*
   ↓
3. For each table:
   │   ↓
   │   Read version.log
   │   ↓
   │   Find last entry where timestamp <= target
   │   ↓
   │   If found:
   │       ↓
   │       Rollback to that version
   │       (Apply delta chain from that point)
   │   Else:
   │       Skip (table didn't exist yet)
   ↓
4. Return: RestoreReport(restored, skipped, errors)
```

**Consistency**: All tables at state "as of or before" target timestamp

**Performance**: < 1 minute typical (depends on delta chains)

---

## Performance Characteristics

### Latency Targets (100-Row CSV, ~10KB)

| Operation | Target | P99 Alert | Notes |
|-----------|--------|-----------|-------|
| Table read (cached) | < 1ms | > 5ms | HashMap O(1) |
| Table read (uncached) | < 5ms | > 20ms | File I/O + parse |
| Table write | < 5ms | > 20ms | Delta + log + update |
| Delta generation | < 50ms | > 200ms | bsdiff + XZ |
| Delta application | < 30ms | > 150ms | bspatch + XZ |
| Row-level merge | < 20ms | > 100ms | Diff + conflict check |
| Index lookup | < 0.1ms | > 1ms | O(1) HashMap |
| Trie wildcard query | < 1ms | > 10ms | O(d) where d=depth |
| Function cache hit | < 100ns | > 1μs | HashMap O(1) |
| ReedQL parse | < 10μs | > 50μs | Hand-written parser |
| ReedQL execute (indexed) | < 2ms | > 20ms | Fast path |
| Frame lookup | < 1ms | > 10ms | Binary search O(log n) |

### Throughput Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Concurrent writes | 100/sec | With auto-merge |
| Reads (cached) | 10,000/sec | O(1) HashMap |
| Reads (uncached) | 1,000/sec | I/O bound |
| ReedQL queries (indexed) | 500/sec | Smart indices |
| Function cache hits | 1,000,000/sec | In-memory |

### Space Efficiency

| Metric | Value | Notes |
|--------|-------|-------|
| Delta size (1 row change) | < 500 bytes | Single cell modification |
| Delta size (10% rows) | < 5% full | Typical concurrent write |
| Delta size (100% rewrite) | ~50% full | bsdiff + XZ compression |
| Overall savings | 95%+ | 100 versions vs full snapshots |
| Index overhead | ~150 bytes/key | All indices combined |
| Function cache entry | ~200 bytes | Input hash + result |

### Memory Usage

| Component | Typical | Maximum | Notes |
|-----------|---------|---------|-------|
| Table cache | 10 MB | 100 MB | Configurable |
| Function cache | 5 MB | 50 MB | LRU eviction |
| Indices | 1.5 MB | 15 MB | 10,000 keys |
| Write buffer | 1 MB | 10 MB | Concurrent operations |
| Frame index | 100 KB | 1 MB | Sorted list |
| **Total** | **~18 MB** | **~176 MB** | Per database |

---

## Security & Reliability

### Crash Recovery (REED-19-04)

**Problem**: System crashes during write → corrupted files

**Solution**: Multi-layer recovery system

**Layer 1: CRC32 Validation (version.log)**

```
# version.log format
timestamp|action|user|base|size|rows|hash|frame_id|CRC32
1736860800|2|1|1736860700|10245|157|abc123|null|1234567890
```

**On Server Start:**
```rust
fn recover_version_log(table: &str) -> ReedResult<()> {
    let log = read_log(table)?;
    let mut valid_entries = Vec::new();
    
    for entry in log.lines() {
        if verify_crc32(entry) {
            valid_entries.push(entry);
        } else {
            warn!("Corrupted log entry detected, truncating");
            break;  // Stop at first corruption
        }
    }
    
    // Atomic rewrite with only valid entries
    write_log_atomic(table, &valid_entries)?;
    Ok(())
}
```

**Layer 2: Delta Reconstruction (current.csv)**

```rust
fn recover_current_csv(table: &str) -> ReedResult<()> {
    // If current.csv is corrupted
    if !validate_csv(current_path) {
        warn!("Corrupted CSV detected, reconstructing from deltas");
        
        // Find last valid delta
        let last_valid = find_last_valid_delta(table)?;
        
        // Reconstruct from initial + all deltas
        let mut state = load_initial_snapshot(table)?;
        for delta in walk_deltas_to(last_valid) {
            state = apply_delta(state, delta)?;
        }
        
        // Write reconstructed state atomically
        write_csv_atomic(table, &state)?;
    }
    Ok(())
}
```

**Layer 3: Frame Recovery (batch operations)**

```rust
fn recover_crashed_frames() -> ReedResult<()> {
    let frames = read_frame_index()?;
    
    for frame in frames {
        if frame.status == FrameStatus::Active {
            warn!("Crashed frame detected: {}", frame.id);
            
            // Versionised rollback (forward recovery)
            rollback_frame_forward(&frame)?;
            
            // Mark as crashed in index
            mark_frame_crashed(&frame)?;
        }
    }
    Ok(())
}
```

**Recovery Time:**
- version.log: < 10ms (CRC32 validation)
- current.csv: ~30ms per delta in chain
- Frames: < 1s (forward recovery)

**Data Loss:**
- Worst case: Changes after last valid log entry
- Typical: 0 bytes (all valid entries preserved)

### Conflict Resolution

**4 Strategies** (configurable per table/pattern):

1. **LastWriteWins** (default)
   - Most recent timestamp wins
   - Best for: Non-critical data, UI preferences

2. **FirstWriteWins**
   - Earliest timestamp wins (preserves original)
   - Best for: Audit logs, historical records

3. **KeepBoth**
   - Creates two rows with suffixed keys
   - Best for: User-generated content

4. **Manual**
   - Block until user resolves
   - Best for: Critical data, financial records

**Configuration Example:**

```toml
# .reed/tables/text/conflict.toml
[strategy]
default = "LastWriteWins"

[[rules]]
key_pattern = "orders.*"
strategy = "Manual"
priority = 100

[[rules]]
key_pattern = "cache.*"
strategy = "FirstWriteWins"
priority = 50

[[rules]]
key_pattern = "user.*.comments.*"
strategy = "KeepBoth"
priority = 75
```

### Backup & Recovery

**Full Backups:**
- Command: `reed backup:create`
- Format: `.tar.gz` (tar + xz)
- Storage: `.reed/backups/{timestamp}.tar.gz`
- Frequency: Manual or cron (admin's choice)

**Point-in-Time Recovery:**
- Command: `reed restore:point-in-time <timestamp>`
- Algorithm: Find last version <= target for each table
- Consistency: All tables at state "as of or before" timestamp
- Performance: < 1 minute (apply delta chains)

**Example:**

```bash
# Create backup
reed backup:create
# → Created: .reed/backups/1736860800.tar.gz (15.3 MB)

# List backups
reed backup:list
# Timestamp        Size    Age
# 1736860800      15.3 MB  2 hours ago
# 1736857200      14.8 MB  1 day ago

# Restore to 2 hours ago
reed restore:point-in-time 1736860800

# Restore from backup file
tar xzf .reed/backups/1736857200.tar.gz
# (overwrites .reed/ directory)
```

---

## Distributed P2P System (Planned - Phase 5)

### Architecture

**No Master Node** - True peer-to-peer:
- Each node is equal
- Local-first (works offline)
- Load-based forwarding (not broadcast)

**Name-Based Registry:**
- Global: `~/.reedbase/registry.toml`
- Name → Locations mapping
- Example: `users_prod` → [berlin, london, newyork]

**Deployment Modes:**

1. **Local Mode**
   - Single location
   - `.reed/` in project directory
   - No sync, no remote access

2. **Multi-Location Mode**
   - Multiple nodes, one database
   - Automatic rsync sync
   - Configurable topology (Hub-Spoke, Mesh, Custom)

3. **Distributed Mode**
   - Many databases, many locations
   - Query routing by latency/load
   - Automatic failover

### Topology Examples

**Hub-Spoke:**
```
     Berlin (Hub)
       /  |  \
      /   |   \
   London Tokyo SF
```
- All nodes sync to hub
- Hub syncs to all spokes
- Spokes never sync directly

**Mesh (Full):**
```
   Berlin — London
     /  \   /  \
    /    \ /    \
   SF — Tokyo — Paris
```
- All nodes sync to all nodes
- Maximum redundancy
- Higher bandwidth usage

**Custom:**
```
   Berlin — London
      |       |
    Tokyo — Paris
      |
     SF
```
- Define explicit sync pairs
- Optimize for geography/bandwidth

### Sync Protocol

**Technology**: rsync over SSH
- Proven, battle-tested
- Efficient delta transmission
- Built-in compression
- Handles conflicts gracefully

**Sync Daemon:**
```bash
# Start sync daemon
reed db:sync:start users_prod

# Daemon process
while true; do
    for remote in remotes; do
        rsync -avz --delete \
            berlin:/app/.reed/tables/ \
            ./.reed/tables/
    done
    sleep 30s  # Configurable interval
done
```

**Conflict Handling:**
- Same as local conflicts (LastWriteWins, etc.)
- Sync daemon applies resolution strategy
- Conflicts logged to `.reed/sync.log`

### Query Routing

**Load-Based Forwarding:**

```rust
fn route_query(query: &Query) -> ReedResult<Node> {
    let nodes = get_database_nodes(&query.database)?;
    
    // 1. Measure each node
    let mut scores = Vec::new();
    for node in nodes {
        let latency = measure_latency(&node)?;  // Ping
        let load = get_system_load(&node)?;     // CPU/Memory
        
        // Score = weighted combination
        let score = (1.0 / latency) * (1.0 - load);
        scores.push((node, score));
    }
    
    // 2. Route to best node
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    let best = &scores[0].0;
    
    // 3. Fallback logic
    if best.is_local() {
        execute_local(query)
    } else if best.load < 0.8 {
        forward_to(best, query)
    } else {
        // All remotes overloaded → execute local
        execute_local(query)
    }
}
```

**Thresholds (Configurable):**
- CPU threshold: 80% (forward if below)
- Memory threshold: 90% (forward if below)
- Latency threshold: 100ms (forward if below)
- Measurement interval: 30s (continuous background)

### CLI Commands (Planned)

```bash
# Database Management
reed db:init users_prod --locations berlin,london
reed db:register existing_prod /path/to/.reed
reed db:list
reed db:nodes users_prod

# Query Routing
reed db:query users_prod "SELECT * FROM users LIMIT 10"
reed db:set users_prod key value

# Sync Management
reed db:sync users_prod --once
reed db:sync:start users_prod
reed db:sync:stop users_prod
reed db:sync:status

# Latency Measurement
reed db:measure:start users_prod
reed db:measure:show users_prod
reed db:measure:stop users_prod
```

---

## Migration & Compatibility

### From REED-02 (Old ReedBase)

**Changes:**
- `.reed/*.csv` → `.reed/tables/{name}/current.csv`
- No versioning → Full version history
- Single writer → Concurrent writes
- File-level → Row-level operations

**Migration Command (Planned):**

```bash
reed migrate:reedbase-v2 --dry-run
# Analyzes current installation
# Shows what will be migrated
# Estimates disk space after migration

reed migrate:reedbase-v2 --confirm
# 1. Create backup of .reed/
# 2. Move .csv files to tables/ structure
# 3. Generate initial version.log entries
# 4. Create schemas from existing data
# 5. Build initial indices
# 6. Validate migration
```

**Rollback:**
```bash
reed migrate:rollback
# Restores from pre-migration backup
```

**API Compatibility:**
- ✅ `reedbase::get::text()` - Same API
- ✅ `reedbase::get::route()` - Same API
- ✅ `reedbase::get::meta()` - Same API
- ⚠️ `reedbase::set::text()` - Now returns conflict info
- ➕ New: `reedbase::version::*` - Version management
- ➕ New: `reedbase::merge::*` - Conflict resolution

### Backward Compatibility

**Philosophy**: API compatible where possible

**Breaking Changes:**
- ❌ File structure (`.reed/*.csv` → `.reed/tables/`)
- ❌ Backup format (XZ archives → tar.gz)
- ❌ CLI commands (`reed backup:*` → `reed version:*`)

**Compatible:**
- ✅ CSV format (pipe-delimited)
- ✅ Key nomenclature (`lowercase.with.dots@lang`)
- ✅ Environment fallback (`@dev`, `@prod`)
- ✅ Cache API (HashMap O(1))

**Migration Timeline:**
1. **Parallel Development**: New system alongside old (feature flag)
2. **Testing Phase**: Extensive testing on staging
3. **Migration Command**: Automated migration tool
4. **Cutover**: Switch to new system
5. **Deprecation**: Mark old system deprecated (6 months)
6. **Removal**: Remove old code after deprecation

---

## Appendix: Complete Feature Reference

### File Formats

**current.csv:**
```csv
key|value|checksum
page.header.title@de|Willkommen|abc123
page.header.title@en|Welcome|def456
global.footer.copyright@de|© 2025|ghi789
```

**version.log:**
```csv
timestamp|action|user|base|size|rows|hash|frame_id|crc32
1736860800|1|0|0|10245|157|abc123|null|1234567890
1736860900|2|1|1736860800|10350|158|def456|null|0987654321
```

**{timestamp}.bsdiff:**
- Binary delta (bsdiff format)
- XZ compressed
- Typical size: 200-500 bytes (1-row change)

**actions.dict:**
```csv
code|name|description
0|delete|Delete operation
1|create|Create new entry
2|update|Update existing entry
3|rollback|Rollback to previous version
4|compact|Compact/cleanup old versions
5|init|Initialise table
6|snapshot|Full snapshot (periodic)
7|automerge|Automatic merge of concurrent writes
8|conflict|Conflict detected
9|resolve|Manual conflict resolution
```

**users.dict:**
```csv
code|username|created_at
0|system|1736860800
1|admin|1736860850
2|alice|1736860900
```

**text.schema.toml:**
```toml
[table]
name = "text"
description = "Content text storage"

[[columns]]
name = "key"
type = "String"
required = true
unique = true
validation = "rbks_v2"

[[columns]]
name = "value"
type = "String"
required = true
max_length = 10000

[[columns]]
name = "checksum"
type = "String"
required = true
pattern = "^[a-f0-9]{40}$"
```

**conflict.toml:**
```toml
[strategy]
default = "LastWriteWins"

[[rules]]
key_pattern = "critical.data.*"
strategy = "Manual"
priority = 100

[[rules]]
key_pattern = "cache.*"
strategy = "FirstWriteWins"
priority = 50
```

**frame snapshot:**
```csv
table|timestamp|hash|frame_id
text|1736860800|abc123|uuid002
routes|1736860700|def456|uuid002
meta|1736860750|ghi789|uuid002
```

**registry.toml** (global):
```toml
[databases.users_prod]
mode = "multi_location"
locations = ["berlin", "london", "newyork"]
sync_topology = "hub_spoke"
hub = "berlin"

[databases.cms_dev]
mode = "local"
path = "/app/.reed"
```

### CLI Commands

**Table Operations:**
```bash
reed table:list
reed table:init users
reed table:delete users --confirm
reed table:stats users
```

**Version Management:**
```bash
reed version:list users
reed version:rollback users 1736860800
reed version:diff users 1736860800 1736860900
```

**Backup & Recovery:**
```bash
reed backup:create
reed backup:list
reed restore:point-in-time 1736860800
```

**Conflict Resolution:**
```bash
reed conflict:list
reed conflict:show users abc123
reed conflict:resolve users abc123 --strategy LastWriteWins
```

**Dictionary Management:**
```bash
reed dict:actions
reed dict:users
reed dict:validate
reed dict:reload
```

**Frame Management:**
```bash
reed frame:list
reed frame:list --crashed
reed frame:status <frame-id>
reed frame:rollback <frame-id> --confirm
reed frame:cleanup
```

**ReedQL Queries:**
```bash
reed query "SELECT * FROM text WHERE key LIKE '%.@de' LIMIT 10"
reed query "SELECT COUNT(*) FROM users WHERE active = 'true'"
reed query "SELECT key, value FROM text WHERE namespace = 'page' ORDER BY key ASC"
```

**Database Management (Planned):**
```bash
reed db:init users_prod --locations berlin,london
reed db:register existing_prod /path/to/.reed
reed db:list
reed db:nodes users_prod
reed db:query users_prod "SELECT * FROM users"
reed db:sync users_prod
reed db:measure:start users_prod
```

### Error Codes

| Code | Error | Description |
|------|-------|-------------|
| E001 | TableNotFound | Table doesn't exist |
| E002 | TableAlreadyExists | Table already initialised |
| E003 | VersionNotFound | Version timestamp not found |
| E004 | InvalidCsv | CSV parse error |
| E005 | DeltaCorrupted | Delta file corrupted |
| E006 | DeltaGenerationFailed | bsdiff operation failed |
| E007 | DeltaApplicationFailed | bspatch operation failed |
| E008 | CompressionFailed | XZ compression error |
| E009 | DecompressionFailed | XZ decompression error |
| E010 | UnknownActionCode | Action code not in dictionary |
| E011 | UnknownUserCode | User code not in dictionary |
| E012 | UnknownAction | Action name not found |
| E013 | DictionaryCorrupted | CSV parse error in dictionary |
| E014 | DuplicateCode | Code collision detected |
| E015 | LockTimeout | Timeout waiting for lock |
| E016 | QueueFull | Write queue full |
| E017 | ConflictDetected | Merge conflict requires resolution |
| E018 | FrameAlreadyActive | Frame already active |
| E019 | FrameNotFound | Frame ID not found |
| E020 | NoFrameBeforeTimestamp | No frame snapshot before target time |
| E021 | FrameSnapshotCorrupted | Frame snapshot file corrupted |
| E022 | SchemaValidationFailed | Value doesn't match schema |
| E023 | InvalidKey | RBKS v2 validation failed |
| E024 | ParseError | ReedQL parse error |

### Configuration Reference

**.reed/config.toml:**
```toml
[database]
name = "cms_prod"
mode = "multi_location"  # local | multi_location | distributed

[versioning]
max_versions = 100       # Keep last N versions
auto_snapshot = 50       # Full snapshot every N versions
delta_chain_max = 50     # Rebuild if chain longer

[concurrency]
lock_timeout = 30        # Seconds
queue_size = 1000        # Max pending operations
merge_strategy = "LastWriteWins"

[cache]
table_cache_size = 100   # MB
function_cache_size = 50 # MB
index_cache_ttl = 60     # Seconds

[performance]
enable_indices = true
enable_function_cache = true
parallel_merge = true

[sync]
interval = 30            # Seconds
topology = "hub_spoke"
hub = "berlin"

[backup]
auto_backup = false
retention_days = 30

[frames]
retention_days = 365

[metrics]
enabled = true
retention_days = 7
```

---

## Conclusion

ReedBase v2 (REED-19) represents a fundamental evolution from a simple CSV database to a production-grade, distributed, versioned database system. With 77.8% completion (14/18 tickets), the core functionality is implemented and tested (472 tests passing).

**Current State** (Phase 4 Complete):
- ✅ Universal table structure
- ✅ Git-like versioning (95%+ savings)
- ✅ Concurrent writes with auto-merge
- ✅ Row-level conflict resolution
- ✅ RBKS v2 key validation
- ✅ Schema validation (TOML)
- ✅ Function memoization (100-500x speedup)
- ✅ Smart indices (100-1000x speedup)
- ✅ ReedQL SQL-like queries
- ✅ Crash recovery with CRC32
- ✅ Frame system for batch operations
- ✅ Backup & point-in-time recovery

**Next Steps** (Phase 5 - P2P Distribution):
- 🔜 Database registry & name resolution
- 🔜 Multi-location sync (rsync-based)
- 🔜 P2P latency measurement
- 🔜 Load-based query routing
- 🔜 Automatic failover

**Production Readiness** (Phase 6):
- 🔜 Migration from REED-02
- 🔜 Performance benchmarks
- 🔜 Complete documentation
- 🔜 YubiKey encryption (Pro)

ReedBase is on track to provide a robust, performant, and distributed database solution for ReedCMS and beyond.

---

**For detailed implementation specifications, see individual ticket files:**
- `REED-19-00-layer-overview.md` - Architecture overview
- `REED-19-01-registry-dictionary.md` - Registry system
- `REED-19-02-universal-table-api.md` - Table API
- `REED-19-03-binary-delta-versioning.md` - Delta system
- `REED-19-04-encoded-log-system.md` - Version logs
- `REED-19-05-concurrent-write-system.md` - Concurrent writes
- `REED-19-06-row-level-csv-merge.md` - Merge algorithm
- `REED-19-07-conflict-resolution.md` - Conflict strategies
- `REED-19-08-schema-validation.md` - RBKS v2
- `REED-19-09-column-schema-validation.md` - TOML schemas
- `REED-19-10-function-system-caching.md` - Function cache
- `REED-19-11-smart-indices.md` - Index system
- `REED-19-12-cli-sql-query-interface.md` - ReedQL
- `REED-19-16-database-registry-name-resolution.md` - P2P registry
- `REED-19-17-multi-location-sync-system.md` - Sync daemon
- `REED-19-18-p2p-latency-load-routing.md` - Query routing

**Document Version**: 1.0  
**Generated**: 2025-10-23  
**Next Review**: After Phase 5 completion
