// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for CSV parser (parse_csv, parse_csv_row).

#[cfg(test)]
mod tests {
    use crate::tables::csv_parser::{parse_csv, parse_csv_row};

    /// Test parse_csv_row with valid single-column row.
    #[test]
    fn test_parse_csv_row_single_column() {
        let result = parse_csv_row("test.key|value", 1).unwrap();
        assert_eq!(result.key, "test.key");
        assert_eq!(result.values, vec!["value"]);
    }

    /// Test parse_csv_row with multiple columns.
    #[test]
    fn test_parse_csv_row_multiple_columns() {
        let result = parse_csv_row("test.key|value1|value2|value3", 1).unwrap();
        assert_eq!(result.key, "test.key");
        assert_eq!(result.values, vec!["value1", "value2", "value3"]);
    }

    /// Test parse_csv_row with empty values.
    #[test]
    fn test_parse_csv_row_empty_values() {
        let result = parse_csv_row("test.key||empty||", 1).unwrap();
        assert_eq!(result.key, "test.key");
        assert_eq!(result.values, vec!["", "empty", "", ""]);
    }

    /// Test parse_csv_row with trailing pipe.
    #[test]
    fn test_parse_csv_row_trailing_pipe() {
        let result = parse_csv_row("test.key|value|", 1).unwrap();
        assert_eq!(result.key, "test.key");
        assert_eq!(result.values, vec!["value", ""]);
    }

    /// Test parse_csv_row with no pipe (invalid).
    #[test]
    fn test_parse_csv_row_no_pipe() {
        let result = parse_csv_row("invalid_row_without_pipe", 5);
        assert!(result.is_err(), "Should fail without pipe delimiter");

        if let Err(e) = result {
            let msg = e.to_string();
            assert!(msg.contains("Invalid CSV"), "Should be InvalidCsv error");
            assert!(msg.contains("line 5"), "Should include line number");
        }
    }

    /// Test parse_csv_row with only key (no values).
    #[test]
    fn test_parse_csv_row_only_key() {
        let result = parse_csv_row("test.key|", 1).unwrap();
        assert_eq!(result.key, "test.key");
        assert_eq!(result.values, vec![""]);
    }

    /// Test parse_csv_row with whitespace.
    #[test]
    fn test_parse_csv_row_whitespace() {
        let result = parse_csv_row("  test.key  |  value  |  other  ", 1).unwrap();
        assert_eq!(result.key, "test.key");
        assert_eq!(
            result.values,
            vec!["value", "other"],
            "Values should be trimmed"
        );
    }

