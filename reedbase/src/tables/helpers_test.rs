// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for table helper functions (list_tables, table_exists, table_stats).

#[cfg(test)]
mod tests {
    use crate::registry::init_registry;
    use crate::tables::helpers::{list_tables, table_exists, table_stats};
    use crate::tables::Table;
    use std::fs;
    use tempfile::TempDir;

    /// Setup test environment with registry initialized.
    fn setup_test(_name: &str) -> TempDir {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        // Set base path for registry
        crate::registry::set_base_path(base_path.to_path_buf());

        // Initialize registry for user codes
        init_registry(base_path).unwrap();

        // Reload dictionaries to load from new path
        crate::registry::reload_dictionaries().unwrap();

        temp
    }

    /// Test list_tables with no tables.
    #[test]
    fn test_list_tables_empty() {
        let temp = setup_test("list_empty");
        let base_path = temp.path();

        let result = list_tables(base_path).unwrap();
        assert_eq!(result.len(), 0, "Should return empty list");
    }

    /// Test list_tables with multiple tables.
    #[test]
    fn test_list_tables_multiple() {
        let temp = setup_test("list_multiple");
        let base_path = temp.path();

        // Create tables using Table API
        let text = Table::new(base_path, "text");
        text.init(b"key|value\n", "testuser").unwrap();

        let routes = Table::new(base_path, "routes");
        routes.init(b"path|target\n", "testuser").unwrap();

        let meta = Table::new(base_path, "meta");
        meta.init(b"key|value\n", "testuser").unwrap();

        let mut result = list_tables(base_path).unwrap();
        result.sort(); // Sort for consistent comparison

        assert_eq!(result.len(), 3, "Should find 3 tables");
        assert_eq!(result, vec!["meta", "routes", "text"]);
    }

    /// Test list_tables ignores directories without current.csv.
    #[test]
    fn test_list_tables_ignores_invalid() {
        let temp = setup_test("list_ignores");
        let base_path = temp.path();

        // Create valid table
        let table = Table::new(base_path, "text");
        table.init(b"key|value\n", "testuser").unwrap();

        // Create invalid directory (no current.csv) in tables directory
        fs::create_dir_all(base_path.join("tables/invalid")).unwrap();

        // Create regular file in tables directory (not a valid table)
        fs::write(base_path.join("tables/readme.txt"), b"test").unwrap();

        let result = list_tables(base_path).unwrap();
        assert_eq!(result.len(), 1, "Should only find valid table");
        assert_eq!(result[0], "text");
    }

    /// Test table_exists returns true for existing table.
    #[test]
    fn test_table_exists_true() {
        let temp = setup_test("exists_true");
        let base_path = temp.path();

        // Create table
        let table = Table::new(base_path, "text");
        table.init(b"key|value\n", "testuser").unwrap();

        assert!(table_exists(base_path, "text"), "Table should exist");
    }

    /// Test table_exists returns false for non-existent table.
    #[test]
    fn test_table_exists_false() {
        let temp = setup_test("exists_false");
        let base_path = temp.path();

        assert!(
            !table_exists(base_path, "nonexistent"),
            "Table should not exist"
        );
    }

    /// Test table_exists with directory but no current.csv.
    #[test]
    fn test_table_exists_no_current_csv() {
        let temp = setup_test("auto");
        let base_path = temp.path();

        // Create directory without current.csv (in correct location)
        fs::create_dir_all(base_path.join("tables/incomplete")).unwrap();

        assert!(
            !table_exists(base_path, "incomplete"),
            "Table should not exist without current.csv"
        );
    }

    /// Test table_stats returns correct statistics.
    #[test]
    fn test_table_stats_basic() {
        let temp = setup_test("auto");
        let base_path = temp.path();

        // Create and initialise table
        let table = Table::new(base_path, "text");
        let initial_content = b"key|value\ntest.key|test value\n";
        table.init(initial_content, "testuser").unwrap();

        let stats = table_stats(base_path, "text").unwrap();

        assert_eq!(stats.name, "text");
        assert_eq!(stats.current_size, initial_content.len() as u64);
        assert_eq!(stats.version_count, 1, "Should have 1 version");
        assert!(stats.latest_version > 0, "Should have valid timestamp");
        assert_eq!(
            stats.oldest_version, stats.latest_version,
            "Single version: oldest == latest"
        );
    }

    /// Test table_stats with multiple versions.
    #[test]
    fn test_table_stats_multiple_versions() {
        let temp = setup_test("auto");
        let base_path = temp.path();

        // Create table
        let table = Table::new(base_path, "text");
        table.init(b"key|value\n", "testuser").unwrap();

        // Add more versions
        table.write(b"key|value\nv2|data\n", "testuser").unwrap();
        table.write(b"key|value\nv3|data\n", "testuser").unwrap();

        let stats = table_stats(base_path, "text").unwrap();

        assert_eq!(stats.version_count, 3, "Should have 3 versions");
        assert!(
            stats.latest_version > stats.oldest_version,
            "Latest should be newer than oldest"
        );
        assert!(stats.current_size > 0, "Should have content");
        assert!(
            stats.deltas_size > 0,
            "Should have delta files (2 versions)"
        );
    }

    /// Test table_stats for non-existent table.
    #[test]
    fn test_table_stats_not_found() {
        let temp = setup_test("auto");
        let base_path = temp.path();

        let result = table_stats(base_path, "nonexistent");
        assert!(result.is_err(), "Should return error");

        if let Err(e) = result {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("not found"),
                "Should be TableNotFound error, got: {}",
                err_msg
            );
        }
    }

    /// Test table_stats with corrupted version.log.
    #[test]
    fn test_table_stats_corrupted_log() {
        let temp = setup_test("auto");
        let base_path = temp.path();

        // Create table
        let table = Table::new(base_path, "text");
        table.init(b"key|value\n", "testuser").unwrap();

        // Corrupt version.log
        fs::write(table.log_path(), b"invalid|log|format\n").unwrap();

        let result = table_stats(base_path, "text");
        assert!(result.is_err(), "Should return error for corrupted log");
    }

    /// Test list_tables on non-existent directory.
    #[test]
    fn test_list_tables_nonexistent_dir() {
        let temp = setup_test("auto");
        let base_path = temp.path().join("nonexistent");

        let result = list_tables(&base_path).unwrap();
        assert_eq!(result.len(), 0, "Should return empty list for missing dir");
    }

    /// Test table_stats calculates delta sizes correctly.
    #[test]
    fn test_table_stats_delta_sizes() {
        let temp = setup_test("auto");
        let base_path = temp.path();

        // Create table with multiple versions
        let table = Table::new(base_path, "text");
        table.init(b"key|value\n", "testuser").unwrap();

        let v2_content = b"key|value\ntest1|data1\ntest2|data2\n";
        table.write(v2_content, "testuser").unwrap();

        let v3_content = b"key|value\ntest1|data1\ntest2|data2\ntest3|data3\n";
        table.write(v3_content, "testuser").unwrap();

        let stats = table_stats(base_path, "text").unwrap();

        // Current size should match latest content
        assert_eq!(stats.current_size, v3_content.len() as u64);

        // Delta size should be sum of v2 and v3 delta files
        // (Currently full content, will be bsdiff in REED-19-03)
        assert!(stats.deltas_size > 0, "Should have accumulated delta sizes");
    }
}
