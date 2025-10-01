// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::matrix::{read_matrix_csv, write_matrix_csv, MatrixRecord, MatrixValue};
    use std::fs;
    use std::path::PathBuf;

    fn get_test_path() -> PathBuf {
        PathBuf::from(format!(
            "/tmp/reed_matrix_write_test_{}.csv",
            rand::random::<u32>()
        ))
    }

    fn cleanup_test_file(path: &PathBuf) {
        let _ = fs::remove_file(path);
        let temp_path = format!("{}.tmp", path.to_string_lossy());
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_write_matrix_csv_basic() {
        let path = get_test_path();

        let mut record1 = MatrixRecord::new();
        record1.add_field(
            "username".to_string(),
            MatrixValue::Single("admin".to_string()),
        );
        record1.add_field(
            "status".to_string(),
            MatrixValue::Single("active".to_string()),
        );
        record1.set_description("System Administrator".to_string());

        let mut record2 = MatrixRecord::new();
        record2.add_field(
            "username".to_string(),
            MatrixValue::Single("editor".to_string()),
        );
        record2.add_field(
            "status".to_string(),
            MatrixValue::Single("inactive".to_string()),
        );
        record2.set_description("Content Editor".to_string());

        let records = vec![record1, record2];

        let result = write_matrix_csv(&path, &records, &["username", "status"]);
        assert!(result.is_ok());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("username|status|desc\n"));
        assert!(content.contains("admin|active|System Administrator"));

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_matrix_csv_with_lists() {
        let path = get_test_path();

        let mut record = MatrixRecord::new();
        record.add_field(
            "username".to_string(),
            MatrixValue::Single("jane".to_string()),
        );
        record.add_field(
            "roles".to_string(),
            MatrixValue::List(vec!["editor".to_string(), "author".to_string()]),
        );
        record.set_description("Multi-role user".to_string());

        let result = write_matrix_csv(&path, &vec![record], &["username", "roles"]);
        assert!(result.is_ok());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("jane|editor,author|Multi-role user"));

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_matrix_csv_with_permissions() {
        let path = get_test_path();

        let mut record = MatrixRecord::new();
        record.add_field(
            "rolename".to_string(),
            MatrixValue::Single("editor".to_string()),
        );
        record.add_field(
            "permissions".to_string(),
            MatrixValue::ModifiedList(vec![
                ("text".to_string(), vec!["rwx".to_string()]),
                ("route".to_string(), vec!["rw-".to_string()]),
            ]),
        );
        record.set_description("Standard Editor".to_string());

        let result = write_matrix_csv(&path, &vec![record], &["rolename", "permissions"]);
        assert!(result.is_ok());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("text[rwx],route[rw-]"));

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_matrix_csv_no_description() {
        let path = get_test_path();

        let mut record = MatrixRecord::new();
        record.add_field("key".to_string(), MatrixValue::Single("value".to_string()));

        let result = write_matrix_csv(&path, &vec![record], &["key"]);
        assert!(result.is_ok());

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "key\nvalue\n");

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_matrix_csv_atomic_no_temp_file() {
        let path = get_test_path();

        let mut record = MatrixRecord::new();
        record.add_field("key".to_string(), MatrixValue::Single("value".to_string()));

        write_matrix_csv(&path, &vec![record], &["key"]).unwrap();

        let temp_path = format!("{}.tmp", path.to_string_lossy());
        assert!(!PathBuf::from(&temp_path).exists());

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_matrix_csv_round_trip() {
        let path = get_test_path();

        let mut record1 = MatrixRecord::new();
        record1.add_field(
            "username".to_string(),
            MatrixValue::Single("admin".to_string()),
        );
        record1.add_field(
            "roles".to_string(),
            MatrixValue::List(vec!["admin".to_string(), "editor".to_string()]),
        );
        record1.set_description("Full access".to_string());

        let original_records = vec![record1];

        write_matrix_csv(&path, &original_records, &["username", "roles"]).unwrap();
        let read_records = read_matrix_csv(&path).unwrap();

        assert_eq!(read_records.len(), original_records.len());

        if let Some(MatrixValue::List(roles)) = read_records[0].get_field("roles") {
            assert_eq!(roles, &vec!["admin", "editor"]);
        } else {
            panic!("Expected List value");
        }

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_matrix_csv_performance() {
        let path = get_test_path();

        let mut records = Vec::new();
        for i in 0..1000 {
            let mut record = MatrixRecord::new();
            record.add_field("key".to_string(), MatrixValue::Single(format!("key{}", i)));
            record.add_field(
                "value".to_string(),
                MatrixValue::Single(format!("value{}", i)),
            );
            record.set_description(format!("Description {}", i));
            records.push(record);
        }

        let start = std::time::Instant::now();
        write_matrix_csv(&path, &records, &["key", "value"]).unwrap();
        let duration = start.elapsed();

        // Should complete in < 20ms
        assert!(
            duration.as_millis() < 20,
            "Writing too slow: {:?}",
            duration
        );

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_matrix_csv_infer_fields_from_record() {
        let path = get_test_path();

        let mut record = MatrixRecord::new();
        record.add_field("c".to_string(), MatrixValue::Single("3".to_string()));
        record.add_field("a".to_string(), MatrixValue::Single("1".to_string()));
        record.add_field("b".to_string(), MatrixValue::Single("2".to_string()));

        // Pass empty field_names to infer from record
        let result = write_matrix_csv(&path, &vec![record], &[]);
        assert!(result.is_ok());

        let content = fs::read_to_string(&path).unwrap();
        // Should preserve field order from record
        assert!(content.starts_with("c|a|b\n"));

        cleanup_test_file(&path);
    }
}
