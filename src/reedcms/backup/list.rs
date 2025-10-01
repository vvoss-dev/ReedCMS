// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Backup Listing Operations
//!
//! Provides list_backups() function for discovering available backups.

use crate::reedcms::reedstream::{io_error, ReedResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Information about a backup file.
#[derive(Debug, Clone, PartialEq)]
pub struct BackupInfo {
    /// Full path to the backup file
    pub path: PathBuf,

    /// Original filename (without timestamp and .xz extension)
    pub original_name: String,

    /// Timestamp string (YYYYMMDD_HHMMSS)
    pub timestamp: String,

    /// File size in bytes (compressed)
    pub size: u64,
}

/// Lists all backups for a given CSV file.
///
/// ## Input
/// - `source_path`: Path to the CSV file to find backups for
///
/// ## Output
/// - `ReedResult<Vec<BackupInfo>>`: List of backup information, sorted by timestamp (newest first)
///
/// ## Performance
/// - O(n log n) where n is number of backups
/// - < 10ms for typical backup directories (< 100 files)
///
/// ## Behaviour
/// - Searches `.reed/backups/` directory
/// - Returns empty vector if backups directory doesn't exist
/// - Filters for matching filename prefix and .xz extension
/// - Sorts by timestamp descending (newest first)
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if directory cannot be read
/// - Skips files with invalid names or metadata
///
/// ## Example Usage
/// ```
/// let backups = list_backups(".reed/text.csv")?;
/// for backup in backups {
///     println!("{}: {} bytes", backup.timestamp, backup.size);
/// }
/// ```
pub fn list_backups<P: AsRef<Path>>(source_path: P) -> ReedResult<Vec<BackupInfo>> {
    let source_ref = source_path.as_ref();
    let source_str = source_ref.to_string_lossy().to_string();

    // Get backup directory
    let backup_dir = source_ref
        .parent()
        .ok_or_else(|| {
            io_error(
                "backup",
                source_str.clone(),
                "No parent directory".to_string(),
            )
        })?
        .join("backups");

    // Return empty list if backups directory doesn't exist
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    // Get original filename
    let original_name = source_ref
        .file_name()
        .ok_or_else(|| io_error("backup", source_str.clone(), "No filename".to_string()))?
        .to_string_lossy()
        .to_string();

    // Read directory
    let entries = fs::read_dir(&backup_dir).map_err(|e| {
        io_error(
            "readdir",
            backup_dir.to_string_lossy().to_string(),
            e.to_string(),
        )
    })?;

    let mut backups = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // Skip invalid entries
        };

        let path = entry.path();
        let filename = match path.file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => continue,
        };

        // Check if this is a backup for our file: {original_name}.{timestamp}.xz
        if !filename.starts_with(&original_name) || !filename.ends_with(".xz") {
            continue;
        }

        // Extract timestamp: remove original_name prefix and .xz suffix
        let timestamp_part = &filename[original_name.len()..filename.len() - 3];
        if !timestamp_part.starts_with('.') {
            continue;
        }
        let timestamp = timestamp_part[1..].to_string();

        // Get file size
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        backups.push(BackupInfo {
            path,
            original_name: original_name.clone(),
            timestamp,
            size: metadata.len(),
        });
    }

    // Sort by timestamp descending (newest first)
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(backups)
}
