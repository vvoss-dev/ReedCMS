// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Conflict resolution logic for ReedBase.
//!
//! This module provides functions for resolving conflicts during CSV merge operations,
//! including automatic resolution strategies and manual conflict file management.

use crate::concurrent::types::CsvRow;
use crate::conflict::types::{ConflictFile, Resolution, ResolutionStrategy};
use crate::error::{ReedError, ReedResult};
use std::fs;
use std::path::{Path, PathBuf};
use toml;

/// Resolve a conflict using the specified strategy.
///
/// ## Input
/// - `base_path`: Base directory (e.g., `.reed`)
/// - `table_name`: Table name (e.g., `text`, `routes`)
/// - `key`: Conflicting key
/// - `base`: Base version (optional)
/// - `change_a`: First change
/// - `change_b`: Second change
/// - `strategy`: Resolution strategy
///
/// ## Output
/// - `Ok(Resolution)`: Resolution result (Automatic, Manual, or KeepBoth)
/// - `Err(ReedError)`: If resolution fails
///
/// ## Performance
/// - LastWriteWins/FirstWriteWins: < 1ms (O(1) operation)
/// - Manual: < 20ms (includes file I/O)
/// - KeepBoth: < 5ms (creates two rows)
///
/// ## Error Conditions
/// - `IoError`: Failed to write manual conflict file
/// - `SerializationError`: Failed to serialize conflict to TOML
///
/// ## Example Usage
/// ```rust
/// let resolution = resolve_conflict(
///     Path::new(".reed"),
///     "text",
///     "test.key",
///     Some(base_row),
///     change_a,
///     change_b,
///     ResolutionStrategy::LastWriteWins,
/// )?;
/// ```
pub fn resolve_conflict(
    base_path: &Path,
    table_name: &str,
    key: &str,
    base: Option<CsvRow>,
    change_a: CsvRow,
    change_b: CsvRow,
    strategy: ResolutionStrategy,
) -> ReedResult<Resolution> {
    match strategy {
        ResolutionStrategy::LastWriteWins => {
            // Accept the newer change (change_b)
            Ok(Resolution::Automatic(change_b))
        }
        ResolutionStrategy::FirstWriteWins => {
            // Keep the earlier change (change_a)
            Ok(Resolution::Automatic(change_a))
        }
        ResolutionStrategy::Manual => {
            // Write conflict to file for human resolution
            let filepath = write_conflict_file(
                base_path, table_name, key, base, change_a, change_b, strategy,
            )?;
            Ok(Resolution::Manual(filepath))
        }
        ResolutionStrategy::KeepBoth => {
            // Create two separate rows with suffixes
            let mut row_a = change_a;
            let mut row_b = change_b;
            row_a.key = format!("{}-a", key);
            row_b.key = format!("{}-b", key);
            Ok(Resolution::KeepBoth(row_a, row_b))
        }
    }
}

/// Write a conflict to a TOML file for manual resolution.
///
/// ## Input
/// - `base_path`: Base directory (e.g., `.reed`)
/// - `table_name`: Table name (e.g., `text`, `routes`)
/// - `key`: Conflicting key
/// - `base`: Base version (optional)
/// - `change_a`: First change
/// - `change_b`: Second change
/// - `strategy`: Resolution strategy (for metadata)
///
/// ## Output
/// - `Ok(String)`: Path to the created conflict file
/// - `Err(ReedError)`: If file creation fails
///
/// ## Performance
/// - < 20ms typical (includes directory creation and TOML serialization)
///
/// ## Error Conditions
/// - `IoError`: Failed to create directory or write file
/// - `SerializationError`: Failed to serialize conflict to TOML
///
/// ## Example Usage
/// ```rust
/// let filepath = write_conflict_file(
///     Path::new(".reed"),
///     "text",
///     "test.key",
///     Some(base_row),
///     change_a,
///     change_b,
///     ResolutionStrategy::Manual,
/// )?;
/// ```
pub fn write_conflict_file(
    base_path: &Path,
    table_name: &str,
    key: &str,
    base: Option<CsvRow>,
    change_a: CsvRow,
    change_b: CsvRow,
    strategy: ResolutionStrategy,
) -> ReedResult<String> {
    // Create conflict directory if it doesn't exist
    let conflict_dir = base_path.join("tables").join(table_name).join("conflicts");
    fs::create_dir_all(&conflict_dir).map_err(|e| ReedError::IoError {
        operation: format!("create conflict directory '{}'", conflict_dir.display()),
        reason: e.to_string(),
    })?;

    // Create conflict file structure
    let conflict = ConflictFile::new(
        table_name.to_string(),
        key.to_string(),
        base,
        change_a,
        change_b,
        strategy,
    );

    // Serialize to TOML
    let toml_string =
        toml::to_string_pretty(&conflict).map_err(|e| ReedError::SerializationError {
            reason: format!("Failed to serialize conflict to TOML: {}", e),
        })?;

    // Write to file
    let filepath = conflict_dir.join(conflict.filename());
    fs::write(&filepath, toml_string).map_err(|e| ReedError::IoError {
        operation: format!("write conflict file '{}'", filepath.display()),
        reason: e.to_string(),
    })?;

    Ok(filepath.display().to_string())
}

