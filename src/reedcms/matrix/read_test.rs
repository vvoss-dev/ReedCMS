// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::matrix::{read_matrix_csv, MatrixValue};
    use std::fs;
    use std::path::PathBuf;

    fn create_test_file(content: &str) -> PathBuf {
        let path = PathBuf::from(format!(
            "/tmp/reed_matrix_test_{}.csv",
            rand::random::<u32>()
        ));
        fs::write(&path, content).unwrap();
        path
    }

    fn cleanup_test_file(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_read_matrix_csv_basic() {
        let content = "username|status|desc\nadmin|active|System Administrator\neditor|inactive|Content Editor\n";
        let path = create_test_file(content);

        let result = read_matrix_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);

        let first = &records[0];
        assert!(matches!(
            first.get_field("username"),
            Some(MatrixValue::Single(_))
        ));
        assert_eq!(first.description, Some("System Administrator".to_string()));

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_matrix_csv_type2_lists() {
        let content = "username|roles|desc\njane|editor,author|Multi-role user\nadmin|admin,editor,author|Full access\n";
        let path = create_test_file(content);

        let result = read_matrix_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);

        if let Some(MatrixValue::List(roles)) = records[0].get_field("roles") {
            assert_eq!(roles.len(), 2);
            assert_eq!(roles[0], "editor");
            assert_eq!(roles[1], "author");
        } else {
            panic!("Expected List value");
        }

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_matrix_csv_type3_modified() {
        let content = "asset|optimization|desc\nmain.css|minify[prod]|Main stylesheet\napp.js|bundle[dev,prod]|App code\n";
        let path = create_test_file(content);

        let result = read_matrix_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);

        if let Some(MatrixValue::Modified(val, mods)) = records[0].get_field("optimization") {
            assert_eq!(val, "minify");
            assert_eq!(mods, &vec!["prod"]);
        } else {
            panic!("Expected Modified value");
        }

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_matrix_csv_type4_permissions() {
        let content = "rolename|permissions|desc\neditor|text[rwx],route[rw-]|Standard Editor\nadmin|*[rwx]|Full Admin\n";
        let path = create_test_file(content);

        let result = read_matrix_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);

        if let Some(MatrixValue::ModifiedList(items)) = records[0].get_field("permissions") {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].0, "text");
            assert_eq!(items[0].1, vec!["rwx"]);
            assert_eq!(items[1].0, "route");
            assert_eq!(items[1].1, vec!["rw-"]);
        } else {
            panic!("Expected ModifiedList value");
        }

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_matrix_csv_skip_comments() {
        let content = "# This is a comment\nusername|status\n# Another comment\nadmin|active\n";
        let path = create_test_file(content);

        let result = read_matrix_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 1);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_matrix_csv_skip_empty_lines() {
        let content = "username|status\n\nadmin|active\n\neditor|inactive\n\n";
        let path = create_test_file(content);

        let result = read_matrix_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_matrix_csv_no_description() {
        let content = "username|status\nadmin|active\neditor|inactive\n";
        let path = create_test_file(content);

        let result = read_matrix_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert!(records[0].description.is_none());

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_matrix_csv_empty_file() {
        let content = "";
        let path = create_test_file(content);

        let result = read_matrix_csv(&path);
        assert!(result.is_err()); // No header

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_matrix_csv_file_not_found() {
        let result = read_matrix_csv("/tmp/nonexistent_matrix_file_12345.csv");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_matrix_csv_performance() {
        let mut content = String::from("key|value1|value2|desc\n");
        for i in 0..1000 {
            content.push_str(&format!(
                "key{}|value{}|item1,item2|Description {}\n",
                i, i, i
            ));
        }

        let path = create_test_file(&content);

        let start = std::time::Instant::now();
        let result = read_matrix_csv(&path);
        let duration = start.elapsed();

        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 1000);

        // Should complete in < 20ms
        assert!(
            duration.as_millis() < 20,
            "Reading too slow: {:?}",
            duration
        );

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_matrix_csv_field_order_preserved() {
        let content = "c|a|b\n3|1|2\n6|4|5\n";
        let path = create_test_file(content);

        let result = read_matrix_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records[0].field_order, vec!["c", "a", "b"]);

        cleanup_test_file(&path);
    }
}
