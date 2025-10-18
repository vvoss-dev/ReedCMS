// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! List available backups.

use crate::backup::types::BackupInfo;
use crate::error::{ReedError, ReedResult};
use std::fs;
use std::path::Path;

/// List all available backups.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory (e.g., `.reed`)
///
/// ## Output
/// - `ReedResult<Vec<BackupInfo>>`: List of backups (newest first)
///
/// ## Performance
/// - < 10ms typical (read directory + stat files)
///
/// ## Error Conditions
/// - IoError: Cannot read backup directory
///
/// ## Example Usage
/// ```no_run
/// use reedbase::backup::list_backups;
/// use std::path::Path;
///
/// let backups = list_backups(Path::new(".reed"))?;
/// for backup in backups {
///     println!("{} - {:.2} MB", backup.timestamp, backup.size_mb);
/// }
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn list_backups(base_path: &Path) -> ReedResult<Vec<BackupInfo>> {
    let backup_dir = base_path.join("backups");

    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(&backup_dir).map_err(|e| ReedError::IoError {
        operation: "read_backup_dir".to_string(),
        reason: e.to_string(),
    })? {
        let entry = entry.map_err(|e| ReedError::IoError {
            operation: "read_dir_entry".to_string(),
            reason: e.to_string(),
        })?;

        let path = entry.path();

        // Only .tar.gz files
        if path.extension().and_then(|s| s.to_str()) != Some("gz") {
            continue;
        }

        // Parse timestamp from filename
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_suffix(".tar"));

        if let Some(ts_str) = filename {
            if let Ok(timestamp) = ts_str.parse::<u64>() {
                let size = entry
                    .metadata()
                    .map_err(|e| ReedError::IoError {
                        operation: "stat_backup".to_string(),
                        reason: e.to_string(),
                    })?
                    .len();

                backups.push(BackupInfo {
                    timestamp,
                    path: path.clone(),
                    size_bytes: size,
                    size_mb: size as f64 / 1_048_576.0,
                });
            }
        }
    }

    // Sort by timestamp (newest first)
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(backups)
}
