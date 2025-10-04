// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Binary compiler for release builds.
//!
//! This module compiles optimised release binaries with LTO, symbol stripping,
//! and optional UPX compression.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use serde::Serialize;
use std::process::Command;

/// Compiles ReedCMS binary for release.
///
/// ## Build Process
/// 1. Clean previous builds
/// 2. Set release flags
/// 3. Compile with cargo
/// 4. Calculate checksums
/// 5. Generate build info
///
/// ## Optimisations
/// - LTO (Link-Time Optimisation): Reduces binary size ~20%
/// - Codegen units = 1: Better optimisation
/// - Strip symbols: Reduces size ~40%
/// - UPX compression: Optional, reduces size ~60%
///
/// ## Performance
/// - Compile time: 2-5 minutes (release build)
/// - Binary size: ~15MB (stripped)
/// - Binary size: ~6MB (UPX compressed)
///
/// ## Error Conditions
/// - `ReedError::BuildError`: Cargo build failed
/// - `ReedError::FileNotFound`: Binary not found after compilation
///
/// ## Example Usage
/// ```rust
/// let build_info = build_release()?;
/// println!("Built v{}", build_info.version);
/// ```
pub fn build_release() -> ReedResult<BuildInfo> {
    println!("🔨 Building ReedCMS v{}...", env!("CARGO_PKG_VERSION"));

    let start_time = std::time::Instant::now();

    // 1. Clean previous builds
    clean_previous_builds()?;

    // 2. Run cargo build --release
    run_cargo_build()?;

    // 3. Get binary path
    let binary_path = "target/release/reedcms";

    if !std::path::Path::new(binary_path).exists() {
        return Err(ReedError::BuildError {
            component: "compiler".to_string(),
            reason: "Binary not found after compilation".to_string(),
        });
    }

    // 4. Get binary size
    let metadata = std::fs::metadata(binary_path).map_err(|e| ReedError::FileNotFound {
        path: binary_path.to_string(),
        reason: format!("Cannot read binary metadata: {}", e),
    })?;
    let binary_size = metadata.len() as usize;

    println!("✓ Compilation complete ({:?})", start_time.elapsed());
    println!(
        "📦 Binary: {} ({:.1} MB)",
        binary_path,
        binary_size as f64 / 1_048_576.0
    );

    // 5. Optional: Compress with UPX
    let compressed_size = if should_use_upx() {
        match compress_with_upx(binary_path) {
            Ok(size) => {
                let reduction = 100 - (size * 100 / binary_size);
                println!(
                    "✓ Compressed: {} ({:.1} MB, -{}%)",
                    binary_path,
                    size as f64 / 1_048_576.0,
                    reduction
                );
                Some(size)
            }
            Err(e) => {
                eprintln!("⚠ UPX compression failed: {:?}", e);
                None
            }
        }
    } else {
        None
    };

    // 6. Calculate checksums
    let sha256 = calculate_sha256(binary_path)?;
    let md5 = calculate_md5(binary_path)?;

    println!("🔐 SHA256: {}", sha256);
    println!("🔐 MD5: {}", md5);

    // 7. Generate build info
    let build_info = BuildInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        binary_path: binary_path.to_string(),
        original_size: binary_size,
        compressed_size,
        sha256,
        md5,
        build_time: chrono::Utc::now().to_rfc3339(),
        build_duration_secs: start_time.elapsed().as_secs(),
    };

    // 8. Write build info to file
    write_build_info(&build_info)?;

    println!("✓ Build complete");

    Ok(build_info)
}

