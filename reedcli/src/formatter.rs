// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Output formatter for ReedCLI.
//!
//! Formats command output in multiple formats (table, JSON, CSV, plain).

use crate::types::{CliError, CliResult, CommandOutput, OutputFormat};
use prettytable::{format, Cell, Row, Table};
use serde_json::Value;

/// Format command output for display.
///
/// ## Input
/// - `output`: Command output with data and format
/// - `colour`: Whether to use colour coding
///
/// ## Output
/// - `CliResult<String>`: Formatted output string
///
/// ## Performance
/// - O(n) where n = number of records
/// - < 5ms typical for 100 records
///
/// ## Error Conditions
/// - FormatError: Data structure incompatible with requested format
///
/// ## Example Usage
/// ```rust
/// let output = CommandOutput {
///     data: json!([{"name": "Alice"}]),
///     format: OutputFormat::Table,
///     exit_code: 0,
/// };
/// let formatted = format_output(&output, true)?;
/// println!("{}", formatted);
/// ```
pub fn format_output(output: &CommandOutput, _colour: bool) -> CliResult<String> {
    match output.format {
        OutputFormat::Table => format_table(&output.data),
        OutputFormat::Json => format_json(&output.data),
        OutputFormat::Csv => format_csv(&output.data),
        OutputFormat::Plain => format_plain(&output.data),
    }
}

/// Format as ASCII table with box-drawing characters.
///
/// ## Input
/// - `data`: JSON value (must be array of objects)
///
/// ## Output
/// - `CliResult<String>`: Formatted table
///
/// ## Performance
/// - O(n*m) where n = rows, m = columns
/// - < 5ms for 100 rows
///
/// ## Error Conditions
/// - FormatError: Data is not an array of objects
///
/// ## Example Usage
/// ```rust
/// let data = json!([{"id": 1, "name": "Alice"}]);
/// let table = format_table(&data)?;
/// ```
fn format_table(data: &Value) -> CliResult<String> {
    let array = data.as_array().ok_or_else(|| CliError::FormatError {
        reason: "Table format requires array".to_string(),
    })?;

    if array.is_empty() {
        return Ok("(empty result set)".to_string());
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);

    // Extract headers from first object
    let first = &array[0];
    let obj = first.as_object().ok_or_else(|| CliError::FormatError {
        reason: "Table format requires array of objects".to_string(),
    })?;

    let headers: Vec<String> = obj.keys().cloned().collect();
    table.add_row(Row::new(headers.iter().map(|h| Cell::new(h)).collect()));

    // Add data rows
    for item in array {
        let obj = item.as_object().ok_or_else(|| CliError::FormatError {
            reason: "Table format requires array of objects".to_string(),
        })?;

        let row = headers
            .iter()
            .map(|h| {
                let value = obj.get(h).unwrap_or(&Value::Null);
                Cell::new(&format_value(value))
            })
            .collect();

        table.add_row(Row::new(row));
    }

    Ok(table.to_string())
}

/// Format as pretty-printed JSON.
///
/// ## Input
/// - `data`: JSON value (any structure)
///
/// ## Output
/// - `CliResult<String>`: Pretty-printed JSON
///
/// ## Performance
/// - O(n) where n = data size
/// - < 20ms for 1000 records
///
/// ## Error Conditions
/// - FormatError: JSON serialisation failure (should never happen for valid Value)
///
/// ## Example Usage
/// ```rust
/// let data = json!({"name": "Alice", "age": 30});
/// let json_str = format_json(&data)?;
/// ```
fn format_json(data: &Value) -> CliResult<String> {
    serde_json::to_string_pretty(data).map_err(|e| CliError::FormatError {
        reason: format!("JSON serialisation error: {}", e),
    })
}

