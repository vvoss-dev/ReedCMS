// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Backup Cleanup Operations
//!
//! Provides cleanup_old_backups() function for maintaining backup retention policy.

use crate::reedcms::backup::list::list_backups;
use crate::reedcms::reedstream::{io_error, ReedResult};
use std::fs;
use std::path::Path;

/// Cleans up old backups, keeping only the most recent 32.
///
/// ## Input
/// - `source_path`: Path to the CSV file to clean backups for
///
/// ## Output
/// - `ReedResult<usize>`: Number of backups deleted or error
///
/// ## Performance
/// - O(n log n) where n is number of backups
/// - < 50ms for typical backup directories
///
/// ## Behaviour
/// - Lists all backups for the file
/// - Sorts by timestamp (newest first)
/// - Keeps the 32 newest backups
/// - Deletes all older backups
/// - Returns 0 if there are 32 or fewer backups
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if directory cannot be read
/// - Returns `ReedError::IoError` if files cannot be deleted
/// - Continues deletion even if some files fail (returns error after attempting all)
///
/// ## Example Usage
/// ```
/// let deleted = cleanup_old_backups(".reed/text.csv")?;
/// println!("Deleted {} old backups", deleted);
/// ```
pub fn cleanup_old_backups<P: AsRef<Path>>(source_path: P) -> ReedResult<usize> {
    const MAX_BACKUPS: usize = 32;

    let source_ref = source_path.as_ref();

    // Get all backups sorted by timestamp (newest first)
    let backups = list_backups(source_ref)?;

    // If we have 32 or fewer backups, nothing to delete
    if backups.len() <= MAX_BACKUPS {
        return Ok(0);
    }

    // Delete backups beyond the 32 newest
    let to_delete = &backups[MAX_BACKUPS..];
    let mut deleted_count = 0;
    let mut last_error: Option<String> = None;

    for backup in to_delete {
        match fs::remove_file(&backup.path) {
            Ok(_) => deleted_count += 1,
            Err(e) => {
                last_error = Some(format!(
                    "Failed to delete {}: {}",
                    backup.path.to_string_lossy(),
                    e
                ));
            }
        }
    }

    // If there was an error during deletion, return it
    if let Some(error_msg) = last_error {
        return Err(io_error(
            "cleanup",
            source_ref.to_string_lossy().to_string(),
            error_msg,
        ));
    }

    Ok(deleted_count)
}
