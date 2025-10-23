// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Aggregation functions for dataset-level operations.
//!
//! Provides count, sum, avg, min, max, and group_by operations with automatic caching.
//! First call scans the CSV file (O(n)), subsequent calls return cached results (<100ns).
//!
//! ## Performance
//!
//! - **First call**: 2-10ms (10k rows, depending on operation)
//! - **Cached calls**: < 100ns (instant)
//! - **Cache invalidation**: Automatic on table writes
//!
//! ## Example Usage
//!
//! ```rust
//! use reedbase::functions::aggregations::{count, sum, avg, min, max};
//!
//! // Count rows
//! let total = count("users")?; // "1250"
//!
//! // Sum column
//! let total_age = sum("users", "age")?; // "43750"
//!
//! // Average
//! let avg_age = avg("users", "age")?; // "35.00"
//!
//! // Min/Max
//! let youngest = min("users", "age")?; // "18"
//! let oldest = max("users", "age")?; // "89"
//! ```

use crate::error::{ReedError, ReedResult};
use crate::functions::cache::{get_cache, CacheKey};
use crate::tables::{parse_csv, Table};
use std::collections::HashMap;
use std::path::Path;

/// Get table path from base directory.
///
/// Assumes tables are in `.reed/tables/{name}/` structure.
fn get_table(table: &str) -> ReedResult<Table> {
    let base_path = Path::new(".reed");
    Ok(Table::new(base_path, table))
}

/// Find column index in CSV rows.
///
/// ## Input
/// - `header` - First row containing column names
/// - `column` - Column name to find
///
/// ## Output
/// - Column index (0-based, excluding key column)
///
/// ## Error Conditions
/// - Column not found → ReedError::InvalidInput
fn get_column_index(header: &[String], column: &str) -> ReedResult<usize> {
    header
        .iter()
        .position(|col| col == column)
        .ok_or_else(|| ReedError::ParseError {
            reason: format!("Column '{}' not found in table", column),
        })
}

