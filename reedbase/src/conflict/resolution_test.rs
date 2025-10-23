// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for conflict resolution logic.

#[cfg(test)]
mod tests {
    use crate::concurrent::types::CsvRow;
    use crate::conflict::resolution::{
        count_conflicts, delete_conflict_file, list_conflicts, load_conflict_file,
        resolve_conflict, write_conflict_file,
    };
    use crate::conflict::types::{Resolution, ResolutionStrategy};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_row(key: &str, value: &str) -> CsvRow {
        CsvRow {
            key: key.to_string(),
            values: vec![value.to_string()],
        }
    }

    #[test]
    fn test_resolve_conflict_last_write_wins() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let base = create_test_row("test.key", "old");
        let change_a = create_test_row("test.key", "new_a");
        let change_b = create_test_row("test.key", "new_b");

        let resolution = resolve_conflict(
            base_path,
            "text",
            "test.key",
            Some(base),
            change_a,
            change_b.clone(),
            ResolutionStrategy::LastWriteWins,
        )
        .unwrap();

        assert!(resolution.is_automatic());
        let rows = resolution.into_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], change_b);
    }

    #[test]
    fn test_resolve_conflict_first_write_wins() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let base = create_test_row("test.key", "old");
        let change_a = create_test_row("test.key", "new_a");
        let change_b = create_test_row("test.key", "new_b");

        let resolution = resolve_conflict(
            base_path,
            "text",
            "test.key",
            Some(base),
            change_a.clone(),
            change_b,
            ResolutionStrategy::FirstWriteWins,
        )
        .unwrap();

        assert!(resolution.is_automatic());
        let rows = resolution.into_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], change_a);
    }

    #[test]
    fn test_resolve_conflict_keep_both() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let base = create_test_row("test.key", "old");
        let change_a = create_test_row("test.key", "new_a");
        let change_b = create_test_row("test.key", "new_b");

        let resolution = resolve_conflict(
            base_path,
            "text",
            "test.key",
            Some(base),
            change_a.clone(),
            change_b.clone(),
            ResolutionStrategy::KeepBoth,
        )
        .unwrap();

        assert!(resolution.is_keep_both());
        let rows = resolution.into_rows();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].key, "test.key-a");
        assert_eq!(rows[0].values, change_a.values);
        assert_eq!(rows[1].key, "test.key-b");
        assert_eq!(rows[1].values, change_b.values);
    }

    #[test]
    fn test_resolve_conflict_manual() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let base = create_test_row("test.key", "old");
        let change_a = create_test_row("test.key", "new_a");
        let change_b = create_test_row("test.key", "new_b");

        let resolution = resolve_conflict(
            base_path,
            "text",
            "test.key",
            Some(base),
            change_a,
            change_b,
            ResolutionStrategy::Manual,
        )
        .unwrap();

        assert!(resolution.is_manual());
        let filepath = resolution.get_filepath().unwrap();
        assert!(filepath.contains("test.key"));
        assert!(filepath.ends_with(".conflict"));

        // Verify file was created
        assert!(PathBuf::from(filepath).exists());
    }

    #[test]
    fn test_write_conflict_file() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let base = create_test_row("test.key", "old");
        let change_a = create_test_row("test.key", "new_a");
        let change_b = create_test_row("test.key", "new_b");

        let filepath = write_conflict_file(
            base_path,
            "text",
            "test.key",
            Some(base),
            change_a,
            change_b,
            ResolutionStrategy::Manual,
        )
        .unwrap();

        // Verify file exists
        let path = PathBuf::from(&filepath);
        assert!(path.exists());

        // Verify content is valid TOML
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("[metadata]"));
        assert!(content.contains("key = \"test.key\""));
        assert!(content.contains("table = \"text\""));
        assert!(content.contains("[base]"));
        assert!(content.contains("[change_a]"));
        assert!(content.contains("[change_b]"));
    }

    #[test]
    fn test_write_conflict_file_no_base() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let change_a = create_test_row("test.key", "new_a");
        let change_b = create_test_row("test.key", "new_b");

        let filepath = write_conflict_file(
            base_path,
            "text",
            "test.key",
            None,
            change_a,
            change_b,
            ResolutionStrategy::Manual,
        )
        .unwrap();

        // Verify file exists
        let path = PathBuf::from(&filepath);
        assert!(path.exists());

        // Verify base is not in TOML
        let content = fs::read_to_string(&path).unwrap();
        assert!(!content.contains("[base]"));
    }

    #[test]
    fn test_load_conflict_file() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let base = create_test_row("test.key", "old");
        let change_a = create_test_row("test.key", "new_a");
        let change_b = create_test_row("test.key", "new_b");

        let filepath = write_conflict_file(
            base_path,
            "text",
            "test.key",
            Some(base.clone()),
            change_a.clone(),
            change_b.clone(),
            ResolutionStrategy::Manual,
        )
        .unwrap();

        // Load the conflict file
        let conflict = load_conflict_file(&PathBuf::from(&filepath)).unwrap();

        assert_eq!(conflict.metadata.key, "test.key");
        assert_eq!(conflict.metadata.table, "text");
        assert_eq!(conflict.metadata.strategy, "manual");
        assert!(conflict.base.is_some());
        assert_eq!(conflict.change_a.key, change_a.key);
        assert_eq!(conflict.change_b.key, change_b.key);
    }

    #[test]
    fn test_list_conflicts_empty() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let conflicts = list_conflicts(base_path, "text").unwrap();
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_list_conflicts_multiple() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        // Create multiple conflict files
        for i in 1..=3 {
            let key = format!("test.key.{}", i);
            let change_a = create_test_row(&key, "new_a");
            let change_b = create_test_row(&key, "new_b");

            write_conflict_file(
                base_path,
                "text",
                &key,
                None,
                change_a,
                change_b,
                ResolutionStrategy::Manual,
            )
            .unwrap();
        }

        let conflicts = list_conflicts(base_path, "text").unwrap();
        assert_eq!(conflicts.len(), 3);
    }

    #[test]
    fn test_list_conflicts_filters_non_conflict_files() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        // Create conflict directory
        let conflict_dir = base_path.join("tables").join("text").join("conflicts");
        fs::create_dir_all(&conflict_dir).unwrap();

        // Create a conflict file
        let change_a = create_test_row("test.key", "new_a");
        let change_b = create_test_row("test.key", "new_b");
        write_conflict_file(
            base_path,
            "text",
            "test.key",
            None,
            change_a,
            change_b,
            ResolutionStrategy::Manual,
        )
        .unwrap();

        // Create a non-conflict file
        fs::write(conflict_dir.join("other.txt"), "not a conflict").unwrap();

        let conflicts = list_conflicts(base_path, "text").unwrap();
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].to_str().unwrap().ends_with(".conflict"));
    }

    #[test]
    fn test_delete_conflict_file() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let change_a = create_test_row("test.key", "new_a");
        let change_b = create_test_row("test.key", "new_b");

        let filepath = write_conflict_file(
            base_path,
            "text",
            "test.key",
            None,
            change_a,
            change_b,
            ResolutionStrategy::Manual,
        )
        .unwrap();

        let path = PathBuf::from(&filepath);
        assert!(path.exists());

        // Delete the conflict file
        delete_conflict_file(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_delete_conflict_file_not_exists() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();
        let nonexistent = base_path.join("nonexistent.conflict");

        let result = delete_conflict_file(&nonexistent);
        assert!(result.is_err());
    }

    #[test]
    fn test_count_conflicts() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        // Initially no conflicts
        let count = count_conflicts(base_path, "text").unwrap();
        assert_eq!(count, 0);

        // Create conflicts
        for i in 1..=5 {
            let key = format!("test.key.{}", i);
            let change_a = create_test_row(&key, "new_a");
            let change_b = create_test_row(&key, "new_b");

            write_conflict_file(
                base_path,
                "text",
                &key,
                None,
                change_a,
                change_b,
                ResolutionStrategy::Manual,
            )
            .unwrap();
        }

        let count = count_conflicts(base_path, "text").unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_conflict_file_persistence() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let base = create_test_row("test.key", "old");
        let change_a = create_test_row("test.key", "new_a");
        let change_b = create_test_row("test.key", "new_b");

        // Write conflict
        let filepath = write_conflict_file(
            base_path,
            "text",
            "test.key",
            Some(base.clone()),
            change_a.clone(),
            change_b.clone(),
            ResolutionStrategy::Manual,
        )
        .unwrap();

        // Load conflict
        let conflict = load_conflict_file(&PathBuf::from(&filepath)).unwrap();

        // Verify all data persisted correctly
        assert_eq!(conflict.metadata.key, "test.key");
        assert_eq!(conflict.metadata.table, "text");

        let loaded_base: CsvRow = conflict.base.unwrap().into();
        assert_eq!(loaded_base.key, base.key);
        assert_eq!(loaded_base.values, base.values);

        let loaded_a: CsvRow = conflict.change_a.into();
        assert_eq!(loaded_a.key, change_a.key);
        assert_eq!(loaded_a.values, change_a.values);

        let loaded_b: CsvRow = conflict.change_b.into();
        assert_eq!(loaded_b.key, change_b.key);
        assert_eq!(loaded_b.values, change_b.values);
    }

    #[test]
    fn test_multiple_tables_conflicts() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        // Create conflicts for different tables
        let change_a = create_test_row("test.key", "new_a");
        let change_b = create_test_row("test.key", "new_b");

        write_conflict_file(
            base_path,
            "text",
            "test.key",
            None,
            change_a.clone(),
            change_b.clone(),
            ResolutionStrategy::Manual,
        )
        .unwrap();

        write_conflict_file(
            base_path,
            "routes",
            "test.key",
            None,
            change_a,
            change_b,
            ResolutionStrategy::Manual,
        )
        .unwrap();

        // Each table should have its own conflicts
        let text_conflicts = list_conflicts(base_path, "text").unwrap();
        let routes_conflicts = list_conflicts(base_path, "routes").unwrap();

        assert_eq!(text_conflicts.len(), 1);
        assert_eq!(routes_conflicts.len(), 1);
        assert_ne!(text_conflicts[0], routes_conflicts[0]);
    }

    #[test]
    fn test_conflict_sorting() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        // Create multiple conflicts with slight delays to ensure different timestamps
        for i in 1..=3 {
            let key = format!("test.key.{}", i);
            let change_a = create_test_row(&key, "new_a");
            let change_b = create_test_row(&key, "new_b");

            write_conflict_file(
                base_path,
                "text",
                &key,
                None,
                change_a,
                change_b,
                ResolutionStrategy::Manual,
            )
            .unwrap();

            // Small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let conflicts = list_conflicts(base_path, "text").unwrap();
        assert_eq!(conflicts.len(), 3);

        // Verify conflicts are sorted by filename (timestamp)
        for i in 0..conflicts.len() - 1 {
            let name1 = conflicts[i].file_name().unwrap().to_str().unwrap();
            let name2 = conflicts[i + 1].file_name().unwrap().to_str().unwrap();
            assert!(
                name1 <= name2,
                "Conflicts should be sorted: {} > {}",
                name1,
                name2
            );
        }
    }
}