/// Load a conflict file from disk.
///
/// ## Input
/// - `filepath`: Path to the conflict file
///
/// ## Output
/// - `Ok(ConflictFile)`: Parsed conflict file
/// - `Err(ReedError)`: If file reading or parsing fails
///
/// ## Performance
/// - < 10ms typical (includes file I/O and TOML parsing)
///
/// ## Error Conditions
/// - `IoError`: Failed to read file
/// - `DeserializationError`: Failed to parse TOML
///
/// ## Example Usage
/// ```rust
/// let conflict = load_conflict_file(Path::new(".reed/tables/text/conflicts/1234567890-test.key.conflict"))?;
/// ```
pub fn load_conflict_file(filepath: &Path) -> ReedResult<ConflictFile> {
    // Read file
    let toml_string = fs::read_to_string(filepath).map_err(|e| ReedError::IoError {
        operation: format!("read conflict file '{}'", filepath.display()),
        reason: e.to_string(),
    })?;

    // Parse TOML
    let conflict: ConflictFile =
        toml::from_str(&toml_string).map_err(|e| ReedError::DeserializationError {
            reason: format!("Failed to parse conflict TOML: {}", e),
        })?;

    Ok(conflict)
}

/// List all conflict files for a table.
///
/// ## Input
/// - `base_path`: Base directory (e.g., `.reed`)
/// - `table_name`: Table name (e.g., `text`, `routes`)
///
/// ## Output
/// - `Ok(Vec<PathBuf>)`: List of conflict file paths
/// - `Err(ReedError)`: If directory reading fails
///
/// ## Performance
/// - < 5ms for typical directory sizes (< 100 conflicts)
///
/// ## Error Conditions
/// - `IoError`: Failed to read directory
///
/// ## Example Usage
/// ```rust
/// let conflicts = list_conflicts(Path::new(".reed"), "text")?;
/// for conflict_path in conflicts {
///     println!("{}", conflict_path.display());
/// }
/// ```
pub fn list_conflicts(base_path: &Path, table_name: &str) -> ReedResult<Vec<PathBuf>> {
    let conflict_dir = base_path.join("tables").join(table_name).join("conflicts");

    // Return empty list if directory doesn't exist
    if !conflict_dir.exists() {
        return Ok(vec![]);
    }

    // Read directory
    let entries = fs::read_dir(&conflict_dir).map_err(|e| ReedError::IoError {
        operation: format!("read conflict directory '{}'", conflict_dir.display()),
        reason: e.to_string(),
    })?;

    // Filter for .conflict files
    let mut conflicts = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| ReedError::IoError {
            operation: format!("read directory entry in '{}'", conflict_dir.display()),
            reason: e.to_string(),
        })?;

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("conflict") {
            conflicts.push(path);
        }
    }

    // Sort by filename (timestamp-based)
    conflicts.sort();

    Ok(conflicts)
}

/// Delete a conflict file.
///
/// ## Input
/// - `filepath`: Path to the conflict file
///
/// ## Output
/// - `Ok(())`: File deleted successfully
/// - `Err(ReedError)`: If deletion fails
///
/// ## Performance
/// - < 5ms typical
///
/// ## Error Conditions
/// - `IoError`: Failed to delete file
///
/// ## Example Usage
/// ```rust
/// delete_conflict_file(Path::new(".reed/tables/text/conflicts/1234567890-test.key.conflict"))?;
/// ```
pub fn delete_conflict_file(filepath: &Path) -> ReedResult<()> {
    fs::remove_file(filepath).map_err(|e| ReedError::IoError {
        operation: format!("delete conflict file '{}'", filepath.display()),
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Count the number of conflicts for a table.
///
/// ## Input
/// - `base_path`: Base directory (e.g., `.reed`)
/// - `table_name`: Table name (e.g., `text`, `routes`)
///
/// ## Output
/// - `Ok(usize)`: Number of conflict files
/// - `Err(ReedError)`: If directory reading fails
///
/// ## Performance
/// - < 5ms for typical directory sizes
///
/// ## Example Usage
/// ```rust
/// let count = count_conflicts(Path::new(".reed"), "text")?;
/// println!("Found {} conflicts", count);
/// ```
pub fn count_conflicts(base_path: &Path, table_name: &str) -> ReedResult<usize> {
    let conflicts = list_conflicts(base_path, table_name)?;
    Ok(conflicts.len())
}
