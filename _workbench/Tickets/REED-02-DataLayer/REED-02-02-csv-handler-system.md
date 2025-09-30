# REED-02-02: CSV Handler System

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function (CONFIG and SYSTEM use identical logic)
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs` - every topic (auth, login,...) has its own scoped rs service file
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-02-02
- **Title**: CSV Handler System
- **Layer**: Data Layer (REED-02)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-01-01

## Summary Reference
- **Section**: CSV Handler Services
- **Lines**: 973-976, 1291-1296 in project_summary.md
- **Key Concepts**: Universal .reed/ file management, atomic operations

## Objective
Implement universal CSV reader and writer for all .reed/ files with atomic write operations and comment preservation.

## Requirements

### 1. CSV Reader (`src/reedcms/csv/reader.rs`)

```rust
/// Reads CSV file and returns parsed entries.
///
/// ## Performance
/// - Streaming read (no full file load)
/// - Memory efficient
pub fn read_csv(file_path: &str) -> ReedResult<Vec<CsvEntry>>

/// Reads specific entry by key.
pub fn get(file_type: &str, key: &str) -> ReedResult<String>

/// Lists all keys matching pattern.
pub fn list_keys(file_type: &str, pattern: &str) -> ReedResult<Vec<String>>
```

### 2. CSV Writer (`src/reedcms/csv/writer.rs`)

```rust
/// Writes CSV entries with atomic operation.
///
/// ## Atomic Write Process
/// 1. Write to temporary file: `.reed/{file}.csv.tmp`
/// 2. Validate CSV structure
/// 3. Atomic rename to final file
///
/// ## Performance
/// - Write time: ~2ms for typical files
/// - Atomic guarantee: No corruption on crash
pub fn write_csv(file_path: &str, entries: &[CsvEntry]) -> ReedResult<()>

/// Sets value with atomic write.
pub fn set(file_type: &str, key: &str, value: &str, comment: &str) -> ReedResult<()>

/// Updates existing entry, preserving comments.
pub fn update(file_type: &str, key: &str, value: &str) -> ReedResult<()>
```

### 3. CSV Entry Structure (`src/reedcms/csv/entry.rs`)

```rust
/// Universal CSV entry structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvEntry {
    pub key: String,
    pub value: String,
    pub comment: String,
}

impl CsvEntry {
    /// Parses CSV line into entry.
    pub fn from_line(line: &str) -> ReedResult<Self>

    /// Formats entry as CSV line.
    pub fn to_line(&self) -> String
}
```

### 4. Comment Preservation (`src/reedcms/csv/comments.rs`)

```rust
/// Preserves existing comments when updating entries.
pub fn get_existing_comment(file_path: &str, key: &str) -> Option<String>

/// Validates comment meets minimum requirements (10 chars).
pub fn validate_comment(comment: &str) -> ReedResult<()>
```

## Implementation Files

### Primary Implementation
- `src/reedcms/csv/reader.rs` - CSV reading
- `src/reedcms/csv/writer.rs` - CSV writing
- `src/reedcms/csv/entry.rs` - Entry structure
- `src/reedcms/csv/comments.rs` - Comment handling

### Test Files
- `src/reedcms/csv/reader.test.rs`
- `src/reedcms/csv/writer.test.rs`
- `src/reedcms/csv/entry.test.rs`
- `src/reedcms/csv/comments.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test CSV parsing with semicolon separator
- [ ] Test quoted value handling
- [ ] Test comment preservation
- [ ] Test atomic write operation
- [ ] Test temporary file cleanup

### Integration Tests
- [ ] Test write followed by read (round-trip)
- [ ] Test concurrent access (multiple writers)
- [ ] Test corruption recovery
- [ ] Test large file handling (1000+ entries)

### Error Handling Tests
- [ ] Test missing file handling
- [ ] Test malformed CSV handling
- [ ] Test permission denied scenarios
- [ ] Test disk full scenarios

### Performance Tests
- [ ] Read operation: < 5ms for 1000 entries
- [ ] Write operation: < 2ms
- [ ] Atomic rename: < 1ms

## CSV Format Standards

### File Format
```csv
key;value;comment
knowledge.title@de;Wissen;German page title
knowledge.title@en;Knowledge;English page title
```

### Quoted Values
When values contain semicolons or newlines:
```csv
key;"value;with;semicolons";"comment"
```

### Multi-line Support
```csv
key;"Multi-line
value
here";"Description"
```

## Acceptance Criteria
- [ ] Universal CSV reader works with all .reed/ files
- [ ] Atomic write prevents corruption
- [ ] Comments preserved on updates
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Thread-safe operations
- [ ] Documentation complete

## Dependencies
- **Requires**: REED-01-01 (ReedStream for error handling)

## Blocks
- REED-02-01 (ReedBase needs CSV handler)
- REED-02-04 (Backup system needs CSV reader)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Summary: Lines 973-976, 1291-1296 in `project_summary.md`

## Notes
Focus on atomic operations to prevent CSV corruption. The temporary file + rename pattern is critical for data integrity. Comment preservation is mandatory per project standards.