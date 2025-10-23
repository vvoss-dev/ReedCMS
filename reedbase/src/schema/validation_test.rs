// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for schema validation.

#[cfg(test)]
mod tests {
    use crate::schema::types::{ColumnDef, Schema};
    use crate::schema::validation::{validate_row, validate_rows, validate_uniqueness, CsvRow};

    fn create_test_schema() -> Schema {
        Schema::new(
            "2.0".to_string(),
            true,
            vec![
                ColumnDef::primary_key("id".to_string(), "integer".to_string()).with_min(1),
                ColumnDef::new("name".to_string(), "string".to_string())
                    .required()
                    .with_min_length(1)
                    .with_max_length(100),
                ColumnDef::new("email".to_string(), "string".to_string())
                    .unique()
                    .with_pattern(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string()),
                ColumnDef::new("age".to_string(), "integer".to_string())
                    .with_min(0)
                    .with_max(150),
            ],
        )
    }

    // ============================================================================
    // Row Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_valid_row() {
        let schema = create_test_schema();
        let row = CsvRow::new(
            "1".to_string(),
            vec![
                "1".to_string(),
                "Alice".to_string(),
                "alice@example.com".to_string(),
                "25".to_string(),
            ],
        );

        assert!(validate_row(&row, &schema).is_ok());
    }

    #[test]
    fn test_validate_field_count_mismatch() {
        let schema = create_test_schema();
        let row = CsvRow::new("1".to_string(), vec!["1".to_string(), "Alice".to_string()]);

        let result = validate_row(&row, &schema);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Field count mismatch"));
    }

    #[test]
    fn test_validate_missing_required_field() {
        let schema = create_test_schema();
        let row = CsvRow::new(
            "1".to_string(),
            vec![
                "1".to_string(),
                "".to_string(), // name is required
                "alice@example.com".to_string(),
                "25".to_string(),
            ],
        );

        let result = validate_row(&row, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Required field"));
    }

    // ============================================================================
    // String Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_string_min_length() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("name".to_string(), "string".to_string()).with_min_length(3)],
        );
        let row = CsvRow::new("1".to_string(), vec!["AB".to_string()]);

        let result = validate_row(&row, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("below minimum"));
    }

    #[test]
    fn test_validate_string_max_length() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("name".to_string(), "string".to_string()).with_max_length(5)],
        );
        let row = CsvRow::new("1".to_string(), vec!["ABCDEF".to_string()]);

        let result = validate_row(&row, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_validate_string_pattern_valid() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("email".to_string(), "string".to_string())
                .with_pattern(r"^[a-z]+@[a-z]+\.[a-z]+$".to_string())],
        );
        let row = CsvRow::new("1".to_string(), vec!["alice@example.com".to_string()]);

        assert!(validate_row(&row, &schema).is_ok());
    }

    #[test]
    fn test_validate_string_pattern_invalid() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("email".to_string(), "string".to_string())
                .with_pattern(r"^[a-z]+@[a-z]+\.[a-z]+$".to_string())],
        );
        let row = CsvRow::new("1".to_string(), vec!["invalid-email".to_string()]);

        let result = validate_row(&row, &schema);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not match pattern"));
    }

    // ============================================================================
    // Integer Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_integer_valid() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("age".to_string(), "integer".to_string())],
        );
        let row = CsvRow::new("1".to_string(), vec!["25".to_string()]);

        assert!(validate_row(&row, &schema).is_ok());
    }

    #[test]
    fn test_validate_integer_invalid() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("age".to_string(), "integer".to_string())],
        );
        let row = CsvRow::new("1".to_string(), vec!["not-a-number".to_string()]);

        let result = validate_row(&row, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid integer"));
    }

    #[test]
    fn test_validate_integer_min() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("age".to_string(), "integer".to_string()).with_min(18)],
        );
        let row = CsvRow::new("1".to_string(), vec!["15".to_string()]);

        let result = validate_row(&row, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("below minimum"));
    }

    #[test]
    fn test_validate_integer_max() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("age".to_string(), "integer".to_string()).with_max(100)],
        );
        let row = CsvRow::new("1".to_string(), vec!["150".to_string()]);

        let result = validate_row(&row, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    // ============================================================================
    // Float Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_float_valid() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("price".to_string(), "float".to_string())],
        );
        let row = CsvRow::new("1".to_string(), vec!["19.99".to_string()]);

        assert!(validate_row(&row, &schema).is_ok());
    }

    #[test]
    fn test_validate_float_invalid() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("price".to_string(), "float".to_string())],
        );
        let row = CsvRow::new("1".to_string(), vec!["not-a-float".to_string()]);

        let result = validate_row(&row, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid float"));
    }

    // ============================================================================
    // Boolean Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_boolean_valid() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("active".to_string(), "boolean".to_string())],
        );

        for value in &["true", "false", "1", "0"] {
            let row = CsvRow::new("1".to_string(), vec![value.to_string()]);
            assert!(validate_row(&row, &schema).is_ok());
        }
    }

    #[test]
    fn test_validate_boolean_invalid() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("active".to_string(), "boolean".to_string())],
        );
        let row = CsvRow::new("1".to_string(), vec!["yes".to_string()]);

        let result = validate_row(&row, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid boolean"));
    }

    // ============================================================================
    // Timestamp Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_timestamp_valid() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new(
                "created_at".to_string(),
                "timestamp".to_string(),
            )],
        );
        let row = CsvRow::new("1".to_string(), vec!["1609459200".to_string()]);

        assert!(validate_row(&row, &schema).is_ok());
    }

    #[test]
    fn test_validate_timestamp_invalid() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new(
                "created_at".to_string(),
                "timestamp".to_string(),
            )],
        );
        let row = CsvRow::new("1".to_string(), vec!["not-a-timestamp".to_string()]);

        let result = validate_row(&row, &schema);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid timestamp"));
    }

    // ============================================================================
    // Uniqueness Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_uniqueness_pass() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![
                ColumnDef::new("id".to_string(), "integer".to_string()).unique(),
                ColumnDef::new("name".to_string(), "string".to_string()),
            ],
        );

        let rows = vec![
            CsvRow::new("1".to_string(), vec!["1".to_string(), "Alice".to_string()]),
            CsvRow::new("2".to_string(), vec!["2".to_string(), "Bob".to_string()]),
            CsvRow::new(
                "3".to_string(),
                vec!["3".to_string(), "Charlie".to_string()],
            ),
        ];

        assert!(validate_uniqueness(&rows, &schema).is_ok());
    }

    #[test]
    fn test_validate_uniqueness_fail() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![
                ColumnDef::new("id".to_string(), "integer".to_string()).unique(),
                ColumnDef::new("name".to_string(), "string".to_string()),
            ],
        );

        let rows = vec![
            CsvRow::new("1".to_string(), vec!["1".to_string(), "Alice".to_string()]),
            CsvRow::new("2".to_string(), vec!["1".to_string(), "Bob".to_string()]), // Duplicate ID
        ];

        let result = validate_uniqueness(&rows, &schema);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Duplicate value violates unique constraint"));
    }

    #[test]
    fn test_validate_uniqueness_empty_optional() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![
                ColumnDef::new("id".to_string(), "integer".to_string()),
                ColumnDef::new("email".to_string(), "string".to_string()).unique(),
            ],
        );

        let rows = vec![
            CsvRow::new("1".to_string(), vec!["1".to_string(), "".to_string()]),
            CsvRow::new("2".to_string(), vec!["2".to_string(), "".to_string()]),
        ];

        // Empty values should be allowed for optional unique columns
        assert!(validate_uniqueness(&rows, &schema).is_ok());
    }

    // ============================================================================
    // Batch Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_rows_all_valid() {
        let schema = create_test_schema();
        let rows = vec![
            CsvRow::new(
                "1".to_string(),
                vec![
                    "1".to_string(),
                    "Alice".to_string(),
                    "alice@example.com".to_string(),
                    "25".to_string(),
                ],
            ),
            CsvRow::new(
                "2".to_string(),
                vec![
                    "2".to_string(),
                    "Bob".to_string(),
                    "bob@example.com".to_string(),
                    "30".to_string(),
                ],
            ),
        ];

        assert!(validate_rows(&rows, &schema).is_ok());
    }

    #[test]
    fn test_validate_rows_with_error() {
        let schema = create_test_schema();
        let rows = vec![
            CsvRow::new(
                "1".to_string(),
                vec![
                    "1".to_string(),
                    "Alice".to_string(),
                    "alice@example.com".to_string(),
                    "25".to_string(),
                ],
            ),
            CsvRow::new(
                "2".to_string(),
                vec![
                    "invalid".to_string(), // Invalid integer
                    "Bob".to_string(),
                    "bob@example.com".to_string(),
                    "30".to_string(),
                ],
            ),
        ];

        let result = validate_rows(&rows, &schema);
        assert!(result.is_err());
    }

    // ============================================================================
    // Optional Field Tests
    // ============================================================================

    #[test]
    fn test_validate_optional_empty_field() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![
                ColumnDef::new("id".to_string(), "integer".to_string()).required(),
                ColumnDef::new("description".to_string(), "string".to_string()), // Optional
            ],
        );

        let row = CsvRow::new("1".to_string(), vec!["1".to_string(), "".to_string()]);

        assert!(validate_row(&row, &schema).is_ok());
    }

    #[test]
    fn test_validate_primary_key() {
        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![
                ColumnDef::primary_key("id".to_string(), "integer".to_string()),
                ColumnDef::new("name".to_string(), "string".to_string()),
            ],
        );

        // Primary key cannot be empty
        let row = CsvRow::new("1".to_string(), vec!["".to_string(), "Alice".to_string()]);
        let result = validate_row(&row, &schema);
        assert!(result.is_err());

        // Valid primary key
        let row = CsvRow::new("1".to_string(), vec!["1".to_string(), "Alice".to_string()]);
        assert!(validate_row(&row, &schema).is_ok());
    }
}
