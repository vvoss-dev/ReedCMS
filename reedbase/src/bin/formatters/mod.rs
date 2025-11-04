// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Output formatters for query results.

use reedbase::reedql::QueryResult;

/// Formats result as human-readable table.
pub fn format_table(result: &QueryResult) -> String {
    match result {
        QueryResult::Rows(rows) => {
            if rows.is_empty() {
                return "0 rows\n".to_string();
            }

            // Get column names (from first row)
            let mut columns: Vec<String> = rows[0].keys().cloned().collect();
            columns.sort();

            // Calculate column widths
            let mut widths: std::collections::HashMap<String, usize> =
                columns.iter().map(|c| (c.clone(), c.len())).collect();

            for row in rows {
                for col in &columns {
                    if let Some(value) = row.get(col) {
                        let current = widths.get(col).copied().unwrap_or(0);
                        widths.insert(col.clone(), current.max(value.len()));
                    }
                }
            }

            // Build table
            let mut output = String::new();

            // Top border
            output.push('+');
            for col in &columns {
                let width = widths.get(col).copied().unwrap_or(0) + 2;
                output.push_str(&"-".repeat(width));
                output.push('+');
            }
            output.push('\n');

            // Header
            output.push('|');
            for col in &columns {
                let width = widths.get(col).copied().unwrap_or(0);
                output.push_str(&format!(" {:<width$} |", col, width = width));
            }
            output.push('\n');

            // Separator
            output.push('+');
            for col in &columns {
                let width = widths.get(col).copied().unwrap_or(0) + 2;
                output.push_str(&"-".repeat(width));
                output.push('+');
            }
            output.push('\n');

            // Rows
            for row in rows {
                output.push('|');
                for col in &columns {
                    let width = widths.get(col).copied().unwrap_or(0);
                    let value = row.get(col).map(|s| s.as_str()).unwrap_or("");
                    output.push_str(&format!(" {:<width$} |", value, width = width));
                }
                output.push('\n');
            }

            // Bottom border
            output.push('+');
            for col in &columns {
                let width = widths.get(col).copied().unwrap_or(0) + 2;
                output.push_str(&"-".repeat(width));
                output.push('+');
            }
            output.push('\n');

            output.push_str(&format!("{} rows\n", rows.len()));
            output
        }

        QueryResult::Aggregation(value) => {
            format!("{}\n", value)
        }
    }
}

/// Formats result as JSON.
pub fn format_json(result: &QueryResult) -> String {
    match result {
        QueryResult::Rows(rows) => {
            if rows.is_empty() {
                return "[]\n".to_string();
            }

            let mut output = String::from("[\n");
            for (i, row) in rows.iter().enumerate() {
                output.push_str("  {");

                let mut keys: Vec<_> = row.keys().collect();
                keys.sort();

                for (j, key) in keys.iter().enumerate() {
                    let value = row.get(*key).map(|s| s.as_str()).unwrap_or("");
                    // Escape quotes in JSON strings
                    let escaped_value = value.replace('"', "\\\"");
                    output.push_str(&format!("\"{}\": \"{}\"", key, escaped_value));
                    if j < keys.len() - 1 {
                        output.push_str(", ");
                    }
                }

                output.push('}');
                if i < rows.len() - 1 {
                    output.push(',');
                }
                output.push('\n');
            }
            output.push_str("]\n");
            output
        }

        QueryResult::Aggregation(value) => {
            format!("{}\n", value)
        }
    }
}

/// Formats result as CSV.
pub fn format_csv(result: &QueryResult, include_header: bool) -> String {
    match result {
        QueryResult::Rows(rows) => {
            if rows.is_empty() {
                return "".to_string();
            }

            let mut output = String::new();

            // Header
            let mut columns: Vec<String> = rows[0].keys().cloned().collect();
            columns.sort();

            if include_header {
                output.push_str(&columns.join(","));
                output.push('\n');
            }

            // Rows
            for row in rows {
                let values: Vec<String> = columns
                    .iter()
                    .map(|col| {
                        let val = row.get(col).map(|s| s.as_str()).unwrap_or("");
                        // Escape CSV values if they contain comma or quotes
                        if val.contains(',') || val.contains('"') {
                            format!("\"{}\"", val.replace('"', "\"\""))
                        } else {
                            val.to_string()
                        }
                    })
                    .collect();
                output.push_str(&values.join(","));
                output.push('\n');
            }

            output
        }

        QueryResult::Aggregation(value) => {
            format!("{}\n", value)
        }
    }
}
