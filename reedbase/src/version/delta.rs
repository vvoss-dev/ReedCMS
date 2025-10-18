// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Binary delta compression for ReedBase versioning.
//!
//! Uses bsdiff for delta generation and bspatch for applying deltas.
//! Deltas are compressed with XZ for optimal storage efficiency.

use crate::error::{ReedError, ReedResult};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

/// Delta metadata.
#[derive(Debug, Clone)]
pub struct DeltaInfo {
    pub size: usize,
    pub original_size: usize,
    pub ratio: u8, // Percentage (0-100)
}

/// Generate binary delta from old version to new version.
///
/// ## Input
/// - `old_path`: Path to previous version CSV
/// - `new_path`: Path to new version CSV
/// - `delta_path`: Path to output delta file
///
/// ## Output
/// - `ReedResult<DeltaInfo>`: Delta metadata (size, compression ratio)
///
/// ## Performance
/// - O(n) where n = file size
/// - < 50ms for 100-row CSV (~10KB)
/// - < 500ms for 1000-row CSV (~100KB)
///
/// ## Error Conditions
/// - IoError: Cannot read old/new file or write delta
/// - DeltaGenerationFailed: bsdiff operation failed
///
/// ## Example Usage
/// ```no_run
/// use reedbase::version::generate_delta;
/// use std::path::Path;
///
/// let info = generate_delta(
///     Path::new("1736860800.csv"),
///     Path::new("current.csv"),
///     Path::new("1736860900.bsdiff")
/// )?;
/// println!("Delta size: {} bytes ({}% of original)", info.size, info.ratio);
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn generate_delta<P: AsRef<Path>>(
    old_path: P,
    new_path: P,
    delta_path: P,
) -> ReedResult<DeltaInfo> {
    let old_data = fs::read(old_path.as_ref()).map_err(|e| ReedError::IoError {
        operation: format!("read_old_file: {}", old_path.as_ref().display()),
        reason: e.to_string(),
    })?;

    let new_data = fs::read(new_path.as_ref()).map_err(|e| ReedError::IoError {
        operation: format!("read_new_file: {}", new_path.as_ref().display()),
        reason: e.to_string(),
    })?;

    let delta = create_bsdiff(&old_data, &new_data)?;
    let compressed = compress_delta(&delta)?;

    fs::write(delta_path.as_ref(), &compressed).map_err(|e| ReedError::IoError {
        operation: format!("write_delta: {}", delta_path.as_ref().display()),
        reason: e.to_string(),
    })?;

    let ratio = if new_data.is_empty() {
        0
    } else {
        ((compressed.len() as f64 / new_data.len() as f64) * 100.0) as u8
    };

    Ok(DeltaInfo {
        size: compressed.len(),
        original_size: new_data.len(),
        ratio,
    })
}

