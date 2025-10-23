// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Conflict resolution types for ReedBase.
//!
//! This module provides types for handling conflicts during CSV merge operations,
//! including resolution strategies and conflict representation.

use crate::concurrent::types::CsvRow;
use serde::{Deserialize, Serialize};

/// Resolution strategy for handling conflicts.
///
/// ## Strategies
/// - `LastWriteWins`: Accept the newer change (change_b)
/// - `FirstWriteWins`: Keep the earlier change (change_a)
/// - `Manual`: Write conflict to file for human resolution
/// - `KeepBoth`: Create two separate rows (append suffix to keys)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    LastWriteWins,
    FirstWriteWins,
    Manual,
    KeepBoth,
}

impl Default for ResolutionStrategy {
    fn default() -> Self {
        ResolutionStrategy::LastWriteWins
    }
}

impl ResolutionStrategy {
    /// Get strategy name as string.
    pub fn name(&self) -> &'static str {
        match self {
            ResolutionStrategy::LastWriteWins => "last-write-wins",
            ResolutionStrategy::FirstWriteWins => "first-write-wins",
            ResolutionStrategy::Manual => "manual",
            ResolutionStrategy::KeepBoth => "keep-both",
        }
    }

    /// Parse strategy from string.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "last-write-wins" => Some(ResolutionStrategy::LastWriteWins),
            "first-write-wins" => Some(ResolutionStrategy::FirstWriteWins),
            "manual" => Some(ResolutionStrategy::Manual),
            "keep-both" => Some(ResolutionStrategy::KeepBoth),
            _ => None,
        }
    }
}

/// Result of conflict resolution.
///
/// ## Variants
/// - `Automatic`: Conflict resolved automatically with resulting row
/// - `Manual`: Conflict written to file for human resolution
/// - `KeepBoth`: Both versions kept as separate rows
#[derive(Debug, Clone, PartialEq)]
pub enum Resolution {
    Automatic(CsvRow),
    Manual(String),
    KeepBoth(CsvRow, CsvRow),
}

impl Resolution {
    /// Check if resolution is automatic.
    pub fn is_automatic(&self) -> bool {
        matches!(self, Resolution::Automatic(_))
    }

    /// Check if resolution requires manual intervention.
    pub fn is_manual(&self) -> bool {
        matches!(self, Resolution::Manual(_))
    }

    /// Check if resolution keeps both versions.
    pub fn is_keep_both(&self) -> bool {
        matches!(self, Resolution::KeepBoth(_, _))
    }

    /// Get the filepath for manual resolution (if applicable).
    pub fn get_filepath(&self) -> Option<&str> {
        match self {
            Resolution::Manual(path) => Some(path),
            _ => None,
        }
    }

    /// Extract resolved rows from resolution.
    pub fn into_rows(self) -> Vec<CsvRow> {
        match self {
            Resolution::Automatic(row) => vec![row],
            Resolution::KeepBoth(row_a, row_b) => vec![row_a, row_b],
            Resolution::Manual(_) => vec![],
        }
    }
}

/// Conflict information for TOML serialization.
///
/// This structure is written to `.conflict` files for manual resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictFile {
    /// Conflict metadata
    pub metadata: ConflictMetadata,
    /// Base version (if available)
    pub base: Option<ConflictRow>,
    /// First change (change_a)
    pub change_a: ConflictRow,
    /// Second change (change_b)
    pub change_b: ConflictRow,
}

/// Conflict metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictMetadata {
    /// Key that has the conflict
    pub key: String,
    /// Table name
    pub table: String,
    /// Timestamp when conflict was detected (Unix timestamp)
    pub timestamp: u64,
    /// Resolution strategy used
    pub strategy: String,
}

/// Row representation in conflict file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRow {
    /// Row key
    pub key: String,
    /// Row values (column values)
    pub values: Vec<String>,
    /// Optional timestamp for the change
    pub timestamp: Option<u64>,
}

impl From<CsvRow> for ConflictRow {
    fn from(row: CsvRow) -> Self {
        ConflictRow {
            key: row.key,
            values: row.values,
            timestamp: None,
        }
    }
}

impl From<ConflictRow> for CsvRow {
    fn from(row: ConflictRow) -> Self {
        CsvRow {
            key: row.key,
            values: row.values,
        }
    }
}

impl ConflictFile {
    /// Create a new conflict file structure.
    pub fn new(
        table: String,
        key: String,
        base: Option<CsvRow>,
        change_a: CsvRow,
        change_b: CsvRow,
        strategy: ResolutionStrategy,
    ) -> Self {
        ConflictFile {
            metadata: ConflictMetadata {
                key: key.clone(),
                table,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                strategy: strategy.name().to_string(),
            },
            base: base.map(ConflictRow::from),
            change_a: ConflictRow::from(change_a),
            change_b: ConflictRow::from(change_b),
        }
    }

