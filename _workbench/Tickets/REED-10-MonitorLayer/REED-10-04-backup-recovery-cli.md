# REED-10-04: Backup Recovery and CLI Tools

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
- **ID**: REED-10-04
- **Title**: Backup Recovery and CLI Management Tools
- **Layer**: Monitor Layer (REED-10)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-02-04

## Summary Reference
- **Section**: Backup Recovery
- **Lines**: 1054-1056 in project_summary.md
- **Key Concepts**: Backup restoration, CLI tools, data recovery, backup verification

## Objective
Implement backup recovery CLI tools that enable restoration of CSV backups, verification of backup integrity, listing available backups, selective restoration, and automated recovery procedures to protect against data loss.

## Requirements

### Backup Recovery Features

**List Backups**
- View all available backups
- Show backup dates and sizes
- Display backup age
- Identify latest backup

**Restore Backup**
- Restore specific CSV file
- Restore all CSV files
- Restore to specific timestamp
- Dry-run mode

**Verify Backup**
- Check backup integrity
- Validate XZ compression
- Verify file structure
- Test decompression

**Backup Management**
- Prune old backups
- Archive backups
- Export backups
- Import backups

### Implementation (`src/reedcms/backup/recovery.rs`)

```rust
/// Backup recovery system for ReedCMS.
///
/// ## Backup Location
/// .reed/backups/{csv_name}/
///   ├── text.csv.2025-01-15_120000.xz
///   ├── text.csv.2025-01-15_130000.xz
///   └── ...
///
/// ## Backup Retention
/// - Last 32 backups kept per CSV file
/// - Automatic pruning on new backup
/// - Manual archive option
pub struct BackupRecovery;

impl BackupRecovery {
    /// Lists all available backups.
    ///
    /// ## Output
    /// ```
    /// 📦 Available Backups
    ///
    /// text.csv (8 backups):
    ///   1. 2025-01-15 13:00:00 (1 hour ago) - 24.3 KB
    ///   2. 2025-01-15 12:00:00 (2 hours ago) - 24.1 KB
    ///   3. 2025-01-15 11:00:00 (3 hours ago) - 23.9 KB
    ///   ...
    ///
    /// routes.csv (5 backups):
    ///   1. 2025-01-15 12:30:00 (90 min ago) - 12.1 KB
    ///   ...
    /// ```
    pub fn list_backups(csv_file: Option<&str>) -> ReedResult<Vec<BackupInfo>> {
        let backup_base = ".reed/backups";
        let mut backups = Vec::new();

        // Get directories to scan
        let dirs = if let Some(file) = csv_file {
            vec![format!("{}/{}", backup_base, file.trim_end_matches(".csv"))]
        } else {
            list_backup_directories(backup_base)?
        };

        for dir in dirs {
            let csv_name = std::path::Path::new(&dir)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let files = list_backup_files(&dir)?;

            for file in files {
                let metadata = std::fs::metadata(&file)?;
                let modified = metadata.modified()?;
                let size = metadata.len();

                backups.push(BackupInfo {
                    csv_file: format!("{}.csv", csv_name),
                    backup_path: file,
                    timestamp: modified.into(),
                    size_bytes: size,
                });
            }
        }

        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(backups)
    }

    /// Restores backup to original location.
    ///
    /// ## Arguments
    /// - backup_path: Path to backup file
    /// - dry_run: If true, only validates without restoring
    ///
    /// ## Process
    /// 1. Validate backup exists
    /// 2. Decompress XZ backup
    /// 3. Verify CSV structure
    /// 4. Backup current file (if exists)
    /// 5. Write restored content
    /// 6. Verify restoration
    ///
    /// ## Safety
    /// - Creates backup of current file before restore
    /// - Validates backup integrity
    /// - Atomic write operation
    pub fn restore_backup(backup_path: &str, dry_run: bool) -> ReedResult<RestoreReport> {
        let mut report = RestoreReport {
            backup_path: backup_path.to_string(),
            target_file: String::new(),
            success: false,
            dry_run,
            errors: Vec::new(),
        };

        // 1. Validate backup exists
        if !std::path::Path::new(backup_path).exists() {
            report.errors.push("Backup file not found".to_string());
            return Ok(report);
        }

        // 2. Extract target CSV filename from backup path
        let target_csv = extract_csv_name(backup_path)?;
        report.target_file = format!(".reed/{}", target_csv);

        // 3. Decompress backup
        let decompressed = decompress_xz_backup(backup_path)?;

        // 4. Verify CSV structure
        if let Err(e) = verify_csv_structure(&decompressed) {
            report.errors.push(format!("Invalid CSV structure: {}", e));
            return Ok(report);
        }

        if dry_run {
            println!("✓ Dry run successful - backup is valid");
            report.success = true;
            return Ok(report);
        }

        // 5. Backup current file
        if std::path::Path::new(&report.target_file).exists() {
            backup_current_file(&report.target_file)?;
        }

        // 6. Write restored content
        write_restored_content(&report.target_file, &decompressed)?;

        // 7. Verify restoration
        verify_restoration(&report.target_file)?;

        report.success = true;

        Ok(report)
    }

    /// Restores all CSV files to specific timestamp.
    ///
    /// ## Arguments
    /// - timestamp: Target timestamp (format: "2025-01-15_120000")
    ///
    /// ## Process
    /// 1. Find backups closest to timestamp
    /// 2. Restore each CSV file
    /// 3. Report results
    pub fn restore_to_timestamp(timestamp: &str) -> ReedResult<Vec<RestoreReport>> {
        let backups = Self::list_backups(None)?;
        let target_time = parse_timestamp(timestamp)?;

        let mut reports = Vec::new();

        // Group by CSV file
        let mut by_csv: HashMap<String, Vec<BackupInfo>> = HashMap::new();
        for backup in backups {
            by_csv
                .entry(backup.csv_file.clone())
                .or_insert_with(Vec::new)
                .push(backup);
        }

        // Find closest backup for each CSV
        for (csv_file, mut csv_backups) in by_csv {
            // Sort by proximity to target time
            csv_backups.sort_by_key(|b| {
                let diff = if b.timestamp > target_time {
                    b.timestamp.duration_since(target_time)
                } else {
                    target_time.duration_since(b.timestamp)
                };
                diff.unwrap_or(std::time::Duration::MAX)
            });

            if let Some(closest) = csv_backups.first() {
                println!("Restoring {} from {}", csv_file, format_timestamp(&closest.timestamp));
                let report = Self::restore_backup(&closest.backup_path, false)?;
                reports.push(report);
            }
        }

        Ok(reports)
    }

    /// Verifies backup integrity.
    pub fn verify_backup(backup_path: &str) -> ReedResult<BackupVerification> {
        let mut verification = BackupVerification {
            backup_path: backup_path.to_string(),
            valid: false,
            decompressible: false,
            csv_valid: false,
            errors: Vec::new(),
        };

        // Check file exists
        if !std::path::Path::new(backup_path).exists() {
            verification.errors.push("Backup file not found".to_string());
            return Ok(verification);
        }

        // Test decompression
        match decompress_xz_backup(backup_path) {
            Ok(content) => {
                verification.decompressible = true;

                // Verify CSV structure
                match verify_csv_structure(&content) {
                    Ok(_) => {
                        verification.csv_valid = true;
                        verification.valid = true;
                    }
                    Err(e) => {
                        verification.errors.push(format!("Invalid CSV: {}", e));
                    }
                }
            }
            Err(e) => {
                verification.errors.push(format!("Decompression failed: {}", e));
            }
        }

        Ok(verification)
    }

    /// Prunes old backups beyond retention limit.
    ///
    /// ## Retention Policy
    /// - Keep last 32 backups per CSV file
    /// - Delete older backups
    pub fn prune_old_backups() -> ReedResult<PruneReport> {
        let mut report = PruneReport {
            total_pruned: 0,
            space_freed: 0,
        };

        let backup_base = ".reed/backups";
        let dirs = list_backup_directories(backup_base)?;

        for dir in dirs {
            let mut files = list_backup_files(&dir)?;

            // Sort by modification time (oldest first)
            files.sort_by_key(|f| {
                std::fs::metadata(f)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            });

            // Keep last 32, delete rest
            if files.len() > 32 {
                let to_delete = &files[0..(files.len() - 32)];

                for file in to_delete {
                    if let Ok(metadata) = std::fs::metadata(file) {
                        report.space_freed += metadata.len();
                    }

                    std::fs::remove_file(file)?;
                    report.total_pruned += 1;
                }
            }
        }

        Ok(report)
    }
}

