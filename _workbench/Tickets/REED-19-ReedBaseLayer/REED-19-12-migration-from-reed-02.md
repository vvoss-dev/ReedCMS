# REED-19-12: Migration from REED-02 Data Structure

**Status**: Not Started  
**Priority**: High  
**Estimated Effort**: 1 week  
**Layer**: ReedBase (Data Layer)  
**Dependencies**: REED-19-01, REED-19-02, REED-19-03, REED-19-08  

---

## Overview

This ticket implements a comprehensive migration system to convert existing REED-02 data structures to the new ReedBase format with versioning, binary deltas, and encoded logs.

**Purpose**: Provide automated migration tools that safely convert existing CSV data to the new universal table structure whilst preserving all content and metadata.

**Scope**:
- Analyse existing REED-02 data structure
- Convert all CSV files to universal table format
- Initialise version logs with migration baseline
- Validate data integrity post-migration
- Provide rollback capability
- Create migration report and audit trail

---

## MANDATORY Development Standards

1. **Language**: All code comments and documentation in BBC English
2. **Principle**: KISS (Keep It Simple, Stupid)
3. **File Naming**: Each file has unique theme and clear responsibility
4. **Files**: One file = One responsibility (no multi-purpose files)
5. **Functions**: One function = One distinctive job (no Swiss Army knives)
6. **Testing**: Separate test files as `{name}.test.rs` (never inline `#[cfg(test)]`)
7. **Avoid**: Generic names like `handler.rs`, `middleware.rs`, `utils.rs`
8. **Templates**: Reference `service-template.md` and `service-template.test.md`

---

## Implementation Files

### 1. `src/reedbase/migration/analyse.rs`

**Purpose**: Analyse existing REED-02 data structure and generate migration plan.

**Functions**:

```rust
/// Scan .reed/ directory and analyse current data structure.
///
/// ## Arguments
/// - reed_path: Path to .reed/ directory
///
/// ## Returns
/// - MigrationPlan with list of tables, row counts, detected issues
///
/// ## Performance
/// - O(n) where n = total rows across all CSVs
/// - < 100ms for 10,000 rows
///
/// ## Error Conditions
/// - ReedError::FileNotFound: .reed/ directory does not exist
/// - ReedError::InvalidFormat: CSV parsing fails
/// - ReedError::PermissionDenied: Cannot read files
///
/// ## Example Usage
/// ```rust
/// let plan = analyse_existing_structure("/project/.reed")?;
/// println!("Found {} tables with {} total rows", plan.table_count, plan.total_rows);
/// ```
pub fn analyse_existing_structure(reed_path: &Path) -> ReedResult<MigrationPlan>

/// Detect schema structure from existing CSV files.
///
/// ## Arguments
/// - csv_path: Path to CSV file to analyse
///
/// ## Returns
/// - DetectedSchema with column names, inferred types, constraints
///
/// ## Performance
/// - O(n) where n = number of rows (samples up to 1000 rows)
/// - < 50ms for typical files
///
/// ## Error Conditions
/// - ReedError::FileNotFound: CSV file does not exist
/// - ReedError::InvalidFormat: Cannot parse CSV structure
///
/// ## Example Usage
/// ```rust
/// let schema = detect_schema(Path::new(".reed/text.csv"))?;
/// for col in &schema.columns {
///     println!("{}: {:?}", col.name, col.inferred_type);
/// }
/// ```
pub fn detect_schema(csv_path: &Path) -> ReedResult<DetectedSchema>

/// Validate data integrity before migration.
///
/// ## Arguments
/// - reed_path: Path to .reed/ directory
///
/// ## Returns
/// - ValidationReport with warnings and errors found
///
/// ## Performance
/// - O(n) where n = total rows
/// - < 200ms for 10,000 rows
///
/// ## Error Conditions
/// - ReedError::DataIntegrity: Duplicate keys, invalid formats detected
///
/// ## Example Usage
/// ```rust
/// let report = validate_integrity("/project/.reed")?;
/// if !report.errors.is_empty() {
///     eprintln!("Found {} errors, aborting migration", report.errors.len());
/// }
/// ```
pub fn validate_integrity(reed_path: &Path) -> ReedResult<ValidationReport>
```

**Key Types**:

```rust
pub struct MigrationPlan {
    pub tables: Vec<TableInfo>,
    pub table_count: usize,
    pub total_rows: usize,
    pub total_size_bytes: u64,
    pub detected_issues: Vec<String>,
    pub estimated_duration_ms: u64,
}

