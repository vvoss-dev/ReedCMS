// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Log file rotation and cleanup management.
//!
//! ## Features
//! - Automatic rotation at 100MB
//! - Gzip compression of rotated files
//! - Keep last 10 rotated files
//! - Timestamp-based file naming
//!
//! ## Performance
//! - Rotation check: < 1ms
//! - Rotation execution: < 100ms

use crate::reedcms::reedstream::{ReedError, ReedResult};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;
use std::path::Path;

/// Log file rotation manager.
pub struct LogFileManager;

impl LogFileManager {
    /// Rotates log file if size exceeds limit.
    ///
    /// ## Arguments
    /// - `log_path`: Path to log file
    ///
    /// ## Rotation Strategy
    /// - Max size: 100MB per file
    /// - Keep last 10 files
    /// - Compress old files with gzip
    ///
    /// ## Performance
    /// - Check: < 1ms
    /// - Rotation: < 100ms
    ///
    /// ## Example
    /// ```rust
    /// LogFileManager::rotate_if_needed(".reed/flow/reedmonitor.log")?;
    /// ```
    pub fn rotate_if_needed(log_path: &str) -> ReedResult<()> {
        let metadata = std::fs::metadata(log_path).map_err(|e| ReedError::IoError {
            operation: "read_log_metadata".to_string(),
            path: log_path.to_string(),
            reason: e.to_string(),
        })?;

        // Rotate if > 100MB
        if metadata.len() > 100 * 1024 * 1024 {
            Self::rotate_log(log_path)?;
        }

        Ok(())
    }

    /// Rotates log file.
    fn rotate_log(log_path: &str) -> ReedResult<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let rotated = format!("{}.{}.gz", log_path, timestamp);

        // Read current log content
        let content = std::fs::read(log_path).map_err(|e| ReedError::IoError {
            operation: "read_log_file".to_string(),
            path: log_path.to_string(),
            reason: e.to_string(),
        })?;

        // Compress and write rotated file
        let compressed = Self::compress_gzip(&content)?;
        std::fs::write(&rotated, compressed).map_err(|e| ReedError::IoError {
            operation: "write_rotated_log".to_string(),
            path: rotated.clone(),
            reason: e.to_string(),
        })?;

        // Clear original file
        std::fs::write(log_path, "").map_err(|e| ReedError::IoError {
            operation: "clear_log_file".to_string(),
            path: log_path.to_string(),
            reason: e.to_string(),
        })?;

        // Cleanup old logs
        Self::cleanup_old_logs(log_path)?;

        Ok(())
    }

    /// Compresses data with gzip.
    fn compress_gzip(data: &[u8]) -> ReedResult<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).map_err(|e| ReedError::IoError {
            operation: "gzip_compress".to_string(),
            path: "memory".to_string(),
            reason: e.to_string(),
        })?;

        encoder.finish().map_err(|e| ReedError::IoError {
            operation: "gzip_finish".to_string(),
            path: "memory".to_string(),
            reason: e.to_string(),
        })
    }

    /// Cleans up old log files (keep last 10).
    fn cleanup_old_logs(log_path: &str) -> ReedResult<()> {
        let dir = Path::new(log_path)
            .parent()
            .ok_or_else(|| ReedError::ConfigError {
                component: "log_manager".to_string(),
                reason: format!("log path '{}' must have parent directory", log_path),
            })?;

        let base_name = Path::new(log_path)
            .file_name()
            .ok_or_else(|| ReedError::ConfigError {
                component: "log_manager".to_string(),
                reason: format!("log path '{}' must have filename", log_path),
            })?
            .to_str()
            .unwrap();

        // Find all rotated log files
        let mut log_files: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| ReedError::IoError {
                operation: "read_log_directory".to_string(),
                path: dir.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name();
                let name_str = name.to_string_lossy();
                name_str.starts_with(base_name) && name_str.ends_with(".gz")
            })
            .collect();

        // Sort by modification time (oldest first)
        log_files.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));

        // Remove oldest files if more than 10
        if log_files.len() > 10 {
            for file in log_files.iter().take(log_files.len() - 10) {
                let _ = std::fs::remove_file(file.path());
            }
        }

        Ok(())
    }
}