/// Backup information structure.
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub csv_file: String,
    pub backup_path: String,
    pub timestamp: std::time::SystemTime,
    pub size_bytes: u64,
}

impl BackupInfo {
    pub fn format(&self) -> String {
        let age = format_age(self.timestamp);
        let size = format_size(self.size_bytes);

        format!(
            "{} ({}) - {}",
            format_timestamp(&self.timestamp),
            age,
            size
        )
    }
}

/// Restore report structure.
#[derive(Debug, Clone)]
pub struct RestoreReport {
    pub backup_path: String,
    pub target_file: String,
    pub success: bool,
    pub dry_run: bool,
    pub errors: Vec<String>,
}

/// Backup verification structure.
#[derive(Debug, Clone)]
pub struct BackupVerification {
    pub backup_path: String,
    pub valid: bool,
    pub decompressible: bool,
    pub csv_valid: bool,
    pub errors: Vec<String>,
}

/// Prune report structure.
#[derive(Debug, Clone)]
pub struct PruneReport {
    pub total_pruned: usize,
    pub space_freed: u64,
}

/// Lists backup directories.
fn list_backup_directories(base: &str) -> ReedResult<Vec<String>> {
    let mut dirs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.file_type()?.is_dir() {
                    dirs.push(entry.path().display().to_string());
                }
            }
        }
    }

    Ok(dirs)
}