    /// Test parse_csv with valid multi-line CSV.
    #[test]
    fn test_parse_csv_basic() {
        let content = b"key|value\ntest.key1|value1\ntest.key2|value2\n";
        let rows = parse_csv(content).unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].key, "key");
        assert_eq!(rows[0].values, vec!["value"]);
        assert_eq!(rows[1].key, "test.key1");
        assert_eq!(rows[1].values, vec!["value1"]);
        assert_eq!(rows[2].key, "test.key2");
        assert_eq!(rows[2].values, vec!["value2"]);
    }

    /// Test parse_csv with empty content.
    #[test]
    fn test_parse_csv_empty() {
        let content = b"";
        let rows = parse_csv(content).unwrap();
        assert_eq!(rows.len(), 0, "Empty content should return empty vec");
    }

    /// Test parse_csv with only whitespace.
    #[test]
    fn test_parse_csv_whitespace_only() {
        let content = b"   \n  \n\t\n";
        let rows = parse_csv(content).unwrap();
        assert_eq!(rows.len(), 0, "Whitespace-only should return empty vec");
    }

    /// Test parse_csv with comments (lines starting with #).
    #[test]
    fn test_parse_csv_with_comments() {
        let content = b"# This is a comment\nkey|value\n# Another comment\ntest.key|value\n";
        let rows = parse_csv(content).unwrap();

        assert_eq!(rows.len(), 2, "Should skip comment lines");
        assert_eq!(rows[0].key, "key");
        assert_eq!(rows[1].key, "test.key");
    }

    /// Test parse_csv with blank lines.
    #[test]
    fn test_parse_csv_blank_lines() {
        let content = b"key|value\n\ntest.key|value\n\n\n";
        let rows = parse_csv(content).unwrap();

        assert_eq!(rows.len(), 2, "Should skip blank lines");
        assert_eq!(rows[0].key, "key");
        assert_eq!(rows[1].key, "test.key");
    }

    /// Test parse_csv with mixed valid and comment lines.
    #[test]
    fn test_parse_csv_mixed_content() {
        let content = b"# Header comment\nkey|value|description\n\n# Data section\ntest.key1|val1|desc1\ntest.key2|val2|desc2\n# Footer\n";
        let rows = parse_csv(content).unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].key, "key");
        assert_eq!(rows[0].values, vec!["value", "description"]);
        assert_eq!(rows[1].key, "test.key1");
        assert_eq!(rows[1].values, vec!["val1", "desc1"]);
        assert_eq!(rows[2].key, "test.key2");
        assert_eq!(rows[2].values, vec!["val2", "desc2"]);
    }

    /// Test parse_csv with invalid row (no pipe).
    #[test]
    fn test_parse_csv_invalid_row() {
        let content = b"key|value\ninvalid_row_no_pipe\ntest.key|value\n";
        let result = parse_csv(content);

        assert!(result.is_err(), "Should fail on invalid row");

        if let Err(e) = result {
            let msg = e.to_string();
            assert!(msg.contains("Invalid CSV"), "Should be InvalidCsv error");
            assert!(
                msg.contains("line 2"),
                "Should report correct line number (1-indexed)"
            );
        }
    }

    /// Test parse_csv with UTF-8 content.
    #[test]
    fn test_parse_csv_utf8() {
        let content = "test.german|Überschrift|Beschreibung\ntest.emoji|🚀|rocket\n".as_bytes();
        let rows = parse_csv(content).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].key, "test.german");
        assert_eq!(rows[0].values, vec!["Überschrift", "Beschreibung"]);
        assert_eq!(rows[1].key, "test.emoji");
        assert_eq!(rows[1].values, vec!["🚀", "rocket"]);
    }

    /// Test parse_csv with invalid UTF-8.
    #[test]
    fn test_parse_csv_invalid_utf8() {
        let mut content = b"key|value\n".to_vec();
        content.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // Invalid UTF-8 bytes
        content.extend_from_slice(b"\ntest.key|value\n");

        let result = parse_csv(&content);
        assert!(result.is_err(), "Should fail on invalid UTF-8");
    }

    /// Test parse_csv with multiple columns.
    #[test]
    fn test_parse_csv_multiple_columns() {
        let content = b"key|col1|col2|col3|col4\ntest.key|a|b|c|d\n";
        let rows = parse_csv(content).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].values, vec!["col1", "col2", "col3", "col4"]);
        assert_eq!(rows[1].values, vec!["a", "b", "c", "d"]);
    }

    /// Test parse_csv with Windows line endings (CRLF).
    #[test]
    fn test_parse_csv_crlf() {
        let content = b"key|value\r\ntest.key1|value1\r\ntest.key2|value2\r\n";
        let rows = parse_csv(content).unwrap();

        assert_eq!(rows.len(), 3, "Should handle CRLF line endings");
        assert_eq!(rows[0].key, "key");
        assert_eq!(rows[1].key, "test.key1");
        assert_eq!(rows[2].key, "test.key2");
    }

    /// Test parse_csv with mixed line endings (LF and CRLF).
    #[test]
    fn test_parse_csv_mixed_line_endings() {
        let content = b"key|value\r\ntest.key1|value1\ntest.key2|value2\r\n";
        let rows = parse_csv(content).unwrap();

        assert_eq!(rows.len(), 3, "Should handle mixed line endings");
        assert_eq!(rows[2].key, "test.key2");
    }

    /// Test parse_csv_row with very long key.
    #[test]
    fn test_parse_csv_row_long_key() {
        let long_key = "a".repeat(500);
        let line = format!("{}|value", long_key);
        let result = parse_csv_row(&line, 1).unwrap();

        assert_eq!(result.key, long_key);
        assert_eq!(result.values, vec!["value"]);
    }

    /// Test parse_csv_row with very long value.
    #[test]
    fn test_parse_csv_row_long_value() {
        let long_value = "x".repeat(10000);
        let line = format!("test.key|{}", long_value);
        let result = parse_csv_row(&line, 1).unwrap();

        assert_eq!(result.key, "test.key");
        assert_eq!(result.values, vec![long_value.as_str()]);
    }

    /// Test parse_csv with large file (performance check).
    #[test]
    fn test_parse_csv_large_file() {
        let mut content = String::new();
        for i in 0..1000 {
            content.push_str(&format!("test.key{}|value{}|desc{}\n", i, i, i));
        }

        let rows = parse_csv(content.as_bytes()).unwrap();
        assert_eq!(rows.len(), 1000, "Should parse 1000 rows");
        assert_eq!(rows[0].key, "test.key0");
        assert_eq!(rows[999].key, "test.key999");
    }
}
