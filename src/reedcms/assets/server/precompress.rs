// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Build-time asset pre-compression for zero runtime overhead.
//!
//! This module pre-compresses static assets during build phase, allowing
//! the server to serve pre-compressed files with zero CPU overhead.

use crate::reedcms::assets::server::compression::{compress_brotli, compress_gzip};
use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Discovers all compressible assets in a directory recursively.
///
/// ## Input
/// - `base_dir`: Base directory to search
///
/// ## Output
/// - `ReedResult<Vec<PathBuf>>`: List of compressible file paths
///
/// ## Performance
/// - Recursive directory traversal
/// - Filters by extension (CSS, JS, HTML, SVG, JSON)
///
/// ## Error Conditions
/// - `ReedError::DirectoryNotFound`: Base directory does not exist
///
/// ## Example Usage
/// ```rust
/// let files = discover_compressible_assets("public")?;
/// // Returns: ["public/style.css", "public/app.js", ...]
/// ```
pub fn discover_compressible_assets(base_dir: &str) -> ReedResult<Vec<PathBuf>> {
    let mut assets = Vec::new();
    discover_recursive(Path::new(base_dir), &mut assets)?;
    Ok(assets)
}

fn discover_recursive(dir: &Path, assets: &mut Vec<PathBuf>) -> ReedResult<()> {
    let entries = fs::read_dir(dir).map_err(|e| ReedError::DirectoryNotFound {
        path: dir.to_string_lossy().to_string(),
        reason: format!("Cannot read directory: {}", e),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ReedError::DirectoryNotFound {
            path: dir.to_string_lossy().to_string(),
            reason: format!("Cannot read entry: {}", e),
        })?;
        let path = entry.path();

        if path.is_dir() {
            discover_recursive(&path, assets)?;
        } else if is_compressible(&path) {
            assets.push(path);
        }
    }

    Ok(())
}

fn is_compressible(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("css")
            | Some("js")
            | Some("html")
            | Some("svg")
            | Some("json")
            | Some("txt")
            | Some("md")
    )
}

/// Pre-compresses a single asset with both gzip and brotli.
///
/// ## Input
/// - `path`: Path to asset file
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - Creates .gz and .br files alongside original
/// - Only compresses if original is newer than compressed versions
/// - Skips if compressed size > original size
///
/// ## Error Conditions
/// - `ReedError::FileNotFound`: Original file does not exist
/// - `ReedError::CompressionFailed`: Compression failed
/// - `ReedError::WriteError`: Cannot write compressed file
///
/// ## Example Usage
/// ```rust
/// precompress_asset(Path::new("public/app.js"))?;
/// // Creates: public/app.js.gz, public/app.js.br
/// ```
pub fn precompress_asset(path: &Path) -> ReedResult<()> {
    // Read original content
    let content = fs::read(path).map_err(|e| ReedError::FileNotFound {
        path: path.to_string_lossy().to_string(),
        reason: format!("Read failed: {}", e),
    })?;

    let original_size = content.len();

    // Check if compression is needed
    let gz_path = path.with_extension(format!(
        "{}.gz",
        path.extension().unwrap().to_str().unwrap()
    ));
    let br_path = path.with_extension(format!(
        "{}.br",
        path.extension().unwrap().to_str().unwrap()
    ));

    let needs_gzip = needs_compression(path, &gz_path)?;
    let needs_brotli = needs_compression(path, &br_path)?;

    // Compress with gzip
    if needs_gzip {
        let compressed_gz = compress_gzip(&content)?;
        if compressed_gz.len() < original_size {
            fs::write(&gz_path, compressed_gz).map_err(|e| ReedError::WriteError {
                path: gz_path.to_string_lossy().to_string(),
                reason: format!("Write failed: {}", e),
            })?;
        }
    }

    // Compress with brotli
    if needs_brotli {
        let compressed_br = compress_brotli(&content)?;
        if compressed_br.len() < original_size {
            fs::write(&br_path, compressed_br).map_err(|e| ReedError::WriteError {
                path: br_path.to_string_lossy().to_string(),
                reason: format!("Write failed: {}", e),
            })?;
        }
    }

    Ok(())
}