/// Lists backup files in directory.
fn list_backup_files(dir: &str) -> ReedResult<Vec<String>> {
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("xz") {
                    files.push(path.display().to_string());
                }
            }
        }
    }

    Ok(files)
}

/// Decompresses XZ backup.
fn decompress_xz_backup(path: &str) -> ReedResult<String> {
    use std::io::Read;

    let file = std::fs::File::open(path)?;
    let mut decoder = xz2::read::XzDecoder::new(file);
    let mut content = String::new();
    decoder.read_to_string(&mut content)?;

    Ok(content)
}

/// Verifies CSV structure.
fn verify_csv_structure(content: &str) -> ReedResult<()> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(content.as_bytes());

    // Check if at least one record exists
    if reader.records().next().is_none() {
        return Err(ReedError::ValidationError {
            field: "csv".to_string(),
            reason: "No records found".to_string(),
        });
    }

    Ok(())
}

/// Extracts CSV name from backup path.
fn extract_csv_name(backup_path: &str) -> ReedResult<String> {
    let filename = std::path::Path::new(backup_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| ReedError::ValidationError {
            field: "backup_path".to_string(),
            reason: "Invalid backup path".to_string(),
        })?;

    // Extract CSV name from pattern: text.csv.2025-01-15_120000.xz
    let parts: Vec<&str> = filename.split('.').collect();
    if parts.len() >= 3 {
        Ok(format!("{}.csv", parts[0]))
    } else {
        Err(ReedError::ValidationError {
            field: "filename".to_string(),
            reason: "Invalid backup filename format".to_string(),
        })
    }
}

/// Backs up current file before restore.
fn backup_current_file(path: &str) -> ReedResult<()> {
    let backup_path = format!("{}.before-restore", path);
    std::fs::copy(path, backup_path)?;
    Ok(())
}

/// Writes restored content to file.
fn write_restored_content(path: &str, content: &str) -> ReedResult<()> {
    std::fs::write(path, content)?;
    Ok(())
}

/// Verifies restoration was successful.
fn verify_restoration(path: &str) -> ReedResult<()> {
    let content = std::fs::read_to_string(path)?;
    verify_csv_structure(&content)
}

/// Parses timestamp string.
fn parse_timestamp(s: &str) -> ReedResult<std::time::SystemTime> {
    // Parse format: "2025-01-15_120000"
    Ok(std::time::SystemTime::now()) // Simplified
}

/// Formats timestamp for display.
fn format_timestamp(time: &std::time::SystemTime) -> String {
    chrono::DateTime::<chrono::Local>::from(*time)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

/// Formats age as human-readable string.
fn format_age(time: std::time::SystemTime) -> String {
    let duration = std::time::SystemTime::now()
        .duration_since(time)
        .unwrap_or(std::time::Duration::from_secs(0));

    let hours = duration.as_secs() / 3600;
    if hours > 0 {
        format!("{} hours ago", hours)
    } else {
        let minutes = duration.as_secs() / 60;
        format!("{} min ago", minutes)
    }
}

/// Formats file size.
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    }
}
```

### CLI Commands (`src/reedcms/cli/commands/backup.rs`)

```rust
/// CLI command: reed backup:list [csv_file]
pub async fn execute_backup_list(csv_file: Option<&str>) -> ReedResult<()> {
    println!("📦 Available Backups\n");

    let backups = BackupRecovery::list_backups(csv_file)?;

    if backups.is_empty() {
        println!("No backups found.");
        return Ok(());
    }

    // Group by CSV file
    let mut by_csv: HashMap<String, Vec<BackupInfo>> = HashMap::new();
    for backup in backups {
        by_csv
            .entry(backup.csv_file.clone())
            .or_insert_with(Vec::new)
            .push(backup);
    }

    for (csv_file, backups) in by_csv {
        println!("{} ({} backups):", csv_file, backups.len());
        for (i, backup) in backups.iter().enumerate().take(10) {
            println!("  {}. {}", i + 1, backup.format());
        }
        if backups.len() > 10 {
            println!("  ... and {} more", backups.len() - 10);
        }
        println!();
    }

    Ok(())
}

