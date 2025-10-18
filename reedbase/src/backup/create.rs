// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Backup creation using tar + xz.

use crate::backup::types::BackupInfo;
use crate::error::{ReedError, ReedResult};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Create full backup of .reed/ directory.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory (e.g., `.reed`)
///
/// ## Output
/// - `ReedResult<BackupInfo>`: Backup metadata
///
/// ## Process
/// 1. Generate timestamp: SystemTime::now()
/// 2. Create backup directory if needed
/// 3. Create tar.gz: `tar czf backups/{timestamp}.tar.gz {base_path}/`
/// 4. Return backup info
///
/// ## Performance
/// - Depends on installation size
/// - Typical: < 30s for 100MB installation
///
/// ## Error Conditions
/// - IoError: Cannot create backup directory
/// - CommandFailed: tar command failed
///
/// ## Example Usage
/// ```no_run
/// use reedbase::backup::create_backup;
/// use std::path::Path;
///
/// let backup = create_backup(Path::new(".reed"))?;
/// println!("Backup created: {}", backup.path.display());
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn create_backup(base_path: &Path) -> ReedResult<BackupInfo> {
    // Generate timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before Unix epoch")
        .as_secs();

    // Ensure backup directory exists
    let backup_dir = base_path.join("backups");
    fs::create_dir_all(&backup_dir).map_err(|e| ReedError::IoError {
        operation: "create_backup_dir".to_string(),
        reason: e.to_string(),
    })?;

    // Backup path
    let backup_path = backup_dir.join(format!("{}.tar.gz", timestamp));

    // Execute tar command
    // tar czf backups/{timestamp}.tar.gz -C {parent} {dirname}
    let parent = base_path.parent().unwrap_or_else(|| Path::new("."));
    let dirname = base_path.file_name().ok_or_else(|| ReedError::IoError {
        operation: "get_basename".to_string(),
        reason: "Invalid base path".to_string(),
    })?;

    let output = Command::new("tar")
        .arg("czf")
        .arg(&backup_path)
        .arg("-C")
        .arg(parent)
        .arg(dirname)
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
    let size = fs::metadata(&backup_path)
        .map_err(|e| ReedError::IoError {
            operation: "stat_backup".to_string(),
            reason: e.to_string(),
        })?
        .len();

    Ok(BackupInfo {
        timestamp,
        path: backup_path,
        size_bytes: size,
        size_mb: size as f64 / 1_048_576.0,
    })
}
