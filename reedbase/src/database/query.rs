// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Query execution (SELECT) via ReedQL.
//!
//! This module handles all SELECT queries through the ReedQL engine.

use crate::database::database::Database;
use crate::database::stats::QueryPattern;
use crate::database::types::QueryMetrics;
use crate::error::{ReedError, ReedResult};
use crate::reedql::{execute, parse, OptimizedExecutor, QueryResult};
use std::time::Instant;

/// Executes a ReedQL SELECT query.
///
/// ## Input
/// - `db`: Database reference
/// - `sql`: ReedQL query string
///
/// ## Output
/// - `Ok(QueryResult)`: Query result
/// - `Err(ReedError)`: Execution failed
///
/// ## Performance
/// - Parse: < 10μs
/// - Execute (with index): < 100μs (exact), < 1ms (range)
/// - Execute (no index): ~10ms for 10k rows
pub fn execute_query(db: &Database, sql: &str) -> ReedResult<QueryResult> {
    let mut metrics = QueryMetrics::new();
    let total_start = Instant::now();

    // Step 1: Parse query
    let parse_start = Instant::now();
    let query = parse(sql)?;
    metrics.parse_time_us = parse_start.elapsed().as_micros() as u64;

    // Step 2: Validate query type (must be SELECT)
    if query.table.is_empty() {
        return Err(ReedError::ParseError {
            reason: "Missing table name".to_string(),
        });
    }

    // Step 3: Load table data
    let table_ref = db.get_table(&query.table)?;
    let content = table_ref.read_current()?;
    let text = std::str::from_utf8(&content).map_err(|e| ReedError::ParseError {
        reason: format!("Invalid UTF-8: {}", e),
    })?;

    // Parse CSV manually to get HashMap format
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Err(ReedError::ParseError {
            reason: "Empty table".to_string(),
        });
    }

    let header_line = lines[0];
    let header_parts: Vec<&str> = header_line.split('|').collect();

    let mut table_data = Vec::new();
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        let mut row_map = std::collections::HashMap::new();
        for (col_idx, col_name) in header_parts.iter().enumerate() {
            if let Some(&value) = parts.get(col_idx) {
                row_map.insert(col_name.to_string(), value.to_string());
            }
        }
        table_data.push(row_map);
    }

    metrics.rows_scanned = table_data.len();

    // Step 5: Track query pattern for auto-indexing
    track_query_pattern(db, &query);

    // Step 6: Execute query (with optimization if indices available)
    let exec_start = Instant::now();
    let result = if db.indices().read().unwrap().is_empty() {
        // No indices available - use basic executor
        execute(&query, &table_data)?
    } else {
        // Use optimized executor with indices
        let indices = db.indices().read().unwrap();
        let index_list: Vec<(String, Box<dyn crate::indices::Index<String, Vec<usize>>>)> =
            Vec::new(); // TODO: Convert Arc<RwLock<HashMap>> to Vec

        let executor = OptimizedExecutor::new(index_list);
        executor.execute_optimized(&query, &table_data)?
    };

    metrics.execution_time_us = exec_start.elapsed().as_micros() as u64;
    metrics.rows_returned = result.row_count();

    // Step 7: Update statistics
    let mut stats = db.stats_mut().write().unwrap();
    stats.query_count += 1;

    // Update average query time
    let total_queries = stats.query_count as u64;
    let new_avg =
        ((stats.avg_query_time_us * (total_queries - 1)) + metrics.total_time_us()) / total_queries;
    stats.avg_query_time_us = new_avg;

    Ok(result)
}