/// Format as CSV with headers.
///
/// ## Input
/// - `data`: JSON value (must be array of objects)
///
/// ## Output
/// - `CliResult<String>`: CSV string with header row
///
/// ## Performance
/// - O(n*m) where n = rows, m = columns
/// - < 10ms for 1000 rows
///
/// ## Error Conditions
/// - FormatError: Data is not an array of objects
///
/// ## Example Usage
/// ```rust
/// let data = json!([{"id": 1, "name": "Alice"}]);
/// let csv = format_csv(&data)?;
/// ```
fn format_csv(data: &Value) -> CliResult<String> {
    let array = data.as_array().ok_or_else(|| CliError::FormatError {
        reason: "CSV format requires array".to_string(),
    })?;

    if array.is_empty() {
        return Ok(String::new());
    }

    let first = &array[0];
    let obj = first.as_object().ok_or_else(|| CliError::FormatError {
        reason: "CSV format requires array of objects".to_string(),
    })?;

    let headers: Vec<String> = obj.keys().cloned().collect();

    let mut output = String::new();

    // Header row
    output.push_str(&headers.join(","));
    output.push('\n');

    // Data rows
    for item in array {
        let obj = item.as_object().ok_or_else(|| CliError::FormatError {
            reason: "CSV format requires array of objects".to_string(),
        })?;

        let row: Vec<String> = headers
            .iter()
            .map(|h| {
                let value = obj.get(h).unwrap_or(&Value::Null);
                escape_csv_value(&format_value(value))
            })
            .collect();

        output.push_str(&row.join(","));
        output.push('\n');
    }

    Ok(output)
}

/// Format as plain text (space-separated values).
///
/// ## Input
/// - `data`: JSON value (must be array)
///
/// ## Output
/// - `CliResult<String>`: Plain text (one line per record)
///
/// ## Performance
/// - O(n*m) where n = rows, m = columns
/// - < 5ms for 1000 rows
///
/// ## Error Conditions
/// - FormatError: Data is not an array
///
/// ## Example Usage
/// ```rust
/// let data = json!([{"id": 1, "name": "Alice"}]);
/// let plain = format_plain(&data)?;
/// ```
fn format_plain(data: &Value) -> CliResult<String> {
    let array = data.as_array().ok_or_else(|| CliError::FormatError {
        reason: "Plain format requires array".to_string(),
    })?;

    let mut output = String::new();

    for item in array {
        if let Some(obj) = item.as_object() {
            let values: Vec<String> = obj.values().map(|v| format_value(v)).collect();
            output.push_str(&values.join(" "));
        } else {
            output.push_str(&format_value(item));
        }
        output.push('\n');
    }

    Ok(output)
}

/// Format a single JSON value as string.
///
/// ## Input
/// - `value`: JSON value (any type)
///
/// ## Output
/// - String representation
///
/// ## Performance
/// - O(1) for primitives, O(n) for arrays/objects
/// - < 10μs typical for primitives
///
/// ## Example Usage
/// ```rust
/// assert_eq!(format_value(&json!(42)), "42");
/// assert_eq!(format_value(&json!("text")), "text");
/// assert_eq!(format_value(&json!(null)), "");
/// ```
pub fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
        }
    }
}

/// Escape CSV value (quote if contains special characters).
///
/// ## Input
/// - `value`: String value to escape
///
/// ## Output
/// - Escaped string (quoted if necessary)
///
/// ## Performance
/// - O(n) where n = string length
/// - < 10μs for typical values
///
/// ## Logic
/// - Quote if contains: comma, quote, newline
/// - Escape quotes by doubling (" → "")
///
/// ## Example Usage
/// ```rust
/// assert_eq!(escape_csv_value("simple"), "simple");
/// assert_eq!(escape_csv_value("with,comma"), "\"with,comma\"");
/// assert_eq!(escape_csv_value("with\"quote"), "\"with\"\"quote\"");
/// ```
pub fn escape_csv_value(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Auto-detect if terminal supports colour.
///
/// ## Output
/// - `bool`: True if colour is supported
///
/// ## Performance
/// - O(1) operation
/// - < 10μs typical
///
/// ## Logic
/// - Check NO_COLOR environment variable (if set, disable colour)
/// - Check if stdout is a TTY (if not, disable colour)
///
/// ## Example Usage
/// ```rust
/// let use_colour = supports_colour();
/// let formatted = format_output(&output, use_colour)?;
/// ```
pub fn supports_colour() -> bool {
    std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout)
}

#[cfg(test)]
#[path = "formatter_test.rs"]
mod tests;
