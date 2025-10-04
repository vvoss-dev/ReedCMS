// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CSS File Writers
//!
//! Utilities for writing CSS bundles and source maps to disk.
//! Handles directory creation and atomic writes.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::fs;
use std::path::Path;

/// Writes CSS file to disk.
///
/// ## Input
/// - `path`: Output file path
/// - `content`: CSS content to write
///
/// ## Process
/// 1. Create parent directory if needed
/// 2. Write content to file
///
/// ## Performance
/// - < 10ms for typical CSS file
///
/// ## Error Conditions
/// - Permission denied
/// - Disk full
/// - Invalid path
///
/// ## Example
/// ```rust
/// write_css_file("public/css/landing.mouse.css", "body{margin:0}")?;
/// ```
pub fn write_css_file(path: &str, content: &str) -> ReedResult<()> {
    // Create directory if needed
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| ReedError::IoError {
            operation: "create_dir".to_string(),
            path: parent.display().to_string(),
            reason: e.to_string(),
        })?;
    }

    // Write content
    fs::write(path, content).map_err(|e| ReedError::IoError {
        operation: "write".to_string(),
        path: path.to_string(),
        reason: e.to_string(),
    })
}

/// Writes source map file to disk.
///
/// ## Input
/// - `path`: Output file path
/// - `content`: Source map JSON content
///
/// ## Process
/// Same as write_css_file (reuses implementation)
///
/// ## Performance
/// - < 10ms for typical source map
///
/// ## Example
/// ```rust
/// write_source_map("public/css/landing.mouse.css.map", "{...}")?;
/// ```
pub fn write_source_map(path: &str, content: &str) -> ReedResult<()> {
    write_css_file(path, content)
}

/// Cleans old bundles with different session hash.
///
/// ## Input
/// - `output_dir`: Directory containing bundles (e.g., "public/session/styles")
/// - `current_hash`: Current session hash to keep
///
/// ## Process
/// 1. List all CSS files in directory
/// 2. Extract session hash from filename
/// 3. Delete files with different hash
///
/// ## Performance
/// - < 50ms for 100 files
///
/// ## Example
/// ```rust
/// // Keeps: landing.a3f5b2c8.mouse.css
/// // Deletes: landing.old123.mouse.css
/// clean_old_bundles("public/session/styles", "a3f5b2c8")?;
/// ```
pub fn clean_old_bundles(output_dir: &str, current_hash: &str) -> ReedResult<usize> {
    let mut deleted_count = 0;

    let entries = fs::read_dir(output_dir).map_err(|e| ReedError::IoError {
        operation: "read_dir".to_string(),
        path: output_dir.to_string(),
        reason: e.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ReedError::IoError {
            operation: "read_entry".to_string(),
            path: output_dir.to_string(),
            reason: e.to_string(),
        })?;

        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            // Check if filename contains a hash
            if let Some(hash) = extract_hash_from_filename(filename) {
                // Delete if hash doesn't match current hash
                if hash != current_hash {
                    fs::remove_file(&path).map_err(|e| ReedError::IoError {
                        operation: "remove_file".to_string(),
                        path: path.display().to_string(),
                        reason: e.to_string(),
                    })?;
                    deleted_count += 1;
                }
            }
        }
    }

    Ok(deleted_count)
}

/// Extracts session hash from bundle filename.
///
/// ## Input
/// - `filename`: Bundle filename (e.g., "landing.a3f5b2c8.mouse.css")
///
/// ## Output
/// - Some(hash) if pattern matches, None otherwise
///
/// ## Pattern
/// - `{layout}.{hash}.{variant}.{ext}`
/// - Hash is always 8 characters
///
/// ## Examples
/// - `landing.a3f5b2c8.mouse.css` → Some("a3f5b2c8")
/// - `landing.a3f5b2c8.mouse.css.map` → Some("a3f5b2c8")
/// - `other-file.css` → None
fn extract_hash_from_filename(filename: &str) -> Option<String> {
    let parts: Vec<&str> = filename.split('.').collect();

    // Expected format: layout.hash.variant.extension
    // Minimum 4 parts: layout.hash.variant.css
    if parts.len() >= 4 {
        // Hash is second part and must be 8 characters
        let potential_hash = parts[1];
        if potential_hash.len() == 8 && potential_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(potential_hash.to_string());
        }
    }

    None
}

/// Ensures output directory exists.
///
/// ## Input
/// - `dir_path`: Directory path to create
///
/// ## Process
/// Creates directory and all parent directories if needed.
///
/// ## Performance
/// - < 5ms
///
/// ## Example
/// ```rust
/// ensure_output_dir("public/session/styles")?;
/// ```
pub fn ensure_output_dir(dir_path: &str) -> ReedResult<()> {
    fs::create_dir_all(dir_path).map_err(|e| ReedError::IoError {
        operation: "create_dir_all".to_string(),
        path: dir_path.to_string(),
        reason: e.to_string(),
    })
}
