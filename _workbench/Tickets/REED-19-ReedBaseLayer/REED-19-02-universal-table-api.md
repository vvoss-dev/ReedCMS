# REED-19-02: Universal Table API

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
- **ID**: REED-19-02
- **Title**: Universal Table API
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-19-01
- **Estimated Time**: 8 hours

## Objective

Implement universal table abstraction that works identically for ALL tables (text, routes, meta, users, etc.). Every table follows the same structure and API regardless of content type.

## Requirements

### Universal Table Structure

Every table follows this pattern:

```
.reed/tables/{table_name}/
├── current.csv          # Active version (always exists)
├── {timestamp}.bsdiff   # Binary deltas (XZ compressed)
└── version.log          # Encoded metadata
```

### Core Principles

1. **One file = One responsibility**: `table.rs` = Table struct only, `helpers.rs` = utility functions only, `csv_parser.rs` = CSV operations only
2. **One function = One job**: `read_current()` reads, `write()` writes, NO function does both
3. **BBC English**: All documentation, comments, error messages
4. **KISS**: Simple, obvious implementations over clever code

## Implementation Files

### Primary Implementation

**`src/reedbase/tables/mod.rs`**
- Module organisation
- Public exports only
- NO implementation logic

**`src/reedbase/tables/table.rs`**

One file = Table struct and its methods ONLY.

