// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Release packager for distribution.
//!
//! This module packages release binaries with all necessary assets for deployment.

use crate::reedcms::build::compiler::BuildInfo;
use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::path::Path;

/// Packages release binary with assets.
///
/// ## Package Contents
/// - reedcms binary
/// - .reed/ directory (config templates)
/// - templates/ directory (layout templates)
/// - README.md
/// - LICENSE
/// - CHANGELOG.md
///
/// ## Package Format
/// - tar.gz for Linux/macOS
/// - zip for Windows
///
/// ## Performance
/// - < 30s for typical project
///
/// ## Error Conditions
/// - `ReedError::DirectoryNotFound`: Required directory not found
/// - `ReedError::IoError`: Archive creation failed
///
/// ## Example Usage
/// ```rust
/// let package_info = package_release(&build_info)?;
/// println!("Package: {}", package_info.archive_path);
/// ```
pub fn package_release(build_info: &BuildInfo) -> ReedResult<PackageInfo> {
    println!("📦 Packaging ReedCMS v{}...", build_info.version);

    let package_name = format!(
        "reedcms-v{}-{}-{}",
        build_info.version,
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let package_dir = format!("target/release/{}", package_name);

    // 1. Create package directory
    std::fs::create_dir_all(&package_dir).map_err(|e| ReedError::IoError {
        operation: "create_dir".to_string(),
        path: package_dir.clone(),
        reason: e.to_string(),
    })?;

    // 2. Copy binary
    let final_size = build_info
        .compressed_size
        .unwrap_or(build_info.original_size);
    println!(
        "  Adding binary: reedcms ({:.1} MB)",
        final_size as f64 / 1_048_576.0
    );
    std::fs::copy(&build_info.binary_path, format!("{}/reedcms", package_dir)).map_err(|e| {
        ReedError::IoError {
            operation: "copy_binary".to_string(),
            path: build_info.binary_path.clone(),
            reason: e.to_string(),
        }
    })?;

    // 3. Copy config templates
    if Path::new(".reed").exists() {
        println!("  Adding configs: .reed/");
        copy_dir_recursive(".reed", &format!("{}/.reed", package_dir))?;
    }

    // 4. Copy templates
    if Path::new("templates").exists() {
        println!("  Adding templates: templates/");
        copy_dir_recursive("templates", &format!("{}/templates", package_dir))?;
    }

    // 5. Copy documentation
    println!("  Adding docs: README.md, LICENSE, CHANGELOG.md");
    let _ = std::fs::copy("README.md", format!("{}/README.md", package_dir));
    let _ = std::fs::copy("LICENSE", format!("{}/LICENSE", package_dir));
    let _ = std::fs::copy("CHANGELOG.md", format!("{}/CHANGELOG.md", package_dir));

    // 6. Create archive
    let archive_path = create_tar_gz_archive(&package_name, &package_dir)?;

    let archive_size = std::fs::metadata(&archive_path)
        .map_err(|e| ReedError::FileNotFound {
            path: archive_path.clone(),
            reason: format!("Cannot read archive metadata: {}", e),
        })?
        .len() as usize;

    println!(
        "✓ Package created: {} ({:.1} MB)",
        archive_path,
        archive_size as f64 / 1_048_576.0
    );

    // 7. Calculate archive checksum
    let sha256 = calculate_archive_sha256(&archive_path)?;

    Ok(PackageInfo {
        package_name,
        archive_path,
        archive_size,
        sha256,
    })
}

/// Recursively copies directory.
///
/// ## Input
/// - `src`: Source directory path
/// - `dst`: Destination directory path
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - < 5s for 1000 files
///
/// ## Error Conditions
/// - `ReedError::DirectoryNotFound`: Source directory not found
/// - `ReedError::IoError`: Copy operation failed
pub(crate) fn copy_dir_recursive(src: &str, dst: &str) -> ReedResult<()> {
    std::fs::create_dir_all(dst).map_err(|e| ReedError::IoError {
        operation: "create_dir".to_string(),
        path: dst.to_string(),
        reason: e.to_string(),
    })?;

    for entry in std::fs::read_dir(src).map_err(|e| ReedError::DirectoryNotFound {
        path: src.to_string(),
        reason: format!("Cannot read directory: {}", e),
    })? {
        let entry = entry.map_err(|e| ReedError::DirectoryNotFound {
            path: src.to_string(),
            reason: format!("Cannot read entry: {}", e),
        })?;

        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let dst_path = format!(
            "{}/{}",
            dst.trim_end_matches('/'),
            file_name.to_string_lossy()
        );

        if path.is_dir() {
            copy_dir_recursive(path.to_str().unwrap(), &dst_path)?;
        } else {
            std::fs::copy(&path, &dst_path).map_err(|e| ReedError::IoError {
                operation: "copy_file".to_string(),
                path: path.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;
        }
    }

    Ok(())
}

/// Creates tar.gz archive.
///
/// ## Input
/// - `package_name`: Package name (used as archive filename)
/// - `package_dir`: Directory to archive
///
/// ## Output
/// - `ReedResult<String>`: Archive file path
///
/// ## Performance
/// - < 10s for typical project
///
/// ## Error Conditions
/// - `ReedError::IoError`: Archive creation failed
pub(crate) fn create_tar_gz_archive(package_name: &str, package_dir: &str) -> ReedResult<String> {
    let archive_path = format!("target/release/{}.tar.gz", package_name);

    let tar_gz = std::fs::File::create(&archive_path).map_err(|e| ReedError::IoError {
        operation: "create_archive".to_string(),
        path: archive_path.clone(),
        reason: e.to_string(),
    })?;

    let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    tar.append_dir_all(package_name, package_dir)
        .map_err(|e| ReedError::IoError {
            operation: "tar_append".to_string(),
            path: archive_path.clone(),
            reason: e.to_string(),
        })?;

    tar.finish().map_err(|e| ReedError::IoError {
        operation: "tar_finish".to_string(),
        path: archive_path.clone(),
        reason: e.to_string(),
    })?;

    Ok(archive_path)
}

/// Calculates SHA256 checksum of archive.
///
/// ## Input
/// - `path`: Path to archive file
///
/// ## Output
/// - `ReedResult<String>`: SHA256 hex string
///
/// ## Performance
/// - < 200ms for 10MB archive
///
/// ## Error Conditions
/// - `ReedError::FileNotFound`: Archive file not found
pub(crate) fn calculate_archive_sha256(path: &str) -> ReedResult<String> {
    use sha2::{Digest, Sha256};

    let content = std::fs::read(path).map_err(|e| ReedError::FileNotFound {
        path: path.to_string(),
        reason: format!("Cannot read archive: {}", e),
    })?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Package information structure.
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub package_name: String,
    pub archive_path: String,
    pub archive_size: usize,
    pub sha256: String,
}
