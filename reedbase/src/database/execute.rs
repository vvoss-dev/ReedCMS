// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Command execution (INSERT/UPDATE/DELETE) via ReedQL.
//!
//! This module handles all data modification operations.

use crate::database::database::Database;
use crate::error::{ReedError, ReedResult};
use std::collections::HashMap;
use std::time::Instant;

/// Execution result for INSERT/UPDATE/DELETE commands.
#[derive(Debug, Clone)]
pub struct ExecuteResult {
    /// Number of rows affected
    pub rows_affected: usize,

    /// Execution time in microseconds
    pub execution_time_us: u64,

    /// Version timestamp created
    pub timestamp: u64,

    /// Delta size in bytes (for versioning)
    pub delta_size: u64,
}

impl ExecuteResult {
    /// Creates a new execution result.
    pub fn new(rows_affected: usize) -> Self {
        Self {
            rows_affected,
            execution_time_us: 0,
            timestamp: 0,
            delta_size: 0,
        }
    }
}

/// Parsed statement types for execution.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecuteStatement {
    /// INSERT INTO table (col1, col2) VALUES (val1, val2)
    Insert {
        table: String,
        columns: Vec<String>,
        values: Vec<String>,
    },

    /// UPDATE table SET col1 = val1, col2 = val2 WHERE condition
    Update {
        table: String,
        assignments: HashMap<String, String>,
        conditions: Vec<FilterCondition>,
    },

    /// DELETE FROM table WHERE condition
    Delete {
        table: String,
        conditions: Vec<FilterCondition>,
    },
}

/// Filter condition (simplified version of ReedQL's FilterCondition).
#[derive(Debug, Clone, PartialEq)]
pub enum FilterCondition {
    Equals { column: String, value: String },
    NotEquals { column: String, value: String },
    Like { column: String, pattern: String },
}

/// Executes a ReedQL command (INSERT/UPDATE/DELETE).
///
/// ## Input
/// - `db`: Database reference
/// - `sql`: ReedQL command string
/// - `user`: Username for audit trail
///
/// ## Output
/// - `Ok(ExecuteResult)`: Execution metadata
/// - `Err(ReedError)`: Execution failed
pub fn execute_command(db: &Database, sql: &str, user: &str) -> ReedResult<ExecuteResult> {
    let start = Instant::now();

    // Parse command
    let statement = parse_execute_statement(sql)?;

    // Execute based on type (using references to avoid move)
    let mut result = match &statement {
        ExecuteStatement::Insert {
            table,
            columns,
            values,
        } => execute_insert(db, table, columns.clone(), values.clone(), user)?,

        ExecuteStatement::Update {
            table,
            assignments,
            conditions,
        } => execute_update(db, table, assignments.clone(), conditions.clone(), user)?,

        ExecuteStatement::Delete { table, conditions } => {
            execute_delete(db, table, conditions.clone(), user)?
        }
    };

    result.execution_time_us = start.elapsed().as_micros() as u64;

    // Update statistics
    let mut stats = db.stats_mut().write().unwrap();
    match statement {
        ExecuteStatement::Insert { .. } => stats.insert_count += 1,
        ExecuteStatement::Update { .. } => stats.update_count += 1,
        ExecuteStatement::Delete { .. } => stats.delete_count += 1,
    }

    Ok(result)
}

/// Parses an execute statement (INSERT/UPDATE/DELETE).
fn parse_execute_statement(sql: &str) -> ReedResult<ExecuteStatement> {
    let sql = sql.trim();

    if sql.to_uppercase().starts_with("INSERT") {
        parse_insert(sql)
    } else if sql.to_uppercase().starts_with("UPDATE") {
        parse_update(sql)
    } else if sql.to_uppercase().starts_with("DELETE") {
        parse_delete(sql)
    } else {
        Err(ReedError::ParseError {
            reason: format!("Unknown statement type: {}", sql),
        })
    }
}