fn needs_compression(original: &Path, compressed: &Path) -> ReedResult<bool> {
    if !compressed.exists() {
        return Ok(true);
    }

    let original_meta = fs::metadata(original).map_err(|e| ReedError::FileNotFound {
        path: original.to_string_lossy().to_string(),
        reason: format!("Metadata read failed: {}", e),
    })?;

    let compressed_meta = fs::metadata(compressed).map_err(|e| ReedError::FileNotFound {
        path: compressed.to_string_lossy().to_string(),
        reason: format!("Metadata read failed: {}", e),
    })?;

    let original_mtime = original_meta
        .modified()
        .map_err(|e| ReedError::InvalidMetadata {
            reason: format!("Cannot read original mtime: {}", e),
        })?;

    let compressed_mtime = compressed_meta
        .modified()
        .map_err(|e| ReedError::InvalidMetadata {
            reason: format!("Cannot read compressed mtime: {}", e),
        })?;

    Ok(original_mtime > compressed_mtime)
}

/// Pre-compresses all assets in a directory.
///
/// ## Input
/// - `base_dir`: Base directory containing assets
///
/// ## Output
/// - `ReedResult<usize>`: Number of assets compressed
///
/// ## Performance
/// - Discovers all compressible files first
/// - Compresses each with gzip and brotli
/// - Skips already-compressed files (incremental)
///
/// ## Error Conditions
/// - `ReedError::DirectoryNotFound`: Base directory does not exist
/// - `ReedError::CompressionFailed`: Compression failed
///
/// ## Example Usage
/// ```rust
/// let count = precompress_all_assets("public")?;
/// println!("Compressed {} assets", count);
/// ```
pub fn precompress_all_assets(base_dir: &str) -> ReedResult<usize> {
    let assets = discover_compressible_assets(base_dir)?;
    let mut count = 0;

    for asset in &assets {
        precompress_asset(asset)?;
        count += 1;
    }

    Ok(count)
}

/// Cleans all pre-compressed files in a directory.
///
/// ## Input
/// - `base_dir`: Base directory containing assets
///
/// ## Output
/// - `ReedResult<usize>`: Number of compressed files deleted
///
/// ## Performance
/// - Discovers all .gz and .br files
/// - Deletes them
///
/// ## Error Conditions
/// - `ReedError::DirectoryNotFound`: Base directory does not exist
///
/// ## Example Usage
/// ```rust
/// let deleted = clean_precompressed_assets("public")?;
/// println!("Deleted {} compressed files", deleted);
/// ```
pub fn clean_precompressed_assets(base_dir: &str) -> ReedResult<usize> {
    let mut count = 0;
    clean_recursive(Path::new(base_dir), &mut count)?;
    Ok(count)
}

fn clean_recursive(dir: &Path, count: &mut usize) -> ReedResult<()> {
    let entries = fs::read_dir(dir).map_err(|e| ReedError::DirectoryNotFound {
        path: dir.to_string_lossy().to_string(),
        reason: format!("Cannot read directory: {}", e),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ReedError::DirectoryNotFound {
            path: dir.to_string_lossy().to_string(),
            reason: format!("Cannot read entry: {}", e),
        })?;
        let path = entry.path();

        if path.is_dir() {
            clean_recursive(&path, count)?;
        } else if is_precompressed(&path) {
            fs::remove_file(&path).map_err(|e| ReedError::WriteError {
                path: path.to_string_lossy().to_string(),
                reason: format!("Delete failed: {}", e),
            })?;
            *count += 1;
        }
    }

    Ok(())
}

fn is_precompressed(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.ends_with(".gz") || path_str.ends_with(".br")
}
