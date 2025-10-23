// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Shared types for CSV merge operations.

use crate::concurrent::types::CsvRow;

/// Row change types.
#[derive(Debug, Clone, PartialEq)]
pub enum RowChange {
    /// Insert new row.
    Insert(CsvRow),

    /// Update existing row.
    Update(CsvRow),

    /// Delete row (key only).
    Delete(String),
}

/// Merge result.
#[derive(Debug)]
pub enum MergeResult {
    /// Successful merge with no conflicts.
    Success(Vec<CsvRow>),

    /// Merge had conflicts requiring manual resolution.
    Conflicts(Vec<Conflict>),
}

/// Merge conflict between two concurrent changes.
#[derive(Debug, Clone)]
pub struct Conflict {
    /// Row key where conflict occurred.
    pub key: String,

    /// Base version of the row (if it existed).
    pub base: Option<CsvRow>,

    /// Change from process A.
    pub change_a: CsvRow,

    /// Change from process B.
    pub change_b: CsvRow,
}

/// Merge statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct MergeStats {
    /// Number of rows added.
    pub added: usize,

    /// Number of rows deleted.
    pub deleted: usize,

    /// Number of rows modified.
    pub modified: usize,

    /// Number of conflicts detected.
    pub conflicts: usize,
}

impl MergeStats {
    /// Creates new merge statistics.
    ///
    /// ## Input
    /// - `added`: Number of rows added
    /// - `deleted`: Number of rows deleted
    /// - `modified`: Number of rows modified
    /// - `conflicts`: Number of conflicts
    ///
    /// ## Output
    /// - `MergeStats`: New statistics
    ///
    /// ## Example Usage
    /// ```rust
    /// let stats = MergeStats::new(5, 2, 10, 1);
    /// assert_eq!(stats.total_changes(), 17);
    /// ```
    pub fn new(added: usize, deleted: usize, modified: usize, conflicts: usize) -> Self {
        Self {
            added,
            deleted,
            modified,
            conflicts,
        }
    }

    /// Calculates total number of changes.
    ///
    /// ## Output
    /// - `usize`: Total changes
    ///
    /// ## Performance
    /// - O(1)
    ///
    /// ## Example Usage
    /// ```rust
    /// let stats = MergeStats::new(5, 2, 10, 1);
    /// assert_eq!(stats.total_changes(), 17);
    /// ```
    pub fn total_changes(&self) -> usize {
        self.added + self.deleted + self.modified
    }

    /// Checks if merge had no conflicts.
    ///
    /// ## Output
    /// - `bool`: True if no conflicts
    ///
    /// ## Performance
    /// - O(1)
    ///
    /// ## Example Usage
    /// ```rust
    /// let stats = MergeStats::new(5, 0, 0, 0);
    /// assert!(stats.is_clean());
    /// ```
    pub fn is_clean(&self) -> bool {
        self.conflicts == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_stats_new() {
        let stats = MergeStats::new(5, 2, 10, 1);
        assert_eq!(stats.added, 5);
        assert_eq!(stats.deleted, 2);
        assert_eq!(stats.modified, 10);
        assert_eq!(stats.conflicts, 1);
    }

    #[test]
    fn test_merge_stats_total_changes() {
        let stats = MergeStats::new(5, 2, 10, 1);
        assert_eq!(stats.total_changes(), 17);
    }

    #[test]
    fn test_merge_stats_is_clean() {
        let clean = MergeStats::new(5, 0, 0, 0);
        assert!(clean.is_clean());

        let dirty = MergeStats::new(5, 0, 0, 1);
        assert!(!dirty.is_clean());
    }
}