    /// Generate filename for this conflict.
    ///
    /// Format: `{timestamp}-{key}.conflict`
    pub fn filename(&self) -> String {
        format!("{}-{}.conflict", self.metadata.timestamp, self.metadata.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_strategy_name() {
        assert_eq!(ResolutionStrategy::LastWriteWins.name(), "last-write-wins");
        assert_eq!(
            ResolutionStrategy::FirstWriteWins.name(),
            "first-write-wins"
        );
        assert_eq!(ResolutionStrategy::Manual.name(), "manual");
        assert_eq!(ResolutionStrategy::KeepBoth.name(), "keep-both");
    }

    #[test]
    fn test_resolution_strategy_from_name() {
        assert_eq!(
            ResolutionStrategy::from_name("last-write-wins"),
            Some(ResolutionStrategy::LastWriteWins)
        );
        assert_eq!(
            ResolutionStrategy::from_name("first-write-wins"),
            Some(ResolutionStrategy::FirstWriteWins)
        );
        assert_eq!(
            ResolutionStrategy::from_name("manual"),
            Some(ResolutionStrategy::Manual)
        );
        assert_eq!(
            ResolutionStrategy::from_name("keep-both"),
            Some(ResolutionStrategy::KeepBoth)
        );
        assert_eq!(ResolutionStrategy::from_name("invalid"), None);
    }

    #[test]
    fn test_resolution_strategy_default() {
        assert_eq!(
            ResolutionStrategy::default(),
            ResolutionStrategy::LastWriteWins
        );
    }

    #[test]
    fn test_resolution_is_automatic() {
        let row = CsvRow {
            key: "test".to_string(),
            values: vec!["value".to_string()],
        };
        let res = Resolution::Automatic(row);
        assert!(res.is_automatic());
        assert!(!res.is_manual());
        assert!(!res.is_keep_both());
    }

    #[test]
    fn test_resolution_is_manual() {
        let res = Resolution::Manual("/path/to/conflict".to_string());
        assert!(!res.is_automatic());
        assert!(res.is_manual());
        assert!(!res.is_keep_both());
    }

    #[test]
    fn test_resolution_is_keep_both() {
        let row_a = CsvRow {
            key: "test-a".to_string(),
            values: vec!["value_a".to_string()],
        };
        let row_b = CsvRow {
            key: "test-b".to_string(),
            values: vec!["value_b".to_string()],
        };
        let res = Resolution::KeepBoth(row_a, row_b);
        assert!(!res.is_automatic());
        assert!(!res.is_manual());
        assert!(res.is_keep_both());
    }

    #[test]
    fn test_resolution_get_filepath() {
        let res = Resolution::Manual("/path/to/conflict".to_string());
        assert_eq!(res.get_filepath(), Some("/path/to/conflict"));

        let row = CsvRow {
            key: "test".to_string(),
            values: vec!["value".to_string()],
        };
        let res = Resolution::Automatic(row);
        assert_eq!(res.get_filepath(), None);
    }

    #[test]
    fn test_resolution_into_rows_automatic() {
        let row = CsvRow {
            key: "test".to_string(),
            values: vec!["value".to_string()],
        };
        let res = Resolution::Automatic(row.clone());
        let rows = res.into_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], row);
    }

    #[test]
    fn test_resolution_into_rows_keep_both() {
        let row_a = CsvRow {
            key: "test-a".to_string(),
            values: vec!["value_a".to_string()],
        };
        let row_b = CsvRow {
            key: "test-b".to_string(),
            values: vec!["value_b".to_string()],
        };
        let res = Resolution::KeepBoth(row_a.clone(), row_b.clone());
        let rows = res.into_rows();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], row_a);
        assert_eq!(rows[1], row_b);
    }

    #[test]
    fn test_resolution_into_rows_manual() {
        let res = Resolution::Manual("/path/to/conflict".to_string());
        let rows = res.into_rows();
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn test_conflict_row_from_csv_row() {
        let csv_row = CsvRow {
            key: "test.key".to_string(),
            values: vec!["value1".to_string(), "value2".to_string()],
        };
        let conflict_row: ConflictRow = csv_row.clone().into();
        assert_eq!(conflict_row.key, csv_row.key);
        assert_eq!(conflict_row.values, csv_row.values);
        assert_eq!(conflict_row.timestamp, None);
    }

    #[test]
    fn test_csv_row_from_conflict_row() {
        let conflict_row = ConflictRow {
            key: "test.key".to_string(),
            values: vec!["value1".to_string(), "value2".to_string()],
            timestamp: Some(1234567890),
        };
        let csv_row: CsvRow = conflict_row.clone().into();
        assert_eq!(csv_row.key, conflict_row.key);
        assert_eq!(csv_row.values, conflict_row.values);
    }

    #[test]
    fn test_conflict_file_new() {
        let base = CsvRow {
            key: "test.key".to_string(),
            values: vec!["old".to_string()],
        };
        let change_a = CsvRow {
            key: "test.key".to_string(),
            values: vec!["new_a".to_string()],
        };
        let change_b = CsvRow {
            key: "test.key".to_string(),
            values: vec!["new_b".to_string()],
        };

        let conflict = ConflictFile::new(
            "text".to_string(),
            "test.key".to_string(),
            Some(base.clone()),
            change_a.clone(),
            change_b.clone(),
            ResolutionStrategy::Manual,
        );

        assert_eq!(conflict.metadata.key, "test.key");
        assert_eq!(conflict.metadata.table, "text");
        assert_eq!(conflict.metadata.strategy, "manual");
        assert!(conflict.base.is_some());
        assert_eq!(conflict.change_a.key, change_a.key);
        assert_eq!(conflict.change_b.key, change_b.key);
    }

    #[test]
    fn test_conflict_file_filename() {
        let change_a = CsvRow {
            key: "test.key".to_string(),
            values: vec!["new_a".to_string()],
        };
        let change_b = CsvRow {
            key: "test.key".to_string(),
            values: vec!["new_b".to_string()],
        };

        let conflict = ConflictFile::new(
            "text".to_string(),
            "test.key".to_string(),
            None,
            change_a,
            change_b,
            ResolutionStrategy::Manual,
        );

        let filename = conflict.filename();
        assert!(filename.starts_with(&conflict.metadata.timestamp.to_string()));
        assert!(filename.ends_with("-test.key.conflict"));
    }
}
