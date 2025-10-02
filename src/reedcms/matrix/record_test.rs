// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::matrix::record::{MatrixRecord, MatrixValue};

    #[test]
    fn test_matrix_value_single() {
        let value = MatrixValue::Single("active".to_string());
        assert!(value.is_single());
        assert_eq!(value.to_csv_string(), "active");
    }

    #[test]
    fn test_matrix_value_list() {
        let value = MatrixValue::List(vec!["editor".to_string(), "author".to_string()]);
        assert!(value.is_list());
        assert_eq!(value.to_csv_string(), "editor,author");
    }

    #[test]
    fn test_matrix_value_modified() {
        let value = MatrixValue::Modified("minify".to_string(), vec!["prod".to_string()]);
        assert!(value.is_modified());
        assert_eq!(value.to_csv_string(), "minify[prod]");
    }

    #[test]
    fn test_matrix_value_modified_list() {
        let value = MatrixValue::ModifiedList(vec![
            ("text".to_string(), vec!["rwx".to_string()]),
            ("route".to_string(), vec!["rw-".to_string()]),
        ]);
        assert!(value.is_modified_list());
        assert_eq!(value.to_csv_string(), "text[rwx],route[rw-]");
    }

    #[test]
    fn test_matrix_value_modified_multiple_mods() {
        let value = MatrixValue::Modified(
            "file".to_string(),
            vec!["dev".to_string(), "prod".to_string()],
        );
        assert_eq!(value.to_csv_string(), "file[dev,prod]");
    }

    #[test]
    fn test_matrix_record_new() {
        let record = MatrixRecord::new();
        assert!(record.fields.is_empty());
        assert!(record.field_order.is_empty());
        assert!(record.description.is_none());
    }

    #[test]
    fn test_matrix_record_add_field() {
        let mut record = MatrixRecord::new();
        record.add_field(
            "username".to_string(),
            MatrixValue::Single("admin".to_string()),
        );
        record.add_field(
            "status".to_string(),
            MatrixValue::Single("active".to_string()),
        );

        assert_eq!(record.fields.len(), 2);
        assert_eq!(record.field_order, vec!["username", "status"]);
    }

    #[test]
    fn test_matrix_record_get_field() {
        let mut record = MatrixRecord::new();
        record.add_field(
            "username".to_string(),
            MatrixValue::Single("admin".to_string()),
        );

        assert!(record.get_field("username").is_some());
        assert!(record.get_field("nonexistent").is_none());
    }

    #[test]
    fn test_matrix_record_to_csv_row() {
        let mut record = MatrixRecord::new();
        record.add_field(
            "username".to_string(),
            MatrixValue::Single("admin".to_string()),
        );
        record.add_field(
            "roles".to_string(),
            MatrixValue::List(vec!["admin".to_string(), "editor".to_string()]),
        );
        record.set_description("System Administrator".to_string());

        let csv_row = record.to_csv_row();
        assert_eq!(csv_row, "admin|admin,editor|System Administrator");
    }

    #[test]
    fn test_matrix_record_to_csv_row_no_description() {
        let mut record = MatrixRecord::new();
        record.add_field(
            "username".to_string(),
            MatrixValue::Single("admin".to_string()),
        );

        let csv_row = record.to_csv_row();
        assert_eq!(csv_row, "admin");
    }

    #[test]
    fn test_matrix_record_field_order_preserved() {
        let mut record = MatrixRecord::new();
        record.add_field("c".to_string(), MatrixValue::Single("3".to_string()));
        record.add_field("a".to_string(), MatrixValue::Single("1".to_string()));
        record.add_field("b".to_string(), MatrixValue::Single("2".to_string()));

        // Field order should match insertion order, not alphabetical
        assert_eq!(record.field_order, vec!["c", "a", "b"]);
        assert_eq!(record.to_csv_row(), "3|1|2");
    }
}