pub struct TableInfo {
    pub name: String,
    pub path: PathBuf,
    pub row_count: usize,
    pub size_bytes: u64,
    pub schema: DetectedSchema,
}

pub struct DetectedSchema {
    pub columns: Vec<ColumnInfo>,
    pub delimiter: char,
    pub has_header: bool,
}

pub struct ColumnInfo {
    pub name: String,
    pub inferred_type: DataType,
    pub nullable: bool,
    pub unique_values: usize,
}

pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub passed: bool,
}
```

---

### 2. `src/reedbase/migration/convert.rs`

**Purpose**: Convert existing CSV files to new universal table structure.

**Functions**:

```rust
/// Migrate single CSV file to universal table format.
///
/// ## Arguments
/// - old_csv_path: Path to existing CSV file
/// - table: Table instance for new format
/// - backup_path: Path for backup of original file
///
/// ## Returns
/// - MigrationResult with rows migrated and any warnings
///
/// ## Performance
/// - O(n) where n = number of rows
/// - ~10,000 rows/second
/// - < 1s for typical tables (< 10,000 rows)
///
/// ## Error Conditions
/// - ReedError::FileNotFound: Source CSV does not exist
/// - ReedError::WriteError: Cannot write to new table
/// - ReedError::DataIntegrity: Invalid data during conversion
///
/// ## Example Usage
/// ```rust
/// let table = Table::new(".reed/tables/text")?;
/// let result = migrate_table(
///     Path::new(".reed/text.csv"),
///     &table,
///     Path::new(".reed/backup/text.csv.bak")
/// )?;
/// println!("Migrated {} rows", result.rows_migrated);
/// ```
pub fn migrate_table(
    old_csv_path: &Path,
    table: &Table,
    backup_path: &Path,
) -> ReedResult<MigrationResult>

/// Convert all tables in .reed/ directory.
///
/// ## Arguments
/// - reed_path: Path to .reed/ directory
/// - plan: MigrationPlan from analyse phase
///
/// ## Returns
/// - OverallMigrationResult with success status and detailed report
///
/// ## Performance
/// - O(n) where n = total rows across all tables
/// - ~10,000 rows/second aggregate
/// - Progress callback every 1000 rows
///
/// ## Error Conditions
/// - ReedError::MigrationFailed: One or more tables failed to migrate
///
/// ## Example Usage
/// ```rust
/// let plan = analyse_existing_structure(".reed")?;
/// let result = migrate_all_tables(".reed", &plan)?;
/// if result.success {
///     println!("Successfully migrated {} tables", result.tables_migrated);
/// }
/// ```
pub fn migrate_all_tables(
    reed_path: &Path,
    plan: &MigrationPlan,
) -> ReedResult<OverallMigrationResult>

/// Initialise version log with migration baseline.
///
/// ## Arguments
/// - table: Table instance
/// - source_file: Original CSV file name
/// - row_count: Number of rows migrated
///
/// ## Returns
/// - () on success
///
/// ## Performance
/// - O(1) - single log entry write
/// - < 1ms
///
/// ## Error Conditions
/// - ReedError::WriteError: Cannot write version.log
///
/// ## Example Usage
/// ```rust
/// initialise_version_log(&table, "text.csv", 5000)?;
/// ```
pub fn initialise_version_log(
    table: &Table,
    source_file: &str,
    row_count: usize,
) -> ReedResult<()>
```

**Key Types**:

```rust
pub struct MigrationResult {
    pub rows_migrated: usize,
    pub rows_skipped: usize,
    pub warnings: Vec<String>,
    pub duration_ms: u64,
}