/// Cleans previous build artefacts.
///
/// ## Input
/// - None
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - < 5s for typical project
///
/// ## Error Conditions
/// - `ReedError::BuildError`: Cargo clean command failed
fn clean_previous_builds() -> ReedResult<()> {
    println!("🧹 Cleaning previous builds...");

    let output =
        Command::new("cargo")
            .arg("clean")
            .output()
            .map_err(|e| ReedError::BuildError {
                component: "cargo_clean".to_string(),
                reason: e.to_string(),
            })?;

    if !output.status.success() {
        return Err(ReedError::BuildError {
            component: "cargo_clean".to_string(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    Ok(())
}

/// Runs cargo build with release profile.
///
/// ## Input
/// - None
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Performance
/// - 2-5 minutes depending on system
///
/// ## Error Conditions
/// - `ReedError::BuildError`: Compilation failed
fn run_cargo_build() -> ReedResult<()> {
    println!("  Compiling with --release");
    println!("  LTO: enabled");
    println!("  Codegen units: 1");
    println!("  Strip: enabled");

    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--lib")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .output()
        .map_err(|e| ReedError::BuildError {
            component: "cargo_build".to_string(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(ReedError::BuildError {
            component: "cargo_build".to_string(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    Ok(())
}

/// Checks if UPX compression should be used.
///
/// ## Input
/// - None
///
/// ## Output
/// - `bool`: true if UPX is available
///
/// ## Performance
/// - < 10ms
pub(crate) fn should_use_upx() -> bool {
    Command::new("upx").arg("--version").output().is_ok()
}

/// Compresses binary with UPX.
///
/// ## Input
/// - `binary_path`: Path to binary file
///
/// ## Output
/// - `ReedResult<usize>`: Compressed file size in bytes
///
/// ## Performance
/// - 10-30s depending on binary size
///
/// ## Error Conditions
/// - `ReedError::BuildError`: UPX compression failed
fn compress_with_upx(binary_path: &str) -> ReedResult<usize> {
    println!("🗜️  Compressing with UPX...");

    let output = Command::new("upx")
        .arg("--best")
        .arg("--lzma")
        .arg(binary_path)
        .output()
        .map_err(|e| ReedError::BuildError {
            component: "upx".to_string(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(ReedError::BuildError {
            component: "upx".to_string(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let metadata = std::fs::metadata(binary_path).map_err(|e| ReedError::FileNotFound {
        path: binary_path.to_string(),
        reason: format!("Cannot read compressed binary metadata: {}", e),
    })?;

    Ok(metadata.len() as usize)
}

/// Calculates SHA256 checksum of binary.
///
/// ## Input
/// - `path`: Path to file
///
/// ## Output
/// - `ReedResult<String>`: SHA256 hex string
///
/// ## Performance
/// - < 100ms for 15MB binary
///
/// ## Error Conditions
/// - `ReedError::FileNotFound`: File does not exist
pub(crate) fn calculate_sha256(path: &str) -> ReedResult<String> {
    use sha2::{Digest, Sha256};

    let content = std::fs::read(path).map_err(|e| ReedError::FileNotFound {
        path: path.to_string(),
        reason: format!("Cannot read file: {}", e),
    })?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Calculates MD5 checksum of binary.
///
/// ## Input
/// - `path`: Path to file
///
/// ## Output
/// - `ReedResult<String>`: MD5 hex string
///
/// ## Performance
/// - < 50ms for 15MB binary
///
/// ## Error Conditions
/// - `ReedError::FileNotFound`: File does not exist
pub(crate) fn calculate_md5(path: &str) -> ReedResult<String> {
    let content = std::fs::read(path).map_err(|e| ReedError::FileNotFound {
        path: path.to_string(),
        reason: format!("Cannot read file: {}", e),
    })?;

    let digest = md5::compute(&content);
    Ok(format!("{:x}", digest))
}

/// Build information structure.
#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    pub version: String,
    pub binary_path: String,
    pub original_size: usize,
    pub compressed_size: Option<usize>,
    pub sha256: String,
    pub md5: String,
    pub build_time: String,
    pub build_duration_secs: u64,
}

/// Writes build info to JSON file.
///
/// ## Input
/// - `info`: Build information
///
/// ## Output
/// - `ReedResult<()>`: Success indicator
///
/// ## Error Conditions
/// - `ReedError::SerializationError`: JSON serialisation failed
/// - `ReedError::WriteError`: File write failed
fn write_build_info(info: &BuildInfo) -> ReedResult<()> {
    let json = serde_json::to_string_pretty(info).map_err(|e| ReedError::IoError {
        operation: "serialize".to_string(),
        path: "build-info.json".to_string(),
        reason: e.to_string(),
    })?;

    std::fs::write("target/release/build-info.json", json).map_err(|e| ReedError::WriteError {
        path: "target/release/build-info.json".to_string(),
        reason: e.to_string(),
    })
}
