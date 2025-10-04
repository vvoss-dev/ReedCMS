// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Session Hash Generator for Asset Bundling
//!
//! Generates MD5 hash over all CSS/JS files for cache-busting and bundle versioning.
//! Hash is stored in .reed/project.csv → project.session_hash
//!
//! ## Performance
//! - Hash generation: < 50ms for 100 files
//! - Cached in project.csv for server runtime

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Generates session hash for asset bundling.
///
/// ## Process
/// 1. Discover all CSS/JS files in templates/
/// 2. Read and concatenate file contents
/// 3. Generate MD5 hash
/// 4. Store in .reed/project.csv → project.session_hash
///
/// ## Performance
/// - Hash generation: < 50ms for 100 files
/// - Cached in project.csv for server runtime
///
/// ## Output
/// - `a3f5b2c8` (8-character hex string)
///
/// ## Example
/// ```rust
/// let hash = generate_session_hash()?;
/// assert_eq!(hash.len(), 8);
/// ```
pub fn generate_session_hash() -> ReedResult<String> {
    // Collect all CSS/JS files
    let css_files = discover_css_files("templates/")?;
    let js_files = discover_js_files("templates/")?;

    // Hash all file contents in sorted order (for deterministic hashing)
    let mut all_files: Vec<PathBuf> = css_files.into_iter().chain(js_files).collect();
    all_files.sort();

    let mut combined_content = Vec::new();
    for file in &all_files {
        let content = fs::read(file).map_err(|e| ReedError::IoError {
            operation: "read".to_string(),
            path: file.display().to_string(),
            reason: e.to_string(),
        })?;
        combined_content.extend_from_slice(&content);
    }

    // Generate MD5 hash
    let digest = md5::compute(&combined_content);
    let hash = format!("{:x}", digest);
    Ok(hash[..8].to_string())
}

/// Discovers all CSS files in templates directory.
///
/// ## Input
/// - `base_path`: Root directory to search (e.g., "templates/")
///
/// ## Output
/// - Vec of absolute paths to all .css files
///
/// ## Performance
/// - < 10ms for 100 files
///
/// ## Example
/// ```rust
/// let files = discover_css_files("templates/")?;
/// // Returns: ["templates/layouts/landing/landing.mouse.css", ...]
/// ```
pub fn discover_css_files<P: AsRef<Path>>(base_path: P) -> ReedResult<Vec<PathBuf>> {
    discover_files_by_extension(base_path, "css")
}

/// Discovers all JS files in templates directory.
///
/// ## Input
/// - `base_path`: Root directory to search (e.g., "templates/")
///
/// ## Output
/// - Vec of absolute paths to all .js files
///
/// ## Performance
/// - < 10ms for 100 files
///
/// ## Example
/// ```rust
/// let files = discover_js_files("templates/")?;
/// // Returns: ["templates/layouts/landing/landing.js", ...]
/// ```
pub fn discover_js_files<P: AsRef<Path>>(base_path: P) -> ReedResult<Vec<PathBuf>> {
    discover_files_by_extension(base_path, "js")
}

/// Discovers all files with given extension recursively.
///
/// ## Input
/// - `base_path`: Root directory to search
/// - `extension`: File extension without dot (e.g., "css", "js")
///
/// ## Output
/// - Vec of absolute paths to all matching files
///
/// ## Performance
/// - < 10ms per 100 files
///
/// ## Error Conditions
/// - Directory not found
/// - Permission denied
fn discover_files_by_extension<P: AsRef<Path>>(
    base_path: P,
    extension: &str,
) -> ReedResult<Vec<PathBuf>> {
    let base_path = base_path.as_ref();
    let mut files = Vec::new();

    if !base_path.exists() {
        return Err(ReedError::NotFound {
            resource: format!("directory: {}", base_path.display()),
            context: None,
        });
    }

    discover_files_recursive(base_path, extension, &mut files)?;

    Ok(files)
}

/// Recursively discovers files with given extension.
///
/// ## Input
/// - `dir`: Current directory to search
/// - `extension`: File extension to match
/// - `files`: Mutable vector to accumulate results
///
/// ## Process
/// 1. Read directory entries
/// 2. For each entry:
///    - If directory: recurse
///    - If file with matching extension: add to results
fn discover_files_recursive(
    dir: &Path,
    extension: &str,
    files: &mut Vec<PathBuf>,
) -> ReedResult<()> {
    let entries = fs::read_dir(dir).map_err(|e| ReedError::IoError {
        operation: "read_dir".to_string(),
        path: dir.display().to_string(),
        reason: e.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ReedError::IoError {
            operation: "read_entry".to_string(),
            path: dir.display().to_string(),
            reason: e.to_string(),
        })?;

        let path = entry.path();

        if path.is_dir() {
            // Recurse into subdirectory
            discover_files_recursive(&path, extension, files)?;
        } else if let Some(ext) = path.extension() {
            // Check if file has matching extension
            if ext == extension {
                files.push(path);
            }
        }
    }

    Ok(())
}

/// Stores session hash in .reed/project.csv.
///
/// ## Input
/// - `hash`: 8-character session hash
///
/// ## Output
/// - Success or error
///
/// ## Process
/// 1. Read existing project.csv via csv::read_csv()
/// 2. Update or insert project.session_hash key
/// 3. Write back via csv::write_csv()
///
/// ## Error Conditions
/// - CSV read/write failure
/// - Permission denied
///
/// ## Example
/// ```rust
/// store_session_hash("a3f5b2c8")?;
/// ```
pub fn store_session_hash(hash: &str) -> ReedResult<()> {
    use crate::reedcms::csv::{read_csv, write_csv, CsvRecord};

    let csv_path = ".reed/project.csv";

    // Read existing CSV
    let mut records = read_csv(csv_path).unwrap_or_else(|_| Vec::new());

    // Find and update existing record or create new one
    let key = "project.session_hash";
    let mut found = false;

    for record in &mut records {
        if record.key == key {
            record.value = hash.to_string();
            found = true;
            break;
        }
    }

    if !found {
        records.push(CsvRecord {
            key: key.to_string(),
            value: hash.to_string(),
            description: Some("Current asset bundle session hash".to_string()),
        });
    }

    // Write back to CSV
    write_csv(csv_path, &records)?;

    Ok(())
}

/// Retrieves session hash from .reed/project.csv.
///
/// ## Output
/// - Session hash string or error
///
/// ## Error Conditions
/// - CSV read failure
/// - Key not found
///
/// ## Example
/// ```rust
/// let hash = get_session_hash()?;
/// assert_eq!(hash.len(), 8);
/// ```
pub fn get_session_hash() -> ReedResult<String> {
    use crate::reedcms::csv::read_csv;

    let csv_path = ".reed/project.csv";
    let records = read_csv(csv_path)?;

    let key = "project.session_hash";
    for record in records {
        if record.key == key {
            return Ok(record.value);
        }
    }

    Err(ReedError::NotFound {
        resource: format!("session_hash key: {}", key),
        context: Some("project.csv".to_string()),
    })
}

/// Generates and stores session hash in one operation.
///
/// ## Output
/// - Generated session hash
///
/// ## Process
/// 1. Generate hash via generate_session_hash()
/// 2. Store hash via store_session_hash()
/// 3. Return hash
///
/// ## Performance
/// - < 60ms total (generation + storage)
///
/// ## Example
/// ```rust
/// let hash = generate_and_store_session_hash()?;
/// println!("New session hash: {}", hash);
/// ```
pub fn generate_and_store_session_hash() -> ReedResult<String> {
    let hash = generate_session_hash()?;
    store_session_hash(&hash)?;
    Ok(hash)
}
