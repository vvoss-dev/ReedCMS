// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Log decoding for version history.
//!
//! Decodes log entries from integer codes to human-readable format with CRC32 validation.

use crate::error::{ReedError, ReedResult};
use crate::log::types::LogEntry;
use crate::registry;
use crc32fast::Hasher;
use uuid::Uuid;

const MAGIC: &str = "REED";

/// Decode log entry from string with CRC32 validation.
///
/// ## Input
/// - `line`: Encoded log line
///
/// ## Output
/// - `ReedResult<LogEntry>`: Decoded log entry
///
/// ## Performance
/// - < 1ms typical (2 dictionary lookups + CRC32 + parsing)
///
/// ## Error Conditions
/// - ParseError: Invalid format or field count
/// - CorruptedLogEntry: CRC32 mismatch or invalid magic bytes
/// - UnknownActionCode: Action code not found
/// - UnknownUserCode: User code not found
///
/// ## Example Usage
/// ```no_run
/// use reedbase::log::decode_log_entry;
///
/// let line = "REED|00000058|1736860900|2|1|1736860800|2500|15|sha256:abc123|n/a|A1B2C3D4";
/// let entry = decode_log_entry(line)?;
/// assert_eq!(entry.action, "update");
/// assert_eq!(entry.user, "admin");
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn decode_log_entry(line: &str) -> ReedResult<LogEntry> {
    let parts: Vec<&str> = line.split('|').collect();

    // New format: REED|length|timestamp|action|user|base|size|rows|hash|frame_id|crc32 (11 fields)
    // Old format: timestamp|action|user|base|size|rows|hash|frame_id (8 fields)
    // Older format: timestamp|action|user|base|size|rows|hash (7 fields)

    if parts.len() == 11 {
        // New format with CRC32 validation
        decode_new_format(line, &parts)
    } else if parts.len() == 8 || parts.len() == 7 {
        // Old format without CRC32 (backward compatibility)
        decode_old_format(&parts)
    } else {
        Err(ReedError::ParseError {
            reason: format!("Expected 7, 8, or 11 fields, got {}", parts.len()),
        })
    }
}

/// Decode new format with CRC32 validation.
fn decode_new_format(line: &str, parts: &[&str]) -> ReedResult<LogEntry> {
    // Validate magic bytes
    if parts[0] != MAGIC {
        return Err(ReedError::CorruptedLogEntry {
            line: 0,
            reason: format!(
                "Invalid magic bytes: expected '{}', got '{}'",
                MAGIC, parts[0]
            ),
        });
    }

    // Parse length
    let expected_length = u32::from_str_radix(parts[1], 16).map_err(|e| ReedError::ParseError {
        reason: format!("Invalid length field: {}", e),
    })? as usize;

    // Validate length matches actual line length
    if line.len() != expected_length {
        return Err(ReedError::CorruptedLogEntry {
            line: 0,
            reason: format!(
                "Length mismatch: expected {}, got {}",
                expected_length,
                line.len()
            ),
        });
    }

    // Extract CRC32 (last field)
    let expected_crc = u32::from_str_radix(parts[10], 16).map_err(|e| ReedError::ParseError {
        reason: format!("Invalid CRC32 field: {}", e),
    })?;

    // Calculate CRC32 of data portion (fields 2-9)
    let data = parts[2..10].join("|");
    let mut hasher = Hasher::new();
    hasher.update(data.as_bytes());
    let actual_crc = hasher.finalize();

    // Validate CRC32
    if actual_crc != expected_crc {
        return Err(ReedError::CorruptedLogEntry {
            line: 0,
            reason: format!(
                "CRC32 mismatch: expected {:08X}, got {:08X}",
                expected_crc, actual_crc
            ),
        });
    }

    // Parse fields
    let timestamp = parts[2].parse::<u64>().map_err(|e| ReedError::ParseError {
        reason: format!("Invalid timestamp: {}", e),
    })?;

    let action_code = parts[3].parse::<u8>().map_err(|e| ReedError::ParseError {
        reason: format!("Invalid action code: {}", e),
    })?;

    let user_code = parts[4].parse::<u32>().map_err(|e| ReedError::ParseError {
        reason: format!("Invalid user code: {}", e),
    })?;

    let base_version = parts[5].parse::<u64>().map_err(|e| ReedError::ParseError {
        reason: format!("Invalid base version: {}", e),
    })?;

    let size = parts[6]
        .parse::<usize>()
        .map_err(|e| ReedError::ParseError {
            reason: format!("Invalid size: {}", e),
        })?;

    let rows = parts[7]
        .parse::<usize>()
        .map_err(|e| ReedError::ParseError {
            reason: format!("Invalid rows: {}", e),
        })?;

    let hash = parts[8].to_string();

    // Parse frame_id
    let frame_id = match parts[9] {
        "n/a" | "" => None,
        uuid_str => Some(
            Uuid::parse_str(uuid_str).map_err(|e| ReedError::ParseError {
                reason: format!("Invalid frame_id UUID: {}", e),
            })?,
        ),
    };

    // Decode codes to names
    let action = registry::get_action_name(action_code)?;
    let user = registry::get_username(user_code)?;

    Ok(LogEntry {
        timestamp,
        action,
        user,
        base_version,
        size,
        rows,
        hash,
        frame_id,
    })
}

