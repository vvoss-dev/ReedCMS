// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Log encoding for version history.
//!
//! Encodes log entries using integer codes from registries with CRC32 validation.

use crate::error::ReedResult;
use crate::log::types::LogEntry;
use crate::registry;
use crc32fast::Hasher;

const MAGIC: &str = "REED";

/// Encode log entry to string with CRC32 validation.
///
/// ## Input
/// - `entry`: Log entry to encode
///
/// ## Output
/// - `ReedResult<String>`: Encoded log line with format:
///   `REED|{length}|{timestamp}|{action_code}|{user_code}|{base_version}|{size}|{rows}|{hash}|{frame_id}|{crc32}`
///
/// ## Performance
/// - < 150μs typical (2 dictionary lookups + CRC32 + string formatting)
///
/// ## Error Conditions
/// - UnknownAction: Action name not found in actions.dict
///
/// ## Example Usage
/// ```no_run
/// use reedbase::log::{LogEntry, encode_log_entry};
///
/// let entry = LogEntry::new(
///     1736860900,
///     "update".to_string(),
///     "admin".to_string(),
///     1736860800,
///     2500,
///     15,
///     "sha256:abc123".to_string(),
///     None,
/// );
/// let encoded = encode_log_entry(&entry)?;
/// // "REED|00000058|1736860900|2|1|1736860800|2500|15|sha256:abc123|n/a|A1B2C3D4"
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn encode_log_entry(entry: &LogEntry) -> ReedResult<String> {
    let action_code = registry::get_action_code(&entry.action)?;
    let user_code = registry::get_or_create_user_code(&entry.user)?;

    let frame_id_str = entry
        .frame_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "n/a".to_string());

    // Build data portion (everything except magic, length, and CRC32)
    let data = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        entry.timestamp,
        action_code,
        user_code,
        entry.base_version,
        entry.size,
        entry.rows,
        entry.hash,
        frame_id_str
    );

    // Calculate CRC32 of data portion
    let mut hasher = Hasher::new();
    hasher.update(data.as_bytes());
    let crc32 = hasher.finalize();

    // Build final entry with magic, placeholder length, data, and CRC32
    // Length field is 8 hex chars (00000000), so we calculate final length including it
    let length_placeholder = "00000000";
    let temp_entry = format!("{}|{}|{}|{:08X}", MAGIC, length_placeholder, data, crc32);
    let actual_length = temp_entry.len();

    // Now build final entry with actual length
    let final_entry = format!("{}|{:08X}|{}|{:08X}", MAGIC, actual_length, data, crc32);

    Ok(final_entry)
}

/// Encode multiple log entries.
///
/// ## Input
/// - `entries`: Log entries to encode
///
/// ## Output
/// - `ReedResult<String>`: Encoded log (newline-separated)
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 150μs per entry
///
/// ## Error Conditions
/// - UnknownAction: Action name not found
///
/// ## Example Usage
/// ```no_run
/// use reedbase::log::{LogEntry, encode_log_entries};
///
/// let entries = vec![entry1, entry2, entry3];
/// let encoded = encode_log_entries(&entries)?;
/// std::fs::write("version.log", encoded)?;
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn encode_log_entries(entries: &[LogEntry]) -> ReedResult<String> {
    let mut lines = Vec::new();

    for entry in entries {
        lines.push(encode_log_entry(entry)?);
    }

    Ok(lines.join("\n"))
}

/// Calculate encoded size vs plain text size.
///
/// ## Input
/// - `entries`: Log entries
///
/// ## Output
/// - `(usize, usize)`: (encoded_size, plain_size) in bytes
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 1ms for 100 entries
///
/// ## Example Usage
/// ```no_run
/// use reedbase::log::calculate_size_savings;
///
/// let (encoded, plain) = calculate_size_savings(&entries)?;
/// let savings = ((plain - encoded) as f64 / plain as f64) * 100.0;
/// println!("Savings: {:.1}%", savings);
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn calculate_size_savings(entries: &[LogEntry]) -> ReedResult<(usize, usize)> {
    let encoded = encode_log_entries(entries)?.len();

    let mut plain_size = 0;
    for entry in entries {
        let frame_id_str = entry
            .frame_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "n/a".to_string());

        plain_size += format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            entry.timestamp,
            entry.action,
            entry.user,
            entry.base_version,
            entry.size,
            entry.rows,
            entry.hash,
            frame_id_str
        )
        .len()
            + 1; // +1 for newline
    }

    Ok((encoded, plain_size))
}
