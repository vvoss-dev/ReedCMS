// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! CSV Record Type-Safe Parsing
//!
//! Provides CsvRecord struct and helper functions for row operations.

use crate::reedcms::reedstream::{parse_error, ReedResult};
use serde::{Deserialize, Serialize};

/// Pipe-delimited CSV record with optional description.
///
/// Format: `key|value|description` or `key|value`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CsvRecord {
    /// Key in dot notation (e.g., "page-header.logo.title@de")
    pub key: String,

    /// Value associated with the key
    pub value: String,

    /// Optional description or metadata
    pub description: Option<String>,
}

impl CsvRecord {
    /// Creates a new CSV record.
    ///
    /// ## Arguments
    /// - `key`: Key in dot notation
    /// - `value`: Value string
    /// - `description`: Optional description
    ///
    /// ## Returns
    /// New CsvRecord instance
    ///
    /// ## Example
    /// ```
    /// let record = CsvRecord::new(
    ///     "page.title@en".to_string(),
    ///     "Welcome".to_string(),
    ///     Some("Homepage title".to_string())
    /// );
    /// ```
    pub fn new(key: String, value: String, description: Option<String>) -> Self {
        Self {
            key,
            value,
            description,
        }
    }
}

/// Parses a pipe-delimited CSV row into a CsvRecord.
///
/// ## Input
/// - `line`: String slice with pipe-delimited fields
///
/// ## Output
/// - `ReedResult<CsvRecord>`: Parsed record or error
///
/// ## Performance
/// - O(n) where n is line length
/// - < 10μs typical for standard rows
///
/// ## Error Conditions
/// - Returns `ReedError::ParseError` if row has < 2 fields
/// - Empty keys are rejected
///
/// ## Example Usage
/// ```
/// let record = parse_row("page.title@en|Welcome|Homepage title")?;
/// assert_eq!(record.key, "page.title@en");
/// assert_eq!(record.value, "Welcome");
/// assert_eq!(record.description, Some("Homepage title".to_string()));
/// ```
pub fn parse_row(line: &str) -> ReedResult<CsvRecord> {
    let parts: Vec<&str> = line.split('|').collect();

    if parts.len() < 2 {
        return Err(parse_error(
            line,
            format!("Expected at least 2 fields, found {}", parts.len()),
        ));
    }

    let key = parts[0].trim();
    if key.is_empty() {
        return Err(parse_error(line, "Key cannot be empty"));
    }

    let value = parts[1].trim();
    let description = if parts.len() >= 3 {
        let desc = parts[2].trim();
        if desc.is_empty() {
            None
        } else {
            Some(desc.to_string())
        }
    } else {
        None
    };

    Ok(CsvRecord {
        key: key.to_string(),
        value: value.to_string(),
        description,
    })
}

/// Creates a pipe-delimited CSV row from a CsvRecord.
///
/// ## Input
/// - `record`: Reference to CsvRecord
///
/// ## Output
/// - Pipe-delimited string
///
/// ## Performance
/// - O(1) string concatenation
/// - < 1μs typical
///
/// ## Example Usage
/// ```
/// let record = CsvRecord::new(
///     "page.title@en".to_string(),
///     "Welcome".to_string(),
///     Some("Homepage title".to_string())
/// );
/// let row = create_row(&record);
/// assert_eq!(row, "page.title@en|Welcome|Homepage title");
/// ```
pub fn create_row(record: &CsvRecord) -> String {
    match &record.description {
        Some(desc) => format!("{}|{}|{}", record.key, record.value, desc),
        None => format!("{}|{}", record.key, record.value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_record_new() {
        let record = CsvRecord::new(
            "test.key@en".to_string(),
            "test value".to_string(),
            Some("test description".to_string()),
        );

        assert_eq!(record.key, "test.key@en");
        assert_eq!(record.value, "test value");
        assert_eq!(record.description, Some("test description".to_string()));
    }

    #[test]
    fn test_csv_record_new_no_description() {
        let record = CsvRecord::new("test.key@en".to_string(), "test value".to_string(), None);

        assert_eq!(record.key, "test.key@en");
        assert_eq!(record.value, "test value");
        assert_eq!(record.description, None);
    }
}
