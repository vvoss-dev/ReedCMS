# REED-18-03: Output Formatter

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-18-03
- **Title**: Output Formatter
- **Layer**: CLI Layer (REED-18)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: None
- **Estimated Time**: 2 days

## Objective

Format command output for display in multiple formats (table, JSON, CSV, plain text) with optional colour coding.

## Requirements

### Input Format

```rust
CommandOutput {
    data: json!([
        {"id": 1, "name": "Alice", "age": 30},
        {"id": 2, "name": "Bob", "age": 25}
    ]),
    format: OutputFormat::Table,
    exit_code: 0,
}
```

### Output Formats

**Table (default for terminal):**
```
┌────┬───────┬─────┐
│ id │ name  │ age │
├────┼───────┼─────┤
│ 1  │ Alice │ 30  │
│ 2  │ Bob   │ 25  │
└────┴───────┴─────┘
```

**JSON:**
```json
[
  {
    "id": 1,
    "name": "Alice",
    "age": 30
  },
  {
    "id": 2,
    "name": "Bob",
    "age": 25
  }
]
```

**CSV:**
```csv
id,name,age
1,Alice,30
2,Bob,25
```

**Plain:**
```
1 Alice 30
2 Bob 25
```

## Implementation Files

### Primary Implementation

**`reedcli/src/formatter.rs`**

One file = Output formatting only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Output formatter for ReedCLI.
//!
//! Formats command output in multiple formats (table, JSON, CSV, plain).

use crate::types::{CommandOutput, OutputFormat, CliResult, CliError};
use prettytable::{Table, Row, Cell, format};
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
pub fn format_output(output: &CommandOutput, colour: bool) -> CliResult<String> {
    match output.format {
        OutputFormat::Table => format_table(&output.data, colour),
        OutputFormat::Json => format_json(&output.data),
        OutputFormat::Csv => format_csv(&output.data),
        OutputFormat::Plain => format_plain(&output.data),
    }
}