/// Apply binary delta to reconstruct version.
///
/// ## Input
/// - `old_path`: Path to base version CSV
/// - `delta_path`: Path to delta file
/// - `output_path`: Path to output reconstructed CSV
///
/// ## Output
/// - `ReedResult<()>`: Success or error
///
/// ## Performance
/// - O(n) where n = file size
/// - < 30ms for 100-row CSV
/// - < 300ms for 1000-row CSV
///
/// ## Error Conditions
/// - IoError: Cannot read base/delta or write output
/// - DeltaApplicationFailed: bspatch operation failed
/// - DecompressionFailed: Delta file corrupted
///
/// ## Example Usage
/// ```no_run
/// use reedbase::version::apply_delta;
/// use std::path::Path;
///
/// apply_delta(
///     Path::new("1736860800.csv"),
///     Path::new("1736860900.bsdiff"),
///     Path::new("reconstructed.csv")
/// )?;
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn apply_delta<P: AsRef<Path>>(old_path: P, delta_path: P, output_path: P) -> ReedResult<()> {
    let old_data = fs::read(old_path.as_ref()).map_err(|e| ReedError::IoError {
        operation: format!("read_base_file: {}", old_path.as_ref().display()),
        reason: e.to_string(),
    })?;

    let compressed = fs::read(delta_path.as_ref()).map_err(|e| ReedError::IoError {
        operation: format!("read_delta_file: {}", delta_path.as_ref().display()),
        reason: e.to_string(),
    })?;

    let delta = decompress_delta(&compressed)?;
    let new_data = apply_bspatch(&old_data, &delta)?;

    // Atomic write: temp file + rename
    let temp_path = output_path.as_ref().with_extension("tmp");
    fs::write(&temp_path, &new_data).map_err(|e| ReedError::IoError {
        operation: format!("write_temp_file: {}", temp_path.display()),
        reason: e.to_string(),
    })?;

    fs::rename(&temp_path, output_path.as_ref()).map_err(|e| ReedError::IoError {
        operation: format!("rename_output: {}", output_path.as_ref().display()),
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Create bsdiff binary delta.
///
/// ## Input
/// - `old_data`: Previous version data
/// - `new_data`: New version data
///
/// ## Output
/// - `ReedResult<Vec<u8>>`: Binary delta (uncompressed)
///
/// ## Performance
/// - O(n) where n = max(old_data.len(), new_data.len())
/// - < 40ms for 10KB files
///
/// ## Error Conditions
/// - DeltaGenerationFailed: bsdiff library error
fn create_bsdiff(old_data: &[u8], new_data: &[u8]) -> ReedResult<Vec<u8>> {
    let mut delta = Vec::new();

    bsdiff::diff(old_data, new_data, &mut delta).map_err(|e| ReedError::DeltaGenerationFailed {
        reason: format!("bsdiff error: {:?}", e),
    })?;

    Ok(delta)
}

/// Apply bspatch to reconstruct data.
///
/// ## Input
/// - `old_data`: Base version data
/// - `delta`: Binary delta (uncompressed)
///
/// ## Output
/// - `ReedResult<Vec<u8>>`: Reconstructed data
///
/// ## Performance
/// - O(n) where n = output size
/// - < 20ms for 10KB output
///
/// ## Error Conditions
/// - DeltaApplicationFailed: bspatch library error
fn apply_bspatch(old_data: &[u8], delta: &[u8]) -> ReedResult<Vec<u8>> {
    let mut new_data = Vec::new();
    let mut delta_reader = std::io::Cursor::new(delta);

    bsdiff::patch(old_data, &mut delta_reader, &mut new_data).map_err(|e| {
        ReedError::DeltaApplicationFailed {
            reason: format!("bspatch error: {:?}", e),
        }
    })?;

    Ok(new_data)
}

/// Compress delta using XZ.
///
/// ## Input
/// - `delta`: Uncompressed delta data
///
/// ## Output
/// - `ReedResult<Vec<u8>>`: Compressed delta
///
/// ## Performance
/// - O(n) where n = delta size
/// - < 10ms for typical deltas (< 1KB)
/// - Compression ratio: ~30-50% of uncompressed
///
/// ## Error Conditions
/// - CompressionFailed: XZ compression error
fn compress_delta(delta: &[u8]) -> ReedResult<Vec<u8>> {
    use xz2::write::XzEncoder;

    let mut encoder = XzEncoder::new(Vec::new(), 6);
    encoder
        .write_all(delta)
        .map_err(|e| ReedError::CompressionFailed {
            reason: format!("XZ write error: {}", e),
        })?;

    encoder.finish().map_err(|e| ReedError::CompressionFailed {
        reason: format!("XZ finish error: {}", e),
    })
}

/// Decompress delta using XZ.
///
/// ## Input
/// - `compressed`: Compressed delta data
///
/// ## Output
/// - `ReedResult<Vec<u8>>`: Uncompressed delta
///
/// ## Performance
/// - O(n) where n = compressed size
/// - < 5ms for typical deltas
///
/// ## Error Conditions
/// - DecompressionFailed: XZ decompression error
fn decompress_delta(compressed: &[u8]) -> ReedResult<Vec<u8>> {
    use xz2::read::XzDecoder;

    let mut decoder = XzDecoder::new(compressed);
    let mut delta = Vec::new();

    decoder
        .read_to_end(&mut delta)
        .map_err(|e| ReedError::DecompressionFailed {
            reason: format!("XZ read error: {}", e),
        })?;

    Ok(delta)
}

/// Calculate delta size savings.
///
/// ## Input
/// - `delta_size`: Size of delta file in bytes
/// - `full_size`: Size of full version in bytes
///
/// ## Output
/// - `f64`: Percentage saved (0.0 to 100.0)
///
/// ## Performance
/// - O(1) operation
/// - < 1μs
///
/// ## Example Usage
/// ```
/// use reedbase::version::calculate_savings;
///
/// let saved = calculate_savings(500, 10000);
/// assert_eq!(saved, 95.0); // 95% savings
/// ```
pub fn calculate_savings(delta_size: usize, full_size: usize) -> f64 {
    if full_size == 0 {
        return 0.0;
    }
    ((full_size - delta_size) as f64 / full_size as f64) * 100.0
}
