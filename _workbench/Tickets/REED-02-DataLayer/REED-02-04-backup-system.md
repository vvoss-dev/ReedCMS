# REED-02-04: Backup System

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
- **ID**: REED-02-04
- **Title**: XZ Compression Backup System
- **Layer**: Data Layer (REED-02)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-02-02

## Summary Reference
- **Section**: Automatic XZ Backup System
- **Lines**: 308-333 in project_summary.md
- **Key Concepts**: Automatic backups before modifications, 32-backup retention

## Objective
Implement automatic XZ compression backup system that runs before any CSV modification to prevent data loss.

## Requirements

### Implementation (`src/reedcms/reedbase/backup.rs`)

```rust
/// Creates compressed backup before CSV modification.
///
/// ## Process
/// 1. Read original CSV file
/// 2. Compress with XZ (LZMA2)
/// 3. Write to .reed/backups/{filename}.{timestamp}.csv.xz
/// 4. Cleanup old backups (keep latest 32)
///
/// ## Performance
/// - Compression: ~5ms for typical CSV files
/// - Storage: ~10x compression ratio
///
/// ## Input
/// - `csv_path`: Path to CSV file to backup
///
/// ## Output
/// - `BackupInfo`: Information about created backup
pub fn create_backup(csv_path: &str) -> ReedResult<BackupInfo>

/// Restores from backup (N steps back).
///
/// ## Input
/// - `csv_path`: Path to CSV file to restore
/// - `steps_back`: Number of backups to go back (1 = latest backup)
///
/// ## Process
/// 1. List available backups sorted by timestamp
/// 2. Select backup N steps back
/// 3. Decompress XZ file
/// 4. Write to original CSV path
pub fn restore_backup(csv_path: &str, steps_back: u32) -> ReedResult<()>

/// Lists available backups for file.
///
/// ## Output
/// - Vector of BackupInfo sorted by timestamp (newest first)
pub fn list_backups(csv_path: &str) -> ReedResult<Vec<BackupInfo>>

/// Cleans up old backups (keeps latest 32).
///
/// ## Input
/// - `directory`: Backup directory path (.reed/backups/)
/// - `keep_count`: Number of backups to keep (default: 32)
///
/// ## Output
/// - `CleanupStats`: Number of backups removed and retained
pub fn cleanup_old_backups(directory: &str, keep_count: u32) -> ReedResult<CleanupStats>

/// Backup information structure
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub timestamp: u64,
    pub file_path: String,
    pub compressed_size: u64,
    pub original_size: u64,
    pub compression_ratio: f32,
}

/// Cleanup statistics
#[derive(Debug, Clone)]
pub struct CleanupStats {
    pub removed_count: u32,
    pub retained_count: u32,
    pub freed_bytes: u64,
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/reedbase/backup.rs` - Backup system

### Test Files
- `src/reedcms/reedbase/backup.test.rs` - Comprehensive tests

## File Structure
```
src/reedcms/reedbase/
├── backup.rs      # Backup implementation
└── backup.test.rs # Tests
```

## Testing Requirements

### Unit Tests
- [ ] Test backup creation
- [ ] Test XZ compression
- [ ] Test backup listing
- [ ] Test restore operation
- [ ] Test cleanup (32-backup retention)

### Integration Tests
- [ ] Test backup before set operation
- [ ] Test restore after data corruption
- [ ] Test concurrent backup operations
- [ ] Test backup with large CSV files (1000+ entries)

### Error Handling Tests
- [ ] Test backup with non-existent file
- [ ] Test restore with invalid backup
- [ ] Test cleanup with permission errors
- [ ] Test disk full scenarios

### Performance Tests
- [ ] Backup creation: < 5ms for typical CSV
- [ ] Compression ratio: > 5x
- [ ] Restore operation: < 10ms
- [ ] Cleanup operation: < 50ms

## Backup Directory Structure
```
.reed/backups/
├── text.1704067200.csv.xz       # Timestamp-based naming
├── text.1704067260.csv.xz
├── text.1704067320.csv.xz
├── routes.1704067200.csv.xz
├── routes.1704067260.csv.xz
└── meta.1704067200.csv.xz
```

## Acceptance Criteria
- [ ] Automatic backup before every CSV write
- [ ] XZ compression working (LZMA2 algorithm)
- [ ] 32-backup retention enforced automatically
- [ ] Restore functionality tested and working
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-02-02 (CSV Handler)

## Blocks
- REED-10-04 (Backup Recovery CLI needs this implementation)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 308-333 in `project_summary.md`

## Notes
The backup system is critical for data integrity. Every CSV modification MUST be preceded by an automatic backup. The 32-backup retention provides a reasonable balance between storage space and recovery options. XZ compression with LZMA2 provides excellent compression ratios (~10x) for text data while maintaining fast compression times (~5ms).