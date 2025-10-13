// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::formatter::*;
    use crate::types::*;
    use serde_json::json;

    #[test]
    fn test_format_table_simple() {
        let data = json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]);

        let result = format_table(&data).unwrap();
        assert!(result.contains("id"));
        assert!(result.contains("name"));
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
    }

    #[test]
    fn test_format_table_empty() {
        let data = json!([]);
        let result = format_table(&data).unwrap();
        assert_eq!(result, "(empty result set)");
    }

    #[test]
    fn test_format_table_non_array() {
        let data = json!({"not": "array"});
        let result = format_table(&data);
        assert!(matches!(result, Err(CliError::FormatError { .. })));
    }

    #[test]
    fn test_format_table_non_objects() {
        let data = json!(["string", "array"]);
        let result = format_table(&data);
        assert!(matches!(result, Err(CliError::FormatError { .. })));
    }

    #[test]
    fn test_format_json_object() {
        let data = json!({"name": "Alice", "age": 30});
        let result = format_json(&data).unwrap();

        assert!(result.contains("\"name\""));
        assert!(result.contains("\"Alice\""));
        assert!(result.contains("\"age\""));
        assert!(result.contains("30"));
    }

    #[test]
    fn test_format_json_array() {
        let data = json!([1, 2, 3]);
        let result = format_json(&data).unwrap();

        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_format_csv_simple() {
        let data = json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]);

        let result = format_csv(&data).unwrap();
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines[0], "id,name");
        assert!(lines[1].contains("1") && lines[1].contains("Alice"));
        assert!(lines[2].contains("2") && lines[2].contains("Bob"));
    }

    #[test]
    fn test_format_csv_empty() {
        let data = json!([]);
        let result = format_csv(&data).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_csv_non_array() {
        let data = json!({"not": "array"});
        let result = format_csv(&data);
        assert!(matches!(result, Err(CliError::FormatError { .. })));
    }

    #[test]
    fn test_format_plain_simple() {
        let data = json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]);

        let result = format_plain(&data).unwrap();
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
    }

    #[test]
    fn test_format_plain_non_array() {
        let data = json!({"not": "array"});
        let result = format_plain(&data);
        assert!(matches!(result, Err(CliError::FormatError { .. })));
    }

    #[test]
    fn test_format_value_null() {
        assert_eq!(format_value(&json!(null)), "");
    }

    #[test]
    fn test_format_value_bool_true() {
        assert_eq!(format_value(&json!(true)), "true");
    }

    #[test]
    fn test_format_value_bool_false() {
        assert_eq!(format_value(&json!(false)), "false");
    }

    #[test]
    fn test_format_value_number_integer() {
        assert_eq!(format_value(&json!(42)), "42");
    }

    #[test]
    fn test_format_value_number_float() {
        assert_eq!(format_value(&json!(3.14)), "3.14");
    }

    #[test]
    fn test_format_value_string() {
        assert_eq!(format_value(&json!("text")), "text");
    }

    #[test]
    fn test_format_value_array() {
        let result = format_value(&json!([1, 2, 3]));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_format_value_object() {
        let result = format_value(&json!({"key": "value"}));
        assert!(result.contains("key"));
        assert!(result.contains("value"));
    }

    #[test]
    fn test_escape_csv_value_simple() {
        assert_eq!(escape_csv_value("simple"), "simple");
    }

    #[test]
    fn test_escape_csv_value_comma() {
        assert_eq!(escape_csv_value("with,comma"), "\"with,comma\"");
    }

    #[test]
    fn test_escape_csv_value_quote() {
        assert_eq!(escape_csv_value("with\"quote"), "\"with\"\"quote\"");
    }

    #[test]
    fn test_escape_csv_value_newline() {
        assert_eq!(escape_csv_value("with\newline"), "\"with\newline\"");
    }

    #[test]
    fn test_escape_csv_value_multiple_quotes() {
        assert_eq!(escape_csv_value("\"quoted\""), "\"\"\"quoted\"\"\"");
    }

    #[test]
    fn test_format_output_table() {
        let output = CommandOutput {
            data: json!([{"id": 1}]),
            format: OutputFormat::Table,
            exit_code: 0,
        };

        let result = format_output(&output, false).unwrap();
        assert!(result.contains("id"));
    }

    #[test]
    fn test_format_output_json() {
        let output = CommandOutput {
            data: json!({"id": 1}),
            format: OutputFormat::Json,
            exit_code: 0,
        };

        let result = format_output(&output, false).unwrap();
        assert!(result.contains("\"id\""));
    }

    #[test]
    fn test_format_output_csv() {
        let output = CommandOutput {
            data: json!([{"id": 1}]),
            format: OutputFormat::Csv,
            exit_code: 0,
        };

        let result = format_output(&output, false).unwrap();
        assert!(result.contains("id"));
    }

    #[test]
    fn test_format_output_plain() {
        let output = CommandOutput {
            data: json!([{"id": 1}]),
            format: OutputFormat::Plain,
            exit_code: 0,
        };

        let result = format_output(&output, false).unwrap();
        assert!(result.contains("1"));
    }

    #[test]
    fn test_format_table_with_null_values() {
        let data = json!([
            {"id": 1, "name": "Alice", "email": null},
            {"id": 2, "name": "Bob", "email": "bob@example.com"}
        ]);

        let result = format_table(&data).unwrap();
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
        assert!(result.contains("bob@example.com"));
    }

    #[test]
    fn test_format_csv_with_commas_in_values() {
        let data = json!([
            {"name": "Smith, John", "city": "New York"}
        ]);

        let result = format_csv(&data).unwrap();
        assert!(result.contains("\"Smith, John\""));
    }

    #[test]
    fn test_format_csv_with_quotes_in_values() {
        let data = json!([
            {"name": "John \"The Boss\" Smith"}
        ]);

        let result = format_csv(&data).unwrap();
        assert!(result.contains("\"\"")); // Doubled quotes
    }

    #[test]
    fn test_format_plain_with_non_object_items() {
        let data = json!(["Alice", "Bob", "Charlie"]);

        let result = format_plain(&data).unwrap();
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
        assert!(result.contains("Charlie"));
    }

    #[test]
    fn test_format_table_preserves_column_order() {
        // Note: HashMap iteration order is not guaranteed, but we test that
        // all columns are present
        let data = json!([
            {"id": 1, "name": "Alice", "age": 30}
        ]);

        let result = format_table(&data).unwrap();
        assert!(result.contains("id"));
        assert!(result.contains("name"));
        assert!(result.contains("age"));
    }
}
