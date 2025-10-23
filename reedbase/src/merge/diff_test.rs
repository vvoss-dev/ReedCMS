// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for diff calculation functionality.

#[cfg(test)]
mod tests {
    use crate::concurrent::types::CsvRow;
    use crate::merge::diff::{apply_changes, calculate_diff, count_changes};
    use crate::merge::types::RowChange;

    fn create_row(key: &str, values: Vec<&str>) -> CsvRow {
        CsvRow::new(key, values)
    }

    #[test]
    fn test_calculate_diff_insert() {
        let old = vec![create_row("1", vec!["Alice", "30"])];

        let new = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];

        let changes = calculate_diff(&old, &new).unwrap();

        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], RowChange::Insert(row) if row.key == "2"));
    }

    #[test]
    fn test_calculate_diff_update() {
        let old = vec![create_row("1", vec!["Alice", "30"])];

        let new = vec![create_row("1", vec!["Alice", "31"])];

        let changes = calculate_diff(&old, &new).unwrap();

        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], RowChange::Update(row) if row.values[1] == "31"));
    }

    #[test]
    fn test_calculate_diff_delete() {
        let old = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];

        let new = vec![create_row("1", vec!["Alice", "30"])];

        let changes = calculate_diff(&old, &new).unwrap();

        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], RowChange::Delete(key) if key == "2"));
    }

    #[test]
    fn test_calculate_diff_multiple() {
        let old = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];

        let new = vec![
            create_row("1", vec!["Alice", "31"]), // Update
            create_row("3", vec!["Charlie", "35"]), // Insert
                                                  // Row 2 deleted
        ];

        let changes = calculate_diff(&old, &new).unwrap();

        assert_eq!(changes.len(), 3);

        let has_update = changes
            .iter()
            .any(|c| matches!(c, RowChange::Update(row) if row.key == "1"));
        let has_insert = changes
            .iter()
            .any(|c| matches!(c, RowChange::Insert(row) if row.key == "3"));
        let has_delete = changes
            .iter()
            .any(|c| matches!(c, RowChange::Delete(key) if key == "2"));

        assert!(has_update);
        assert!(has_insert);
        assert!(has_delete);
    }

    #[test]
    fn test_calculate_diff_no_changes() {
        let old = vec![create_row("1", vec!["Alice", "30"])];

        let new = vec![create_row("1", vec!["Alice", "30"])];

        let changes = calculate_diff(&old, &new).unwrap();

        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_apply_changes() {
        let base = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];

        let changes = vec![
            RowChange::Update(create_row("1", vec!["Alice", "31"])),
            RowChange::Delete("2".to_string()),
            RowChange::Insert(create_row("3", vec!["Charlie", "35"])),
        ];

        let result = apply_changes(&base, &changes).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, "1");
        assert_eq!(result[0].values[1], "31");
        assert_eq!(result[1].key, "3");
    }

    #[test]
    fn test_apply_changes_insert_only() {
        let base = vec![create_row("1", vec!["Alice", "30"])];

        let changes = vec![RowChange::Insert(create_row("2", vec!["Bob", "25"]))];

        let result = apply_changes(&base, &changes).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, "1");
        assert_eq!(result[1].key, "2");
    }

    #[test]
    fn test_apply_changes_delete_only() {
        let base = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];

        let changes = vec![RowChange::Delete("2".to_string())];

        let result = apply_changes(&base, &changes).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, "1");
    }

    #[test]
    fn test_apply_changes_update_only() {
        let base = vec![create_row("1", vec!["Alice", "30"])];

        let changes = vec![RowChange::Update(create_row("1", vec!["Alice", "31"]))];

        let result = apply_changes(&base, &changes).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].values[1], "31");
    }

    #[test]
    fn test_count_changes() {
        let changes = vec![
            RowChange::Insert(create_row("1", vec![])),
            RowChange::Insert(create_row("2", vec![])),
            RowChange::Update(create_row("3", vec![])),
            RowChange::Delete("4".to_string()),
        ];

        let (ins, upd, del) = count_changes(&changes);

        assert_eq!(ins, 2);
        assert_eq!(upd, 1);
        assert_eq!(del, 1);
    }

    #[test]
    fn test_count_changes_empty() {
        let changes = vec![];

        let (ins, upd, del) = count_changes(&changes);

        assert_eq!(ins, 0);
        assert_eq!(upd, 0);
        assert_eq!(del, 0);
    }

    #[test]
    fn test_apply_changes_empty() {
        let base = vec![create_row("1", vec!["Alice", "30"])];

        let changes = vec![];

        let result = apply_changes(&base, &changes).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, "1");
    }

    #[test]
    fn test_apply_changes_sorts_by_key() {
        let base = vec![];

        let changes = vec![
            RowChange::Insert(create_row("3", vec!["Charlie"])),
            RowChange::Insert(create_row("1", vec!["Alice"])),
            RowChange::Insert(create_row("2", vec!["Bob"])),
        ];

        let result = apply_changes(&base, &changes).unwrap();

        assert_eq!(result[0].key, "1");
        assert_eq!(result[1].key, "2");
        assert_eq!(result[2].key, "3");
    }

    #[test]
    fn test_calculate_diff_empty_old() {
        let old = vec![];
        let new = vec![create_row("1", vec!["Alice", "30"])];

        let changes = calculate_diff(&old, &new).unwrap();

        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], RowChange::Insert(row) if row.key == "1"));
    }

    #[test]
    fn test_calculate_diff_empty_new() {
        let old = vec![create_row("1", vec!["Alice", "30"])];
        let new = vec![];

        let changes = calculate_diff(&old, &new).unwrap();

        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], RowChange::Delete(key) if key == "1"));
    }
}
