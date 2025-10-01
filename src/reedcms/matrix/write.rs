// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Matrix CSV Writing Operations
//!
//! Provides write_matrix_csv() function with atomic write pattern.

use crate::reedcms::matrix::record::MatrixRecord;
use crate::reedcms::reedstream::{io_error, ReedResult};
use std::fs;
use std::path::Path;

/// Writes Matrix CSV records to file atomically.
///
/// ## Input
/// - `path`: Destination path for matrix CSV file
/// - `records`: Vector of MatrixRecord to write
/// - `field_names`: Field names for header (in order)
///
/// ## Output
/// - `ReedResult<()>`: Success or error
///
/// ## Performance
/// - O(n) where n is number of records
/// - < 20ms for < 1000 rows
///
/// ## Behaviour
/// - Writes header with field names
/// - Follows field_order from first record if field_names is empty
/// - Uses temp file + rename for atomic writes
/// - Adds 'desc' column if any record has description
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if temp file cannot be created
/// - Returns `ReedError::IoError` if write fails
/// - Returns `ReedError::IoError` if rename fails
/// - Cleans up temp file on error
///
/// ## Example Usage
/// ```
/// let mut records = vec![];
/// // ... populate records ...
/// write_matrix_csv(
///     ".reed/users.matrix.csv",
///     &records,
///     &["username", "status", "roles", "desc"]
/// )?;
/// ```
pub fn write_matrix_csv<P: AsRef<Path>>(
    path: P,
    records: &[MatrixRecord],
    field_names: &[&str],
) -> ReedResult<()> {
    let path_ref = path.as_ref();
    let path_str = path_ref.to_string_lossy().to_string();

    // Determine field names
    let fields: Vec<String> = if field_names.is_empty() {
        // Use field_order from first record
        if let Some(first) = records.first() {
            first.field_order.clone()
        } else {
            Vec::new()
        }
    } else {
        field_names.iter().map(|s| s.to_string()).collect()
    };

    if fields.is_empty() && !records.is_empty() {
        return Err(io_error(
            "write",
            path_str.clone(),
            "No field names provided and no records to infer from".to_string(),
        ));
    }

    // Check if any record has description
    let has_description = records.iter().any(|r| r.description.is_some());

    // Build header
    let mut header = fields.join("|");
    if has_description {
        header.push_str("|desc");
    }

    // Build content
    let mut content = String::new();
    content.push_str(&header);
    content.push('\n');

    for record in records {
        let mut parts: Vec<String> = Vec::new();

        for field_name in &fields {
            if let Some(value) = record.fields.get(field_name) {
                parts.push(value.to_csv_string());
            } else {
                parts.push(String::new());
            }
        }

        if has_description {
            if let Some(desc) = &record.description {
                parts.push(desc.clone());
            } else {
                parts.push(String::new());
            }
        }

        content.push_str(&parts.join("|"));
        content.push('\n');
    }

    // Create temp file path
    let temp_path = format!("{}.tmp", path_str);
    let temp_path_ref = Path::new(&temp_path);

    // Write to temp file
    fs::write(temp_path_ref, content)
        .map_err(|e| io_error("write", temp_path.clone(), e.to_string()))?;

    // Atomic rename
    fs::rename(temp_path_ref, path_ref).map_err(|e| {
        let _ = fs::remove_file(temp_path_ref);
        io_error("rename", path_str.clone(), e.to_string())
    })?;

    Ok(())
}
