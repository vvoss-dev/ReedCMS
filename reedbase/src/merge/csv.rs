// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Row-level CSV merging for concurrent writes.
//!
//! Automatically merges non-conflicting changes at row level.

use crate::concurrent::types::CsvRow;
use crate::error::ReedResult;
use crate::merge::types::{Conflict, MergeResult, MergeStats};
use std::collections::HashMap;

/// Merges two sets of changes into base CSV.
///
/// ## Input
/// - `base`: Base CSV rows
/// - `changes_a`: Changes from process A
/// - `changes_b`: Changes from process B
///
/// ## Output
/// - `ReedResult<MergeResult>`: Merged rows or conflicts
///
/// ## Performance
/// - O(n) where n = total rows
/// - < 50ms for 100 rows (no conflicts)
///
/// ## Error Conditions
/// - None (pure computation)
///
/// ## Example Usage
/// ```no_run
/// use reedbase::merge::{merge_changes, MergeResult};
/// use reedbase::concurrent::types::CsvRow;
///
/// let base = vec![CsvRow::new("1", vec!["Alice", "30"])];
/// let changes_a = vec![CsvRow::new("1", vec!["Alice", "31"])];
/// let changes_b = vec![CsvRow::new("2", vec!["Bob", "25"])];
///
/// let result = merge_changes(&base, &changes_a, &changes_b)?;
/// match result {
///     MergeResult::Success(rows) => println!("Merged {} rows", rows.len()),
///     MergeResult::Conflicts(conflicts) => println!("{} conflicts", conflicts.len()),
/// }
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn merge_changes(
    base: &[CsvRow],
    changes_a: &[CsvRow],
    changes_b: &[CsvRow],
) -> ReedResult<MergeResult> {
    let mut merged = build_row_map(base);
    let mut conflicts = Vec::new();

    // Apply changes from A
    for row in changes_a {
        merged.insert(row.key.clone(), row.clone());
    }

    // Apply changes from B, detecting conflicts
    for row in changes_b {
        if let Some(existing) = merged.get(&row.key) {
            // Check if this row was also modified by A
            if changes_a.iter().any(|a| a.key == row.key) {
                // Conflict: both A and B modified same row
                conflicts.push(Conflict {
                    key: row.key.clone(),
                    base: base.iter().find(|b| b.key == row.key).cloned(),
                    change_a: existing.clone(),
                    change_b: row.clone(),
                });
                continue;
            }
        }
        merged.insert(row.key.clone(), row.clone());
    }

    if conflicts.is_empty() {
        let mut rows: Vec<_> = merged.into_values().collect();
        rows.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(MergeResult::Success(rows))
    } else {
        Ok(MergeResult::Conflicts(conflicts))
    }
}

/// Merges single change set into base.
///
/// ## Input
/// - `base`: Base CSV rows
/// - `changes`: Changes to apply
///
/// ## Output
/// - `ReedResult<Vec<CsvRow>>`: Merged rows
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 20ms for 100 rows
///
/// ## Error Conditions
/// - None (pure computation)
///
/// ## Example Usage
/// ```no_run
/// use reedbase::merge::merge_single;
/// use reedbase::concurrent::types::CsvRow;
///
/// let base = vec![CsvRow::new("1", vec!["Alice", "30"])];
/// let changes = vec![CsvRow::new("2", vec!["Bob", "25"])];
///
/// let merged = merge_single(&base, &changes)?;
/// assert_eq!(merged.len(), 2);
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn merge_single(base: &[CsvRow], changes: &[CsvRow]) -> ReedResult<Vec<CsvRow>> {
    let mut merged = build_row_map(base);

    for row in changes {
        merged.insert(row.key.clone(), row.clone());
    }

    let mut rows: Vec<_> = merged.into_values().collect();
    rows.sort_by(|a, b| a.key.cmp(&b.key));

    Ok(rows)
}

/// Builds HashMap from CSV rows for fast lookup.
///
/// ## Input
/// - `rows`: CSV rows
///
/// ## Output
/// - `HashMap<String, CsvRow>`: Row map (key → row)
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 5ms for 100 rows
///
/// ## Example Usage
/// ```rust
/// use reedbase::merge::csv::build_row_map;
/// use reedbase::concurrent::types::CsvRow;
///
/// let rows = vec![CsvRow::new("1", vec!["Alice"])];
/// let map = build_row_map(&rows);
/// assert!(map.contains_key("1"));
/// ```
pub fn build_row_map(rows: &[CsvRow]) -> HashMap<String, CsvRow> {
    rows.iter()
        .map(|row| (row.key.clone(), row.clone()))
        .collect()
}

/// Detects conflicts between two change sets.
///
/// ## Input
/// - `changes_a`: Changes from process A
/// - `changes_b`: Changes from process B
///
/// ## Output
/// - `Vec<String>`: List of conflicting row keys
///
/// ## Performance
/// - O(n*m) where n,m = number of changes
/// - < 5ms for 100 changes each
///
/// ## Example Usage
/// ```no_run
/// use reedbase::merge::detect_conflicts;
/// use reedbase::concurrent::types::CsvRow;
///
/// let changes_a = vec![CsvRow::new("1", vec!["Alice", "31"])];
/// let changes_b = vec![CsvRow::new("1", vec!["Alice", "32"])];
///
/// let conflicts = detect_conflicts(&changes_a, &changes_b);
/// assert_eq!(conflicts.len(), 1);
/// ```
pub fn detect_conflicts(changes_a: &[CsvRow], changes_b: &[CsvRow]) -> Vec<String> {
    let keys_a: Vec<&String> = changes_a.iter().map(|r| &r.key).collect();

    changes_b
        .iter()
        .filter(|row| keys_a.contains(&&row.key))
        .map(|row| row.key.clone())
        .collect()
}

/// Checks if rows have same values.
///
/// ## Input
/// - `row_a`: First row
/// - `row_b`: Second row
///
/// ## Output
/// - `bool`: True if values match
///
/// ## Performance
/// - O(n) where n = number of columns
/// - < 1μs typical
///
/// ## Example Usage
/// ```no_run
/// use reedbase::merge::rows_equal;
/// use reedbase::concurrent::types::CsvRow;
///
/// let row_a = CsvRow::new("1", vec!["Alice", "30"]);
/// let row_b = CsvRow::new("1", vec!["Alice", "30"]);
///
/// assert!(rows_equal(&row_a, &row_b));
/// ```
pub fn rows_equal(row_a: &CsvRow, row_b: &CsvRow) -> bool {
    row_a.key == row_b.key && row_a.values == row_b.values
}

/// Calculates merge statistics.
///
/// ## Input
/// - `base_count`: Base rows count
/// - `merged_count`: Merged rows count
/// - `conflicts`: Number of conflicts
///
/// ## Output
/// - `MergeStats`: Merge statistics
///
/// ## Performance
/// - O(1) operation
/// - < 1μs
///
/// ## Example Usage
/// ```no_run
/// use reedbase::merge::calculate_merge_stats;
///
/// let stats = calculate_merge_stats(100, 105, 2);
/// assert_eq!(stats.added, 5);
/// assert_eq!(stats.conflicts, 2);
/// ```
pub fn calculate_merge_stats(
    base_count: usize,
    merged_count: usize,
    conflicts: usize,
) -> MergeStats {
    MergeStats {
        added: if merged_count > base_count {
            merged_count - base_count
        } else {
            0
        },
        deleted: if base_count > merged_count {
            base_count - merged_count
        } else {
            0
        },
        modified: 0, // TODO: Track modifications separately
        conflicts,
    }
}
