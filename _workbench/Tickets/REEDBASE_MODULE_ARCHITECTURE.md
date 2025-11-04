# ReedBase Module Architecture Summary

**Generated**: 2025-11-04  
**Purpose**: Comprehensive overview of all ReedBase modules for implementing new modules (e.g., B+-Tree)

---

## Table of Contents

1. [Overview](#overview)
2. [Core Architecture](#core-architecture)
3. [Module Details](#module-details)
4. [Common Patterns](#common-patterns)
5. [Dependencies Between Modules](#dependencies-between-modules)
6. [Implementation Guidelines](#implementation-guidelines)

---

## Overview

ReedBase is a CSV-based versioned distributed database with Git-like versioning and P2P distribution capabilities. It consists of 13 core modules, each with specific responsibilities following the KISS principle.

### Key Features

- **Binary Delta Versioning**: Space-efficient versioning using bsdiff
- **Three Deployment Modes**: Global, local, and distributed
- **Frame-System**: Coordinated batch operations with shared timestamp
- **Concurrent Writes**: File locking with automatic conflict resolution
- **Smart Indices**: O(1) lookups via HashMap indexing
- **Metrics & Observability**: Built-in performance monitoring

---

## Core Architecture

### Entry Point: `lib.rs`

**Public Exports**:
```rust
// Main modules
pub mod backup;
pub mod concurrent;
pub mod conflict;
pub mod error;
pub mod functions;
pub mod indices;
pub mod log;
pub mod merge;
pub mod metrics;
pub mod reedql;
pub mod registry;
pub mod schema;
pub mod tables;
pub mod version;

// Commonly used re-exports
pub use backup::{create_backup, list_backups, restore_point_in_time, BackupInfo, RestoreReport};
pub use error::{ReedError, ReedResult};
pub use metrics::{Metric, MetricType, MetricUnit, MetricsCollector};
```

### Error Handling: `error.rs`

**Type**: `ReedResult<T> = Result<T, ReedError>`

**Error Variants** (28 total):
```rust
pub enum ReedError {
    // Registry/Dictionary errors
    UnknownActionCode { code: u8 },
    UnknownUserCode { code: u32 },
    UnknownAction { name: String },
    DictionaryCorrupted { file: String, reason: String, line: usize },
    DuplicateCode { code: String, file: String },
    
    // File operation errors
    IoError { operation: String, reason: String },
    PermissionDenied { path: String },
    CsvError { file: String, operation: String, reason: String },
    
    // Table errors
    TableNotFound { name: String },
    TableAlreadyExists { name: String },
    
    // Version errors
    VersionNotFound { timestamp: u64 },
    LogCorrupted { reason: String },
    DeltaCorrupted { timestamp: u64, reason: String },
    DeltaGenerationFailed { reason: String },
    DeltaApplicationFailed { reason: String },
    
    // CSV format errors
    InvalidCsv { reason: String, line: usize },
    CorruptedLogEntry { line: usize, reason: String },
    
    // Compression errors
    CompressionFailed { reason: String },
    DecompressionFailed { reason: String },
    
    // Operation errors
    NotConfirmed { operation: String },
    ParseError { reason: String },
    CommandFailed { command: String, error: String },
    
    // Backup/Restore errors
    NoTablesFound,
    TableRestoreFailed { table: String, reason: String },
    
    // Concurrency errors
    LockTimeout { table: String, timeout_secs: u64 },
    QueueFull { table: String, size: usize },
    InvalidQueueFile { path: String },
    
    // Serialisation errors
    SerializationError { reason: String },
    DeserializationError { reason: String },
    
    // Schema errors
    SchemaNotFound { table: String },
    InvalidSchema { reason: String },
    ValidationError { column: String, reason: String, value: Option<String> },
}
```

**Convenience Conversions**:
```rust
impl From<std::io::Error> for ReedError {
    fn from(err: std::io::Error) -> Self {
        ReedError::IoError {
            operation: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}
```

---

## Module Details

### 1. Backup Module (`backup/`)

**Purpose**: Backup creation, listing, and point-in-time recovery using tar and existing version.log infrastructure.

**Structure**:
```
backup/
├── mod.rs          # Public API exports
├── create.rs       # Backup creation
├── list.rs         # Backup enumeration
├── restore.rs      # Point-in-time recovery
├── types.rs        # BackupInfo, RestoreReport
└── tests.rs        # Test suite
```

**Public API**:
```rust
pub use create::create_backup;
pub use list::list_backups;
pub use restore::restore_point_in_time;
pub use types::{BackupInfo, RestoreReport};
```

**Key Types**:
- `BackupInfo`: Metadata about a backup (timestamp, size, tables)
- `RestoreReport`: Results of a restore operation

**Usage Pattern**:
```rust
use reedbase::backup::{create_backup, list_backups, restore_point_in_time};

// Create backup
create_backup(base_path, "admin")?;

// List available backups
let backups = list_backups(base_path)?;

// Restore to specific point
restore_point_in_time(base_path, timestamp)?;
```

---

### 2. Concurrent Module (`concurrent/`)

**Purpose**: File locking, write queuing, and coordination for concurrent writes.

**Structure**:
```
concurrent/
├── mod.rs          # Public API exports
├── lock.rs         # File locking primitives
├── queue.rs        # Write queue management
├── types.rs        # CsvRow, PendingWrite, WriteOperation
├── lock_test.rs    # Lock tests
└── queue_test.rs   # Queue tests
```

**Public API**:
```rust
pub use lock::{acquire_lock, is_locked, wait_for_unlock, TableLock};
pub use queue::{count_pending, get_next_pending, queue_write, remove_from_queue};
pub use types::{CsvRow, PendingWrite, WriteOperation};
```

**Key Types**:
- `TableLock`: RAII lock guard for exclusive table access
- `PendingWrite`: Serialized write operation in queue
- `WriteOperation`: Insert/Update/Delete operations
- `CsvRow`: Row representation (key + values)

**Usage Pattern**:
```rust
use reedbase::concurrent::{acquire_lock, queue_write, WriteOperation};

// Try to acquire lock
match acquire_lock(base_path, "text", timeout_secs) {
    Ok(lock) => {
        // Perform write with exclusive access
        // Lock automatically released on drop
    }
    Err(_) => {
        // Queue write for later
        queue_write(base_path, "text", WriteOperation::Insert(row))?;
    }
}
```

---

### 3. Conflict Module (`conflict/`)

**Purpose**: Conflict resolution for concurrent CSV merge operations with multiple strategies.

**Structure**:
```
conflict/
├── mod.rs              # Public API exports
├── resolution.rs       # Conflict resolution logic
├── types.rs            # ConflictFile, Resolution, ResolutionStrategy
└── resolution_test.rs  # Resolution tests
```

**Public API**:
```rust
pub use resolution::{
    count_conflicts, delete_conflict_file, list_conflicts, 
    load_conflict_file, resolve_conflict, write_conflict_file,
};
pub use types::{ConflictFile, Resolution, ResolutionStrategy};
```

**Key Types**:
- `ResolutionStrategy`: LastWriteWins, FirstWriteWins, Manual, KeepBoth
- `ConflictFile`: TOML-based conflict representation
- `Resolution`: Resolved or Unresolved (with file path)

**Usage Pattern**:
```rust
use reedbase::conflict::{resolve_conflict, ResolutionStrategy};

let resolution = resolve_conflict(
    base_path,
    "text",
    "test.key",
    Some(base_row),
    change_a,
    change_b,
    ResolutionStrategy::LastWriteWins,
)?;

match resolution {
    Resolution::Resolved(row) => {
        // Apply resolved row
    }
    Resolution::Unresolved(conflict_path) => {
        // Manual resolution required
    }
}
```

---

### 4. Functions Module (`functions/`)

**Purpose**: Function system with automatic memoization caching for computed, aggregation, and transformation functions.

**Structure**:
```
functions/
├── mod.rs                  # Public API exports
├── aggregations.rs         # COUNT, SUM, AVG, MIN, MAX, GROUP_BY
├── cache.rs                # Global memoization cache
├── computed.rs             # calculate_age, full_name, days_since, etc.
├── transformations.rs      # normalize_email, slugify, trim, etc.
├── aggregations_test.rs    # Aggregation tests
├── cache_test.rs           # Cache tests
├── computed_test.rs        # Computed function tests
└── transformations_test.rs # Transformation tests
```

**Public API**:
```rust
pub mod aggregations;
pub mod cache;
pub mod computed;
pub mod transformations;

pub use cache::{get_cache, CacheKey, CacheStats, FunctionCache};
```

**Key Types**:
- `FunctionCache`: Global singleton with HashMap-based memoization
- `CacheKey`: Hash-based cache key (function + args)
- `CacheStats`: Hit rate, entry count, cache size

**Performance**:
- Cache hit: < 100ns (instant)
- Cache insert: < 10μs
- Computed functions: < 1μs (first), < 100ns (cached)
- Aggregations: 2-10ms (first, 10k rows), < 100ns (cached)
- Transformations: < 2μs (first), < 100ns (cached)

**Usage Pattern**:
```rust
use reedbase::functions::{computed, aggregations, transformations};

// Computed
let age = computed::calculate_age("1990-05-15")?; // "35"

// Aggregation
let total = aggregations::count("users")?; // "1250"
let avg_age = aggregations::avg("users", "age")?; // "35.00"

// Transformation
let email = transformations::normalize_email("John@Example.COM")?; // "john@example.com"
let slug = transformations::slugify("Hello World!")?; // "hello-world"

// Cache management
let stats = get_cache().stats();
println!("Hit rate: {:.2}%", stats.hit_rate());
get_cache().clear();
get_cache().invalidate_table("users");
```

---

### 5. Indices Module (`indices/`)

**Purpose**: Smart Indices for 100-1000x faster queries with O(1) lookups.

**Structure**:
```
indices/
├── mod.rs          # Public API exports
├── hierarchy.rs    # HierarchyTrie for wildcard queries
├── manager.rs      # IndexManager coordinator
├── modifier.rs     # ModifierIndex for language/environment/etc.
├── namespace.rs    # NamespaceIndex for prefix lookups
├── types.rs        # KeyIndex, Modifiers, QueryFilter
└── indices_test.rs # Integration tests
```

**Public API**:
```rust
pub use hierarchy::HierarchyTrie;
pub use manager::{IndexManager, IndexStats};
pub use modifier::ModifierIndex;
pub use namespace::NamespaceIndex;
pub use types::{KeyIndex, Modifiers, QueryFilter};
```

**Key Types**:
- `IndexManager`: Coordinates all indices with set intersection
- `QueryFilter`: Builder for combined queries
- `KeyIndex`: Row number mapping for O(1) lookups
- `Modifiers`: Language, environment, season, variant extractors

**Performance**:
- Single index lookup: < 1μs (O(1) HashMap)
- Hierarchy query: < 10μs (O(d) trie walk, d typically 2-4)
- Combined query (3 filters): < 50μs (3x O(1) + set intersection)
- Index build: < 50ms for 10,000 keys
- Memory: ~150 bytes/key (~1.5MB for 10k keys)

**Usage Pattern**:
```rust
use reedbase::indices::{IndexManager, QueryFilter};

// Build indices
let mut manager = IndexManager::new();
manager.build(base_path, "text")?;

// Single filter query
let filter = QueryFilter::new().with_language("de");
let rows = manager.query(&filter)?;

// Combined filter query
let filter = QueryFilter::new()
    .with_namespace("page")
    .with_language("de")
    .with_environment("prod");
let rows = manager.query(&filter)?; // Intersection of all 3 filters

// Hierarchical wildcard query
let filter = QueryFilter::new()
    .with_hierarchy(vec!["page".into(), "header".into(), "*".into()]);
let rows = manager.query(&filter)?; // All descendants of page.header
```

---

### 6. Log Module (`log/`)

**Purpose**: Encoded log system for version history with CRC32 validation.

**Structure**:
```
log/
├── mod.rs              # Public API exports
├── decoder.rs          # Log entry decoding
├── encoder.rs          # Log entry encoding
├── types.rs            # LogEntry, ValidationReport
├── validator.rs        # CRC32 validation
├── decoder_test.rs     # Decoder tests
├── encoder_test.rs     # Encoder tests
└── validator_test.rs   # Validator tests
```

**Public API**:
```rust
pub use decoder::{
    decode_log_entries, decode_log_entry, 
    filter_by_action, filter_by_time_range, filter_by_user,
};
pub use encoder::{calculate_size_savings, encode_log_entries, encode_log_entry};
pub use types::{LogEntry, ValidationReport};
pub use validator::{validate_and_truncate_log, validate_log};
```

**Key Types**:
- `LogEntry`: Timestamp, user, action, table, metadata
- `ValidationReport`: Valid count, corrupted lines, truncated status

**Usage Pattern**:
```rust
use reedbase::log::{encode_log_entry, decode_log_entry, validate_log};

// Encode log entry
let entry = LogEntry::new(timestamp, "admin", "update", "text", "Updated key");
let encoded = encode_log_entry(&entry)?;

// Decode log entry
let decoded = decode_log_entry(&encoded)?;

// Validate log file
let report = validate_log(log_path)?;
if report.corrupted_lines > 0 {
    // Handle corrupted entries
}
```

---

### 7. Merge Module (`merge/`)

**Purpose**: Row-level CSV merge with intelligent conflict detection.

**Structure**:
```
merge/
├── mod.rs      # Public API exports
├── csv.rs      # CSV merge operations
├── diff.rs     # Diff calculation and application
├── types.rs    # Conflict, MergeResult, MergeStats, RowChange
├── csv_test.rs # CSV merge tests
└── diff_test.rs # Diff tests
```

**Public API**:
```rust
pub use csv::{
    build_row_map, calculate_merge_stats, detect_conflicts, 
    merge_changes, merge_single, rows_equal,
};
pub use diff::{apply_changes, calculate_diff, count_changes};
pub use types::{Conflict, MergeResult, MergeStats, RowChange};
```

**Key Types**:
- `MergeResult`: Success or Conflicts (with conflict list)
- `MergeStats`: Added, modified, deleted, unchanged, conflicts
- `RowChange`: Insert, Update, Delete
- `Conflict`: Key, base, change_a, change_b

**Usage Pattern**:
```rust
use reedbase::merge::{merge_changes, calculate_diff};

// Calculate diff
let changes = calculate_diff(&base_rows, &new_rows)?;

// Merge changes
let result = merge_changes(&base_rows, &changes_a, &changes_b)?;

match result {
    MergeResult::Success(merged_rows) => {
        // Write merged rows
    }
    MergeResult::Conflicts(conflicts) => {
        // Resolve conflicts
    }
}
```

---

### 8. Metrics Module (`metrics/`)

**Purpose**: Performance monitoring and observability infrastructure.

**Structure**:
```
metrics/
├── mod.rs          # Public API exports
├── aggregator.rs   # Statistical calculations (p50, p95, p99)
├── collector.rs    # Global singleton collector
├── storage.rs      # CSV-based persistent storage
├── types.rs        # Metric, MetricType, MetricUnit
├── aggregator_test.rs
├── collector_test.rs
├── mod_test.rs
├── storage_test.rs
└── types_test.rs
```

**Public API**:
```rust
pub use aggregator::{calculate_stats, p50, p95, p99, MetricStats};
pub use collector::MetricsCollector;
pub use types::{Metric, MetricType, MetricUnit};
```

**Key Types**:
- `MetricsCollector`: Global singleton with RwLock
- `Metric`: Name, value, unit, timestamp, tags
- `MetricType`: Counter, Gauge, Histogram, Timer
- `MetricUnit`: Microseconds, Milliseconds, Bytes, Count, Percent

**Performance**:
- Record: O(1) - lock + push to buffer
- Flush: O(n) - write batched metrics to CSV
- Aggregation: O(n log n) - sorting for percentiles
- Storage: Append-only CSV (no seeks)

**Usage Pattern**:
```rust
use reedbase::metrics::{MetricsCollector, Metric, MetricUnit};

// Record a metric
let metric = Metric::new("query_duration", 1250.0, MetricUnit::Microseconds)
    .with_tag("table", "text")
    .with_tag("operation", "get");

MetricsCollector::global().record(metric);

// Flush to storage periodically
MetricsCollector::global().flush();
```

---

### 9. ReedQL Module (`reedql/`)

**Purpose**: SQL-like query interface optimized for ReedBase operations.

**Structure**:
```
reedql/
├── mod.rs      # Public API exports
├── executor.rs # Query execution engine
├── parser.rs   # Custom hand-written parser
└── types.rs    # ParsedQuery, FilterCondition, etc.
```

**Public API**:
```rust
pub use executor::execute;
pub use parser::parse;
pub use types::{
    AggregationFunction, AggregationType, FilterCondition, 
    LimitOffset, OrderBy, ParsedQuery, QueryResult, SortDirection,
};
```

**Key Features**:
- Fast Parsing: < 10μs parse time (10x faster than generic SQL parsers)
- ReedBase Optimized: Key pattern fast paths for 10x query speedup
- Subquery Support: Recursive IN subquery execution
- Aggregations: COUNT, SUM, AVG, MIN, MAX
- CLI-Only: No API exposure (security-by-design)

**Supported SQL Syntax**:
```sql
SELECT column1, column2, ... FROM table
WHERE condition1 AND condition2 ...
ORDER BY column ASC|DESC
LIMIT n OFFSET m

-- Aggregations
SELECT COUNT(*) FROM text
SELECT AVG(column) FROM text WHERE condition

-- Subqueries
SELECT * FROM text WHERE key IN (SELECT key FROM routes)
```

**Fast Paths**:
- `key LIKE '%.@de'` → Language filter (< 1ms for 10k rows)
- `key LIKE 'page.%'` → Namespace filter (< 1ms for 10k rows)
- `namespace = 'page'` → Direct index lookup (< 100μs)

**Performance Targets**:
- Parse: < 10μs
- Execute key LIKE pattern: < 1ms for 10k rows
- Execute simple filter: < 10ms for 10k rows
- Execute subquery: < 20ms for 10k + 10k rows

**Usage Pattern**:
```rust
use reedbase::reedql::{parse, execute};

// Parse query
let query = parse("SELECT * FROM text WHERE key LIKE '%.@de' LIMIT 10")?;

// Execute query
let result = execute(&query, &table)?;
```

---

### 10. Registry Module (`registry/`)

**Purpose**: Dictionary system for efficient integer encoding of frequently-used values.

**Structure**:
```
registry/
├── mod.rs              # Public API exports
├── dictionary.rs       # Dictionary lookup operations
├── init.rs             # Registry initialization
├── dictionary_test.rs  # Dictionary tests
└── init_test.rs        # Init tests
```

**Public API**:
```rust
pub use dictionary::{
    get_action_code, get_action_name, get_or_create_user_code, 
    get_username, reload_dictionaries, set_base_path,
};
pub use init::{init_registry, validate_dictionaries};
```

**Dictionary Structure**:
```text
.reed/registry/
├── actions.dict    # code|name|description
└── users.dict      # code|username|created_at
```

**Performance**:
- Lookups: O(1) HashMap cached, < 100ns
- User creation: < 10ms (CSV append + cache update)
- Memory: < 50 KB for typical dictionaries

**Thread Safety**:
- Read operations: Lock-free via `OnceLock`
- Write operations: Synchronized via `RwLock`
- User creation: Atomic via file locking

**Usage Pattern**:
```rust
use reedbase::registry::{get_action_code, get_or_create_user_code};

// Get action code
let code = get_action_code("update")?; // u8

// Get or create user code
let code = get_or_create_user_code("admin")?; // u32
```

---

### 11. Schema Module (`schema/`)

**Purpose**: Schema validation for keys (RBKS v2) and columns.

**Structure**:
```
schema/
├── mod.rs              # Public API exports
├── loader.rs           # Schema file operations
├── rbks.rs             # RBKS v2 key validation
├── types.rs            # ColumnDef, Schema
├── validation.rs       # Row/column validation
├── loader_test.rs      # Loader tests
├── rbks_test.rs        # RBKS tests
└── validation_test.rs  # Validation tests
```

**Public API**:
```rust
// RBKS v2 (key validation)
pub use rbks::{
    normalize_key, parse_key, validate_key, Modifiers, ParsedKey, 
    KNOWN_ENVIRONMENTS, KNOWN_LANGUAGES, KNOWN_SEASONS, KNOWN_VARIANTS, RBKS_V2_PATTERN,
};

// Column schema validation
pub use loader::{create_default_schema, delete_schema, load_schema, save_schema, schema_exists};
pub use types::{ColumnDef, Schema};
pub use validation::{validate_row, validate_rows, validate_uniqueness, CsvRow};
```

**RBKS v2 Key Format**: `<namespace>.<hierarchy>[<modifier,modifier>]`

**Key Structure Rules**:
- Lowercase only
- Dots for hierarchy
- Angle brackets for modifiers
- Comma-separated modifiers
- Order-independent modifiers
- Depth 2-8 levels

**Modifier Categories**:
- **Language**: ISO 639-1 codes (de, en, fr, etc.) - max 1
- **Environment**: dev/prod/staging/test - max 1
- **Season**: christmas/easter/summer/winter - max 1
- **Variant**: mobile/desktop/tablet - max 1
- **Custom**: Any other identifier - multiple allowed

**Column Types**:
- **string**: Text data with length and pattern constraints
- **integer**: Whole numbers with min/max range
- **float**: Decimal numbers
- **boolean**: True/false values
- **timestamp**: Unix timestamps

**Constraints**:
- **required**: Cannot be empty
- **unique**: No duplicate values
- **primary_key**: Required + unique
- **min/max**: Range constraints for integer/float
- **min_length/max_length**: Length constraints for string
- **pattern**: Regex validation for string

**Performance**:
- Key validation: < 20μs
- Row validation: < 1ms
- Schema load: < 5ms
- Total overhead: < 30μs per write

**Usage Pattern**:
```rust
use reedbase::schema::{validate_key, parse_key, normalize_key};

// Key validation
validate_key("page.header.title<de,prod>")?;

// Parse key
let parsed = parse_key("page.header.title<de,prod>")?;
assert_eq!(parsed.namespace, "page");
assert_eq!(parsed.hierarchy, vec!["header", "title"]);
assert_eq!(parsed.modifiers.language, Some("de".to_string()));

// Column validation
let schema = Schema::new("2.0".to_string(), true, vec![
    ColumnDef::primary_key("id".to_string(), "integer".to_string()),
    ColumnDef::new("name".to_string(), "string".to_string()).required(),
]);

let row = CsvRow::new("1".to_string(), vec!["1".to_string(), "Alice".to_string()]);
validate_row(&row, &schema)?;
```

---

### 12. Tables Module (`tables/`)

**Purpose**: Universal table API for all ReedBase tables with versioning.

**Structure**:
```
tables/
├── mod.rs              # Public API exports
├── csv_parser.rs       # CSV parsing
├── helpers.rs          # Table utilities
├── table.rs            # Core Table struct
├── types.rs            # CsvRow, TableStats, VersionInfo, WriteResult
├── csv_parser_test.rs  # Parser tests
├── helpers_test.rs     # Helper tests
└── table_test.rs       # Table tests
```

**Public API**:
```rust
pub use csv_parser::{parse_csv, parse_csv_row};
pub use helpers::{list_tables, table_exists, table_stats};
pub use table::Table;
pub use types::{CsvRow, TableStats, VersionInfo, WriteResult};
```

**Table Structure**:
```text
.reed/tables/{table_name}/
├── current.csv          # Active version
├── {timestamp}.bsdiff   # Binary deltas (XZ compressed)
└── version.log          # Encoded metadata
```

**Key Features**:
- Universal: Same API for all tables
- Versioned: Git-like history with binary deltas
- Efficient: XZ-compressed deltas (95%+ space savings)
- Simple: Read current, write new, rollback to any version

**Usage Pattern**:
```rust
use reedbase::tables::Table;

// Create table reference
let table = Table::new(base_path, "text");

// Initialize new table
table.init(b"key|value\nfoo|bar\n", "admin")?;

// Read current version
let content = table.read_current()?;

// Write new version
table.write(b"key|value\nfoo|baz\n", "admin")?;

// List versions
let versions = table.list_versions()?;

// Rollback to version
table.rollback(timestamp, "admin")?;
```

---

### 13. Version Module (`version/`)

**Purpose**: Binary delta compression for efficient versioning.

**Structure**:
```
version/
├── mod.rs          # Public API exports
├── delta.rs        # Delta generation/application
└── delta_test.rs   # Delta tests
```

**Public API**:
```rust
pub use delta::{apply_delta, calculate_savings, generate_delta, DeltaInfo};
```

**Key Types**:
- `DeltaInfo`: Original size, delta size, compressed size, savings ratio

**Usage Pattern**:
```rust
use reedbase::version::{generate_delta, apply_delta};

// Generate delta
let delta = generate_delta(&old_data, &new_data)?;

// Apply delta
let restored = apply_delta(&old_data, &delta)?;
assert_eq!(restored, new_data);
```

---

## Common Patterns

### 1. Module Structure Pattern

**Every module follows this structure**:

```
module_name/
├── mod.rs              # Public API exports, module documentation
├── core_logic.rs       # Main implementation
├── types.rs            # Data structures
├── helpers.rs          # Utility functions (if needed)
├── core_logic_test.rs  # Tests (separate files, NOT inline)
└── types_test.rs       # Type tests (if complex)
```

**Key Rules**:
- `mod.rs`: Only exports and module-level documentation
- Separate test files (NOT `#[cfg(test)]` inline modules)
- One file = one responsibility
- Test files mirror source files: `foo.rs` → `foo_test.rs`

### 2. Error Handling Pattern

**All functions return `ReedResult<T>`**:

```rust
use crate::error::{ReedError, ReedResult};

pub fn my_function() -> ReedResult<String> {
    // Business logic
    if something_wrong {
        return Err(ReedError::SpecificError { 
            context: "detailed context".to_string() 
        });
    }
    
    Ok(result)
}
```

**Error Creation Pattern**:
```rust
// Specific error with context
Err(ReedError::TableNotFound { 
    name: table_name.to_string() 
})

// I/O error with context
std::fs::read_to_string(path).map_err(|e| ReedError::IoError {
    operation: format!("read file '{}'", path.display()),
    reason: e.to_string(),
})?
```

### 3. Public API Re-export Pattern

**`mod.rs` always re-exports commonly used types**:

```rust
// Internal modules (private)
mod implementation;
mod types;
mod helpers;

#[cfg(test)]
mod implementation_test;

// Re-export public API
pub use implementation::{main_function, helper_function};
pub use types::{MainType, HelperType};
```

### 4. Documentation Pattern

**Module-level documentation in `mod.rs`**:

```rust
//! Brief module description.
//!
//! Detailed explanation of what this module does.
//!
//! ## Features
//! - Feature 1
//! - Feature 2
//!
//! ## Performance
//! - Operation X: < Yμs
//! - Operation Z: O(1)
//!
//! ## Example Usage
//! ```rust
//! use reedbase::module::{function};
//!
//! let result = function(args)?;
//! # Ok::<(), reedbase::ReedError>(())
//! ```
```

**Function-level documentation**:

```rust
/// Brief one-line description.
///
/// ## Arguments
/// - `param1`: Description
/// - `param2`: Description
///
/// ## Returns
/// Description of return value
///
/// ## Errors
/// - `ErrorType1` - When condition X
/// - `ErrorType2` - When condition Y
///
/// ## Performance
/// - Time: O(1), < 100ns typical
/// - Space: O(n) for n items
///
/// ## Example
/// ```rust
/// let result = function(arg1, arg2)?;
/// assert_eq!(result, expected);
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn function(param1: Type1, param2: Type2) -> ReedResult<ReturnType> {
    // Implementation
}
```

### 5. Type Definition Pattern

**Clear, documented types in `types.rs`**:

```rust
/// Brief description of the type.
///
/// ## Fields
/// - `field1`: Description
/// - `field2`: Description
#[derive(Debug, Clone, PartialEq)]
pub struct MyType {
    /// Field description
    pub field1: String,
    
    /// Field description
    pub field2: u64,
}

impl MyType {
    /// Constructor with validation.
    pub fn new(field1: String, field2: u64) -> ReedResult<Self> {
        // Validation
        Ok(Self { field1, field2 })
    }
}
```

### 6. Test Pattern

**Separate test files with comprehensive coverage**:

```rust
// my_function_test.rs
use super::*;
use crate::error::ReedError;

#[test]
fn test_happy_path() {
    let result = my_function(valid_args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_error_case() {
    let result = my_function(invalid_args);
    assert!(result.is_err());
    match result.unwrap_err() {
        ReedError::SpecificError { .. } => (),
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_edge_case() {
    // Edge case testing
}
```

### 7. Singleton Pattern (for global state)

**Used in `metrics` and `functions` modules**:

```rust
use std::sync::{OnceLock, RwLock};

pub struct GlobalState {
    // State fields
}

static GLOBAL_INSTANCE: OnceLock<RwLock<GlobalState>> = OnceLock::new();

impl GlobalState {
    /// Get global singleton instance.
    pub fn global() -> &'static RwLock<GlobalState> {
        GLOBAL_INSTANCE.get_or_init(|| {
            RwLock::new(GlobalState::new())
        })
    }
    
    fn new() -> Self {
        Self {
            // Initialization
        }
    }
}
```

**Thread-safe access**:
```rust
// Read access
let guard = GlobalState::global().read().unwrap();
let value = guard.some_field;
drop(guard);

// Write access
let mut guard = GlobalState::global().write().unwrap();
guard.some_field = new_value;
drop(guard);
```

### 8. Builder Pattern (for complex types)

**Used in `indices::QueryFilter`**:

```rust
#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    namespace: Option<String>,
    language: Option<String>,
    environment: Option<String>,
}

impl QueryFilter {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }
    
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }
}
```

### 9. RAII Pattern (for resource management)

**Used in `concurrent::TableLock`**:

```rust
pub struct TableLock {
    lock_file: PathBuf,
}

impl TableLock {
    pub fn new(lock_file: PathBuf) -> Self {
        Self { lock_file }
    }
}

impl Drop for TableLock {
    fn drop(&mut self) {
        // Automatic cleanup on drop
        let _ = std::fs::remove_file(&self.lock_file);
    }
}
```

### 10. Type Alias Pattern

**For common Result types**:

```rust
/// Standard Result type for all operations.
pub type ReedResult<T> = Result<T, ReedError>;
```

---

## Dependencies Between Modules

### Dependency Graph

```text
                    ┌─────────────┐
                    │   error.rs  │ ◄─── Foundation (used by ALL)
                    └─────────────┘
                           ▲
                           │
        ┌──────────────────┴──────────────────┐
        │                                     │
   ┌────┴────┐                          ┌────┴────┐
   │ metrics │                          │ tables  │
   └────┬────┘                          └────┬────┘
        │                                     │
        │                                     ├──► version (delta)
        │                                     │
        │                                     └──► registry (logging)
        │
   ┌────┴────────────────────────────────────┴────┐
   │                                              │
┌──┴──┐  ┌──────────┐  ┌──────────┐  ┌──────────┤
│ log │  │ schema   │  │ indices  │  │ backup   │
└─────┘  └──────────┘  └──────────┘  └──────────┘
             │              │              │
             │              │              │
   ┌─────────┴──────────────┴──────────────┴─────┐
   │                                              │
┌──┴────────┐  ┌────────────┐  ┌────────────────┴┐
│concurrent │  │ conflict   │  │  functions      │
└───────────┘  └────────────┘  └─────────────────┘
     │              │                   │
     └──────────────┴───────────────────┘
                    │
              ┌─────┴─────┐
              │   merge   │
              └───────────┘
                    │
              ┌─────┴─────┐
              │  reedql   │ (uses indices, schema, tables)
              └───────────┘
```

### Module Dependencies by Layer

**Layer 0: Foundation**
- `error`: No dependencies (used by all)

**Layer 1: Core Infrastructure**
- `metrics`: → error
- `tables`: → error, version, registry
- `version`: → error
- `registry`: → error

**Layer 2: Data Management**
- `log`: → error, registry
- `schema`: → error, tables
- `indices`: → error, tables, schema
- `backup`: → error, tables, version

**Layer 3: Coordination**
- `concurrent`: → error, tables
- `conflict`: → error, concurrent
- `functions`: → error, tables, indices

**Layer 4: High-Level Operations**
- `merge`: → error, concurrent, conflict
- `reedql`: → error, tables, indices, schema, functions

### Import Guidelines

**When creating a new module, consider dependencies**:

1. **Prefer using `tables`** over direct file operations
2. **Always use `error` types** (never `anyhow` or custom errors)
3. **Use `metrics`** for performance tracking
4. **Use `schema`** for validation (don't implement custom validation)
5. **Use `indices`** for queries (don't scan CSV manually)
6. **Avoid circular dependencies** (structure matters!)

---

## Implementation Guidelines

### For B+-Tree Module (or any new module)

#### 1. Choose Appropriate Layer

Based on the dependency graph:
- **Layer 2** if depends only on tables/error
- **Layer 3** if needs concurrent/indices
- **Layer 4** if needs merge/reedql

**B+-Tree recommendation**: Layer 2 (parallel to `indices`)

#### 2. Module Structure

```
btree/
├── mod.rs              # Public API exports + documentation
├── node.rs             # B+-Tree node implementation
├── tree.rs             # Main B+-Tree struct
├── types.rs            # Key, Value, NodeId, etc.
├── insert.rs           # Insertion logic
├── search.rs           # Search logic
├── delete.rs           # Deletion logic (if needed)
├── iterator.rs         # Range query iterator
├── node_test.rs        # Node tests
├── tree_test.rs        # Tree tests
├── insert_test.rs      # Insert tests
├── search_test.rs      # Search tests
└── iterator_test.rs    # Iterator tests
```

#### 3. Types to Define

```rust
// types.rs
use crate::error::{ReedError, ReedResult};

/// B+-Tree key type (supports multiple key types).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BTreeKey {
    String(String),
    Integer(i64),
    Float(ordered_float::OrderedFloat<f64>),
}

/// B+-Tree node identifier.
pub type NodeId = u64;

/// B+-Tree configuration.
#[derive(Debug, Clone)]
pub struct BTreeConfig {
    pub order: usize,
    pub max_keys: usize,
    pub min_keys: usize,
}

/// B+-Tree node (internal or leaf).
#[derive(Debug, Clone)]
pub enum Node {
    Internal(InternalNode),
    Leaf(LeafNode),
}

/// Internal node with keys and child pointers.
#[derive(Debug, Clone)]
pub struct InternalNode {
    pub keys: Vec<BTreeKey>,
    pub children: Vec<NodeId>,
}

/// Leaf node with key-value pairs and next pointer.
#[derive(Debug, Clone)]
pub struct LeafNode {
    pub entries: Vec<(BTreeKey, String)>,
    pub next: Option<NodeId>,
}
```

#### 4. Main API

```rust
// tree.rs
use crate::btree::types::*;
use crate::error::{ReedError, ReedResult};

/// B+-Tree for indexed CSV access.
///
/// Provides O(log n) lookups, range queries, and ordered iteration.
///
/// ## Performance
/// - Search: O(log n)
/// - Insert: O(log n)
/// - Range query: O(log n + k) where k = results
/// - Memory: O(n) where n = number of keys
pub struct BTree {
    config: BTreeConfig,
    root: NodeId,
    nodes: HashMap<NodeId, Node>,
    next_id: NodeId,
}

impl BTree {
    /// Create new B+-Tree with specified order.
    pub fn new(order: usize) -> ReedResult<Self> {
        // Validation
        if order < 3 {
            return Err(ReedError::InvalidSchema {
                reason: "B+-Tree order must be >= 3".to_string(),
            });
        }
        
        // Initialize
        Ok(Self {
            config: BTreeConfig {
                order,
                max_keys: 2 * order - 1,
                min_keys: order - 1,
            },
            root: 0,
            nodes: HashMap::new(),
            next_id: 1,
        })
    }
    
    /// Insert key-value pair.
    pub fn insert(&mut self, key: BTreeKey, value: String) -> ReedResult<()> {
        // Implementation
    }
    
    /// Search for key.
    pub fn search(&self, key: &BTreeKey) -> ReedResult<Option<String>> {
        // Implementation
    }
    
    /// Range query [start, end).
    pub fn range(&self, start: &BTreeKey, end: &BTreeKey) -> ReedResult<Vec<(BTreeKey, String)>> {
        // Implementation
    }
    
    /// Get all keys in sorted order.
    pub fn keys(&self) -> ReedResult<Vec<BTreeKey>> {
        // Implementation
    }
}
```

#### 5. Integration with Tables

```rust
// integration example
use reedbase::tables::Table;
use reedbase::btree::BTree;

// Load CSV into B+-Tree
let table = Table::new(base_path, "text");
let content = table.read_current()?;
let rows = parse_csv(&content)?;

let mut btree = BTree::new(50)?; // order = 50
for row in rows {
    btree.insert(BTreeKey::String(row.key), row.values.join("|"))?;
}

// Fast search
let result = btree.search(&BTreeKey::String("page.header.title".into()))?;

// Range query
let results = btree.range(
    &BTreeKey::String("page.".into()),
    &BTreeKey::String("page.zzzz".into())
)?;
```

#### 6. Error Handling

```rust
// Use existing ReedError variants or add new ones
use crate::error::ReedError;

// For B+-Tree specific errors, add to error.rs:
pub enum ReedError {
    // ... existing variants ...
    
    /// B+-Tree node corrupted.
    BTreeNodeCorrupted { node_id: u64, reason: String },
    
    /// B+-Tree invariant violated.
    BTreeInvariantViolated { reason: String },
}
```

#### 7. Testing Strategy

```rust
// tree_test.rs
use super::*;

#[test]
fn test_insert_and_search() {
    let mut tree = BTree::new(3).unwrap();
    tree.insert(BTreeKey::String("key1".into()), "value1".into()).unwrap();
    
    let result = tree.search(&BTreeKey::String("key1".into())).unwrap();
    assert_eq!(result, Some("value1".to_string()));
}

#[test]
fn test_range_query() {
    let mut tree = BTree::new(3).unwrap();
    // Insert many keys
    
    let results = tree.range(
        &BTreeKey::String("page.".into()),
        &BTreeKey::String("page.zzzz".into())
    ).unwrap();
    
    assert!(results.len() > 0);
}

#[test]
fn test_node_split() {
    // Test node splitting logic
}
```

#### 8. Documentation

```rust
// mod.rs
//! B+-Tree index for fast CSV lookups.
//!
//! Provides O(log n) searches and range queries for CSV data.
//!
//! ## Features
//! - **Fast Search**: O(log n) lookup time
//! - **Range Queries**: O(log n + k) for k results
//! - **Ordered Iteration**: Sorted key traversal
//! - **Multiple Key Types**: String, Integer, Float
//!
//! ## Performance
//! - Insert: < 100μs (typical)
//! - Search: < 50μs (typical)
//! - Range query: < 1ms for 1000 results
//! - Memory: ~150 bytes/key
//!
//! ## Example Usage
//! ```rust
//! use reedbase::btree::{BTree, BTreeKey};
//!
//! let mut tree = BTree::new(50)?;
//! tree.insert(BTreeKey::String("key".into()), "value".into())?;
//!
//! let result = tree.search(&BTreeKey::String("key".into()))?;
//! assert_eq!(result, Some("value".to_string()));
//! # Ok::<(), reedbase::ReedError>(())
//! ```
```

#### 9. Integration with Metrics

```rust
use reedbase::metrics::{MetricsCollector, Metric, MetricUnit};

impl BTree {
    pub fn search(&self, key: &BTreeKey) -> ReedResult<Option<String>> {
        let start = std::time::Instant::now();
        
        // Perform search
        let result = self.search_internal(key)?;
        
        // Record metric
        let duration = start.elapsed().as_micros() as f64;
        let metric = Metric::new("btree_search", duration, MetricUnit::Microseconds)
            .with_tag("operation", "search");
        MetricsCollector::global().record(metric);
        
        Ok(result)
    }
}
```

#### 10. Checklist Before Implementation

- [ ] Module structure planned (files, responsibilities)
- [ ] Types defined (`types.rs`)
- [ ] Error variants added (if needed)
- [ ] Public API designed (`mod.rs`)
- [ ] Performance targets defined
- [ ] Test strategy planned
- [ ] Documentation outline ready
- [ ] Integration points identified
- [ ] Metrics integration planned
- [ ] No dependency on Layer 3+ modules (for Layer 2 module)

---

## Summary

### Key Takeaways

1. **Consistent Structure**: All modules follow the same pattern (mod.rs, core logic, types, tests)
2. **Separate Test Files**: Never inline `#[cfg(test)]`, always `foo_test.rs`
3. **Rich Error Handling**: Use `ReedResult<T>` with specific `ReedError` variants
4. **Performance Focus**: Document performance characteristics (< Xμs, O(n))
5. **Re-export Pattern**: `mod.rs` re-exports public API from internal modules
6. **Documentation**: Comprehensive module and function documentation
7. **Type Safety**: Strong types in `types.rs`, avoid stringly-typed APIs
8. **Thread Safety**: Use `RwLock` for shared state, `OnceLock` for singletons
9. **Metrics Integration**: Record performance metrics for observability
10. **Layer Awareness**: Respect dependency layers, avoid circular dependencies

### Module Categories

**Infrastructure** (0 dependencies):
- `error`: Foundation error types

**Core** (minimal dependencies):
- `metrics`: Performance monitoring
- `tables`: Universal table API
- `version`: Binary delta compression
- `registry`: Dictionary encoding

**Data Management** (depends on core):
- `log`: Encoded version history
- `schema`: RBKS v2 + column validation
- `indices`: Smart indices for queries
- `backup`: Point-in-time recovery

**Coordination** (depends on data):
- `concurrent`: Lock + queue management
- `conflict`: Merge conflict resolution
- `functions`: Computed/aggregation/transformation with caching

**High-Level** (depends on coordination):
- `merge`: Row-level CSV merge
- `reedql`: SQL-like query interface

### Implementation Patterns

1. **Error handling**: Always `ReedResult<T>` with specific variants
2. **Public API**: Re-export from `mod.rs`
3. **Types**: Dedicated `types.rs` file
4. **Tests**: Separate `foo_test.rs` files
5. **Documentation**: Module-level in `mod.rs`, function-level in implementation
6. **Singletons**: `OnceLock + RwLock` pattern
7. **Builders**: Fluent API with `self` methods
8. **RAII**: `Drop` for resource cleanup
9. **Performance**: Document and measure with metrics
10. **Integration**: Use existing modules (tables, indices, schema)

---

**End of Document**
