// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Backup Creation with XZ Compression
//!
//! Provides create_backup() function for creating compressed backups.

use crate::reedcms::reedstream::{io_error, ReedResult};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use xz2::write::XzEncoder;

/// Creates an XZ-compressed backup of a CSV file.
///
/// ## Input
/// - `source_path`: Path to the CSV file to back up
///
/// ## Output
/// - `ReedResult<PathBuf>`: Path to the created backup file or error
///
/// ## Performance
/// - LZMA2 compression with level 6 (balanced)
/// - ~10x compression ratio for CSV files
/// - < 100ms for typical CSV files (< 100KB uncompressed)
///
/// ## Behaviour
/// - Creates `.reed/backups/` directory if it doesn't exist
/// - Backup filename: `{original_name}.{timestamp}.xz`
/// - Timestamp format: ISO 8601 (YYYYMMDD_HHMMSS)
/// - Atomic operation (writes to temp file, then renames)
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if source file cannot be read
/// - Returns `ReedError::IoError` if backup directory cannot be created
/// - Returns `ReedError::IoError` if compression fails
/// - Returns `ReedError::IoError` if backup file cannot be written
///
/// ## Example Usage
/// ```
/// let backup_path = create_backup(".reed/text.csv")?;
/// // backup_path: ".reed/backups/text.csv.20250102_143022.xz"
/// ```
pub fn create_backup<P: AsRef<Path>>(source_path: P) -> ReedResult<PathBuf> {
    let source_ref = source_path.as_ref();
    let source_str = source_ref.to_string_lossy().to_string();

    // Read source file
    let mut source_file =
        File::open(source_ref).map_err(|e| io_error("open", source_str.clone(), e.to_string()))?;

    let mut source_content = Vec::new();
    source_file
        .read_to_end(&mut source_content)
        .map_err(|e| io_error("read", source_str.clone(), e.to_string()))?;

    // Create backup directory
    let backup_dir = source_ref
        .parent()
        .ok_or_else(|| {
            io_error(
                "backup",
                source_str.clone(),
                "No parent directory".to_string(),
            )
        })?
        .join("backups");

    fs::create_dir_all(&backup_dir).map_err(|e| {
        io_error(
            "mkdir",
            backup_dir.to_string_lossy().to_string(),
            e.to_string(),
        )
    })?;

    // Generate backup filename with ISO 8601 timestamp
    let filename = source_ref
        .file_name()
        .ok_or_else(|| io_error("backup", source_str.clone(), "No filename".to_string()))?
        .to_string_lossy();

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_filename = format!("{}.{}.xz", filename, timestamp);
    let backup_path = backup_dir.join(&backup_filename);
    let backup_str = backup_path.to_string_lossy().to_string();

    // Create temp file for atomic write
    let temp_path = backup_dir.join(format!("{}.tmp", backup_filename));
    let temp_str = temp_path.to_string_lossy().to_string();

    // Compress and write to temp file
    let temp_file = File::create(&temp_path)
        .map_err(|e| io_error("create", temp_str.clone(), e.to_string()))?;

    let mut encoder = XzEncoder::new(temp_file, 6); // Level 6: balanced compression
    encoder.write_all(&source_content).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        io_error("compress", temp_str.clone(), e.to_string())
    })?;

    encoder.finish().map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        io_error("compress_finish", temp_str.clone(), e.to_string())
    })?;

    // Atomic rename
    fs::rename(&temp_path, &backup_path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        io_error("rename", backup_str.clone(), e.to_string())
    })?;

    Ok(backup_path)
}