/// Parses INSERT statement.
///
/// Format: INSERT INTO table (col1, col2) VALUES (val1, val2)
fn parse_insert(sql: &str) -> ReedResult<ExecuteStatement> {
    let sql = sql.trim();

    // Extract table name
    let after_into = sql
        .to_uppercase()
        .find("INTO")
        .ok_or_else(|| ReedError::ParseError {
            reason: "Missing INTO keyword".to_string(),
        })?;

    let rest = &sql[after_into + 4..].trim();

    // Find table name (up to opening parenthesis)
    let paren_pos = rest.find('(').ok_or_else(|| ReedError::ParseError {
        reason: "Missing column list".to_string(),
    })?;

    let table = rest[..paren_pos].trim().to_string();

    // Extract columns
    let values_pos = rest
        .to_uppercase()
        .find("VALUES")
        .ok_or_else(|| ReedError::ParseError {
            reason: "Missing VALUES keyword".to_string(),
        })?;

    let columns_str = &rest[paren_pos + 1..values_pos];
    let columns_end = columns_str
        .rfind(')')
        .ok_or_else(|| ReedError::ParseError {
            reason: "Unclosed column list".to_string(),
        })?;

    let columns: Vec<String> = columns_str[..columns_end]
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Extract values
    let values_rest = &rest[values_pos + 6..].trim();
    let values_start = values_rest.find('(').ok_or_else(|| ReedError::ParseError {
        reason: "Missing values list".to_string(),
    })?;
    let values_end = values_rest
        .rfind(')')
        .ok_or_else(|| ReedError::ParseError {
            reason: "Unclosed values list".to_string(),
        })?;

    let values: Vec<String> = values_rest[values_start + 1..values_end]
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            // Remove quotes
            if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
                || (trimmed.starts_with('"') && trimmed.ends_with('"'))
            {
                trimmed[1..trimmed.len() - 1].to_string()
            } else {
                trimmed.to_string()
            }
        })
        .collect();

    if columns.len() != values.len() {
        return Err(ReedError::ParseError {
            reason: format!(
                "Column count ({}) doesn't match value count ({})",
                columns.len(),
                values.len()
            ),
        });
    }

    Ok(ExecuteStatement::Insert {
        table,
        columns,
        values,
    })
}

/// Parses UPDATE statement.
///
/// Format: UPDATE table SET col1 = val1, col2 = val2 WHERE condition
fn parse_update(sql: &str) -> ReedResult<ExecuteStatement> {
    let sql = sql.trim();

    // Extract table name
    let set_pos = sql
        .to_uppercase()
        .find("SET")
        .ok_or_else(|| ReedError::ParseError {
            reason: "Missing SET keyword".to_string(),
        })?;

    let table = sql[6..set_pos].trim().to_string(); // Skip "UPDATE"

    // Extract assignments
    let where_pos = sql.to_uppercase().find("WHERE");
    let assignments_str = if let Some(pos) = where_pos {
        &sql[set_pos + 3..pos]
    } else {
        &sql[set_pos + 3..]
    };

    let mut assignments = HashMap::new();
    for assignment in assignments_str.split(',') {
        let parts: Vec<&str> = assignment.split('=').collect();
        if parts.len() != 2 {
            return Err(ReedError::ParseError {
                reason: format!("Invalid assignment: {}", assignment),
            });
        }

        let column = parts[0].trim().to_string();
        let value = parts[1].trim();
        let value_clean = if (value.starts_with('\'') && value.ends_with('\''))
            || (value.starts_with('"') && value.ends_with('"'))
        {
            value[1..value.len() - 1].to_string()
        } else {
            value.to_string()
        };

        assignments.insert(column, value_clean);
    }

    // Parse WHERE conditions (if present)
    let conditions = if let Some(pos) = where_pos {
        parse_simple_where(&sql[pos + 5..])?
    } else {
        Vec::new()
    };

    Ok(ExecuteStatement::Update {
        table,
        assignments,
        conditions,
    })
}

/// Parses DELETE statement.
///
/// Format: DELETE FROM table WHERE condition
fn parse_delete(sql: &str) -> ReedResult<ExecuteStatement> {
    let sql = sql.trim();

    // Extract table name
    let from_pos = sql
        .to_uppercase()
        .find("FROM")
        .ok_or_else(|| ReedError::ParseError {
            reason: "Missing FROM keyword".to_string(),
        })?;

    let where_pos = sql.to_uppercase().find("WHERE");
    let table = if let Some(pos) = where_pos {
        sql[from_pos + 4..pos].trim().to_string()
    } else {
        sql[from_pos + 4..].trim().to_string()
    };

    // Parse WHERE conditions (if present)
    let conditions = if let Some(pos) = where_pos {
        parse_simple_where(&sql[pos + 5..])?
    } else {
        Vec::new()
    };

    Ok(ExecuteStatement::Delete { table, conditions })
}