/// Tracks query pattern for auto-indexing.
fn track_query_pattern(db: &Database, query: &crate::reedql::types::ParsedQuery) {
    if !db.auto_index_config().enabled {
        return;
    }

    let mut tracker = db.pattern_tracker().write().unwrap();

    // Track each condition
    for condition in &query.conditions {
        let (column, operation) = match condition {
            crate::reedql::types::FilterCondition::Equals { column, .. } => {
                (column.clone(), "equals".to_string())
            }
            crate::reedql::types::FilterCondition::LessThan { column, .. }
            | crate::reedql::types::FilterCondition::GreaterThan { column, .. }
            | crate::reedql::types::FilterCondition::LessThanOrEqual { column, .. }
            | crate::reedql::types::FilterCondition::GreaterThanOrEqual { column, .. } => {
                (column.clone(), "range".to_string())
            }
            crate::reedql::types::FilterCondition::Like { column, .. } => {
                (column.clone(), "like".to_string())
            }
            crate::reedql::types::FilterCondition::InList { column, .. }
            | crate::reedql::types::FilterCondition::InSubquery { column, .. } => {
                (column.clone(), "in".to_string())
            }
            _ => continue,
        };

        let pattern = QueryPattern::new(query.table.clone(), column.clone(), operation.clone());
        let count = tracker.record(pattern.clone());

        // Check if should create index
        let threshold = db.auto_index_config().threshold;
        if tracker.should_create_index(&pattern, threshold) {
            tracker.mark_indexed(pattern.clone());

            // Drop lock before creating index (create_index needs write access)
            drop(tracker);

            // Attempt to create index with auto_created flag (ignore errors - best effort)
            let _ = crate::database::index::create_index_internal(db, &query.table, &column, true);

            // Don't continue tracking - index created, we're done
            return;
        }
    }
}

/// Query result formatter for different output formats.
pub struct QueryResultFormatter;

impl QueryResultFormatter {
    /// Formats result as human-readable table.
    ///
    /// ## Example Output
    /// ```text
    /// +----------------------+---------------+
    /// | key                  | value         |
    /// +----------------------+---------------+
    /// | page.title@de        | Willkommen    |
    /// | page.title@en        | Welcome       |
    /// +----------------------+---------------+
    /// 2 rows
    /// ```
    pub fn format_table(result: &QueryResult) -> String {
        match result {
            QueryResult::Rows(rows) => {
                if rows.is_empty() {
                    return "0 rows".to_string();
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
    ///
    /// ## Example Output
    /// ```json
    /// [
    ///   {"key": "page.title@de", "value": "Willkommen"},
    ///   {"key": "page.title@en", "value": "Welcome"}
    /// ]
    /// ```
    pub fn format_json(result: &QueryResult) -> String {
        match result {
            QueryResult::Rows(rows) => {
                if rows.is_empty() {
                    return "[]".to_string();
                }

                let mut output = String::from("[\n");
                for (i, row) in rows.iter().enumerate() {
                    output.push_str("  {");

                    let mut keys: Vec<_> = row.keys().collect();
                    keys.sort();

                    for (j, key) in keys.iter().enumerate() {
                        let value = row.get(*key).map(|s| s.as_str()).unwrap_or("");
                        output.push_str(&format!("\"{}\": \"{}\"", key, value));
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
    ///
    /// ## Example Output
    /// ```text
    /// key,value
    /// page.title@de,Willkommen
    /// page.title@en,Welcome
    /// ```
    pub fn format_csv(result: &QueryResult) -> String {
        match result {
            QueryResult::Rows(rows) => {
                if rows.is_empty() {
                    return "".to_string();
                }

                let mut output = String::new();

                // Header
                let mut columns: Vec<String> = rows[0].keys().cloned().collect();
                columns.sort();
                output.push_str(&columns.join(","));
                output.push('\n');

                // Rows
                for row in rows {
                    let values: Vec<String> = columns
                        .iter()
                        .map(|col| row.get(col).map(|s| s.as_str()).unwrap_or(""))
                        .map(|s| s.to_string())
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_table_empty() {
        let result = QueryResult::Rows(Vec::new());
        let output = QueryResultFormatter::format_table(&result);
        assert_eq!(output, "0 rows");
    }

    #[test]
    fn test_format_json_empty() {
        let result = QueryResult::Rows(Vec::new());
        let output = QueryResultFormatter::format_json(&result);
        assert_eq!(output, "[]");
    }

    #[test]
    fn test_format_csv_empty() {
        let result = QueryResult::Rows(Vec::new());
        let output = QueryResultFormatter::format_csv(&result);
        assert_eq!(output, "");
    }

    #[test]
    fn test_format_aggregation() {
        let result = QueryResult::Aggregation(42.5);
        assert_eq!(QueryResultFormatter::format_table(&result), "42.5\n");
        assert_eq!(QueryResultFormatter::format_json(&result), "42.5\n");
        assert_eq!(QueryResultFormatter::format_csv(&result), "42.5\n");
    }
}