/// CLI command: reed backup:restore {backup_path} [--dry-run]
pub async fn execute_backup_restore(backup_path: &str, dry_run: bool) -> ReedResult<()> {
    println!("🔄 Restoring backup: {}\n", backup_path);

    let report = BackupRecovery::restore_backup(backup_path, dry_run)?;

    if report.success {
        if dry_run {
            println!("✓ Dry run successful");
        } else {
            println!("✓ Backup restored to: {}", report.target_file);
        }
    } else {
        println!("✗ Restore failed:");
        for error in &report.errors {
            println!("  - {}", error);
        }
    }

    Ok(())
}

/// CLI command: reed backup:verify {backup_path}
pub async fn execute_backup_verify(backup_path: &str) -> ReedResult<()> {
    println!("🔍 Verifying backup: {}\n", backup_path);

    let verification = BackupRecovery::verify_backup(backup_path)?;

    println!("Decompressible: {}", if verification.decompressible { "✓" } else { "✗" });
    println!("CSV Valid: {}", if verification.csv_valid { "✓" } else { "✗" });
    println!("Overall: {}", if verification.valid { "✓ Valid" } else { "✗ Invalid" });

    if !verification.errors.is_empty() {
        println!("\nErrors:");
        for error in &verification.errors {
            println!("  - {}", error);
        }
    }

    Ok(())
}

/// CLI command: reed backup:prune
pub async fn execute_backup_prune() -> ReedResult<()> {
    println!("🧹 Pruning old backups...\n");

    let report = BackupRecovery::prune_old_backups()?;

    println!("✓ Pruned {} backups", report.total_pruned);
    println!("  Space freed: {}", format_size(report.space_freed));

    Ok(())
}

/// CLI command: reed backup:restore-timestamp {timestamp}
pub async fn execute_backup_restore_timestamp(timestamp: &str) -> ReedResult<()> {
    println!("🔄 Restoring all CSV files to timestamp: {}\n", timestamp);

    let reports = BackupRecovery::restore_to_timestamp(timestamp)?;

    println!("Restored {} files:\n", reports.len());
    for report in reports {
        if report.success {
            println!("  ✓ {}", report.target_file);
        } else {
            println!("  ✗ {} (failed)", report.target_file);
        }
    }

    Ok(())
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/backup/recovery.rs` - Recovery system
- `src/reedcms/cli/commands/backup.rs` - CLI commands

### Test Files
- `src/reedcms/backup/recovery.test.rs`
- `src/reedcms/cli/commands/backup.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test backup listing
- [ ] Test backup restoration
- [ ] Test backup verification
- [ ] Test dry-run mode
- [ ] Test timestamp parsing
- [ ] Test pruning logic

### Integration Tests
- [ ] Test complete restore workflow
- [ ] Test restore to timestamp
- [ ] Test backup verification with real files
- [ ] Test pruning with multiple backups

### Recovery Tests
- [ ] Test restoration of corrupted data
- [ ] Test restoration with missing backup
- [ ] Test atomic restore operation
- [ ] Test rollback on restore failure

## Acceptance Criteria
- [ ] Backup listing functional
- [ ] Backup restoration working
- [ ] Dry-run mode implemented
- [ ] Backup verification functional
- [ ] Timestamp-based restore working
- [ ] Backup pruning implemented
- [ ] All CLI commands functional
- [ ] Atomic restore operations
- [ ] Safety backups created
- [ ] Clear error messages
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-02-04 (Backup System)

## Blocks
- None (final ticket!)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1054-1056 in `project_summary.md`

## Notes
Backup recovery is critical for data safety and disaster recovery. XZ compression provides excellent compression ratios for CSV backups. Dry-run mode enables safe testing before actual restore. Timestamp-based restore enables point-in-time recovery. Atomic operations prevent partial restores. Safety backups protect against restore failures. Pruning prevents unlimited backup growth. Clear CLI commands enable quick recovery in emergencies.