/// Count rows in CSV table.
///
/// ## Input
/// - `table` - Table name (e.g., "text", "users", "routes")
///
/// ## Output
/// - Number of rows as string (excluding header)
///
/// ## Performance
/// - First call: 2-5ms (10k rows)
/// - Cached: < 100ns
///
/// ## Error Conditions
/// - Table not found → ReedError::TableNotFound
/// - CSV read error → ReedError::IoError
///
/// ## Example Usage
/// ```rust
/// let count = count("text")?; // "1247"
/// let users = count("users")?; // "5000"
/// ```
pub fn count(table: &str) -> ReedResult<String> {
    let key = CacheKey::new("count", vec![table]);

    // Check cache first
    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    // Read CSV
    let tbl = get_table(table)?;
    let content = tbl.read_current().map_err(|_| ReedError::TableNotFound {
        name: table.to_string(),
    })?;

    let rows = parse_csv(&content)?;

    // Exclude header row (first row)
    let count = if rows.is_empty() { 0 } else { rows.len() - 1 };

    let result = count.to_string();

    // Cache result
    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Sum numeric column values.
///
/// ## Input
/// - `table` - Table name
/// - `column` - Column name to sum (must be numeric)
///
/// ## Output
/// - Sum as string with 2 decimal places
///
/// ## Performance
/// - First call: 5-10ms (10k rows)
/// - Cached: < 100ns
///
/// ## Error Conditions
/// - Table not found → ReedError::TableNotFound
/// - Column not found → ReedError::InvalidInput
/// - Non-numeric values are skipped (not an error)
///
/// ## Example Usage
/// ```rust
/// let total = sum("users", "age")?; // "43750.00"
/// let revenue = sum("sales", "amount")?; // "125840.50"
/// ```
pub fn sum(table: &str, column: &str) -> ReedResult<String> {
    let key = CacheKey::new("sum", vec![table, column]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    // Read CSV
    let tbl = get_table(table)?;
    let content = tbl.read_current().map_err(|_| ReedError::TableNotFound {
        name: table.to_string(),
    })?;

    let rows = parse_csv(&content)?;

    if rows.is_empty() {
        return Ok("0.00".to_string());
    }

    // First row is header (contains column names as values)
    let header = &rows[0].values;
    let col_idx = get_column_index(header, column)?;

    let mut total: f64 = 0.0;

    // Sum numeric values (skip header row)
    for row in &rows[1..] {
        if let Some(value) = row.values.get(col_idx) {
            if let Ok(num) = value.parse::<f64>() {
                total += num;
            }
            // Non-numeric values are silently skipped
        }
    }

    let result = format!("{:.2}", total);

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Calculate average of numeric column.
///
/// ## Input
/// - `table` - Table name
/// - `column` - Column name to average (must be numeric)
///
/// ## Output
/// - Average as string with 2 decimal places
///
/// ## Performance
/// - First call: 5-10ms (10k rows)
/// - Cached: < 100ns
///
/// ## Error Conditions
/// - Table not found → ReedError::TableNotFound
/// - Column not found → ReedError::InvalidInput
/// - No valid numeric values → Returns "0.00"
///
/// ## Example Usage
/// ```rust
/// let avg_age = avg("users", "age")?; // "35.50"
/// let avg_price = avg("products", "price")?; // "29.99"
/// ```
pub fn avg(table: &str, column: &str) -> ReedResult<String> {
    let key = CacheKey::new("avg", vec![table, column]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    // Read CSV
    let tbl = get_table(table)?;
    let content = tbl.read_current().map_err(|_| ReedError::TableNotFound {
        name: table.to_string(),
    })?;

    let rows = parse_csv(&content)?;

    if rows.is_empty() {
        return Ok("0.00".to_string());
    }

    let header = &rows[0].values;
    let col_idx = get_column_index(header, column)?;

    let mut total: f64 = 0.0;
    let mut count: usize = 0;

    // Calculate sum and count (skip header)
    for row in &rows[1..] {
        if let Some(value) = row.values.get(col_idx) {
            if let Ok(num) = value.parse::<f64>() {
                total += num;
                count += 1;
            }
        }
    }

    let average = if count > 0 { total / count as f64 } else { 0.0 };

    let result = format!("{:.2}", average);

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Find minimum value in numeric column.
///
/// ## Input
/// - `table` - Table name
/// - `column` - Column name (must be numeric)
///
/// ## Output
/// - Minimum value as string with 2 decimal places
///
/// ## Performance
/// - First call: 5-10ms (10k rows)
/// - Cached: < 100ns
///
/// ## Error Conditions
/// - Table not found → ReedError::TableNotFound
/// - Column not found → ReedError::InvalidInput
/// - No valid values → Returns "0.00"
///
/// ## Example Usage
/// ```rust
/// let youngest = min("users", "age")?; // "18.00"
/// let cheapest = min("products", "price")?; // "5.99"
/// ```
pub fn min(table: &str, column: &str) -> ReedResult<String> {
    let key = CacheKey::new("min", vec![table, column]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let tbl = get_table(table)?;
    let content = tbl.read_current().map_err(|_| ReedError::TableNotFound {
        name: table.to_string(),
    })?;

    let rows = parse_csv(&content)?;

    if rows.is_empty() {
        return Ok("0.00".to_string());
    }

    let header = &rows[0].values;
    let col_idx = get_column_index(header, column)?;

    let mut min_val: Option<f64> = None;

    for row in &rows[1..] {
        if let Some(value) = row.values.get(col_idx) {
            if let Ok(num) = value.parse::<f64>() {
                min_val = Some(min_val.map_or(num, |current| current.min(num)));
            }
        }
    }

    let result = format!("{:.2}", min_val.unwrap_or(0.0));

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Find maximum value in numeric column.
///
/// ## Input
/// - `table` - Table name
/// - `column` - Column name (must be numeric)
///
/// ## Output
/// - Maximum value as string with 2 decimal places
///
/// ## Performance
/// - First call: 5-10ms (10k rows)
/// - Cached: < 100ns
///
/// ## Error Conditions
/// - Table not found → ReedError::TableNotFound
/// - Column not found → ReedError::InvalidInput
/// - No valid values → Returns "0.00"
///
/// ## Example Usage
/// ```rust
/// let oldest = max("users", "age")?; // "89.00"
/// let expensive = max("products", "price")?; // "999.99"
/// ```
pub fn max(table: &str, column: &str) -> ReedResult<String> {
    let key = CacheKey::new("max", vec![table, column]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let tbl = get_table(table)?;
    let content = tbl.read_current().map_err(|_| ReedError::TableNotFound {
        name: table.to_string(),
    })?;

    let rows = parse_csv(&content)?;

    if rows.is_empty() {
        return Ok("0.00".to_string());
    }

    let header = &rows[0].values;
    let col_idx = get_column_index(header, column)?;

    let mut max_val: Option<f64> = None;

    for row in &rows[1..] {
        if let Some(value) = row.values.get(col_idx) {
            if let Ok(num) = value.parse::<f64>() {
                max_val = Some(max_val.map_or(num, |current| current.max(num)));
            }
        }
    }

    let result = format!("{:.2}", max_val.unwrap_or(0.0));

    get_cache().insert(key, result.clone());

    Ok(result)
}

/// Group by column and count occurrences.
///
/// ## Input
/// - `table` - Table name
/// - `column` - Column name to group by
///
/// ## Output
/// - JSON object as string: `{"value1": count1, "value2": count2, ...}`
///
/// ## Performance
/// - First call: 5-10ms (10k rows)
/// - Cached: < 100ns
///
/// ## Error Conditions
/// - Table not found → ReedError::TableNotFound
/// - Column not found → ReedError::InvalidInput
///
/// ## Example Usage
/// ```rust
/// let status_counts = group_by("users", "status")?;
/// // Output: {"active": 850, "inactive": 120, "pending": 30}
///
/// let lang_counts = group_by("text", "lang")?;
/// // Output: {"en": 1200, "de": 800, "fr": 450}
/// ```
pub fn group_by(table: &str, column: &str) -> ReedResult<String> {
    let key = CacheKey::new("group_by", vec![table, column]);

    if let Some(cached) = get_cache().get(&key) {
        return Ok(cached);
    }

    let tbl = get_table(table)?;
    let content = tbl.read_current().map_err(|_| ReedError::TableNotFound {
        name: table.to_string(),
    })?;

    let rows = parse_csv(&content)?;

    if rows.is_empty() {
        return Ok("{}".to_string());
    }

    let header = &rows[0].values;
    let col_idx = get_column_index(header, column)?;

    let mut counts: HashMap<String, usize> = HashMap::new();

    for row in &rows[1..] {
        if let Some(value) = row.values.get(col_idx) {
            *counts.entry(value.clone()).or_insert(0) += 1;
        }
    }

    let result = serde_json::to_string(&counts).map_err(|e| ReedError::InvalidCsv {
        reason: format!("Failed to serialize group_by result: {}", e),
        line: 0,
    })?;

    get_cache().insert(key, result.clone());

    Ok(result)
}
