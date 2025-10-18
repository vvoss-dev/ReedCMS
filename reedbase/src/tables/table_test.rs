// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for Table struct.

#[cfg(test)]
mod tests {
    use crate::registry::init_registry;
    use crate::tables::Table;
    use std::fs;

    fn setup_test(name: &str) -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir().join(format!("reedbase_table_test_{}", name));
        let _ = fs::remove_dir_all(&temp_dir);

        // Set base path for registry
        crate::registry::set_base_path(temp_dir.clone());

        // Initialize registry for user codes
        init_registry(&temp_dir).unwrap();

        // Reload dictionaries to load from new path
        crate::registry::reload_dictionaries().unwrap();

        temp_dir
    }

    #[test]
    fn test_table_new() {
        let temp_dir = setup_test("new");
        let table = Table::new(&temp_dir, "test");

        assert!(!table.exists());
        assert_eq!(
            table.current_path(),
            temp_dir.join("tables/test/current.csv")
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_init() {
        let temp_dir = setup_test("init");
        let table = Table::new(&temp_dir, "test");

        let content = b"key|value\nfoo|bar\n";
        table.init(content, "testuser").unwrap();

        assert!(table.exists());
        assert!(table.current_path().exists());
        assert!(table.log_path().exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_init_already_exists() {
        let temp_dir = setup_test("init_exists");
        let table = Table::new(&temp_dir, "test");

        let content = b"key|value\nfoo|bar\n";
        table.init(content, "testuser").unwrap();

        // Second init should fail
        let result = table.init(content, "testuser");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_read_current() {
        let temp_dir = setup_test("read");
        let table = Table::new(&temp_dir, "test");

        let content = b"key|value\nfoo|bar\n";
        table.init(content, "testuser").unwrap();

        let read_content = table.read_current().unwrap();
        assert_eq!(read_content, content);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_read_current_not_found() {
        let temp_dir = setup_test("read_not_found");
        let table = Table::new(&temp_dir, "nonexistent");

        let result = table.read_current();
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_read_current_as_rows() {
        let temp_dir = setup_test("read_rows");
        let table = Table::new(&temp_dir, "test");

        let content = b"key|value\nfoo|bar\nbaz|qux\n";
        table.init(content, "testuser").unwrap();

        let rows = table.read_current_as_rows().unwrap();
        assert_eq!(rows.len(), 3, "Should include header + 2 data rows");
        assert_eq!(rows[0].key, "key", "First row is header");
        assert_eq!(rows[0].values[0], "value");
        assert_eq!(rows[1].key, "foo");
        assert_eq!(rows[1].values[0], "bar");
        assert_eq!(rows[2].key, "baz");
        assert_eq!(rows[2].values[0], "qux");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_write() {
        let temp_dir = setup_test("write");
        let table = Table::new(&temp_dir, "test");

        let initial = b"key|value\nfoo|bar\n";
        table.init(initial, "testuser").unwrap();

        let updated = b"key|value\nfoo|baz\n";
        let result = table.write(updated, "testuser").unwrap();

        assert!(result.timestamp > 0);
        assert!(result.delta_size > 0);
        assert_eq!(result.current_size, updated.len() as u64);

        let read_content = table.read_current().unwrap();
        assert_eq!(read_content, updated);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_write_not_found() {
        let temp_dir = setup_test("write_not_found");
        let table = Table::new(&temp_dir, "nonexistent");

        let content = b"key|value\nfoo|bar\n";
        let result = table.write(content, "testuser");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_list_versions() {
        let temp_dir = setup_test("list_versions");
        let table = Table::new(&temp_dir, "test");

        let v1 = b"key|value\nfoo|bar\n";
        table.init(v1, "testuser").unwrap();

        let v2 = b"key|value\nfoo|baz\n";
        table.write(v2, "testuser").unwrap();

        let v3 = b"key|value\nfoo|qux\n";
        table.write(v3, "testuser").unwrap();

        let versions = table.list_versions().unwrap();
        assert_eq!(versions.len(), 3);

        // Newest first
        assert_eq!(versions[0].action, "update");
        assert_eq!(versions[1].action, "update");
        assert_eq!(versions[2].action, "init");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_rollback() {
        let temp_dir = setup_test("rollback");
        let table = Table::new(&temp_dir, "test");

        let v1 = b"key|value\nfoo|bar\n";
        table.init(v1, "testuser").unwrap();

        let v2 = b"key|value\nfoo|baz\n";
        table.write(v2, "testuser").unwrap();

        let v3 = b"key|value\nfoo|qux\n";
        table.write(v3, "testuser").unwrap();

        // Get versions
        let versions = table.list_versions().unwrap();

        // Rollback to v1 (oldest)
        table.rollback(versions[2].timestamp, "testuser").unwrap();

        let content = table.read_current().unwrap();
        assert_eq!(content, v1);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_rollback_version_not_found() {
        let temp_dir = setup_test("rollback_not_found");
        let table = Table::new(&temp_dir, "test");

        let content = b"key|value\nfoo|bar\n";
        table.init(content, "testuser").unwrap();

        let result = table.rollback(999999, "testuser");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_delete() {
        let temp_dir = setup_test("delete");
        let table = Table::new(&temp_dir, "test");

        let content = b"key|value\nfoo|bar\n";
        table.init(content, "testuser").unwrap();

        assert!(table.exists());

        table.delete(true).unwrap();

        assert!(!table.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_table_delete_not_confirmed() {
        let temp_dir = setup_test("delete_not_confirmed");
        let table = Table::new(&temp_dir, "test");

        let content = b"key|value\nfoo|bar\n";
        table.init(content, "testuser").unwrap();

        let result = table.delete(false);
        assert!(result.is_err());
        assert!(table.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
