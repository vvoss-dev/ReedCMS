// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Index management for Database API.
//!
//! Handles index creation, listing, and statistics.

use crate::btree::Order;
use crate::database::database::Database;
use crate::database::types::{IndexBackend, IndexInfo, IndexMetadata};
use crate::error::{ReedError, ReedResult};
use crate::indices::{BTreeIndex, HashMapIndex, Index};

/// Creates an index on a table column with specified backend.
///
/// ## Input
/// - `db`: Database reference
/// - `table_name`: Table name
/// - `column`: Column name
/// - `backend`: Index backend (Hash or BTree)
/// - `auto_created`: Whether this index was auto-created (default: false)
///
/// ## Output
/// - `Ok(())`: Index created successfully
/// - `Err(ReedError)`: Creation failed
///
/// ## Performance
/// - HashMap: < 10ms for 10k rows (in-memory)
/// - B+-Tree: < 50ms for 10k rows (persistent to disk)
pub fn create_index_with_backend(
    db: &Database,
    table_name: &str,
    column: &str,
    backend: IndexBackend,
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

    // Build index based on backend type
    let index: Box<dyn Index<String, Vec<usize>>> = match backend {
        IndexBackend::Hash => {
            // Build HashMap index (in-memory)
            let mut hash_index: HashMapIndex<String, Vec<usize>> = HashMapIndex::new();

            for (row_id, line) in lines.iter().skip(1).enumerate() {
                if line.trim().is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split('|').collect();
                if let Some(&value) = parts.get(column_index) {
                    let value_str = value.to_string();
                    if let Ok(Some(mut existing)) = hash_index.get(&value_str) {
                        existing.push(row_id);
                        let _ = hash_index.insert(value_str, existing);
                    } else {
                        let _ = hash_index.insert(value_str, vec![row_id]);
                    }
                }
            }

            Box::new(hash_index)
        }

        IndexBackend::BTree => {
            // Build B+-Tree index (persistent)
            let indices_dir = db.base_path().join("indices");
            std::fs::create_dir_all(&indices_dir).map_err(|e| ReedError::IoError {
                operation: "create_indices_dir".to_string(),
                reason: e.to_string(),
            })?;

            let index_path = indices_dir.join(format!("{}.btree", index_key));

            // Create B+-Tree with order 100 (optimal for most cases)
            let order = Order::new(100).map_err(|e| ReedError::IoError {
                operation: "create_btree_order".to_string(),
                reason: format!("Invalid order: {}", e),
            })?;

            let mut btree_index: BTreeIndex<String, Vec<usize>> =
                BTreeIndex::open(&index_path, order)?;

            for (row_id, line) in lines.iter().skip(1).enumerate() {
                if line.trim().is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split('|').collect();
                if let Some(&value) = parts.get(column_index) {
                    let value_str = value.to_string();
                    if let Ok(Some(mut existing)) = btree_index.get(&value_str) {
                        existing.push(row_id);
                        btree_index.insert(value_str, existing)?;
                    } else {
                        btree_index.insert(value_str, vec![row_id])?;
                    }
                }
            }

            Box::new(btree_index)
        }
    };

    // Store index
    let mut indices = db.indices().write().unwrap();
    indices.insert(index_key.clone(), index);

    // Store auto-created flag
    if auto_created {
        let mut auto_flags = db.auto_created_indices().write().unwrap();
        auto_flags.insert(index_key.clone(), true);
    }

    // Save metadata
    let mut metadata = IndexMetadata::new(table_name.to_string(), column.to_string(), backend);
    metadata.auto_created = auto_created;
    save_index_metadata(db, metadata)?;

    // Update statistics
    let mut stats = db.stats_mut().write().unwrap();
    stats.index_count += 1;
    if auto_created {
        stats.auto_index_count += 1;
    }

    Ok(())
}

/// Determines optimal index backend based on query pattern.
///
/// ## Input
/// - `operation`: Query operation type ("equals", "range", "prefix", etc.)
///
/// ## Output
/// - Optimal backend for the operation
///
/// ## Rules
/// - Exact match (=) → Hash (O(1) faster for point lookups)
/// - Range (<, >, <=, >=) → BTree (only backend supporting ranges)
/// - Prefix (LIKE 'foo%') → BTree (efficient prefix scans)
/// - Unknown → BTree (persistent, supports all operations)
pub fn select_backend_for_operation(operation: &str) -> IndexBackend {
    IndexBackend::for_operation(operation)
}

