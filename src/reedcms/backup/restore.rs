// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Backup Restoration Operations
//!
//! Provides restore_backup() function for restoring from compressed backups.

use crate::reedcms::reedstream::{io_error, ReedResult};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use xz2::read::XzDecoder;

/// Restores a CSV file from an XZ-compressed backup.
///
/// ## Input
/// - `backup_path`: Path to the backup file (.xz)
/// - `destination_path`: Path where to restore the file
///
/// ## Output
/// - `ReedResult<()>`: Success or error
///
/// ## Performance
/// - LZMA2 decompression
/// - < 100ms for typical backup files
///
/// ## Behaviour
/// - Decompresses backup file
/// - Writes to temp file first, then atomic rename
/// - Overwrites existing destination file
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if backup file cannot be read
/// - Returns `ReedError::IoError` if decompression fails
/// - Returns `ReedError::IoError` if destination cannot be written
/// - Cleans up temp file on error
///
/// ## Example Usage
/// ```
/// restore_backup(
///     ".reed/backups/text.csv.20250102_143022.xz",
///     ".reed/text.csv"
/// )?;
/// ```
pub fn restore_backup<P: AsRef<Path>, Q: AsRef<Path>>(
    backup_path: P,
    destination_path: Q,
) -> ReedResult<()> {
    let backup_ref = backup_path.as_ref();
    let backup_str = backup_ref.to_string_lossy().to_string();

    let dest_ref = destination_path.as_ref();
    let dest_str = dest_ref.to_string_lossy().to_string();

    // Open backup file
    let backup_file =
        File::open(backup_ref).map_err(|e| io_error("open", backup_str.clone(), e.to_string()))?;

    // Decompress
    let mut decoder = XzDecoder::new(backup_file);
    let mut decompressed_content = Vec::new();
    decoder
        .read_to_end(&mut decompressed_content)
        .map_err(|e| io_error("decompress", backup_str.clone(), e.to_string()))?;

    // Create temp file for atomic write
    let temp_path = format!("{}.tmp", dest_str);
    let temp_path_ref = Path::new(&temp_path);

    // Write decompressed content to temp file
    fs::write(temp_path_ref, &decompressed_content)
        .map_err(|e| io_error("write", temp_path.clone(), e.to_string()))?;

    // Atomic rename
    fs::rename(temp_path_ref, dest_ref).map_err(|e| {
        let _ = fs::remove_file(temp_path_ref);
        io_error("rename", dest_str.clone(), e.to_string())
    })?;

    Ok(())
}