```rust
/// Universal table abstraction.
///
/// All tables (text, routes, meta, users, etc.) use identical structure.
/// This provides consistent API regardless of table contents.
///
/// ## Structure
/// - `current.csv`: Active version
/// - `{timestamp}.bsdiff`: Binary deltas
/// - `version.log`: Encoded metadata
///
/// ## Performance
/// - read_current(): < 1ms (cached)
/// - write(): < 5ms (create delta + update)
/// - list_versions(): < 5ms (parse log)
///
/// ## Thread Safety
/// - Multiple readers: Yes (concurrent reads safe)
/// - Multiple writers: NO (use WriteSession from REED-19-04)
pub struct Table {
    base_path: PathBuf,
    name: String,
}

impl Table {
    /// Create new table reference.
    ///
    /// Does NOT create table on disk, only creates reference.
    ///
    /// ## Input
    /// - `base_path`: Path to ReedBase directory
    /// - `name`: Table name
    ///
    /// ## Output
    /// - `Table`: Table reference
    ///
    /// ## Example Usage
    /// ```rust
    /// let table = Table::new(Path::new(".reed"), "text");
    /// ```
    pub fn new(base_path: &Path, name: &str) -> Self
    
    /// Get path to current.csv.
    ///
    /// ## Output
    /// - `PathBuf`: Full path to current.csv
    ///
    /// ## Performance
    /// - O(1), < 10ns
    pub fn current_path(&self) -> PathBuf
    
    /// Get path to delta file.
    ///
    /// ## Input
    /// - `timestamp`: Version timestamp
    ///
    /// ## Output
    /// - `PathBuf`: Full path to {timestamp}.bsdiff
    pub fn delta_path(&self, timestamp: u64) -> PathBuf
    
    /// Get path to version.log.
    ///
    /// ## Output
    /// - `PathBuf`: Full path to version.log
    pub fn log_path(&self) -> PathBuf
    
    /// Check if table exists on disk.
    ///
    /// ## Output
    /// - `bool`: True if current.csv exists
    ///
    /// ## Performance
    /// - < 100μs (file system check)
    pub fn exists(&self) -> bool
    
    /// Initialise new table.
    ///
    /// Creates directory and initial current.csv.
    ///
    /// ## Input
    /// - `initial_content`: CSV content (with header)
    /// - `user`: Username for audit
    ///
    /// ## Output
    /// - `Result<()>`: Success or error
    ///
    /// ## Performance
    /// - < 20ms (create dir + write file + log)
    ///
    /// ## Error Conditions
    /// - TableAlreadyExists: Table already initialised
    /// - IoError: Cannot create files
    pub fn init(&self, initial_content: &[u8], user: &str) -> ReedResult<()>
    
    /// Read current version as bytes.
    ///
    /// ## Output
    /// - `Result<Vec<u8>>`: CSV content
    ///
    /// ## Performance
    /// - < 1ms for typical tables (< 100 KB)
    ///
    /// ## Error Conditions
    /// - TableNotFound: Table doesn't exist
    /// - IoError: Cannot read file
    pub fn read_current(&self) -> ReedResult<Vec<u8>>
    
    /// Read current version as parsed rows.
    ///
    /// ## Output
    /// - `Result<Vec<CsvRow>>`: Parsed CSV rows
    ///
    /// ## Performance
    /// - < 5ms for typical tables (< 1000 rows)
    ///
    /// ## Error Conditions
    /// - TableNotFound: Table doesn't exist
    /// - InvalidCsv: Parse error
    pub fn read_current_as_rows(&self) -> ReedResult<Vec<CsvRow>>
    
    /// Write new version.
    ///
    /// Creates delta automatically, updates current.csv, logs to version.log.
    ///
    /// ## Input
    /// - `content`: New CSV content
    /// - `user`: Username for audit
    ///
    /// ## Output
    /// - `Result<WriteResult>`: Write metadata
    ///
    /// ## Performance
    /// - < 5ms typical (bsdiff + xz + write)
    ///
    /// ## Error Conditions
    /// - TableNotFound: Table doesn't exist (use init() first)
    /// - IoError: Cannot write files
    pub fn write(&self, content: &[u8], user: &str) -> ReedResult<WriteResult>
    
    /// List all versions.
    ///
    /// Parses version.log and returns metadata for each version.
    ///
    /// ## Output
    /// - `Result<Vec<VersionInfo>>`: Version metadata (newest first)
    ///
    /// ## Performance
    /// - < 5ms for typical logs (< 100 versions)
    ///
    /// ## Error Conditions
    /// - TableNotFound: Table doesn't exist
    /// - LogCorrupted: version.log parse error
    pub fn list_versions(&self) -> ReedResult<Vec<VersionInfo>>
    
    /// Rollback to specific version.
    ///
    /// Reconstructs version from deltas and writes as current.
    ///
    /// ## Input
    /// - `timestamp`: Target version timestamp
    /// - `user`: Username for audit
    ///
    /// ## Output
    /// - `Result<()>`: Success or error
    ///
    /// ## Performance
    /// - < 100ms per 50 deltas (typical)
    ///
    /// ## Error Conditions
    /// - VersionNotFound: Timestamp not in log
    /// - DeltaCorrupted: Cannot apply delta
    pub fn rollback(&self, timestamp: u64, user: &str) -> ReedResult<()>
    
    /// Delete table and all versions.
    ///
    /// ## Input
    /// - `confirm`: Safety flag (must be true)
    ///
    /// ## Output
    /// - `Result<()>`: Success or error
    ///
    /// ## Error Conditions
    /// - NotConfirmed: confirm was false
    /// - IoError: Cannot delete files
    pub fn delete(&self, confirm: bool) -> ReedResult<()>
}
```

**`src/reedbase/tables/helpers.rs`**

One file = Helper functions ONLY. NO struct definitions.

```rust
/// List all tables in database.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
///
/// ## Output
/// - `Result<Vec<String>>`: Table names
///
/// ## Performance
/// - < 10ms for typical installations (< 50 tables)
pub fn list_tables(base_path: &Path) -> ReedResult<Vec<String>>

/// Check if table exists.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
/// - `name`: Table name
///
/// ## Output
/// - `bool`: True if table exists
pub fn table_exists(base_path: &Path, name: &str) -> bool

/// Get table statistics.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
/// - `name`: Table name
///
/// ## Output
/// - `Result<TableStats>`: Statistics
///
/// ## Performance
/// - < 10ms (read log + file sizes)
pub fn table_stats(base_path: &Path, name: &str) -> ReedResult<TableStats>
```

**`src/reedbase/tables/csv_parser.rs`**

One file = CSV parsing ONLY. NO other logic.

```rust
/// Parse CSV bytes to rows.
///
/// ## Input
/// - `content`: CSV bytes (pipe-delimited)
///
/// ## Output
/// - `Result<Vec<CsvRow>>`: Parsed rows
///
/// ## Performance
/// - < 5ms for 1000 rows
///
/// ## Error Conditions
/// - InvalidCsv: Malformed CSV
pub fn parse_csv(content: &[u8]) -> ReedResult<Vec<CsvRow>>

