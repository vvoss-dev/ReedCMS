// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Index rebuild functionality for version log maintenance.
//!
//! Provides tools to rebuild indices from version log after corruption
//! or during initial setup.

use crate::error::{ReedError, ReedResult};
use crate::version::index::{FrameId, Timestamp, VersionId, VersionIndices};
use std::fs;
use std::path::Path;

/// Version entry from log file.
#[derive(Debug, Clone)]
pub struct VersionEntry {
    pub version_id: VersionId,
    pub timestamp: Timestamp,
    pub frame_id: FrameId,
}

/// Rebuild indices from version log (for corruption recovery).
///
/// ## Arguments
/// - `log_path`: Path to version log CSV
/// - `timestamp_index_path`: Path to timestamp B+-Tree
/// - `frame_index_path`: Path to frame B+-Tree
///
/// ## Algorithm
/// 1. Delete old index files
/// 2. Create new empty indices
/// 3. Parse all versions from log
/// 4. Insert all versions into indices
///
/// ## Performance
/// - <5s for 100k versions
/// - Progress printed every 1000 versions
///
/// ## Example
/// ```rust,ignore
/// use reedbase::version::rebuild_indices;
///
/// rebuild_indices(
///     ".reed/flow/versions.log",
///     ".reed/indices/versions_timestamp.btree",
///     ".reed/indices/versions_frame.btree",
/// )?;
/// ```
pub fn rebuild_indices<P: AsRef<Path>>(
    log_path: P,
    timestamp_index_path: P,
    frame_index_path: P,
) -> ReedResult<()> {
    println!("Rebuilding version indices from log...");

    // 1. Delete old index files
    let _ = fs::remove_file(timestamp_index_path.as_ref());
    let _ = fs::remove_file(frame_index_path.as_ref());

    // 2. Create new indices
    let mut indices = VersionIndices::open_or_create(&timestamp_index_path, &frame_index_path)?;

    // 3. Read all versions from log
    let content = fs::read_to_string(log_path.as_ref()).map_err(|e| ReedError::IoError {
        operation: format!("read_version_log: {}", log_path.as_ref().display()),
        reason: e.to_string(),
    })?;

    let versions = parse_version_log(&content)?;

    println!("Indexing {} versions...", versions.len());

    // 4. Insert all versions into indices
    for (i, version) in versions.iter().enumerate() {
        indices.insert(
            version.version_id,
            version.timestamp.clone(),
            version.frame_id.clone(),
        )?;

        if (i + 1) % 1000 == 0 {
            println!(
                "  Progress: {}/{} ({:.1}%)",
                i + 1,
                versions.len(),
                (i + 1) as f64 / versions.len() as f64 * 100.0
            );
        }
    }

    println!("✓ Rebuild complete");

    // 5. Show statistics
    let stats = indices.stats();
    println!(
        "  Timestamp index: {} (disk: {})",
        format_bytes(stats.timestamp_memory),
        format_bytes(stats.timestamp_disk)
    );
    println!(
        "  Frame index: {} (disk: {})",
        format_bytes(stats.frame_memory),
        format_bytes(stats.frame_disk)
    );

    Ok(())
}

/// Parse version log CSV into entries.
///
/// ## Format
/// ```text
/// version_id|timestamp|frame_id|key|value_hash|delta_bytes|metadata
/// 1|2025-10-28T08:15:23.001Z|F001|page.title@de|abc123|142|user=alice
/// ```
///
/// ## Arguments
/// - `content`: CSV file content
///
/// ## Returns
/// - Vec<VersionEntry> with version_id, timestamp, frame_id
fn parse_version_log(content: &str) -> ReedResult<Vec<VersionEntry>> {
    let mut entries = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        // Skip header line
        if line_num == 0 && line.starts_with("version_id|") {
            continue;
        }

        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() < 3 {
            return Err(ReedError::ParseError {
                reason: format!(
                    "Invalid version log format at line {}: expected 7 columns, found {}",
                    line_num + 1,
                    parts.len()
                ),
            });
        }

        let version_id = parts[0]
            .parse::<usize>()
            .map_err(|_| ReedError::ParseError {
                reason: format!(
                    "Invalid version_id at line {}: '{}'",
                    line_num + 1,
                    parts[0]
                ),
            })?;

        let timestamp = parts[1].to_string();
        let frame_id = parts[2].to_string();

        entries.push(VersionEntry {
            version_id,
            timestamp,
            frame_id,
        });
    }

    Ok(entries)
}

/// Format bytes as human-readable string.
fn format_bytes(bytes: usize) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }

    let units = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", size, units[unit_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_log_empty() {
        let content = "";
        let entries = parse_version_log(content).unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_parse_version_log_header_only() {
        let content = "version_id|timestamp|frame_id|key|value_hash|delta_bytes|metadata";
        let entries = parse_version_log(content).unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_parse_version_log_single_entry() {
        let content = "version_id|timestamp|frame_id|key|value_hash|delta_bytes|metadata\n\
                       1|2025-10-28T08:15:23.001Z|F001|page.title@de|abc123|142|user=alice";
        let entries = parse_version_log(content).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].version_id, 1);
        assert_eq!(entries[0].timestamp, "2025-10-28T08:15:23.001Z");
        assert_eq!(entries[0].frame_id, "F001");
    }

    #[test]
    fn test_parse_version_log_multiple_entries() {
        let content = "version_id|timestamp|frame_id|key|value_hash|delta_bytes|metadata\n\
                       1|2025-10-28T08:15:23.001Z|F001|page.title@de|abc123|142|user=alice\n\
                       2|2025-10-28T08:15:23.001Z|F001|page.desc@de|def456|89|user=alice\n\
                       3|2025-10-28T08:16:45.500Z|F002|page.title@de|ghi789|67|user=bob";
        let entries = parse_version_log(content).unwrap();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].version_id, 1);
        assert_eq!(entries[1].version_id, 2);
        assert_eq!(entries[2].version_id, 3);
    }

    #[test]
    fn test_parse_version_log_invalid_version_id() {
        let content = "version_id|timestamp|frame_id|key|value_hash|delta_bytes|metadata\n\
                       invalid|2025-10-28T08:15:23.001Z|F001|page.title@de|abc123|142|user=alice";
        let result = parse_version_log(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_version_log_insufficient_columns() {
        let content = "1|2025-10-28T08:15:23.001Z";
        let result = parse_version_log(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_bytes_zero() {
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn test_format_bytes_bytes() {
        assert_eq!(format_bytes(512), "512.0 B");
    }

    #[test]
    fn test_format_bytes_kilobytes() {
        assert_eq!(format_bytes(2048), "2.0 KB");
    }

    #[test]
    fn test_format_bytes_megabytes() {
        assert_eq!(format_bytes(10_485_760), "10.0 MB");
    }

    #[test]
    fn test_format_bytes_gigabytes() {
        assert_eq!(format_bytes(2_147_483_648), "2.0 GB");
    }
}