/// Creates an index with smart backend selection based on query pattern.
///
/// ## Input
/// - `db`: Database reference
/// - `table_name`: Table name
/// - `column`: Column name
/// - `operation`: Query operation that triggered creation ("equals", "range", etc.)
/// - `auto_created`: Whether this index was auto-created
///
/// ## Output
/// - `Ok(())`: Index created successfully
/// - `Err(ReedError)`: Creation failed
pub fn create_index_with_smart_selection(
    db: &Database,
    table_name: &str,
    column: &str,
    operation: &str,
    auto_created: bool,
) -> ReedResult<()> {
    let backend = select_backend_for_operation(operation);
    create_index_with_backend(db, table_name, column, backend, auto_created)
}

/// Creates an index on a table column (internal - delegates to create_index_with_backend).
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
    // Default to B+-Tree for new indices (persistent, better for most use cases)
    create_index_with_backend(db, table_name, column, IndexBackend::BTree, auto_created)
}

/// Legacy implementation (kept for reference, no longer used).
#[allow(dead_code)]
fn create_index_internal_legacy(
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
///
/// ## Features
/// - Shows backend type (hash or btree)
/// - Reports memory and disk usage
/// - Includes usage count from metadata
pub fn list_indices(db: &Database) -> Vec<IndexInfo> {
    let indices = db.indices().read().unwrap();
    let auto_flags = db.auto_created_indices().read().unwrap();

    // Load metadata for usage tracking
    let metadata_map: std::collections::HashMap<String, IndexMetadata> = load_index_metadata(db)
        .ok()
        .unwrap_or_default()
        .into_iter()
        .map(|m| (m.index_key(), m))
        .collect();

    let mut result = Vec::new();

    for (key, index) in indices.iter() {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() >= 2 {
            let table = parts[0].to_string();
            let column = parts[1..].join(".");

            // Get backend type from index trait
            let backend_name = index.backend_type();
            let backend = if backend_name == "btree" {
                IndexBackend::BTree
            } else {
                IndexBackend::Hash
            };

            let auto_created = auto_flags.get(key).copied().unwrap_or(false);

            // Get memory and disk usage from index
            let memory_bytes = index.memory_usage();
            let disk_bytes = index.disk_usage();

            // Get usage count from metadata
            let usage_count = metadata_map.get(key).map(|m| m.usage_count).unwrap_or(0);

            let mut info = IndexInfo::new(table, column, backend_name.to_string(), backend);
            info.auto_created = auto_created;
            info.memory_bytes = memory_bytes;
            info.disk_bytes = disk_bytes;
            info.usage_count = usage_count;
            // entry_count would require iterating the index - skip for performance

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

/// Saves index metadata to .reed/indices/metadata.json
///
/// ## Input
/// - `db`: Database reference
/// - `metadata`: Metadata to save
///
/// ## Output
/// - `Ok(())`: Metadata saved successfully
/// - `Err(ReedError)`: Save failed
fn save_index_metadata(db: &Database, metadata: IndexMetadata) -> ReedResult<()> {
    let indices_dir = db.base_path().join("indices");
    std::fs::create_dir_all(&indices_dir).map_err(|e| ReedError::IoError {
        operation: "create_indices_dir".to_string(),
        reason: e.to_string(),
    })?;

    let metadata_path = indices_dir.join("metadata.json");

    // Load existing metadata
    let mut all_metadata: Vec<IndexMetadata> = if metadata_path.exists() {
        let content = std::fs::read_to_string(&metadata_path).map_err(|e| ReedError::IoError {
            operation: "read_metadata".to_string(),
            reason: e.to_string(),
        })?;

        serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };

    // Update or add metadata
    let index_key = metadata.index_key();
    if let Some(existing) = all_metadata.iter_mut().find(|m| m.index_key() == index_key) {
        *existing = metadata;
    } else {
        all_metadata.push(metadata);
    }

    // Save back to file
    let json = serde_json::to_string_pretty(&all_metadata).map_err(|e| ReedError::IoError {
        operation: "serialize_metadata".to_string(),
        reason: e.to_string(),
    })?;

    std::fs::write(&metadata_path, json).map_err(|e| ReedError::IoError {
        operation: "write_metadata".to_string(),
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Loads all index metadata from .reed/indices/metadata.json
///
/// ## Input
/// - `db`: Database reference
///
/// ## Output
/// - `Ok(Vec<IndexMetadata>)`: All metadata entries
/// - `Err(ReedError)`: Load failed
pub fn load_index_metadata(db: &Database) -> ReedResult<Vec<IndexMetadata>> {
    let metadata_path = db.base_path().join("indices").join("metadata.json");

    if !metadata_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&metadata_path).map_err(|e| ReedError::IoError {
        operation: "read_metadata".to_string(),
        reason: e.to_string(),
    })?;

    let metadata: Vec<IndexMetadata> =
        serde_json::from_str(&content).map_err(|e| ReedError::IoError {
            operation: "parse_metadata".to_string(),
            reason: e.to_string(),
        })?;

    Ok(metadata)
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
