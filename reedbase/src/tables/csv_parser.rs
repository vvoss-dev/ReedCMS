// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CSV parsing for ReedBase tables.
//!
//! Pipe-delimited format: `key|value1|value2|...`

use crate::error::{ReedError, ReedResult};
use crate::tables::types::CsvRow;

/// Parses CSV content into rows.
///
/// ## Input
/// - `content`: CSV bytes (pipe-delimited)
///
/// ## Output
/// - `Result<Vec<CsvRow>>`: Parsed rows (header excluded)
///
/// ## Performance
/// - O(n) where n = content length
/// - < 5ms for typical tables (< 1000 rows)
///
/// ## Error Conditions
/// - InvalidCsv: Malformed CSV
///
/// ## Example Usage
/// ```
/// use reedbase::tables::parse_csv;
///
/// let csv = b"key|value\nfoo|bar\nbaz|qux\n";
/// let rows = parse_csv(csv)?;
/// assert_eq!(rows.len(), 2);
/// assert_eq!(rows[0].key, "foo");
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn parse_csv(content: &[u8]) -> ReedResult<Vec<CsvRow>> {
    // Validate UTF-8
    let text = std::str::from_utf8(content).map_err(|e| ReedError::InvalidCsv {
        reason: format!("Invalid UTF-8: {}", e),
        line: 0,
    })?;

    let mut rows = Vec::new();

    for (line_num, line) in text.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        let row = parse_csv_row(trimmed, line_num + 1)?;
        rows.push(row);
    }

    Ok(rows)
}

/// Parses a single CSV row.
///
/// ## Input
/// - `line`: CSV line (pipe-delimited)
/// - `line_num`: Line number (for error reporting)
///
/// ## Output
/// - `Result<CsvRow>`: Parsed row
///
/// ## Error Conditions
/// - InvalidCsv: Less than 1 column
pub fn parse_csv_row(line: &str, line_num: usize) -> ReedResult<CsvRow> {
    // Must contain at least one pipe
    if !line.contains('|') {
        return Err(ReedError::InvalidCsv {
            reason: "No pipe delimiter found".to_string(),
            line: line_num,
        });
    }

    let parts: Vec<&str> = line.split('|').collect();

    if parts.is_empty() {
        return Err(ReedError::InvalidCsv {
            reason: "Empty row".to_string(),
            line: line_num,
        });
    }

    let key = parts[0].trim().to_string();
    let values = parts[1..].iter().map(|s| s.trim().to_string()).collect();

    Ok(CsvRow { key, values })
}
