// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Shared types for concurrent write operations.

use serde::{Deserialize, Serialize};

/// Pending write operation queued for processing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PendingWrite {
    /// CSV rows to write.
    pub rows: Vec<CsvRow>,

    /// Timestamp when write was queued (nanoseconds).
    pub timestamp: u64,

    /// Type of write operation.
    pub operation: WriteOperation,
}

/// Type of write operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WriteOperation {
    /// Insert new rows.
    Insert,

    /// Update existing rows.
    Update,

    /// Delete rows.
    Delete,
}

/// Single CSV row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CsvRow {
    /// Row key (first column).
    pub key: String,

    /// Row values (remaining columns).
    pub values: Vec<String>,
}

impl CsvRow {
    /// Creates new CSV row.
    ///
    /// ## Input
    /// - `key`: Row key
    /// - `values`: Row values
    ///
    /// ## Output
    /// - `CsvRow`: New row
    ///
    /// ## Example Usage
    /// ```rust
    /// let row = CsvRow::new("user:1", vec!["Alice", "alice@example.com"]);
    /// ```
    pub fn new<S: Into<String>>(key: S, values: Vec<S>) -> Self {
        Self {
            key: key.into(),
            values: values.into_iter().map(|v| v.into()).collect(),
        }
    }

    /// Converts row to CSV format.
    ///
    /// ## Output
    /// - `String`: CSV-formatted row
    ///
    /// ## Example Usage
    /// ```rust
    /// let row = CsvRow::new("user:1", vec!["Alice", "alice@example.com"]);
    /// assert_eq!(row.to_csv(), "user:1|Alice|alice@example.com");
    /// ```
    pub fn to_csv(&self) -> String {
        format!("{}|{}", self.key, self.values.join("|"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_row_new() {
        let row = CsvRow::new("user:1", vec!["Alice", "alice@example.com"]);
        assert_eq!(row.key, "user:1");
        assert_eq!(row.values, vec!["Alice", "alice@example.com"]);
    }

    #[test]
    fn test_csv_row_to_csv() {
        let row = CsvRow::new("user:1", vec!["Alice", "alice@example.com"]);
        assert_eq!(row.to_csv(), "user:1|Alice|alice@example.com");
    }

    #[test]
    fn test_pending_write_serialization() {
        let write = PendingWrite {
            rows: vec![CsvRow::new("user:1", vec!["Alice"])],
            timestamp: 1736860900000000000,
            operation: WriteOperation::Insert,
        };

        let json = serde_json::to_string(&write).unwrap();
        let deserialized: PendingWrite = serde_json::from_str(&json).unwrap();

        assert_eq!(write, deserialized);
    }
}
