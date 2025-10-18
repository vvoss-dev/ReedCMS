// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Helper functions for table operations.

use crate::error::{ReedError, ReedResult};
use crate::tables::types::TableStats;
use std::fs;
use std::path::Path;

/// Lists all tables in database.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
///
/// ## Output
/// - `Result<Vec<String>>`: Table names (sorted)
///
/// ## Performance
/// - < 10ms for typical installations (< 50 tables)
///
/// ## Example Usage
/// ```no_run
/// use reedbase::tables::list_tables;
/// use std::path::Path;
///
/// let tables = list_tables(Path::new(".reed"))?;
/// for table in tables {
///     println!("Table: {}", table);
/// }
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn list_tables(base_path: &Path) -> ReedResult<Vec<String>> {
    let tables_dir = base_path.join("tables");

    if !tables_dir.exists() {
        return Ok(Vec::new());
    }

    let mut tables = Vec::new();

    for entry in fs::read_dir(&tables_dir).map_err(|e| ReedError::IoError {
        operation: "list_tables".to_string(),
        reason: e.to_string(),
    })? {
        let entry = entry.map_err(|e| ReedError::IoError {
            operation: "read_dir_entry".to_string(),
            reason: e.to_string(),
        })?;

        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                // Only include if current.csv exists
                if table_exists(base_path, name) {
                    tables.push(name.to_string());
                }
            }
        }
    }

    tables.sort();
    Ok(tables)
}

/// Checks if table exists.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
/// - `name`: Table name
///
/// ## Output
/// - `bool`: True if table exists
///
/// ## Performance
/// - < 100μs (file system check)
pub fn table_exists(base_path: &Path, name: &str) -> bool {
    let current_path = base_path.join("tables").join(name).join("current.csv");
    current_path.exists()
}

/// Gets table statistics.
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
///
/// ## Error Conditions
/// - TableNotFound: Table doesn't exist
///
/// ## Example Usage
/// ```no_run
/// use reedbase::tables::table_stats;
/// use std::path::Path;
///
/// let stats = table_stats(Path::new(".reed"), "text")?;
/// println!("Versions: {}", stats.version_count);
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn table_stats(base_path: &Path, name: &str) -> ReedResult<TableStats> {
    let table_dir = base_path.join("tables").join(name);

    if !table_exists(base_path, name) {
        return Err(ReedError::TableNotFound {
            name: name.to_string(),
        });
    }

    // Get current.csv size
    let current_path = table_dir.join("current.csv");
    let current_size = fs::metadata(&current_path)
        .map_err(|e| ReedError::IoError {
            operation: "stat_current".to_string(),
            reason: e.to_string(),
        })?
        .len();

    // Parse version.log for version count and timestamps
    let log_path = table_dir.join("version.log");
    let log_content = fs::read_to_string(&log_path).map_err(|e| ReedError::IoError {
        operation: "read_version_log".to_string(),
        reason: e.to_string(),
    })?;

    let mut version_count = 0usize;
    let mut latest_version = 0u64;
    let mut oldest_version = u64::MAX;
    let mut line_num = 0usize;

    for line in log_content.lines() {
        line_num += 1;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse: timestamp|action_code|user_code|delta_size
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 4 {
            return Err(ReedError::LogCorrupted {
                reason: format!(
                    "Invalid format at line {}: expected 4 fields, got {}",
                    line_num,
                    parts.len()
                ),
            });
        }

        let timestamp = parts[0]
            .parse::<u64>()
            .map_err(|_| ReedError::LogCorrupted {
                reason: format!("Invalid timestamp at line {}: '{}'", line_num, parts[0]),
            })?;

        version_count += 1;
        latest_version = latest_version.max(timestamp);
        oldest_version = oldest_version.min(timestamp);
    }

    // Count delta files and total size
    let mut deltas_size = 0u64;
    for entry in fs::read_dir(&table_dir).map_err(|e| ReedError::IoError {
        operation: "read_table_dir".to_string(),
        reason: e.to_string(),
    })? {
        let entry = entry.map_err(|e| ReedError::IoError {
            operation: "read_dir_entry".to_string(),
            reason: e.to_string(),
        })?;

        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "bsdiff" {
                let size = entry
                    .metadata()
                    .map_err(|e| ReedError::IoError {
                        operation: "stat_delta".to_string(),
                        reason: e.to_string(),
                    })?
                    .len();
                deltas_size += size;
            }
        }
    }

    Ok(TableStats {
        name: name.to_string(),
        current_size,
        deltas_size,
        version_count,
        latest_version,
        oldest_version,
    })
}
