// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for CSV merge functionality.

#[cfg(test)]
mod tests {
    use crate::concurrent::types::CsvRow;
    use crate::merge::csv::{
        build_row_map, calculate_merge_stats, detect_conflicts, merge_changes, merge_single,
        rows_equal,
    };
    use crate::merge::types::MergeResult;

    fn create_row(key: &str, values: Vec<&str>) -> CsvRow {
        CsvRow::new(key, values)
    }

    #[test]
    fn test_merge_different_rows() {
        let base = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];

        let changes_a = vec![
            create_row("1", vec!["Alice", "31"]), // Update row 1
        ];

        let changes_b = vec![
            create_row("3", vec!["Charlie", "35"]), // Insert row 3
        ];

        let result = merge_changes(&base, &changes_a, &changes_b).unwrap();

        match result {
            MergeResult::Success(rows) => {
                assert_eq!(rows.len(), 3);
                assert_eq!(rows[0].key, "1");
                assert_eq!(rows[0].values[1], "31");
                assert_eq!(rows[2].key, "3");
            }
            MergeResult::Conflicts(_) => panic!("Expected success, got conflicts"),
        }
    }

    #[test]
    fn test_merge_same_row_conflict() {
        let base = vec![create_row("1", vec!["Alice", "30"])];

        let changes_a = vec![create_row("1", vec!["Alice", "31"])];

        let changes_b = vec![create_row("1", vec!["Alice", "32"])];

        let result = merge_changes(&base, &changes_a, &changes_b).unwrap();

        match result {
            MergeResult::Success(_) => panic!("Expected conflicts, got success"),
            MergeResult::Conflicts(conflicts) => {
                assert_eq!(conflicts.len(), 1);
                assert_eq!(conflicts[0].key, "1");
                assert_eq!(conflicts[0].change_a.values[1], "31");
                assert_eq!(conflicts[0].change_b.values[1], "32");
            }
        }
    }

    #[test]
    fn test_merge_single() {
        let base = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];

        let changes = vec![
            create_row("1", vec!["Alice", "31"]),
            create_row("3", vec!["Charlie", "35"]),
        ];

        let merged = merge_single(&base, &changes).unwrap();

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].values[1], "31");
        assert_eq!(merged[2].key, "3");
    }

    #[test]
    fn test_merge_insert_only() {
        let base = vec![create_row("1", vec!["Alice", "30"])];

        let changes_a = vec![create_row("2", vec!["Bob", "25"])];

        let changes_b = vec![create_row("3", vec!["Charlie", "35"])];

        let result = merge_changes(&base, &changes_a, &changes_b).unwrap();

        match result {
            MergeResult::Success(rows) => {
                assert_eq!(rows.len(), 3);
                assert_eq!(rows[0].key, "1");
                assert_eq!(rows[1].key, "2");
                assert_eq!(rows[2].key, "3");
            }
            MergeResult::Conflicts(_) => panic!("Expected success, got conflicts"),
        }
    }

    #[test]
    fn test_detect_conflicts() {
        let changes_a = vec![
            create_row("1", vec!["Alice", "31"]),
            create_row("2", vec!["Bob", "26"]),
        ];

        let changes_b = vec![
            create_row("1", vec!["Alice", "32"]),
            create_row("3", vec!["Charlie", "35"]),
        ];

        let conflicts = detect_conflicts(&changes_a, &changes_b);

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0], "1");
    }

    #[test]
    fn test_detect_conflicts_none() {
        let changes_a = vec![create_row("1", vec!["Alice", "31"])];

        let changes_b = vec![create_row("2", vec!["Bob", "25"])];

        let conflicts = detect_conflicts(&changes_a, &changes_b);

        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_rows_equal() {
        let row_a = create_row("1", vec!["Alice", "30"]);
        let row_b = create_row("1", vec!["Alice", "30"]);
        let row_c = create_row("1", vec!["Alice", "31"]);
        let row_d = create_row("2", vec!["Alice", "30"]);

        assert!(rows_equal(&row_a, &row_b));
        assert!(!rows_equal(&row_a, &row_c));
        assert!(!rows_equal(&row_a, &row_d));
    }

    #[test]
    fn test_calculate_merge_stats() {
        let stats = calculate_merge_stats(100, 105, 2);

        assert_eq!(stats.added, 5);
        assert_eq!(stats.deleted, 0);
        assert_eq!(stats.conflicts, 2);
    }

    #[test]
    fn test_calculate_merge_stats_deletions() {
        let stats = calculate_merge_stats(100, 95, 0);

        assert_eq!(stats.added, 0);
        assert_eq!(stats.deleted, 5);
        assert_eq!(stats.conflicts, 0);
    }

    #[test]
    fn test_build_row_map() {
        let rows = vec![
            create_row("1", vec!["Alice"]),
            create_row("2", vec!["Bob"]),
            create_row("3", vec!["Charlie"]),
        ];

        let map = build_row_map(&rows);

        assert_eq!(map.len(), 3);
        assert!(map.contains_key("1"));
        assert!(map.contains_key("2"));
        assert!(map.contains_key("3"));
        assert_eq!(map.get("1").unwrap().values[0], "Alice");
    }

    #[test]
    fn test_merge_empty_base() {
        let base = vec![];

        let changes_a = vec![create_row("1", vec!["Alice", "30"])];

        let changes_b = vec![create_row("2", vec!["Bob", "25"])];

        let result = merge_changes(&base, &changes_a, &changes_b).unwrap();

        match result {
            MergeResult::Success(rows) => {
                assert_eq!(rows.len(), 2);
            }
            MergeResult::Conflicts(_) => panic!("Expected success, got conflicts"),
        }
    }

    #[test]
    fn test_merge_empty_changes() {
        let base = vec![create_row("1", vec!["Alice", "30"])];

        let changes_a = vec![];
        let changes_b = vec![];

        let result = merge_changes(&base, &changes_a, &changes_b).unwrap();

        match result {
            MergeResult::Success(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].key, "1");
            }
            MergeResult::Conflicts(_) => panic!("Expected success, got conflicts"),
        }
    }

    #[test]
    fn test_merge_sorts_by_key() {
        let base = vec![];

        let changes_a = vec![create_row("3", vec!["Charlie"])];

        let changes_b = vec![create_row("1", vec!["Alice"]), create_row("2", vec!["Bob"])];

        let result = merge_changes(&base, &changes_a, &changes_b).unwrap();

        match result {
            MergeResult::Success(rows) => {
                assert_eq!(rows[0].key, "1");
                assert_eq!(rows[1].key, "2");
                assert_eq!(rows[2].key, "3");
            }
            MergeResult::Conflicts(_) => panic!("Expected success, got conflicts"),
        }
    }
}
