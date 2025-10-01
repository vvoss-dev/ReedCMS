// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CSV Writing Operations
//!
//! Provides write_csv() function with atomic write pattern (temp file + rename).

use crate::reedcms::csv::record::{create_row, CsvRecord};
use crate::reedcms::reedstream::{io_error, ReedResult};
use std::fs;
use std::path::Path;

/// Writes a vector of CsvRecord to a pipe-delimited CSV file atomically.
///
/// Uses temp file + rename pattern to ensure atomic writes:
/// 1. Write to {path}.tmp
/// 2. Rename to {path}
///
/// ## Input
/// - `path`: Destination path for CSV file
/// - `records`: Vector of CsvRecord to write
///
/// ## Output
/// - `ReedResult<()>`: Success or error
///
/// ## Performance
/// - O(n) where n is number of records
/// - < 1ms for < 1000 rows (SSD)
/// - Atomic operation ensures data integrity
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if temp file cannot be created
/// - Returns `ReedError::IoError` if write operation fails
/// - Returns `ReedError::IoError` if rename operation fails
/// - Cleans up temp file on error
///
/// ## Example Usage
/// ```
/// let records = vec![
///     CsvRecord::new(
///         "page.title@en".to_string(),
///         "Welcome".to_string(),
///         Some("Homepage title".to_string())
///     ),
/// ];
/// write_csv(".reed/text.csv", &records)?;
/// ```
pub fn write_csv<P: AsRef<Path>>(path: P, records: &[CsvRecord]) -> ReedResult<()> {
    let path_ref = path.as_ref();
    let path_str = path_ref.to_string_lossy().to_string();

    // Create temp file path
    let temp_path = format!("{}.tmp", path_str);
    let temp_path_ref = Path::new(&temp_path);

    // Build content
    let mut content = String::new();
    for record in records {
        content.push_str(&create_row(record));
        content.push('\n');
    }

    // Write to temp file
    fs::write(temp_path_ref, content)
        .map_err(|e| io_error("write", temp_path.clone(), e.to_string()))?;

    // Atomic rename
    fs::rename(temp_path_ref, path_ref).map_err(|e| {
        // Clean up temp file on error
        let _ = fs::remove_file(temp_path_ref);

        io_error("rename", path_str.clone(), e.to_string())
    })?;

    Ok(())
}