pub struct OverallMigrationResult {
    pub success: bool,
    pub tables_migrated: usize,
    pub tables_failed: usize,
    pub total_rows: usize,
    pub duration_ms: u64,
    pub table_results: HashMap<String, MigrationResult>,
}
```

---

### 3. `src/reedbase/migration/rollback.rs`

**Purpose**: Provide rollback capability to restore original REED-02 structure.

**Functions**:

```rust
/// Create complete backup before migration.
///
/// ## Arguments
/// - reed_path: Path to .reed/ directory
/// - backup_dir: Directory to store backup
///
/// ## Returns
/// - Path to backup archive (.tar.xz)
///
/// ## Performance
/// - O(n) where n = total file size
/// - ~50MB/s compression speed
/// - ~100ms for typical 5MB .reed/ directory
///
/// ## Error Conditions
/// - ReedError::FileNotFound: Source directory does not exist
/// - ReedError::WriteError: Cannot create backup archive
/// - ReedError::PermissionDenied: Cannot read source files
///
/// ## Example Usage
/// ```rust
/// let backup_path = create_migration_backup(
///     Path::new(".reed"),
///     Path::new(".reed/backups")
/// )?;
/// println!("Created backup: {}", backup_path.display());
/// ```
pub fn create_migration_backup(
    reed_path: &Path,
    backup_dir: &Path,
) -> ReedResult<PathBuf>

/// Restore original REED-02 structure from backup.
///
/// ## Arguments
/// - backup_path: Path to backup archive
/// - reed_path: Destination .reed/ directory
///
/// ## Returns
/// - () on success
///
/// ## Performance
/// - O(n) where n = backup archive size
/// - ~100MB/s decompression speed
/// - ~50ms for typical backup
///
/// ## Error Conditions
/// - ReedError::FileNotFound: Backup archive does not exist
/// - ReedError::WriteError: Cannot extract backup
/// - ReedError::DataIntegrity: Backup archive corrupted
///
/// ## Example Usage
/// ```rust
/// rollback_migration(
///     Path::new(".reed/backups/2025-01-15_pre-migration.tar.xz"),
///     Path::new(".reed")
/// )?;
/// println!("Successfully rolled back to REED-02 format");
/// ```
pub fn rollback_migration(
    backup_path: &Path,
    reed_path: &Path,
) -> ReedResult<()>

/// Verify backup integrity before migration.
///
/// ## Arguments
/// - backup_path: Path to backup archive
///
/// ## Returns
/// - BackupInfo with file count, size, and integrity status
///
/// ## Performance
/// - O(n) where n = archive size
/// - < 100ms for typical backups
///
/// ## Error Conditions
/// - ReedError::FileNotFound: Backup does not exist
/// - ReedError::DataIntegrity: Checksum mismatch
///
/// ## Example Usage
/// ```rust
/// let info = verify_backup(Path::new(".reed/backups/migration.tar.xz"))?;
/// if info.integrity_ok {
///     println!("Backup valid: {} files, {} bytes", info.file_count, info.size_bytes);
/// }
/// ```
pub fn verify_backup(backup_path: &Path) -> ReedResult<BackupInfo>
```

**Key Types**:

```rust
pub struct BackupInfo {
    pub file_count: usize,
    pub size_bytes: u64,
    pub compressed_bytes: u64,
    pub checksum: String,
    pub integrity_ok: bool,
    pub created_at: SystemTime,
}
```

---

### 4. `src/reedbase/migration/report.rs`

**Purpose**: Generate detailed migration reports and audit trail.

**Functions**:

```rust
/// Generate human-readable migration report.
///
/// ## Arguments
/// - result: OverallMigrationResult from migration
/// - output_path: Path to write report file
///
/// ## Returns
/// - () on success
///
/// ## Performance
/// - O(n) where n = number of tables
/// - < 10ms
///
/// ## Error Conditions
/// - ReedError::WriteError: Cannot write report file
///
/// ## Example Usage
/// ```rust
/// generate_report(&migration_result, Path::new(".reed/migration-report.txt"))?;
/// ```
pub fn generate_report(
    result: &OverallMigrationResult,
    output_path: &Path,
) -> ReedResult<()>

