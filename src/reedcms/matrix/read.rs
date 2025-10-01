// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Matrix CSV Reading Operations
//!
//! Provides read_matrix_csv() function for loading structured CSV files.

use crate::reedcms::matrix::parse::parse_matrix_value;
use crate::reedcms::matrix::record::MatrixRecord;
use crate::reedcms::reedstream::{io_error, parse_error, ReedResult};
use std::fs;
use std::path::Path;

/// Reads a pipe-delimited Matrix CSV file.
///
/// ## Input
/// - `path`: Path to matrix CSV file
///
/// ## Output
/// - `ReedResult<Vec<MatrixRecord>>`: Vector of parsed records or error
///
/// ## Performance
/// - O(n) where n is number of rows
/// - < 20ms for < 1000 rows
///
/// ## Behaviour
/// - First line is header with field names
/// - Subsequent lines are data rows
/// - Last field may be 'desc' or 'description' (optional)
/// - Skips empty lines and comments (#)
/// - Automatically detects value types per field
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if file cannot be read
/// - Returns `ReedError::ParseError` if header missing
/// - Returns `ReedError::ParseError` if row has wrong field count
///
/// ## Example Usage
/// ```
/// let records = read_matrix_csv(".reed/users.matrix.csv")?;
/// for record in records {
///     if let Some(username) = record.get_field("username") {
///         println!("User: {:?}", username);
///     }
/// }
/// ```
pub fn read_matrix_csv<P: AsRef<Path>>(path: P) -> ReedResult<Vec<MatrixRecord>> {
    let path_ref = path.as_ref();
    let path_str = path_ref.to_string_lossy().to_string();

    // Read file content
    let content = fs::read_to_string(path_ref)
        .map_err(|e| io_error("read", path_str.clone(), e.to_string()))?;

    let mut lines = content.lines().enumerate();

    // Parse header
    let header = loop {
        if let Some((line_num, line)) = lines.next() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            break (line_num, trimmed);
        } else {
            return Err(parse_error(path_str, "No header found in matrix CSV file"));
        }
    };

    let field_names: Vec<String> = header.1.split('|').map(|s| s.trim().to_string()).collect();

    if field_names.is_empty() {
        return Err(parse_error(path_str, "Empty header in matrix CSV file"));
    }

    // Check if last field is description
    let has_description = field_names
        .last()
        .map(|name| name == "desc" || name == "description")
        .unwrap_or(false);

    let data_field_count = if has_description {
        field_names.len() - 1
    } else {
        field_names.len()
    };

    // Parse data rows
    let mut records = Vec::new();

    for (line_num, line) in lines {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split('|').collect();

        if parts.len() < data_field_count {
            return Err(parse_error(
                format!("{}:{}", path_str, line_num + 1),
                format!(
                    "Expected at least {} fields, found {}",
                    data_field_count,
                    parts.len()
                ),
            ));
        }

        let mut record = MatrixRecord::new();

        // Parse data fields
        for (i, field_name) in field_names.iter().take(data_field_count).enumerate() {
            let value = parts[i].trim();
            let matrix_value = parse_matrix_value(value);
            record.add_field(field_name.clone(), matrix_value);
        }

        // Parse description if present
        if has_description && parts.len() >= field_names.len() {
            let desc_idx = field_names.len() - 1;
            if desc_idx < parts.len() {
                let desc = parts[desc_idx].trim();
                if !desc.is_empty() {
                    record.set_description(desc.to_string());
                }
            }
        }

        records.push(record);
    }

    Ok(records)
}
