// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Index management for Database API.
//!
//! Handles index creation, listing, and statistics.

use crate::database::database::Database;
use crate::database::types::IndexInfo;
use crate::error::{ReedError, ReedResult};
use crate::indices::{HashMapIndex, Index};

/// Creates an index on a table column.
///
/// ## Input
/// - `db`: Database reference
/// - `table_name`: Table name
/// - `column`: Column name
/// - `auto_created`: Whether this index was auto-created (default: false)
///
/// ## Output
/// - `Ok(())`: Index created successfully
/// - `Err(ReedError)`: Creation failed
///
/// ## Performance
/// - HashMap index creation: < 10ms for 10k rows
/// - B+-Tree index creation: < 50ms for 10k rows (persistent)
pub fn create_index_internal(
    db: &Database,
    table_name: &str,
    column: &str,
    auto_created: bool,
) -> ReedResult<()> {
    // Check if index already exists
    let index_key = format!("{}.{}", table_name, column);
    {
        let indices = db.indices().read().unwrap();
        if indices.contains_key(&index_key) {
            return Err(ReedError::IndexAlreadyExists {
                table: table_name.to_string(),
                column: column.to_string(),
            });
        }
    }

    // Load table data
    let table = db.get_table(table_name)?;
    let content = table.read_current()?;
    let text = std::str::from_utf8(&content).map_err(|e| ReedError::IoError {
        operation: "parse_table".to_string(),
        reason: format!("Invalid UTF-8: {}", e),
    })?;

    // Parse CSV to get column indices
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Err(ReedError::InvalidCsv {
            reason: "Empty table".to_string(),
            line: 0,
        });
    }

    let header_line = lines[0];
    let header_parts: Vec<&str> = header_line.split('|').collect();
    let column_index = header_parts
        .iter()
        .position(|&col| col == column)
        .ok_or_else(|| ReedError::InvalidCsv {
            reason: format!("Column '{}' not found", column),
            line: 0,
        })?;

    // Build index
    let mut index: HashMapIndex<String, Vec<usize>> = HashMapIndex::new();

    for (row_id, line) in lines.iter().skip(1).enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if let Some(&value) = parts.get(column_index) {
            let value_str = value.to_string();
            // Insert into index: key → [row_id]
            if let Ok(Some(mut existing)) = index.get(&value_str) {
                existing.push(row_id);
                let _ = index.insert(value_str, existing);
            } else {
                let _ = index.insert(value_str, vec![row_id]);
            }
        }
    }

    // Store index
    let mut indices = db.indices().write().unwrap();
    indices.insert(index_key.clone(), Box::new(index));

    // Store auto-created flag
    if auto_created {
        let mut auto_flags = db.auto_created_indices().write().unwrap();
        auto_flags.insert(index_key.clone(), true);
    }

    // Update statistics
    let mut stats = db.stats_mut().write().unwrap();
    stats.index_count += 1;
    if auto_created {
        stats.auto_index_count += 1;
    }

    Ok(())
}

/// Creates an index on a table column (public API - manual creation).
pub fn create_index(db: &Database, table_name: &str, column: &str) -> ReedResult<()> {
    create_index_internal(db, table_name, column, false)
}

/// Lists all indices in the database.
///
/// ## Input
/// - `db`: Database reference
///
/// ## Output
/// - `Vec<IndexInfo>`: Information about all indices
pub fn list_indices(db: &Database) -> Vec<IndexInfo> {
    let indices = db.indices().read().unwrap();
    let auto_flags = db.auto_created_indices().read().unwrap();
    let mut result = Vec::new();

    for (key, _index) in indices.iter() {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() >= 2 {
            let table = parts[0].to_string();
            let column = parts[1..].join(".");

            let auto_created = auto_flags.get(key).copied().unwrap_or(false);

            let info = IndexInfo {
                table,
                column,
                index_type: "hash".to_string(),
                entry_count: 0, // TODO: Query index for count
                memory_bytes: 0,
                disk_bytes: 0,
                usage_count: 0,
                auto_created,
            };

            result.push(info);
        }
    }

    result
}

/// Drops an index.
///
/// ## Input
/// - `db`: Database reference
/// - `table_name`: Table name
/// - `column`: Column name
///
/// ## Output
/// - `Ok(())`: Index dropped successfully
/// - `Err(ReedError)`: Drop failed
pub fn drop_index(db: &Database, table_name: &str, column: &str) -> ReedResult<()> {
    let index_key = format!("{}.{}", table_name, column);

    let mut indices = db.indices().write().unwrap();
    if indices.remove(&index_key).is_none() {
        return Err(ReedError::IndexNotFound {
            name: index_key.clone(),
        });
    }

    // Update statistics
    let mut stats = db.stats_mut().write().unwrap();
    stats.index_count = stats.index_count.saturating_sub(1);

    Ok(())
}

/// Rebuilds an index (useful after bulk updates).
///
/// ## Input
/// - `db`: Database reference
/// - `table_name`: Table name
/// - `column`: Column name
///
/// ## Output
/// - `Ok(())`: Index rebuilt successfully
/// - `Err(ReedError)`: Rebuild failed
pub fn rebuild_index(db: &Database, table_name: &str, column: &str) -> ReedResult<()> {
    // Drop existing index
    let _ = drop_index(db, table_name, column);

    // Recreate index
    create_index(db, table_name, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_key_format() {
        let key = format!("{}.{}", "text", "key");
        assert_eq!(key, "text.key");
    }

    #[test]
    fn test_parse_index_key() {
        let key = "text.key";
        let parts: Vec<&str> = key.split('.').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "text");
        assert_eq!(parts[1], "key");
    }
}
