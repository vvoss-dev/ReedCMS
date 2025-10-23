// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Schema types for column validation.
//!
//! Defines the structure for TOML-based table schemas with type and constraint validation.

use serde::{Deserialize, Serialize};

/// Table schema definition.
///
/// Defines column types and constraints for a CSV table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Schema {
    /// Schema version (e.g., "2.0")
    pub version: String,

    /// Strict mode: reject writes that violate schema
    pub strict: bool,

    /// Column definitions
    pub columns: Vec<ColumnDef>,
}

impl Schema {
    /// Create a new schema.
    pub fn new(version: String, strict: bool, columns: Vec<ColumnDef>) -> Self {
        Schema {
            version,
            strict,
            columns,
        }
    }

    /// Get column by name.
    pub fn get_column(&self, name: &str) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Get column count.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Check if schema is empty.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

/// Column definition with type and constraints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnDef {
    /// Column name
    pub name: String,

    /// Column type: "string", "integer", "float", "boolean", "timestamp"
    #[serde(rename = "type")]
    pub col_type: String,

    /// Required field (cannot be empty)
    #[serde(default)]
    pub required: bool,

    /// Unique constraint
    #[serde(default)]
    pub unique: bool,

    /// Primary key (implies required + unique)
    #[serde(default)]
    pub primary_key: bool,

    /// Minimum value (for integer/float)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,

    /// Maximum value (for integer/float)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,

    /// Minimum length (for string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Maximum length (for string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Regex pattern (for string validation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

impl ColumnDef {
    /// Create a new column definition.
    pub fn new(name: String, col_type: String) -> Self {
        ColumnDef {
            name,
            col_type,
            required: false,
            unique: false,
            primary_key: false,
            min: None,
            max: None,
            min_length: None,
            max_length: None,
            pattern: None,
        }
    }

    /// Create a primary key column.
    pub fn primary_key(name: String, col_type: String) -> Self {
        ColumnDef {
            name,
            col_type,
            required: true,
            unique: true,
            primary_key: true,
            min: None,
            max: None,
            min_length: None,
            max_length: None,
            pattern: None,
        }
    }

    /// Set as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set as unique.
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Set min value.
    pub fn with_min(mut self, min: i64) -> Self {
        self.min = Some(min);
        self
    }

    /// Set max value.
    pub fn with_max(mut self, max: i64) -> Self {
        self.max = Some(max);
        self
    }

    /// Set min length.
    pub fn with_min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }

    /// Set max length.
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set pattern.
    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.pattern = Some(pattern);
        self
    }

    /// Check if column is required (either explicitly or via primary_key).
    pub fn is_required(&self) -> bool {
        self.required || self.primary_key
    }

    /// Check if column is unique (either explicitly or via primary_key).
    pub fn is_unique(&self) -> bool {
        self.unique || self.primary_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_new() {
        let columns = vec![
            ColumnDef::new("id".to_string(), "integer".to_string()),
            ColumnDef::new("name".to_string(), "string".to_string()),
        ];
        let schema = Schema::new("2.0".to_string(), true, columns);

        assert_eq!(schema.version, "2.0");
        assert!(schema.strict);
        assert_eq!(schema.columns.len(), 2);
    }

    #[test]
    fn test_schema_get_column() {
        let columns = vec![
            ColumnDef::new("id".to_string(), "integer".to_string()),
            ColumnDef::new("name".to_string(), "string".to_string()),
        ];
        let schema = Schema::new("2.0".to_string(), true, columns);

        let col = schema.get_column("name");
        assert!(col.is_some());
        assert_eq!(col.unwrap().name, "name");

        let col = schema.get_column("nonexistent");
        assert!(col.is_none());
    }

    #[test]
    fn test_schema_column_count() {
        let columns = vec![
            ColumnDef::new("id".to_string(), "integer".to_string()),
            ColumnDef::new("name".to_string(), "string".to_string()),
        ];
        let schema = Schema::new("2.0".to_string(), true, columns);

        assert_eq!(schema.column_count(), 2);
    }

    #[test]
    fn test_schema_is_empty() {
        let schema = Schema::new("2.0".to_string(), true, vec![]);
        assert!(schema.is_empty());

        let columns = vec![ColumnDef::new("id".to_string(), "integer".to_string())];
        let schema = Schema::new("2.0".to_string(), true, columns);
        assert!(!schema.is_empty());
    }

    #[test]
    fn test_column_def_new() {
        let col = ColumnDef::new("id".to_string(), "integer".to_string());

        assert_eq!(col.name, "id");
        assert_eq!(col.col_type, "integer");
        assert!(!col.required);
        assert!(!col.unique);
        assert!(!col.primary_key);
    }

    #[test]
    fn test_column_def_primary_key() {
        let col = ColumnDef::primary_key("id".to_string(), "integer".to_string());

        assert_eq!(col.name, "id");
        assert!(col.required);
        assert!(col.unique);
        assert!(col.primary_key);
    }

    #[test]
    fn test_column_def_builder() {
        let col = ColumnDef::new("age".to_string(), "integer".to_string())
            .required()
            .with_min(0)
            .with_max(150);

        assert!(col.required);
        assert_eq!(col.min, Some(0));
        assert_eq!(col.max, Some(150));
    }

    #[test]
    fn test_column_def_string_constraints() {
        let col = ColumnDef::new("email".to_string(), "string".to_string())
            .required()
            .unique()
            .with_min_length(5)
            .with_max_length(100)
            .with_pattern(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string());

        assert!(col.required);
        assert!(col.unique);
        assert_eq!(col.min_length, Some(5));
        assert_eq!(col.max_length, Some(100));
        assert!(col.pattern.is_some());
    }

    #[test]
    fn test_column_def_is_required() {
        let col = ColumnDef::new("name".to_string(), "string".to_string());
        assert!(!col.is_required());

        let col = col.required();
        assert!(col.is_required());

        let col = ColumnDef::primary_key("id".to_string(), "integer".to_string());
        assert!(col.is_required());
    }

    #[test]
    fn test_column_def_is_unique() {
        let col = ColumnDef::new("name".to_string(), "string".to_string());
        assert!(!col.is_unique());

        let col = col.unique();
        assert!(col.is_unique());

        let col = ColumnDef::primary_key("id".to_string(), "integer".to_string());
        assert!(col.is_unique());
    }
}