/// Parses simple WHERE clause (column = 'value' AND column = 'value').
fn parse_simple_where(where_clause: &str) -> ReedResult<Vec<FilterCondition>> {
    let mut conditions = Vec::new();

    for condition_str in where_clause.split("AND") {
        let condition_str = condition_str.trim();

        if condition_str.contains("!=") {
            let parts: Vec<&str> = condition_str.split("!=").collect();
            if parts.len() == 2 {
                let column = parts[0].trim().to_string();
                let value = clean_value(parts[1].trim());
                conditions.push(FilterCondition::NotEquals { column, value });
            }
        } else if condition_str.to_uppercase().contains("LIKE") {
            let parts: Vec<&str> = condition_str.split_whitespace().collect();
            if parts.len() >= 3 {
                let column = parts[0].to_string();
                let pattern = clean_value(parts[2]);
                conditions.push(FilterCondition::Like { column, pattern });
            }
        } else if condition_str.contains('=') {
            let parts: Vec<&str> = condition_str.split('=').collect();
            if parts.len() == 2 {
                let column = parts[0].trim().to_string();
                let value = clean_value(parts[1].trim());
                conditions.push(FilterCondition::Equals { column, value });
            }
        }
    }

    Ok(conditions)
}

/// Cleans value by removing quotes.
fn clean_value(value: &str) -> String {
    let trimmed = value.trim();
    if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        || (trimmed.starts_with('"') && trimmed.ends_with('"'))
    {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Executes INSERT statement.
fn execute_insert(
    db: &Database,
    table_name: &str,
    columns: Vec<String>,
    values: Vec<String>,
    user: &str,
) -> ReedResult<ExecuteResult> {
    let table = db.get_table(table_name)?;

    // Build new row based on columns
    let key = columns
        .iter()
        .zip(values.iter())
        .find(|(col, _)| col.as_str() == "key")
        .map(|(_, val)| val.clone())
        .unwrap_or_default();

    let row_values: Vec<String> = columns
        .iter()
        .skip(1) // Skip key column
        .zip(values.iter().skip(1))
        .map(|(_, val)| val.clone())
        .collect();

    // Create new row line
    let mut new_row_parts = vec![key];
    new_row_parts.extend(row_values);
    let new_row_line = new_row_parts.join("|");

    // Use atomic read-modify-write to prevent race conditions
    let write_result = table.read_modify_write(
        |content| {
            // Append new row to existing content
            let mut new_content = content.to_vec();
            new_content.extend_from_slice(new_row_line.as_bytes());
            new_content.push(b'\n');
            new_content
        },
        user,
    )?;

    Ok(ExecuteResult {
        rows_affected: 1,
        execution_time_us: 0, // Will be set by caller
        timestamp: write_result.timestamp,
        delta_size: write_result.delta_size,
    })
}

/// Executes UPDATE statement.
fn execute_update(
    db: &Database,
    table_name: &str,
    assignments: HashMap<String, String>,
    conditions: Vec<FilterCondition>,
    user: &str,
) -> ReedResult<ExecuteResult> {
    let table = db.get_table(table_name)?;

    // Read current content
    let content = table.read_current()?;
    let text = std::str::from_utf8(&content).map_err(|e| ReedError::InvalidCsv {
        reason: format!("Invalid UTF-8: {}", e),
        line: 0,
    })?;

    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Err(ReedError::InvalidCsv {
            reason: "Empty table".to_string(),
            line: 0,
        });
    }

    let header_line = lines[0];
    let header_parts: Vec<&str> = header_line.split('|').collect();

    let mut updated = 0;
    let mut new_lines = vec![header_line.to_string()];

    // Process each row
    for (i, line) in lines.iter().skip(1).enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        let mut row_map = HashMap::new();
        for (col_idx, col_name) in header_parts.iter().enumerate() {
            if let Some(value) = parts.get(col_idx) {
                row_map.insert(col_name.to_string(), value.to_string());
            }
        }

        if matches_conditions(&row_map, &conditions) {
            // Apply updates
            for (col, val) in &assignments {
                row_map.insert(col.clone(), val.clone());
            }
            updated += 1;
        }

        // Rebuild row line
        let row_values: Vec<String> = header_parts
            .iter()
            .map(|col| row_map.get(*col).cloned().unwrap_or_default())
            .collect();
        new_lines.push(row_values.join("|"));
    }

    // Write back
    let new_content = new_lines.join("\n") + "\n";
    let write_result = table.write(new_content.as_bytes(), user)?;

    Ok(ExecuteResult {
        rows_affected: updated,
        execution_time_us: 0,
        timestamp: write_result.timestamp,
        delta_size: write_result.delta_size,
    })
}

