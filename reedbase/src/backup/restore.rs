// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Point-in-time recovery using version.log timestamps.

use crate::backup::types::RestoreReport;
use crate::error::{ReedError, ReedResult};
use crate::tables::Table;
use std::fs;
use std::path::Path;

/// Restore all tables to consistent point-in-time.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory (e.g., `.reed`)
/// - `target_timestamp`: Unix timestamp (nanoseconds) to restore to
///
/// ## Output
/// - `ReedResult<RestoreReport>`: What was restored
///
/// ## Process (KISS Algorithm)
/// 1. List all tables in {base_path}/tables/
/// 2. For each table:
///    - Read version.log
///    - Find last entry where timestamp <= target
///    - Restore table to that version
/// 3. Return report of restored states
///
/// ## Consistency Guarantee
/// All tables will be at their state as of or before target_timestamp.
/// No mixed states (e.g., users@14:05 + orders@13:58 for target 14:00).
///
/// ## Performance
/// - < 1 minute typical (depends on delta chain length)
/// - Each table restored independently
///
/// ## Error Conditions
/// - NoTablesFound: No tables exist
/// - TableRestoreFailed: One or more tables failed to restore
///
/// ## Example Usage
/// ```no_run
/// use reedbase::backup::restore_point_in_time;
/// use std::path::Path;
/// use std::time::{SystemTime, UNIX_EPOCH};
///
/// // Restore to 2 hours ago
/// let two_hours_ago = SystemTime::now()
///     .duration_since(UNIX_EPOCH)
///     .unwrap()
///     .as_nanos() as u64 - (7200 * 1_000_000_000);
///
/// let report = restore_point_in_time(Path::new(".reed"), two_hours_ago)?;
/// println!("Restored {} tables", report.tables_restored.len());
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn restore_point_in_time(base_path: &Path, target_timestamp: u64) -> ReedResult<RestoreReport> {
    let tables_dir = base_path.join("tables");

    if !tables_dir.exists() {
        return Err(ReedError::NoTablesFound);
    }

    let mut report = RestoreReport::new(target_timestamp);

    // Iterate all tables
    for entry in fs::read_dir(&tables_dir).map_err(|e| ReedError::IoError {
        operation: "read_tables_dir".to_string(),
        reason: e.to_string(),
    })? {
        let entry = entry.map_err(|e| ReedError::IoError {
            operation: "read_dir_entry".to_string(),
            reason: e.to_string(),
        })?;

        if !entry
            .file_type()
            .map_err(|e| ReedError::IoError {
                operation: "get_file_type".to_string(),
                reason: e.to_string(),
            })?
            .is_dir()
        {
            continue;
        }

        let table_name = entry.file_name().to_string_lossy().to_string();
        let table = Table::new(base_path, &table_name);

        // Read version.log
        match table.list_versions() {
            Ok(versions) => {
                // Find best match: last version <= target
                let best_match = versions
                    .iter()
                    .filter(|v| v.timestamp <= target_timestamp)
                    .max_by_key(|v| v.timestamp);

                match best_match {
                    Some(version) => {
                        // Restore to this version
                        match table.rollback(version.timestamp, "system") {
                            Ok(_) => {
                                report
                                    .tables_restored
                                    .push((table_name.clone(), version.timestamp));
                            }
                            Err(e) => {
                                report.errors.push((table_name.clone(), e));
                            }
                        }
                    }
                    None => {
                        // Table didn't exist at target time
                        report.tables_skipped.push(table_name.clone());
                    }
                }
            }
            Err(e) => {
                report.errors.push((table_name.clone(), e));
            }
        }
    }

    Ok(report)
}