/// Generate JSON audit trail for programmatic processing.
///
/// ## Arguments
/// - result: OverallMigrationResult from migration
/// - plan: Original MigrationPlan
/// - output_path: Path to write JSON file
///
/// ## Returns
/// - () on success
///
/// ## Performance
/// - O(n) where n = number of tables
/// - < 10ms
///
/// ## Error Conditions
/// - ReedError::WriteError: Cannot write JSON file
/// - ReedError::SerializationError: Cannot serialise to JSON
///
/// ## Example Usage
/// ```rust
/// generate_audit_trail(&result, &plan, Path::new(".reed/migration-audit.json"))?;
/// ```
pub fn generate_audit_trail(
    result: &OverallMigrationResult,
    plan: &MigrationPlan,
    output_path: &Path,
) -> ReedResult<()>

/// Create diff report showing changes between old and new format.
///
/// ## Arguments
/// - old_reed_path: Path to backup of old .reed/
/// - new_reed_path: Path to new .reed/ structure
///
/// ## Returns
/// - DiffReport with detailed comparison
///
/// ## Performance
/// - O(n) where n = total rows
/// - < 500ms for typical datasets
///
/// ## Error Conditions
/// - ReedError::FileNotFound: Cannot find source directories
/// - ReedError::DataIntegrity: Data mismatch detected
///
/// ## Example Usage
/// ```rust
/// let diff = create_diff_report(
///     Path::new(".reed/backups/old"),
///     Path::new(".reed")
/// )?;
/// println!("Content identical: {}", diff.content_matches);
/// ```
pub fn create_diff_report(
    old_reed_path: &Path,
    new_reed_path: &Path,
) -> ReedResult<DiffReport>
```

**Key Types**:

```rust
pub struct DiffReport {
    pub content_matches: bool,
    pub row_count_diff: HashMap<String, (usize, usize)>, // (old, new)
    pub schema_changes: Vec<SchemaChange>,
    pub data_differences: Vec<DataDifference>,
}

pub struct SchemaChange {
    pub table: String,
    pub change_type: String, // "added_column", "removed_column", "type_change"
    pub details: String,
}

pub struct DataDifference {
    pub table: String,
    pub row_identifier: String,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}
```

---

### 5. `src/reedbase/migration/mod.rs`

**Purpose**: Public API for migration system.

**Functions**:

```rust
/// Execute complete migration with all safety checks.
///
/// ## Arguments
/// - reed_path: Path to .reed/ directory
/// - dry_run: If true, analyse only without making changes
///
/// ## Returns
/// - OverallMigrationResult with complete report
///
/// ## Performance
/// - Depends on dataset size
/// - ~10,000 rows/second
/// - Typical 10,000 row dataset: < 2 seconds
///
/// ## Error Conditions
/// - ReedError::MigrationFailed: Migration failed validation or execution
/// - ReedError::DataIntegrity: Data validation failed
///
/// ## Example Usage
/// ```rust
/// // Dry run first
/// let dry_result = execute_migration(".reed", true)?;
/// if dry_result.success {
///     // Execute actual migration
///     let result = execute_migration(".reed", false)?;
///     println!("Migrated {} tables", result.tables_migrated);
/// }
/// ```
pub fn execute_migration(
    reed_path: &Path,
    dry_run: bool,
) -> ReedResult<OverallMigrationResult>
```

---

## CLI Commands

### `reed migrate:plan`
**Purpose**: Analyse existing data structure and show migration plan.

```bash
# Analyse current .reed/ directory
reed migrate:plan

# Output format:
# Migration Plan
# ==============
# Tables found: 5
# Total rows: 12,450
# Estimated duration: 1.2s
#
# Table: text.csv (5,000 rows, 2.1 MB)
#   - Detected schema: key|value|description
#   - Column types: string|string|string
#   - Issues: None
#
# Table: routes.csv (150 rows, 25 KB)
#   ...
```

### `reed migrate:execute`
**Purpose**: Execute migration with safety checks.

```bash
# Execute migration (requires confirmation)
reed migrate:execute

# Dry run mode (no changes)
reed migrate:execute --dry-run

# Skip confirmation prompt
reed migrate:execute --force

