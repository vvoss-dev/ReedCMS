// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Write queue for concurrent write coordination.
//!
//! Queues pending writes when table is locked.

use crate::concurrent::types::PendingWrite;
use crate::error::{ReedError, ReedResult};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Queues a write operation.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
/// - `table_name`: Table name
/// - `operation`: Write operation to queue
///
/// ## Output
/// - `ReedResult<String>`: Queue ID (UUID)
///
/// ## Performance
/// - < 5ms typical (write small file)
///
/// ## Error Conditions
/// - QueueFull: Queue has reached maximum size (100 pending)
/// - IoError: Cannot write queue file
///
/// ## Example Usage
/// ```no_run
/// use reedbase::concurrent::{queue_write, types::{PendingWrite, WriteOperation, CsvRow}};
/// use std::path::Path;
///
/// let write = PendingWrite {
///     rows: vec![CsvRow::new("user:1", vec!["Alice"])],
///     timestamp: 1736860900000000000,
///     operation: WriteOperation::Insert,
/// };
/// let queue_id = queue_write(Path::new(".reed"), "users", write)?;
/// println!("Queued with ID: {}", queue_id);
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn queue_write(
    base_path: &Path,
    table_name: &str,
    operation: PendingWrite,
) -> ReedResult<String> {
    let queue_dir = get_queue_dir(base_path, table_name);
    fs::create_dir_all(&queue_dir).map_err(|e| ReedError::IoError {
        operation: "create_queue_dir".to_string(),
        reason: e.to_string(),
    })?;

    // Check queue size
    let queue_size = count_pending(base_path, table_name)?;
    if queue_size >= 100 {
        return Err(ReedError::QueueFull {
            table: table_name.to_string(),
            size: queue_size,
        });
    }

    let queue_id = Uuid::new_v4().to_string();
    let queue_path = queue_dir.join(format!("{}.pending", queue_id));

    let json = serde_json::to_string(&operation).map_err(|e| ReedError::SerializationError {
        reason: format!("Failed to serialise write: {}", e),
    })?;

    fs::write(&queue_path, json).map_err(|e| ReedError::IoError {
        operation: "write_queue_file".to_string(),
        reason: e.to_string(),
    })?;

    Ok(queue_id)
}

/// Gets next pending write from queue.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
/// - `table_name`: Table name
///
/// ## Output
/// - `ReedResult<Option<(String, PendingWrite)>>`: (queue_id, write) or None if empty
///
/// ## Performance
/// - < 10ms typical
///
/// ## Error Conditions
/// - IoError: Cannot read queue directory
/// - DeserializationError: Corrupted queue file
///
/// ## Example Usage
/// ```no_run
/// use reedbase::concurrent::{get_next_pending, remove_from_queue};
/// use std::path::Path;
///
/// while let Some((id, write)) = get_next_pending(Path::new(".reed"), "users")? {
///     // Process write...
///     remove_from_queue(Path::new(".reed"), "users", &id)?;
/// }
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn get_next_pending(
    base_path: &Path,
    table_name: &str,
) -> ReedResult<Option<(String, PendingWrite)>> {
    let queue_dir = get_queue_dir(base_path, table_name);

    if !queue_dir.exists() {
        return Ok(None);
    }

    let mut entries: Vec<_> = fs::read_dir(&queue_dir)
        .map_err(|e| ReedError::IoError {
            operation: "read_queue_dir".to_string(),
            reason: e.to_string(),
        })?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pending"))
        .collect();

    if entries.is_empty() {
        return Ok(None);
    }

    // Sort by creation time (oldest first)
    entries.sort_by_key(|e| e.metadata().ok().and_then(|m| m.created().ok()));

    let entry = &entries[0];
    let queue_id = entry
        .path()
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ReedError::InvalidQueueFile {
            path: entry.path().to_string_lossy().to_string(),
        })?
        .to_string();

    let json = fs::read_to_string(entry.path()).map_err(|e| ReedError::IoError {
        operation: "read_queue_file".to_string(),
        reason: e.to_string(),
    })?;

    let write: PendingWrite =
        serde_json::from_str(&json).map_err(|e| ReedError::DeserializationError {
            reason: format!("Failed to deserialise write: {}", e),
        })?;

    Ok(Some((queue_id, write)))
}

/// Removes write from queue.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
/// - `table_name`: Table name
/// - `queue_id`: Queue ID (UUID)
///
/// ## Output
/// - `ReedResult<()>`: Success or error
///
/// ## Performance
/// - < 5ms typical
///
/// ## Error Conditions
/// - IoError: Cannot delete queue file
///
/// ## Example Usage
/// ```no_run
/// use reedbase::concurrent::remove_from_queue;
/// use std::path::Path;
///
/// remove_from_queue(Path::new(".reed"), "users", "550e8400-e29b-41d4-a716-446655440000")?;
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn remove_from_queue(base_path: &Path, table_name: &str, queue_id: &str) -> ReedResult<()> {
    let queue_path = get_queue_dir(base_path, table_name).join(format!("{}.pending", queue_id));

    fs::remove_file(&queue_path).map_err(|e| ReedError::IoError {
        operation: "remove_queue_file".to_string(),
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Counts pending writes in queue.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
/// - `table_name`: Table name
///
/// ## Output
/// - `ReedResult<usize>`: Number of pending writes
///
/// ## Performance
/// - < 5ms typical
///
/// ## Error Conditions
/// - IoError: Cannot read queue directory
///
/// ## Example Usage
/// ```no_run
/// use reedbase::concurrent::count_pending;
/// use std::path::Path;
///
/// let pending = count_pending(Path::new(".reed"), "users")?;
/// println!("{} writes pending", pending);
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn count_pending(base_path: &Path, table_name: &str) -> ReedResult<usize> {
    let queue_dir = get_queue_dir(base_path, table_name);

    if !queue_dir.exists() {
        return Ok(0);
    }

    let count = fs::read_dir(&queue_dir)
        .map_err(|e| ReedError::IoError {
            operation: "count_queue_dir".to_string(),
            reason: e.to_string(),
        })?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pending"))
        .count();

    Ok(count)
}

/// Gets queue directory path.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
/// - `table_name`: Table name
///
/// ## Output
/// - `PathBuf`: Queue directory path
///
/// ## Performance
/// - O(1) operation
/// - < 1μs
fn get_queue_dir(base_path: &Path, table_name: &str) -> PathBuf {
    base_path.join("tables").join(table_name).join("queue")
}