/// Executes DELETE statement.
fn execute_delete(
    db: &Database,
    table_name: &str,
    conditions: Vec<FilterCondition>,
    user: &str,
) -> ReedResult<ExecuteResult> {
    let table = db.get_table(table_name)?;

    // Read current content
    let content = table.read_current()?;
    let text = std::str::from_utf8(&content).map_err(|e| ReedError::InvalidCsv {
        reason: format!("Invalid UTF-8: {}", e),
        line: 0,
    })?;

    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Err(ReedError::InvalidCsv {
            reason: "Empty table".to_string(),
            line: 0,
        });
    }

    let header_line = lines[0];
    let header_parts: Vec<&str> = header_line.split('|').collect();

    let mut deleted = 0;
    let mut new_lines = vec![header_line.to_string()];

    // Process each row
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        let mut row_map = HashMap::new();
        for (col_idx, col_name) in header_parts.iter().enumerate() {
            if let Some(value) = parts.get(col_idx) {
                row_map.insert(col_name.to_string(), value.to_string());
            }
        }

        if matches_conditions(&row_map, &conditions) {
            // Skip this row (delete it)
            deleted += 1;
        } else {
            // Keep this row
            new_lines.push(line.to_string());
        }
    }

    // Write back
    let new_content = new_lines.join("\n") + "\n";
    let write_result = table.write(new_content.as_bytes(), user)?;

    Ok(ExecuteResult {
        rows_affected: deleted,
        execution_time_us: 0,
        timestamp: write_result.timestamp,
        delta_size: write_result.delta_size,
    })
}

/// Checks if row matches all conditions.
fn matches_conditions(row: &HashMap<String, String>, conditions: &[FilterCondition]) -> bool {
    if conditions.is_empty() {
        return true; // No conditions = match all
    }

    for condition in conditions {
        match condition {
            FilterCondition::Equals { column, value } => {
                if row.get(column) != Some(value) {
                    return false;
                }
            }
            FilterCondition::NotEquals { column, value } => {
                if row.get(column) == Some(value) {
                    return false;
                }
            }
            FilterCondition::Like { column, pattern } => {
                if let Some(val) = row.get(column) {
                    if !matches_like_pattern(val, pattern) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
    }

    true
}

/// Matches SQL LIKE pattern (simplified).
fn matches_like_pattern(value: &str, pattern: &str) -> bool {
    if pattern.ends_with('%') && !pattern[..pattern.len() - 1].contains('%') {
        value.starts_with(&pattern[..pattern.len() - 1])
    } else if pattern.starts_with('%') && !pattern[1..].contains('%') {
        value.ends_with(&pattern[1..])
    } else if pattern.starts_with('%')
        && pattern.ends_with('%')
        && pattern.matches('%').count() == 2
    {
        value.contains(&pattern[1..pattern.len() - 1])
    } else {
        value == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_insert() {
        let sql = "INSERT INTO text (key, value) VALUES ('page.title', 'Welcome')";
        let stmt = parse_insert(sql).unwrap();

        match stmt {
            ExecuteStatement::Insert {
                table,
                columns,
                values,
            } => {
                assert_eq!(table, "text");
                assert_eq!(columns, vec!["key", "value"]);
                assert_eq!(values, vec!["page.title", "Welcome"]);
            }
            _ => panic!("Expected Insert statement"),
        }
    }

    #[test]
    fn test_parse_update() {
        let sql = "UPDATE text SET value = 'Hello' WHERE key = 'page.title'";
        let stmt = parse_update(sql).unwrap();

        match stmt {
            ExecuteStatement::Update {
                table,
                assignments,
                conditions,
            } => {
                assert_eq!(table, "text");
                assert_eq!(assignments.get("value"), Some(&"Hello".to_string()));
                assert_eq!(conditions.len(), 1);
            }
            _ => panic!("Expected Update statement"),
        }
    }

    #[test]
    fn test_parse_delete() {
        let sql = "DELETE FROM text WHERE key = 'page.title'";
        let stmt = parse_delete(sql).unwrap();

        match stmt {
            ExecuteStatement::Delete { table, conditions } => {
                assert_eq!(table, "text");
                assert_eq!(conditions.len(), 1);
            }
            _ => panic!("Expected Delete statement"),
        }
    }

    #[test]
    fn test_matches_like_pattern() {
        assert!(matches_like_pattern("page.title@de", "%.@de"));
        assert!(matches_like_pattern("page.title@de", "page.%"));
        assert!(matches_like_pattern("page.title@de", "%title%"));
        assert!(!matches_like_pattern("page.title@en", "%.@de"));
    }
}
