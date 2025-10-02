// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! CSV Reading Operations
//!
//! Provides read_csv() function for loading pipe-delimited CSV files.

use crate::reedcms::csv::record::{parse_row, CsvRecord};
use crate::reedcms::reedstream::{io_error, ReedError, ReedResult};
use std::fs;
use std::path::Path;

/// Reads a pipe-delimited CSV file into a vector of CsvRecord.
///
/// ## Input
/// - `path`: Path to CSV file (pipe-delimited)
///
/// ## Output
/// - `ReedResult<Vec<CsvRecord>>`: Vector of parsed records or error
///
/// ## Performance
/// - O(n) where n is number of rows
/// - < 1ms for < 1000 rows (SSD)
/// - Memory: ~100 bytes per record
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if file cannot be read
/// - Returns `ReedError::ParseError` if row format is invalid
/// - Skips empty lines and comment lines starting with '#'
///
/// ## Example Usage
/// ```
/// let records = read_csv(".reed/text.csv")?;
/// for record in records {
///     println!("{}: {}", record.key, record.value);
/// }
/// ```
pub fn read_csv<P: AsRef<Path>>(path: P) -> ReedResult<Vec<CsvRecord>> {
    let path_ref = path.as_ref();
    let path_str = path_ref.to_string_lossy().to_string();

    // Read file content
    let content = fs::read_to_string(path_ref)
        .map_err(|e| io_error("read", path_str.clone(), e.to_string()))?;

    // Parse rows
    let mut records = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse row with line number context
        let record = parse_row(trimmed).map_err(|e| match e {
            ReedError::ParseError { input, reason } => ReedError::ParseError {
                input: format!("{}:{} - {}", path_str, line_num + 1, input),
                reason,
            },
            other => other,
        })?;

        records.push(record);
    }

    Ok(records)
}