/// Decode old format without CRC32 (backward compatibility).
fn decode_old_format(parts: &[&str]) -> ReedResult<LogEntry> {
    let timestamp = parts[0].parse::<u64>().map_err(|e| ReedError::ParseError {
        reason: format!("Invalid timestamp: {}", e),
    })?;

    let action_code = parts[1].parse::<u8>().map_err(|e| ReedError::ParseError {
        reason: format!("Invalid action code: {}", e),
    })?;

    let user_code = parts[2].parse::<u32>().map_err(|e| ReedError::ParseError {
        reason: format!("Invalid user code: {}", e),
    })?;

    let base_version = parts[3].parse::<u64>().map_err(|e| ReedError::ParseError {
        reason: format!("Invalid base version: {}", e),
    })?;

    let size = parts[4]
        .parse::<usize>()
        .map_err(|e| ReedError::ParseError {
            reason: format!("Invalid size: {}", e),
        })?;

    let rows = parts[5]
        .parse::<usize>()
        .map_err(|e| ReedError::ParseError {
            reason: format!("Invalid rows: {}", e),
        })?;

    let hash = parts[6].to_string();

    // Parse frame_id if present (8 fields)
    let frame_id = if parts.len() == 8 {
        match parts[7] {
            "n/a" | "" => None,
            uuid_str => Some(
                Uuid::parse_str(uuid_str).map_err(|e| ReedError::ParseError {
                    reason: format!("Invalid frame_id UUID: {}", e),
                })?,
            ),
        }
    } else {
        None // Old format without frame_id
    };

    // Decode codes to names
    let action = registry::get_action_name(action_code)?;
    let user = registry::get_username(user_code)?;

    Ok(LogEntry {
        timestamp,
        action,
        user,
        base_version,
        size,
        rows,
        hash,
        frame_id,
    })
}

/// Decode multiple log entries.
///
/// ## Input
/// - `content`: Encoded log content (newline-separated)
///
/// ## Output
/// - `ReedResult<Vec<LogEntry>>`: Decoded log entries
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 1ms per entry
/// - < 50ms for 1000 entries
///
/// ## Error Conditions
/// - ParseError: Invalid format
/// - CorruptedLogEntry: CRC32 mismatch or invalid magic bytes
/// - UnknownActionCode: Action code not found
/// - UnknownUserCode: User code not found
///
/// ## Example Usage
/// ```no_run
/// use reedbase::log::decode_log_entries;
///
/// let content = std::fs::read_to_string("version.log")?;
/// let entries = decode_log_entries(&content)?;
/// for entry in entries {
///     println!("{} - {} by {}", entry.timestamp, entry.action, entry.user);
/// }
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn decode_log_entries(content: &str) -> ReedResult<Vec<LogEntry>> {
    let mut entries = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        entries.push(decode_log_entry(line).map_err(|e| match e {
            ReedError::CorruptedLogEntry { reason, .. } => ReedError::CorruptedLogEntry {
                line: line_num + 1,
                reason,
            },
            ReedError::ParseError { reason } => ReedError::ParseError {
                reason: format!("Line {}: {}", line_num + 1, reason),
            },
            other => other,
        })?);
    }

    Ok(entries)
}

/// Filter log entries by action.
///
/// ## Input
/// - `entries`: Log entries to filter
/// - `action`: Action name to filter by
///
/// ## Output
/// - `Vec<&LogEntry>`: Filtered entries
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 10ms for 1000 entries
///
/// ## Example Usage
/// ```no_run
/// use reedbase::log::filter_by_action;
///
/// let updates = filter_by_action(&entries, "update");
/// println!("Found {} update operations", updates.len());
/// ```
pub fn filter_by_action<'a>(entries: &'a [LogEntry], action: &str) -> Vec<&'a LogEntry> {
    entries.iter().filter(|e| e.action == action).collect()
}

/// Filter log entries by user.
///
/// ## Input
/// - `entries`: Log entries to filter
/// - `user`: Username to filter by
///
/// ## Output
/// - `Vec<&LogEntry>`: Filtered entries
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 10ms for 1000 entries
///
/// ## Example Usage
/// ```no_run
/// use reedbase::log::filter_by_user;
///
/// let admin_actions = filter_by_user(&entries, "admin");
/// println!("Admin performed {} actions", admin_actions.len());
/// ```
pub fn filter_by_user<'a>(entries: &'a [LogEntry], user: &str) -> Vec<&'a LogEntry> {
    entries.iter().filter(|e| e.user == user).collect()
}

/// Filter log entries by time range.
///
/// ## Input
/// - `entries`: Log entries to filter
/// - `start`: Start timestamp (inclusive)
/// - `end`: End timestamp (inclusive)
///
/// ## Output
/// - `Vec<&LogEntry>`: Filtered entries
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 10ms for 1000 entries
///
/// ## Example Usage
/// ```no_run
/// use reedbase::log::filter_by_time_range;
///
/// let yesterday = now() - 86400;
/// let recent = filter_by_time_range(&entries, yesterday, now());
/// println!("Last 24h: {} operations", recent.len());
/// ```
pub fn filter_by_time_range<'a>(
    entries: &'a [LogEntry],
    start: u64,
    end: u64,
) -> Vec<&'a LogEntry> {
    entries
        .iter()
        .filter(|e| e.timestamp >= start && e.timestamp <= end)
        .collect()
}
