// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Static asset server with ETag caching and compression support.
//!
//! This module provides HTTP asset serving with conditional requests,
//! automatic compression, and security headers.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use crate::reedcms::assets::server::compression::{compress_with_method, get_compression_method, CompressionMethod};
use actix_web::{HttpRequest, HttpResponse};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Generates ETag from file metadata (modification time + size).
///
/// ## Input
/// - `path`: Path to file
///
/// ## Output
/// - `ReedResult<String>`: ETag value (quoted hex string)
///
/// ## Performance
/// - O(1) metadata read
/// - No file content hashing (fast)
///
/// ## Error Conditions
/// - `ReedError::FileNotFound`: File does not exist
/// - `ReedError::InvalidMetadata`: Cannot read file metadata
///
/// ## Example Usage
/// ```rust
/// let etag = generate_etag("public/style.css")?;
/// // Returns: "\"1a2b3c4d5e6f7890\""
/// ```
pub fn generate_etag(path: &Path) -> ReedResult<String> {
    let metadata = fs::metadata(path).map_err(|e| ReedError::FileNotFound {
        path: path.to_string_lossy().to_string(),
        reason: format!("Metadata read failed: {}", e),
    })?;

    let mtime = metadata
        .modified()
        .map_err(|e| ReedError::InvalidMetadata {
            reason: format!("Cannot read mtime: {}", e),
        })?
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| ReedError::InvalidMetadata {
            reason: format!("Invalid mtime: {}", e),
        })?
        .as_secs();

    let size = metadata.len();
    let etag = format!("\"{:x}{:x}\"", mtime, size);
    Ok(etag)
}

/// Detects MIME type from file extension.
///
/// ## Input
/// - `path`: Path to file
///
/// ## Output
/// - `&str`: MIME type string
///
/// ## Performance
/// - O(1) extension lookup
///
/// ## Example Usage
/// ```rust
/// assert_eq!(detect_mime_type(Path::new("style.css")), "text/css");
/// assert_eq!(detect_mime_type(Path::new("logo.svg")), "image/svg+xml");
/// ```
pub fn detect_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("html") => "text/html",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain",
        Some("md") => "text/markdown",
        _ => "application/octet-stream",
    }
}

/// Determines Cache-Control header based on file type.
///
/// ## Input
/// - `path`: Path to file
///
/// ## Output
/// - `&str`: Cache-Control header value
///
/// ## Performance
/// - O(1) extension lookup
///
/// ## Example Usage
/// ```rust
/// // CSS/JS: 1 year cache
/// assert_eq!(get_cache_control(Path::new("app.css")), "public, max-age=31536000, immutable");
/// // Images: 30 days cache
/// assert_eq!(get_cache_control(Path::new("logo.png")), "public, max-age=2592000");
/// ```
pub fn get_cache_control(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("css") | Some("js") => "public, max-age=31536000, immutable",
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("svg") => {
            "public, max-age=2592000"
        }
        Some("woff") | Some("woff2") | Some("ttf") | Some("otf") => "public, max-age=31536000",
        _ => "public, max-age=86400",
    }
}

/// Prevents directory traversal attacks by validating path.
///
/// ## Input
/// - `requested_path`: User-provided path from URL
/// - `base_dir`: Base directory for static assets
///
/// ## Output
/// - `ReedResult<PathBuf>`: Validated absolute path
///
/// ## Performance
/// - O(1) canonicalization
///
/// ## Error Conditions
/// - `ReedError::SecurityViolation`: Path traversal attempt detected
///
/// ## Example Usage
/// ```rust
/// // Safe: public/style.css
/// validate_path("style.css", "public")?;
/// // Unsafe: ../etc/passwd
/// validate_path("../etc/passwd", "public")?; // Error
/// ```
pub fn validate_path(requested_path: &str, base_dir: &str) -> ReedResult<PathBuf> {
    let base = PathBuf::from(base_dir);
    let requested = base.join(requested_path);

    let canonical_base = base.canonicalize().map_err(|e| ReedError::SecurityViolation {
        reason: format!("Invalid base directory: {}", e),
    })?;

    let canonical_requested = requested
        .canonicalize()
        .map_err(|e| ReedError::FileNotFound {
            path: requested_path.to_string(),
            reason: format!("Path not found: {}", e),
        })?;

    if !canonical_requested.starts_with(&canonical_base) {
        return Err(ReedError::SecurityViolation {
            reason: format!("Path traversal attempt: {}", requested_path),
        });
    }

    Ok(canonical_requested)
}

/// Serves static asset with compression and caching support.
///
/// ## Input
/// - `req`: Actix-Web HTTP request
/// - `file_path`: Path to static file
/// - `base_dir`: Base directory for assets
///
/// ## Output
/// - `ReedResult<HttpResponse>`: HTTP response with asset content
///
/// ## Performance
/// - Checks If-None-Match for 304 responses (no content transfer)
/// - Compresses on-the-fly based on Accept-Encoding
/// - Sets long-lived Cache-Control headers
///
/// ## Error Conditions
/// - `ReedError::FileNotFound`: File does not exist
/// - `ReedError::SecurityViolation`: Path traversal attempt
/// - `ReedError::CompressionFailed`: Compression failed
///
/// ## Example Usage
/// ```rust
/// async fn serve_css(req: HttpRequest) -> HttpResponse {
///     serve_static_asset(&req, "bundle.css", "public").await
/// }
/// ```
pub async fn serve_static_asset(
    req: &HttpRequest,
    file_path: &str,
    base_dir: &str,
) -> ReedResult<HttpResponse> {
    // Validate path security
    let full_path = validate_path(file_path, base_dir)?;

    // Generate ETag
    let etag = generate_etag(&full_path)?;

    // Check If-None-Match for 304 response
    if let Some(if_none_match) = req.headers().get("If-None-Match") {
        if let Ok(header_etag) = if_none_match.to_str() {
            if header_etag == etag {
                return Ok(HttpResponse::NotModified()
                    .insert_header(("ETag", etag))
                    .finish());
            }
        }
    }

    // Read file content
    let content = fs::read(&full_path).map_err(|e| ReedError::FileNotFound {
        path: full_path.to_string_lossy().to_string(),
        reason: format!("Read failed: {}", e),
    })?;

    // Detect MIME type
    let mime_type = detect_mime_type(&full_path);

    // Determine compression method
    let accept_encoding = req
        .headers()
        .get("Accept-Encoding")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let compression_method = get_compression_method(accept_encoding);

    // Build response
    let mut response = HttpResponse::Ok();

    // Set headers
    response.insert_header(("Content-Type", mime_type));
    response.insert_header(("ETag", etag));
    response.insert_header(("Cache-Control", get_cache_control(&full_path)));
    response.insert_header(("X-Content-Type-Options", "nosniff"));
    response.insert_header(("X-Frame-Options", "DENY"));

    // Apply compression if supported
    if let Some(method) = compression_method {
        let compressed = compress_with_method(&content, method)?;
        let encoding = match method {
            CompressionMethod::Gzip => "gzip",
            CompressionMethod::Brotli => "br",
        };
        response.insert_header(("Content-Encoding", encoding));
        Ok(response.body(compressed))
    } else {
        Ok(response.body(content))
    }
}
