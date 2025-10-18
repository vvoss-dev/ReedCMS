// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Log validation and crash recovery.
//!
//! Provides CRC32 validation and automatic truncation of corrupted entries.

use crate::error::{ReedError, ReedResult};
use crate::log::decoder::decode_log_entry;
use crate::log::types::ValidationReport;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Validate log file and return detailed report.
///
/// ## Input
/// - `log_path`: Path to version.log file
///
/// ## Output
/// - `ReedResult<ValidationReport>`: Validation report with corruption details
///
/// ## Performance
/// - < 1ms per entry
/// - < 50ms for 1000 entries
///
/// ## Error Conditions
/// - IoError: Cannot read log file
///
/// ## Example Usage
/// ```no_run
/// use reedbase::log::validate_log;
/// use std::path::Path;
///
/// let report = validate_log(Path::new(".reedbase/tables/text/version.log"))?;
/// if !report.is_healthy() {
///     println!("Found {} corrupted entries", report.corrupted_count);
/// }
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn validate_log(log_path: &Path) -> ReedResult<ValidationReport> {
    let mut report = ValidationReport::new();

    // Check if log exists
    if !log_path.exists() {
        return Ok(report); // Empty log is valid
    }

    let content = fs::read_to_string(log_path).map_err(|e| ReedError::IoError {
        operation: "read_log".to_string(),
        reason: e.to_string(),
    })?;

    for (line_num, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        report.total_entries += 1;

        match decode_log_entry(line) {
            Ok(_) => {
                report.valid_entries += 1;
            }
            Err(ReedError::CorruptedLogEntry { .. }) | Err(ReedError::ParseError { .. }) => {
                report.corrupted_count += 1;
                report.corrupted_lines.push(line_num + 1);
            }
            Err(e) => {
                // Other errors (like UnknownActionCode) are also treated as corruption
                report.corrupted_count += 1;
                report.corrupted_lines.push(line_num + 1);
                eprintln!("Warning: Line {}: {}", line_num + 1, e);
            }
        }
    }

    Ok(report)
}

/// Validate log and truncate corrupted entries (crash recovery).
///
/// ## Input
/// - `log_path`: Path to version.log file
///
/// ## Output
/// - `ReedResult<ValidationReport>`: Validation report (truncated = true if modified)
///
/// ## Performance
/// - < 1ms per entry validation
/// - < 10ms for truncation write
///
/// ## Error Conditions
/// - IoError: Cannot read or write log file
///
/// ## Example Usage
/// ```no_run
/// use reedbase::log::validate_and_truncate_log;
/// use std::path::Path;
///
/// // Call on startup to ensure log consistency
/// let report = validate_and_truncate_log(Path::new(".reedbase/tables/text/version.log"))?;
/// if report.truncated {
///     println!("Removed {} corrupted entries", report.corrupted_count);
/// }
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn validate_and_truncate_log(log_path: &Path) -> ReedResult<ValidationReport> {
    let mut report = validate_log(log_path)?;

    // If no corruption found, return as-is
    if report.corrupted_count == 0 {
        return Ok(report);
    }

    // Find first corrupted line
    let first_corruption = report.corrupted_lines.iter().min().copied().unwrap_or(0);

    if first_corruption == 0 {
        return Ok(report); // Empty or all corrupted
    }

    // Read log content
    let content = fs::read_to_string(log_path).map_err(|e| ReedError::IoError {
        operation: "read_log_for_truncation".to_string(),
        reason: e.to_string(),
    })?;

    // Keep only valid lines before first corruption
    let valid_lines: Vec<&str> = content
        .lines()
        .enumerate()
        .filter(|(idx, line)| {
            let line_num = idx + 1;
            !line.trim().is_empty() && line_num < first_corruption
        })
        .map(|(_, line)| line)
        .collect();

    // Write truncated log
    let truncated_content = valid_lines.join("\n");
    if !truncated_content.is_empty() {
        fs::write(log_path, format!("{}\n", truncated_content)).map_err(|e| {
            ReedError::IoError {
                operation: "write_truncated_log".to_string(),
                reason: e.to_string(),
            }
        })?;
    } else {
        // Remove empty log file
        let _ = fs::remove_file(log_path);
    }

    report.truncated = true;
    report.valid_entries = valid_lines.len();
    report.total_entries = valid_lines.len();

    Ok(report)
}

/// Append validated entry to log file.
///
/// ## Input
/// - `log_path`: Path to version.log file
/// - `encoded_entry`: Encoded log entry (already validated)
///
/// ## Output
/// - `ReedResult<()>`: Success or error
///
/// ## Performance
/// - < 5ms (append + flush)
///
/// ## Error Conditions
/// - IoError: Cannot write to log file
///
/// ## Example Usage
/// ```no_run
/// use reedbase::log::{encode_log_entry, append_entry};
/// use std::path::Path;
///
/// let encoded = encode_log_entry(&entry)?;
/// append_entry(Path::new(".reedbase/tables/text/version.log"), &encoded)?;
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn append_entry(log_path: &Path, encoded_entry: &str) -> ReedResult<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| ReedError::IoError {
            operation: "open_log_for_append".to_string(),
            reason: e.to_string(),
        })?;

    writeln!(file, "{}", encoded_entry).map_err(|e| ReedError::IoError {
        operation: "append_log_entry".to_string(),
        reason: e.to_string(),
    })?;

    file.flush().map_err(|e| ReedError::IoError {
        operation: "flush_log".to_string(),
        reason: e.to_string(),
    })?;

    Ok(())
}