/// Serialise rows to CSV bytes.
///
/// ## Input
/// - `rows`: CSV rows
///
/// ## Output
/// - `Result<Vec<u8>>`: CSV bytes
///
/// ## Performance
/// - < 5ms for 1000 rows
pub fn serialize_csv(rows: &[CsvRow]) -> ReedResult<Vec<u8>>

/// Count rows in CSV.
///
/// ## Input
/// - `content`: CSV bytes
///
/// ## Output
/// - `Result<usize>`: Row count (excluding header)
pub fn count_rows(content: &[u8]) -> ReedResult<usize>
```

**`src/reedbase/tables/types.rs`**

One file = Type definitions ONLY.

```rust
#[derive(Debug, Clone)]
pub struct WriteResult {
    pub timestamp: u64,
    pub delta_size: u64,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub timestamp: u64,
    pub action: String,
    pub user: String,
    pub delta_size: u64,
    pub rows_changed: u32,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct CsvRow {
    pub key: String,
    pub values: Vec<String>,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct TableStats {
    pub name: String,
    pub current_size: u64,
    pub version_count: usize,
    pub total_delta_size: u64,
    pub oldest_version: Option<u64>,
    pub newest_version: Option<u64>,
    pub last_modified_by: String,
}
```

### Test Files

**`src/reedbase/tables/table.test.rs`**

One test = One specific behaviour.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_table_new() {
        // Test table reference creation
    }
    
    #[test]
    fn test_table_paths() {
        // Test path generation methods
    }
    
    #[test]
    fn test_table_exists_false() {
        // Test non-existent table
    }
    
    #[test]
    fn test_table_init() {
        // Test table initialisation
    }
    
    #[test]
    fn test_table_init_already_exists() {
        // Test init fails for existing table
    }
    
    #[test]
    fn test_read_current() {
        // Test reading current version
    }
    
    #[test]
    fn test_read_current_not_found() {
        // Test error for non-existent table
    }
    
    #[test]
    fn test_write_creates_delta() {
        // Test write creates delta file
    }
    
    #[test]
    fn test_list_versions() {
        // Test version listing
    }
    
    #[test]
    fn test_rollback() {
        // Test rollback functionality
    }
    
    #[test]
    fn test_delete_requires_confirm() {
        // Test delete safety flag
    }
}
```

**`src/reedbase/tables/helpers.test.rs`**
**`src/reedbase/tables/csv_parser.test.rs`**

## Performance Requirements

- Table init: < 20ms
- read_current(): < 1ms
- write(): < 5ms (typical)
- list_versions(): < 5ms
- rollback(): < 100ms per 50 deltas
- Memory: < 10 MB per table operation

## Error Conditions

- **TableNotFound**: Table doesn't exist
- **TableAlreadyExists**: Table already initialised
- **VersionNotFound**: Version timestamp not found
- **InvalidCsv**: CSV parse error
- **DeltaCorrupted**: Cannot apply delta
- **NotConfirmed**: Safety flag not set
- **IoError**: File system errors

## Acceptance Criteria

- [ ] Table struct with all methods implemented
- [ ] Universal API works for any table name
- [ ] read_current() loads CSV correctly
- [ ] write() creates delta and updates current
- [ ] list_versions() parses version.log
- [ ] rollback() restores old version
- [ ] Helper functions work (list, stats)
- [ ] CSV parser handles edge cases
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks meet targets
- [ ] All code in BBC English
- [ ] Each file has one clear responsibility
- [ ] All functions have proper documentation
- [ ] No Swiss Army knife functions

## Dependencies
- **Requires**: REED-19-01 (Registry System - for logging)

## Blocks
- REED-19-03 (Binary Delta Versioning - uses Table API)
- REED-19-04 (Concurrent Writes - extends Table API)
- REED-19-05 (Row-Level Merge - uses Table API)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-19-00: Layer Overview

## Notes

The universal Table API is the core abstraction of ReedBase. Every table uses the EXACT same structure and API regardless of content type.

This consistency provides:
- **Simple mental model**: Learn once, use everywhere
- **Code reuse**: No duplicate table handling logic
- **Easy extension**: Add new tables without new code
- **Testability**: Test once, applies to all tables

**Implementation Note**: Keep Table struct simple. Complex operations (merge, conflict resolution) belong in separate modules (REED-19-04, REED-19-05).