# Output:
# Creating backup: .reed/backups/2025-01-15_pre-migration.tar.xz
# Validating data integrity... OK
# Migrating text.csv... 5000 rows ✓ (0.5s)
# Migrating routes.csv... 150 rows ✓ (0.02s)
# ...
# Migration complete: 5 tables, 12,450 rows in 1.2s
```

### `reed migrate:rollback`
**Purpose**: Restore original REED-02 structure from backup.

```bash
# List available backups
reed migrate:rollback --list

# Rollback to specific backup
reed migrate:rollback .reed/backups/2025-01-15_pre-migration.tar.xz

# Rollback to latest backup
reed migrate:rollback --latest
```

### `reed migrate:verify`
**Purpose**: Verify migration integrity by comparing old and new data.

```bash
# Compare backup with current state
reed migrate:verify .reed/backups/2025-01-15_pre-migration.tar.xz

# Output:
# Verification Report
# ===================
# Content match: ✓ All rows identical
# Row counts: ✓ All match
# Schema changes: 0
# Data differences: 0
#
# Migration verified successfully.
```

---

## Test Files

### `src/reedbase/migration/analyse.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_analyse_empty_directory()
// Verify: Empty .reed/ returns empty plan

#[test]
fn test_analyse_multiple_tables()
// Verify: Correctly counts tables and rows

#[test]
fn test_detect_schema_with_types()
// Verify: Infers correct column types from data

#[test]
fn test_detect_schema_delimiter()
// Verify: Detects pipe delimiter correctly

#[test]
fn test_validate_integrity_duplicate_keys()
// Verify: Reports duplicate key errors

#[test]
fn test_validate_integrity_clean_data()
// Verify: Returns passed=true for valid data

#[test]
fn test_performance_large_dataset()
// Verify: Analyses 10,000 rows < 100ms
```

### `src/reedbase/migration/convert.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_migrate_table_basic()
// Verify: Migrates simple CSV to universal table

#[test]
fn test_migrate_table_preserves_content()
// Verify: All rows and data preserved exactly

#[test]
fn test_migrate_all_tables()
// Verify: Migrates multiple tables successfully

#[test]
fn test_initialise_version_log()
// Verify: Creates version.log with correct baseline entry

#[test]
fn test_migrate_table_error_handling()
// Verify: Returns appropriate errors for invalid data

#[test]
fn test_performance_migration_speed()
// Verify: Migrates ~10,000 rows/second
```

### `src/reedbase/migration/rollback.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_create_backup()
// Verify: Creates compressed backup archive

#[test]
fn test_rollback_restores_original()
// Verify: Rollback exactly restores original state

#[test]
fn test_verify_backup_integrity()
// Verify: Detects corrupted backups

#[test]
fn test_verify_backup_checksum()
// Verify: Checksum validation works correctly

#[test]
fn test_backup_compression_ratio()
// Verify: Achieves reasonable compression ratio
```

### `src/reedbase/migration/report.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_generate_report_format()
// Verify: Report contains all required sections

#[test]
fn test_generate_audit_trail_json()
// Verify: Valid JSON structure

#[test]
fn test_diff_report_identical_data()
// Verify: Reports match when data identical

#[test]
fn test_diff_report_detects_differences()
// Verify: Reports differences when data changed