/// Format as ASCII table with box-drawing characters.
///
/// ## Input
/// - `data`: JSON value (must be array of objects)
/// - `colour`: Whether to use colour (currently unused, reserved for future)
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
/// let table = format_table(&data, false)?;
/// ```
fn format_table(data: &Value, colour: bool) -> CliResult<String> {
    let array = data.as_array()
        .ok_or_else(|| CliError::FormatError {
            reason: "Table format requires array".to_string(),
        })?;
    
    if array.is_empty() {
        return Ok("(empty result set)".to_string());
    }
    
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    
    // Extract headers from first object
    let first = &array[0];
    let obj = first.as_object()
        .ok_or_else(|| CliError::FormatError {
            reason: "Table format requires array of objects".to_string(),
        })?;
    
    let headers: Vec<String> = obj.keys().cloned().collect();
    table.add_row(Row::new(
        headers.iter().map(|h| Cell::new(h)).collect()
    ));
    
    // Add data rows
    for item in array {
        let obj = item.as_object()
            .ok_or_else(|| CliError::FormatError {
                reason: "Table format requires array of objects".to_string(),
            })?;
        
        let row = headers.iter().map(|h| {
            let value = obj.get(h).unwrap_or(&Value::Null);
            Cell::new(&format_value(value))
        }).collect();
        
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
    serde_json::to_string_pretty(data)
        .map_err(|e| CliError::FormatError {
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
    let array = data.as_array()
        .ok_or_else(|| CliError::FormatError {
            reason: "CSV format requires array".to_string(),
        })?;
    
    if array.is_empty() {
        return Ok(String::new());
    }
    
    let first = &array[0];
    let obj = first.as_object()
        .ok_or_else(|| CliError::FormatError {
            reason: "CSV format requires array of objects".to_string(),
        })?;
    
    let headers: Vec<String> = obj.keys().cloned().collect();
    
    let mut output = String::new();
    
    // Header row
    output.push_str(&headers.join(","));
    output.push('\n');
    
    // Data rows
    for item in array {
        let obj = item.as_object()
            .ok_or_else(|| CliError::FormatError {
                reason: "CSV format requires array of objects".to_string(),
            })?;
        
        let row: Vec<String> = headers.iter().map(|h| {
            let value = obj.get(h).unwrap_or(&Value::Null);
            escape_csv_value(&format_value(value))
        }).collect();
        
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
    let array = data.as_array()
        .ok_or_else(|| CliError::FormatError {
            reason: "Plain format requires array".to_string(),
        })?;
    
    let mut output = String::new();
    
    for item in array {
        if let Some(obj) = item.as_object() {
            let values: Vec<String> = obj.values()
                .map(|v| format_value(v))
                .collect();
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
fn format_value(value: &Value) -> String {
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
fn escape_csv_value(value: &str) -> String {
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
```

**`reedcli/src/types.rs`** (additions)

```rust
/// Output format options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Table,    // ASCII table with box-drawing characters
    Json,     // Pretty-printed JSON
    Csv,      // CSV with headers
    Plain,    // Plain text (space-separated)
}

/// Command output from tool.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub data: serde_json::Value,
    pub format: OutputFormat,
    pub exit_code: i32,
}

/// Additional CLI errors.
#[derive(Error, Debug)]
pub enum CliError {
    // ... (existing errors from REED-18-01, REED-18-02)
    
    #[error("Format error: {reason}")]
    FormatError {
        reason: String,
    },
}
```

### Test Files

**`reedcli/src/formatter.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_format_table_simple() {
        let data = json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]);
        
        let result = format_table(&data, false).unwrap();
        assert!(result.contains("id"));
        assert!(result.contains("name"));
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
    }
    
    #[test]
    fn test_format_table_empty() {
        let data = json!([]);
        let result = format_table(&data, false).unwrap();
        assert_eq!(result, "(empty result set)");
    }
    
    #[test]
    fn test_format_table_non_array() {
        let data = json!({"not": "array"});
        let result = format_table(&data, false);
        assert!(matches!(result, Err(CliError::FormatError { .. })));
    }
    
    #[test]
    fn test_format_table_non_objects() {
        let data = json!(["string", "array"]);
        let result = format_table(&data, false);
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
    fn test_format_value_bool() {
        assert_eq!(format_value(&json!(true)), "true");
        assert_eq!(format_value(&json!(false)), "false");
    }
    
    #[test]
    fn test_format_value_number() {
        assert_eq!(format_value(&json!(42)), "42");
        assert_eq!(format_value(&json!(3.14)), "3.14");
    }
    
    #[test]
    fn test_format_value_string() {
        assert_eq!(format_value(&json!("text")), "text");
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
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Format 10 records as table | < 1ms |
| Format 100 records as table | < 5ms |
| Format 1000 records as JSON | < 20ms |
| Format 1000 records as CSV | < 10ms |
| Format 1000 records as plain | < 5ms |
| Escape CSV value | < 10μs |

## Error Conditions

- **FormatError**: Data structure incompatible with requested format (e.g., object instead of array for table format)

## CLI Commands

Not applicable - this is an internal formatting module, not a CLI command.

## Acceptance Criteria

- [ ] Format as ASCII table with box-drawing characters
- [ ] Format as pretty-printed JSON
- [ ] Format as CSV with headers
- [ ] Format as plain text (space-separated)
- [ ] Escape CSV values containing commas/quotes/newlines
- [ ] Handle empty result sets gracefully ("(empty result set)" for table, empty string for CSV)
- [ ] Return FormatError for incompatible data structures
- [ ] Auto-detect terminal colour support (NO_COLOR env var + TTY check)
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation (Input/Output/Performance/Error Conditions/Example Usage)
- [ ] No Swiss Army knife functions (each function = one job)
- [ ] Separate test file as `formatter.test.rs`

## Dependencies

**Requires**: None

**Blocks**: 
- REED-18-04 (Interactive Shell - needs formatter for output)
- REED-18-06 (Tool Integration - needs formatter for responses)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-18-00: Layer Overview

## Notes

The formatter does NOT:
- Validate data (assumes valid JSON from tools)
- Paginate output (future enhancement)
- Apply colours (reserved for future, infrastructure in place)

The formatter ONLY:
- Formats JSON data → formatted strings
- Pure data transformation, no I/O

This separation ensures the formatter can be tested in isolation and reused for different output destinations (stdout, files, logs).
