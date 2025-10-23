// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Schema validation logic for CSV rows.
//!
//! Validates rows against schema definitions with type and constraint checking.

use crate::error::{ReedError, ReedResult};
use crate::schema::types::{ColumnDef, Schema};
use regex::Regex;
use std::collections::HashSet;

/// CSV row representation.
#[derive(Debug, Clone, PartialEq)]
pub struct CsvRow {
    pub key: String,
    pub values: Vec<String>,
}

impl CsvRow {
    pub fn new(key: String, values: Vec<String>) -> Self {
        CsvRow { key, values }
    }
}

/// Validate row against schema.
///
/// ## Input
/// - `row`: CSV row to validate
/// - `schema`: Table schema
///
/// ## Output
/// - `Ok(())`: Row is valid
/// - `Err(ReedError::ValidationError)`: Validation failed
///
/// ## Performance
/// - < 1ms per row typical
pub fn validate_row(row: &CsvRow, schema: &Schema) -> ReedResult<()> {
    // Check field count
    if row.values.len() != schema.columns.len() {
        return Err(ReedError::ValidationError {
            column: "".to_string(),
            reason: format!(
                "Field count mismatch: expected {}, got {}",
                schema.columns.len(),
                row.values.len()
            ),
            value: None,
        });
    }

    // Validate each field
    for (i, column) in schema.columns.iter().enumerate() {
        let value = &row.values[i];
        validate_field(value, column)?;
    }

    Ok(())
}

/// Validate single field against column definition.
fn validate_field(value: &str, column: &ColumnDef) -> ReedResult<()> {
    // Check required
    if column.is_required() && value.is_empty() {
        return Err(ReedError::ValidationError {
            column: column.name.clone(),
            reason: "Required field is empty".to_string(),
            value: None,
        });
    }

    // Skip validation for empty optional fields
    if value.is_empty() && !column.is_required() {
        return Ok(());
    }

    // Type validation
    match column.col_type.as_str() {
        "string" => validate_string(value, column)?,
        "integer" => validate_integer(value, column)?,
        "float" => validate_float(value, column)?,
        "boolean" => validate_boolean(value, column)?,
        "timestamp" => validate_timestamp(value, column)?,
        _ => {
            return Err(ReedError::ValidationError {
                column: column.name.clone(),
                reason: format!("Unknown column type: {}", column.col_type),
                value: Some(value.to_string()),
            })
        }
    }

    Ok(())
}

/// Validate string field.
fn validate_string(value: &str, column: &ColumnDef) -> ReedResult<()> {
    // Check length constraints
    if let Some(min_length) = column.min_length {
        if value.len() < min_length {
            return Err(ReedError::ValidationError {
                column: column.name.clone(),
                reason: format!(
                    "String length {} is below minimum {}",
                    value.len(),
                    min_length
                ),
                value: Some(value.to_string()),
            });
        }
    }

    if let Some(max_length) = column.max_length {
        if value.len() > max_length {
            return Err(ReedError::ValidationError {
                column: column.name.clone(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    value.len(),
                    max_length
                ),
                value: Some(value.to_string()),
            });
        }
    }

    // Check pattern constraint
    if let Some(ref pattern) = column.pattern {
        let regex = Regex::new(pattern).map_err(|e| ReedError::ValidationError {
            column: column.name.clone(),
            reason: format!("Invalid regex pattern: {}", e),
            value: None,
        })?;

        if !regex.is_match(value) {
            return Err(ReedError::ValidationError {
                column: column.name.clone(),
                reason: format!("Value does not match pattern: {}", pattern),
                value: Some(value.to_string()),
            });
        }
    }

    Ok(())
}

/// Validate integer field.
fn validate_integer(value: &str, column: &ColumnDef) -> ReedResult<()> {
    let num = value
        .parse::<i64>()
        .map_err(|_| ReedError::ValidationError {
            column: column.name.clone(),
            reason: "Invalid integer value".to_string(),
            value: Some(value.to_string()),
        })?;

    if let Some(min) = column.min {
        if num < min {
            return Err(ReedError::ValidationError {
                column: column.name.clone(),
                reason: format!("Value {} is below minimum {}", num, min),
                value: Some(value.to_string()),
            });
        }
    }

    if let Some(max) = column.max {
        if num > max {
            return Err(ReedError::ValidationError {
                column: column.name.clone(),
                reason: format!("Value {} exceeds maximum {}", num, max),
                value: Some(value.to_string()),
            });
        }
    }

    Ok(())
}

/// Validate float field.
fn validate_float(value: &str, column: &ColumnDef) -> ReedResult<()> {
    value
        .parse::<f64>()
        .map_err(|_| ReedError::ValidationError {
            column: column.name.clone(),
            reason: "Invalid float value".to_string(),
            value: Some(value.to_string()),
        })?;

    Ok(())
}

/// Validate boolean field.
fn validate_boolean(value: &str, column: &ColumnDef) -> ReedResult<()> {
    if !matches!(value, "true" | "false" | "1" | "0") {
        return Err(ReedError::ValidationError {
            column: column.name.clone(),
            reason: "Invalid boolean value (expected: true, false, 1, or 0)".to_string(),
            value: Some(value.to_string()),
        });
    }

    Ok(())
}

/// Validate timestamp field.
fn validate_timestamp(value: &str, column: &ColumnDef) -> ReedResult<()> {
    value
        .parse::<u64>()
        .map_err(|_| ReedError::ValidationError {
            column: column.name.clone(),
            reason: "Invalid timestamp value (expected: Unix timestamp)".to_string(),
            value: Some(value.to_string()),
        })?;

    Ok(())
}

/// Validate uniqueness constraints across all rows.
///
/// ## Performance
/// - O(n*m) where n = rows, m = unique columns
/// - < 10ms for 100 rows with 2 unique columns
pub fn validate_uniqueness(rows: &[CsvRow], schema: &Schema) -> ReedResult<()> {
    for (col_idx, column) in schema.columns.iter().enumerate() {
        if column.is_unique() {
            let mut seen = HashSet::new();

            for row in rows {
                if col_idx >= row.values.len() {
                    continue;
                }

                let value = &row.values[col_idx];

                // Skip empty values for optional unique columns
                if value.is_empty() && !column.is_required() {
                    continue;
                }

                if !value.is_empty() && !seen.insert(value.clone()) {
                    return Err(ReedError::ValidationError {
                        column: column.name.clone(),
                        reason: "Duplicate value violates unique constraint".to_string(),
                        value: Some(value.clone()),
                    });
                }
            }
        }
    }

    Ok(())
}

/// Validate multiple rows in batch.
///
/// ## Performance
/// - O(n) for row validation + O(n*m) for uniqueness
/// - < 100ms for 100 rows typical
pub fn validate_rows(rows: &[CsvRow], schema: &Schema) -> ReedResult<()> {
    // Validate each row individually
    for row in rows {
        validate_row(row, schema)?;
    }

    // Check uniqueness constraints
    validate_uniqueness(rows, schema)?;

    Ok(())
}