#[test]
fn test_schema_change_detection()
// Verify: Detects column additions/removals
```

---

## Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Analyse structure (10k rows) | < 100ms | Wall time |
| Detect schema per table | < 50ms | Wall time |
| Validate integrity (10k rows) | < 200ms | Wall time |
| Migrate table (10k rows) | < 1s | Wall time |
| Migration speed | ~10,000 rows/s | Throughput |
| Create backup | < 100ms | Wall time (5MB) |
| Rollback restore | < 50ms | Wall time |
| Generate report | < 10ms | Wall time |
| Diff comparison (10k rows) | < 500ms | Wall time |

---

## Error Conditions

### `ReedError::MigrationFailed`
**When**: Migration process fails validation or execution.  
**Context**: Table name, row number, reason for failure.  
**Recovery**: Review error details, fix data issues, retry migration.

### `ReedError::DataIntegrity`
**When**: Data validation fails during migration.  
**Context**: Specific integrity violations (duplicate keys, invalid formats).  
**Recovery**: Fix source data, run validation again.

### `ReedError::BackupFailed`
**When**: Cannot create or restore backup.  
**Context**: Backup path, I/O error details.  
**Recovery**: Check disk space, permissions, retry.

### `ReedError::RollbackFailed`
**When**: Rollback operation cannot complete.  
**Context**: Backup path, destination path, error details.  
**Recovery**: Manual restoration from backup archive.

---

## Acceptance Criteria

- [ ] `analyse.rs` implements structure analysis and schema detection
- [ ] `convert.rs` migrates CSV files to universal table format
- [ ] `rollback.rs` provides backup and restore functionality
- [ ] `report.rs` generates human and machine-readable reports
- [ ] All migration operations preserve data integrity (100% match)
- [ ] Migration creates complete backup before making changes
- [ ] Version logs initialised with migration baseline
- [ ] CLI commands provide clear progress feedback
- [ ] Dry-run mode allows safe preview of changes
- [ ] Rollback capability tested and verified
- [ ] Performance targets met for typical datasets (< 10k rows)
- [ ] Error handling covers all failure scenarios
- [ ] Test coverage 100% for all modules
- [ ] All tests pass
- [ ] Documentation complete with examples

---

## Dependencies

- **REED-19-01**: Dictionary system for version log encoding
- **REED-19-02**: Universal table API for new format
- **REED-19-03**: Binary delta system (not used during migration, but initialised)
- **REED-19-08**: Schema validation for converted tables

---

## Notes

### Migration Process Flow

1. **Analysis Phase**:
   - Scan .reed/ directory
   - Count tables and rows
   - Detect schemas
   - Validate data integrity
   - Generate migration plan

2. **Backup Phase**:
   - Create compressed backup (.tar.xz)
   - Verify backup integrity
   - Store in .reed/backups/

3. **Migration Phase**:
   - For each CSV file:
     - Create universal table directory
     - Copy data to current.csv
     - Initialise version.log with baseline
     - Create dictionaries (actions.dict, users.dict)
   - Progress reporting every 1000 rows

4. **Verification Phase**:
   - Compare old and new data
   - Verify row counts match
   - Check data integrity
   - Generate diff report

5. **Completion Phase**:
   - Generate human-readable report
   - Generate JSON audit trail
   - Display summary statistics

### Data Preservation Guarantees

- **100% content preservation**: Every row and value transferred exactly
- **Schema preservation**: Column names, order, types maintained
- **Metadata preservation**: Original file structure documented in version logs
- **Atomic operation**: Either complete success or full rollback (no partial state)

### Rollback Strategy

- Backups stored in `.reed/backups/` with timestamp
- Backup naming: `YYYY-MM-DD_HH-MM-SS_pre-migration.tar.xz`
- XZ compression for efficient storage
- Rollback restores exact original structure
- Verification step ensures rollback success

### Migration Safety Features

1. **Dry run mode**: Preview changes without executing
2. **Confirmation prompts**: Require explicit user confirmation
3. **Automatic backups**: Created before any changes
4. **Integrity validation**: Pre and post-migration checks
5. **Progress reporting**: Real-time feedback during migration
6. **Error recovery**: Clear error messages with recovery instructions
7. **Audit trail**: Complete log of all actions taken

### Performance Considerations

- **Streaming processing**: Process rows incrementally (no full load into memory)
- **Batch writes**: Write current.csv in batches of 1000 rows
- **Parallel compression**: XZ compression in separate thread
- **Progress callbacks**: Update UI every 1000 rows (not every row)

### Edge Cases

- **Empty tables**: Create table structure with empty current.csv
- **Large files**: Stream processing for files > 100MB
- **Invalid CSV**: Report errors but continue with valid tables
- **Duplicate keys**: Report as warning but allow migration
- **Missing columns**: Fill with empty strings, report as warning
- **Encoding issues**: Detect and report non-UTF8 content

---

## References

- Service Template: `_workbench/Tickets/templates/service-template.md`
- Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-19-02: Universal Table API
- REED-19-08: Schema Validation System
