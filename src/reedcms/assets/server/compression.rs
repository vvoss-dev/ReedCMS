// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Asset compression utilities for gzip and brotli compression.
//!
//! This module provides efficient compression for static assets with configurable
//! compression levels. Supports both runtime and build-time compression strategies.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use flate2::write::GzEncoder;
use flate2::Compression as GzipCompression;
use std::io::Write;

/// Compresses data using gzip algorithm.
///
/// ## Input
/// - `data`: Raw bytes to compress
///
/// ## Output
/// - `ReedResult<Vec<u8>>`: Compressed data
///
/// ## Performance
/// - Level 6 compression (balanced speed/size)
/// - ~60-70% reduction for text assets
/// - ~20-30% reduction for binary assets
///
/// ## Error Conditions
/// - `ReedError::CompressionFailed`: Compression algorithm failed
///
/// ## Example Usage
/// ```rust
/// let compressed = compress_gzip(b"console.log('Hello');")?;
/// ```
pub fn compress_gzip(data: &[u8]) -> ReedResult<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), GzipCompression::default());
    encoder
        .write_all(data)
        .map_err(|e| ReedError::CompressionFailed {
            reason: format!("Gzip write failed: {}", e),
        })?;
    encoder.finish().map_err(|e| ReedError::CompressionFailed {
        reason: format!("Gzip finish failed: {}", e),
    })
}

/// Compresses data using Brotli algorithm.
///
/// ## Input
/// - `data`: Raw bytes to compress
///
/// ## Output
/// - `ReedResult<Vec<u8>>`: Compressed data
///
/// ## Performance
/// - Quality 6 compression (balanced speed/size)
/// - ~65-75% reduction for text assets (better than gzip)
/// - ~25-35% reduction for binary assets
/// - Slower than gzip but better compression
///
/// ## Error Conditions
/// - `ReedError::CompressionFailed`: Compression algorithm failed
///
/// ## Example Usage
/// ```rust
/// let compressed = compress_brotli(b"body { margin: 0; }")?;
/// ```
pub fn compress_brotli(data: &[u8]) -> ReedResult<Vec<u8>> {
    let mut output = Vec::new();
    let mut compressor = brotli::CompressorWriter::new(&mut output, 4096, 6, 22);
    compressor
        .write_all(data)
        .map_err(|e| ReedError::CompressionFailed {
            reason: format!("Brotli write failed: {}", e),
        })?;
    compressor
        .flush()
        .map_err(|e| ReedError::CompressionFailed {
            reason: format!("Brotli flush failed: {}", e),
        })?;
    drop(compressor);
    Ok(output)
}

/// Determines best compression method based on Accept-Encoding header.
///
/// ## Input
/// - `accept_encoding`: Accept-Encoding header value from HTTP request
///
/// ## Output
/// - `Option<CompressionMethod>`: Best supported method or None
///
/// ## Performance
/// - O(1) string comparison
/// - Prioritizes Brotli > Gzip > None
///
/// ## Example Usage
/// ```rust
/// let method = get_compression_method("gzip, deflate, br");
/// assert_eq!(method, Some(CompressionMethod::Brotli));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionMethod {
    Gzip,
    Brotli,
}

pub fn get_compression_method(accept_encoding: &str) -> Option<CompressionMethod> {
    let lower = accept_encoding.to_lowercase();
    if lower.contains("br") {
        Some(CompressionMethod::Brotli)
    } else if lower.contains("gzip") {
        Some(CompressionMethod::Gzip)
    } else {
        None
    }
}

/// Compresses data using the specified compression method.
///
/// ## Input
/// - `data`: Raw bytes to compress
/// - `method`: Compression method to use
///
/// ## Output
/// - `ReedResult<Vec<u8>>`: Compressed data
///
/// ## Performance
/// - Delegates to compress_gzip() or compress_brotli()
///
/// ## Error Conditions
/// - `ReedError::CompressionFailed`: Compression failed
///
/// ## Example Usage
/// ```rust
/// let compressed = compress_with_method(data, CompressionMethod::Brotli)?;
/// ```
pub fn compress_with_method(data: &[u8], method: CompressionMethod) -> ReedResult<Vec<u8>> {
    match method {
        CompressionMethod::Gzip => compress_gzip(data),
        CompressionMethod::Brotli => compress_brotli(data),
    }
}
