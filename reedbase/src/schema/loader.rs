// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Schema loading and saving with TOML format.
//!
//! Provides functions to load and save table schemas from/to TOML files.

use crate::error::{ReedError, ReedResult};
use crate::schema::types::{ColumnDef, Schema};
use std::fs;
use std::path::{Path, PathBuf};

/// Load schema from TOML file.
///
/// ## Performance
/// - < 5ms typical (TOML parsing)
///
/// ## Example
/// ```rust
/// let schema = load_schema(Path::new(".reed"), "users")?;
/// ```
pub fn load_schema(base_path: &Path, table_name: &str) -> ReedResult<Schema> {
    let schema_path = get_schema_path(base_path, table_name);

    if !schema_path.exists() {
        return Err(ReedError::SchemaNotFound {
            table: table_name.to_string(),
        });
    }

    let content = fs::read_to_string(&schema_path).map_err(|e| ReedError::IoError {
        operation: format!("read schema file '{}'", schema_path.display()),
        reason: e.to_string(),
    })?;

    parse_schema(&content)
}

/// Parse schema from TOML string.
fn parse_schema(content: &str) -> ReedResult<Schema> {
    let schema: Schema = toml::from_str(content).map_err(|e| ReedError::InvalidSchema {
        reason: format!("TOML parse error: {}", e),
    })?;

    // Validate schema
    if schema.columns.is_empty() {
        return Err(ReedError::InvalidSchema {
            reason: "Schema must have at least one column".to_string(),
        });
    }

    // Validate column types
    for column in &schema.columns {
        if !matches!(
            column.col_type.as_str(),
            "string" | "integer" | "float" | "boolean" | "timestamp"
        ) {
            return Err(ReedError::InvalidSchema {
                reason: format!(
                    "Invalid column type '{}' for column '{}'",
                    column.col_type, column.name
                ),
            });
        }
    }

    Ok(schema)
}

/// Save schema to TOML file.
///
/// ## Performance
/// - < 10ms typical (TOML serialization + write)
pub fn save_schema(base_path: &Path, table_name: &str, schema: &Schema) -> ReedResult<()> {
    let schema_path = get_schema_path(base_path, table_name);

    // Create parent directory
    if let Some(parent) = schema_path.parent() {
        fs::create_dir_all(parent).map_err(|e| ReedError::IoError {
            operation: format!("create schema directory '{}'", parent.display()),
            reason: e.to_string(),
        })?;
    }

    // Serialize to TOML
    let toml_string =
        toml::to_string_pretty(schema).map_err(|e| ReedError::SerializationError {
            reason: format!("TOML serialization error: {}", e),
        })?;

    // Write to file
    fs::write(&schema_path, toml_string).map_err(|e| ReedError::IoError {
        operation: format!("write schema file '{}'", schema_path.display()),
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Create default schema from column names.
///
/// All columns default to:
/// - type: "string"
/// - required: false
/// - strict: false (lenient mode)
pub fn create_default_schema(column_names: &[String]) -> Schema {
    let columns = column_names
        .iter()
        .map(|name| ColumnDef::new(name.clone(), "string".to_string()))
        .collect();

    Schema::new("2.0".to_string(), false, columns)
}

/// Get schema file path.
fn get_schema_path(base_path: &Path, table_name: &str) -> PathBuf {
    base_path
        .join("tables")
        .join(table_name)
        .join("schema.toml")
}

/// Check if schema exists for table.
pub fn schema_exists(base_path: &Path, table_name: &str) -> bool {
    get_schema_path(base_path, table_name).exists()
}

/// Delete schema file.
pub fn delete_schema(base_path: &Path, table_name: &str) -> ReedResult<()> {
    let schema_path = get_schema_path(base_path, table_name);

    if !schema_path.exists() {
        return Ok(());
    }

    fs::remove_file(&schema_path).map_err(|e| ReedError::IoError {
        operation: format!("delete schema file '{}'", schema_path.display()),
        reason: e.to_string(),
    })?;

    Ok(())
}
