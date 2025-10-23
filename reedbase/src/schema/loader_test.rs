// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for schema loader.

#[cfg(test)]
mod tests {
    use crate::schema::loader::{
        create_default_schema, delete_schema, load_schema, save_schema, schema_exists,
    };
    use crate::schema::types::{ColumnDef, Schema};
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_schema() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![
                ColumnDef::primary_key("id".to_string(), "integer".to_string()),
                ColumnDef::new("name".to_string(), "string".to_string())
                    .required()
                    .with_min_length(1),
            ],
        );

        // Save schema
        save_schema(base_path, "users", &schema).unwrap();

        // Load schema
        let loaded = load_schema(base_path, "users").unwrap();

        assert_eq!(loaded.version, schema.version);
        assert_eq!(loaded.strict, schema.strict);
        assert_eq!(loaded.columns.len(), schema.columns.len());
        assert_eq!(loaded.columns[0].name, "id");
        assert_eq!(loaded.columns[1].name, "name");
    }

    #[test]
    fn test_load_nonexistent_schema() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let result = load_schema(base_path, "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Schema not found"));
    }

    #[test]
    fn test_schema_exists() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        assert!(!schema_exists(base_path, "users"));

        let schema = Schema::new("2.0".to_string(), true, vec![]);
        save_schema(base_path, "users", &schema).unwrap();

        assert!(schema_exists(base_path, "users"));
    }

    #[test]
    fn test_delete_schema() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let schema = Schema::new("2.0".to_string(), true, vec![]);
        save_schema(base_path, "users", &schema).unwrap();

        assert!(schema_exists(base_path, "users"));

        delete_schema(base_path, "users").unwrap();

        assert!(!schema_exists(base_path, "users"));
    }

    #[test]
    fn test_delete_nonexistent_schema() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        // Should not error
        assert!(delete_schema(base_path, "nonexistent").is_ok());
    }

    #[test]
    fn test_create_default_schema() {
        let columns = vec!["id".to_string(), "name".to_string(), "email".to_string()];
        let schema = create_default_schema(&columns);

        assert_eq!(schema.version, "2.0");
        assert!(!schema.strict); // Lenient by default
        assert_eq!(schema.columns.len(), 3);

        for (i, col) in schema.columns.iter().enumerate() {
            assert_eq!(col.name, columns[i]);
            assert_eq!(col.col_type, "string");
            assert!(!col.required);
            assert!(!col.unique);
        }
    }

    #[test]
    fn test_save_schema_with_constraints() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![
                ColumnDef::primary_key("id".to_string(), "integer".to_string()).with_min(1),
                ColumnDef::new("email".to_string(), "string".to_string())
                    .unique()
                    .with_pattern(r"^[a-z]+@[a-z]+\.[a-z]+$".to_string()),
                ColumnDef::new("age".to_string(), "integer".to_string())
                    .with_min(0)
                    .with_max(150),
            ],
        );

        save_schema(base_path, "users", &schema).unwrap();
        let loaded = load_schema(base_path, "users").unwrap();

        // Check constraints preserved
        assert_eq!(loaded.columns[0].primary_key, true);
        assert_eq!(loaded.columns[0].min, Some(1));

        assert_eq!(loaded.columns[1].unique, true);
        assert!(loaded.columns[1].pattern.is_some());

        assert_eq!(loaded.columns[2].min, Some(0));
        assert_eq!(loaded.columns[2].max, Some(150));
    }

    #[test]
    fn test_invalid_schema_empty_columns() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let schema = Schema::new("2.0".to_string(), true, vec![]);
        save_schema(base_path, "users", &schema).unwrap();

        let result = load_schema(base_path, "users");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least one column"));
    }

    #[test]
    fn test_invalid_schema_wrong_type() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        // Manually create invalid schema file
        let schema_path = base_path.join("tables/users/schema.toml");
        std::fs::create_dir_all(schema_path.parent().unwrap()).unwrap();
        std::fs::write(
            &schema_path,
            r#"
version = "2.0"
strict = true

[[columns]]
name = "id"
type = "invalid_type"
"#,
        )
        .unwrap();

        let result = load_schema(base_path, "users");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid column type"));
    }

    #[test]
    fn test_schema_toml_format() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("id".to_string(), "integer".to_string()).required()],
        );

        save_schema(base_path, "users", &schema).unwrap();

        let schema_path = base_path.join("tables/users/schema.toml");
        let content = std::fs::read_to_string(&schema_path).unwrap();

        assert!(content.contains("version = \"2.0\""));
        assert!(content.contains("strict = true"));
        assert!(content.contains("[[columns]]"));
        assert!(content.contains("name = \"id\""));
        assert!(content.contains("type = \"integer\""));
        assert!(content.contains("required = true"));
    }

    #[test]
    fn test_roundtrip_all_column_types() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let schema = Schema::new(
            "2.0".to_string(),
            true,
            vec![
                ColumnDef::new("str_col".to_string(), "string".to_string()),
                ColumnDef::new("int_col".to_string(), "integer".to_string()),
                ColumnDef::new("float_col".to_string(), "float".to_string()),
                ColumnDef::new("bool_col".to_string(), "boolean".to_string()),
                ColumnDef::new("ts_col".to_string(), "timestamp".to_string()),
            ],
        );

        save_schema(base_path, "test", &schema).unwrap();
        let loaded = load_schema(base_path, "test").unwrap();

        assert_eq!(loaded.columns[0].col_type, "string");
        assert_eq!(loaded.columns[1].col_type, "integer");
        assert_eq!(loaded.columns[2].col_type, "float");
        assert_eq!(loaded.columns[3].col_type, "boolean");
        assert_eq!(loaded.columns[4].col_type, "timestamp");
    }

    #[test]
    fn test_schema_lenient_mode() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let schema = Schema::new("2.0".to_string(), false, vec![]); // strict = false
        save_schema(base_path, "users", &schema).unwrap();

        // Should fail due to empty columns, regardless of strict mode
        let result = load_schema(base_path, "users");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_schemas() {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path();

        let schema1 = Schema::new(
            "2.0".to_string(),
            true,
            vec![ColumnDef::new("id".to_string(), "integer".to_string())],
        );

        let schema2 = Schema::new(
            "2.0".to_string(),
            false,
            vec![ColumnDef::new("name".to_string(), "string".to_string())],
        );

        save_schema(base_path, "users", &schema1).unwrap();
        save_schema(base_path, "products", &schema2).unwrap();

        let loaded1 = load_schema(base_path, "users").unwrap();
        let loaded2 = load_schema(base_path, "products").unwrap();

        assert_eq!(loaded1.columns[0].name, "id");
        assert_eq!(loaded2.columns[0].name, "name");
    }
}
