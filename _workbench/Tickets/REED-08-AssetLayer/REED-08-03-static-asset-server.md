# REED-08-03: Static Asset Server

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function (CONFIG and SYSTEM use identical logic)
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs` - every topic (auth, login,...) has its own scoped rs service file
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-08-03
- **Title**: Static Asset Server with Caching
- **Layer**: Asset Layer (REED-08)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-06-01

## Summary Reference
- **Section**: Static Asset Server
- **Lines**: 1031-1033 in project_summary.md
- **Key Concepts**: Static file serving, cache headers, ETags, compression, security headers

## Objective
Implement static asset server for serving CSS, JavaScript, images, fonts, and other files with proper cache headers, ETags for conditional requests, gzip/brotli compression, and security headers to prevent XSS and clickjacking.

## Requirements

### Asset Types Served
- **CSS**: .css files from public/css/
- **JavaScript**: .js files from public/js/
- **Images**: .png, .jpg, .gif, .svg, .webp from public/images/
- **Fonts**: .woff, .woff2, .ttf, .otf from public/fonts/
- **Documents**: .pdf, .doc, .docx from public/docs/
- **Other**: .ico, .txt, .xml, .json

### Asset Directory Structure
```
public/
├── css/
│   ├── knowledge.mouse.css
│   └── knowledge.mouse.css.map
├── js/
│   ├── knowledge.mouse.js
│   └── knowledge.mouse.js.map
├── images/
│   ├── logo.svg
│   ├── banner.jpg
│   └── icons/
│       ├── search.svg
│       └── menu.svg
├── fonts/
│   ├── inter.woff2
│   └── inter-bold.woff2
└── docs/
    └── manual.pdf
```

### Implementation (`src/reedcms/assets/server/static_server.rs`)

```rust
/// Serves static assets with caching and compression.
///
/// ## Features
/// - Automatic MIME type detection
/// - Cache-Control headers with long TTL
/// - ETag generation for conditional requests
/// - Gzip/Brotli compression
/// - Range request support (for large files)
/// - Security headers (X-Content-Type-Options, etc.)
///
/// ## Cache Strategy
/// - CSS/JS: max-age=31536000 (1 year) - with cache busting via filenames
/// - Images: max-age=2592000 (30 days)
/// - Fonts: max-age=31536000 (1 year) - fonts rarely change
/// - Documents: max-age=3600 (1 hour)
///
/// ## Performance
/// - File serving: < 5ms (cached in memory for small files)
/// - ETag comparison: < 1ms
/// - Compression: On-the-fly or pre-compressed
pub async fn serve_static_asset(req: HttpRequest) -> Result<HttpResponse, Error> {
    let path = req.path();

    // Extract asset path
    let asset_path = extract_asset_path(path)?;

    // Security check - prevent directory traversal
    if asset_path.contains("..") {
        return Ok(HttpResponse::Forbidden().body("Invalid path"));
    }

    // Build full file path
    let file_path = format!("public/{}", asset_path);

    // Check if file exists
    if !std::path::Path::new(&file_path).exists() {
        return Ok(HttpResponse::NotFound().body("Asset not found"));
    }

    // Load file metadata
    let metadata = std::fs::metadata(&file_path).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to read file metadata")
    })?;

    // Generate ETag
    let etag = generate_etag(&file_path, &metadata)?;

    // Check If-None-Match header for conditional request
    if let Some(if_none_match) = req.headers().get("If-None-Match") {
        if if_none_match.to_str().ok() == Some(&etag) {
            return Ok(HttpResponse::NotModified().finish());
        }
    }

    // Read file content
    let content = std::fs::read(&file_path).map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to read file")
    })?;

    // Detect MIME type
    let mime_type = detect_mime_type(&file_path);

    // Determine cache TTL
    let cache_ttl = get_cache_ttl(&file_path);

    // Check if compression is supported
    let accept_encoding = req
        .headers()
        .get("Accept-Encoding")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // Build response
    let mut response = HttpResponse::Ok();

    // Set content type
    response.insert_header(("Content-Type", mime_type));

    // Set cache headers
    response.insert_header(("Cache-Control", format!("public, max-age={}", cache_ttl)));
    response.insert_header(("ETag", etag));

    // Set security headers
    response.insert_header(("X-Content-Type-Options", "nosniff"));
    response.insert_header(("X-Frame-Options", "SAMEORIGIN"));

    // Compress if supported
    if should_compress(&file_path, &content) {
        if accept_encoding.contains("br") {
            // Brotli compression
            let compressed = compress_brotli(&content)?;
            response.insert_header(("Content-Encoding", "br"));
            return Ok(response.body(compressed));
        } else if accept_encoding.contains("gzip") {
            // Gzip compression
            let compressed = compress_gzip(&content)?;
            response.insert_header(("Content-Encoding", "gzip"));
            return Ok(response.body(compressed));
        }
    }

    // Serve uncompressed
    Ok(response.body(content))
}

/// Extracts asset path from request path.
///
/// ## Examples
/// - /css/knowledge.mouse.css → css/knowledge.mouse.css
/// - /images/logo.svg → images/logo.svg
fn extract_asset_path(path: &str) -> Result<String, Error> {
    let path = path.trim_start_matches('/');

    // Validate asset type
    let valid_prefixes = vec!["css/", "js/", "images/", "fonts/", "docs/"];

    if !valid_prefixes.iter().any(|prefix| path.starts_with(prefix)) {
        return Err(actix_web::error::ErrorBadRequest("Invalid asset path"));
    }

    Ok(path.to_string())
}

/// Generates ETag from file metadata.
///
/// ## ETag Format
/// "{mtime}-{size}"
///
/// ## Example
/// "1703001234-12345"
fn generate_etag(path: &str, metadata: &std::fs::Metadata) -> Result<String, Error> {
    use std::time::UNIX_EPOCH;

    let mtime = metadata
        .modified()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to get mtime"))?
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let size = metadata.len();

    Ok(format!("\"{}-{}\"", mtime, size))
}

/// Detects MIME type from file extension.
fn detect_mime_type(path: &str) -> &'static str {
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension {
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "pdf" => "application/pdf",
        "ico" => "image/x-icon",
        "xml" => "application/xml",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Gets cache TTL for asset type.
fn get_cache_ttl(path: &str) -> u32 {
    if path.ends_with(".css") || path.ends_with(".js") {
        31536000 // 1 year (with cache busting)
    } else if path.ends_with(".woff") || path.ends_with(".woff2") {
        31536000 // 1 year (fonts rarely change)
    } else if path.contains("/images/") {
        2592000 // 30 days
    } else {
        3600 // 1 hour (default)
    }
}

/// Checks if content should be compressed.
fn should_compress(path: &str, content: &[u8]) -> bool {
    // Only compress text-based assets
    let compressible_types = vec![".css", ".js", ".json", ".svg", ".xml", ".txt"];

    let is_compressible = compressible_types
        .iter()
        .any(|ext| path.ends_with(ext));

    // Only compress if size > 1KB
    is_compressible && content.len() > 1024
}
```

### Compression (`src/reedcms/assets/server/compression.rs`)

```rust
/// Compresses content with Gzip.
///
/// ## Compression Level
/// - Level 6 (default) - good balance of speed and compression
///
/// ## Performance
/// - Compression: ~5ms per 100KB
/// - Compression ratio: ~70% for text assets
pub fn compress_gzip(content: &[u8]) -> Result<Vec<u8>, Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(content)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Gzip compression failed"))?;

    encoder
        .finish()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Gzip finish failed"))
}

/// Compresses content with Brotli.
///
/// ## Compression Level
/// - Level 6 (default) - good balance of speed and compression
///
/// ## Performance
/// - Compression: ~8ms per 100KB
/// - Compression ratio: ~75% for text assets (better than gzip)
///
/// ## Browser Support
/// - Modern browsers only (Chrome 50+, Firefox 44+, Safari 11+)
pub fn compress_brotli(content: &[u8]) -> Result<Vec<u8>, Error> {
    use brotli::enc::BrotliEncoderParams;

    let mut output = Vec::new();
    let params = BrotliEncoderParams {
        quality: 6,
        ..Default::default()
    };

    brotli::BrotliCompress(
        &mut std::io::Cursor::new(content),
        &mut output,
        &params,
    )
    .map_err(|_| actix_web::error::ErrorInternalServerError("Brotli compression failed"))?;

    Ok(output)
}
```

### Pre-compression (`src/reedcms/assets/server/precompress.rs`)

```rust
/// Pre-compresses assets at build time.
///
/// ## Process
/// 1. Find all compressible assets in public/
/// 2. Generate .gz and .br versions
/// 3. Store alongside original files
///
/// ## Benefits
/// - No runtime compression overhead
/// - Better compression ratios (can use higher levels)
/// - Instant serving
///
/// ## Output
/// ```
/// public/css/knowledge.mouse.css
/// public/css/knowledge.mouse.css.gz
/// public/css/knowledge.mouse.css.br
/// ```
pub fn precompress_assets() -> ReedResult<PrecompressStats> {
    println!("🗜️  Pre-compressing assets...");

    let mut stats = PrecompressStats {
        total_files: 0,
        total_original_size: 0,
        total_gzip_size: 0,
        total_brotli_size: 0,
    };

    // Find all compressible files
    let files = find_compressible_files("public")?;

    for file_path in files {
        let content = std::fs::read(&file_path)?;
        let original_size = content.len();

        // Generate gzip version
        let gzip_content = compress_gzip(&content).map_err(|_| ReedError::CompressionError {
            algorithm: "gzip".to_string(),
            reason: "Gzip compression failed".to_string(),
        })?;
        let gzip_path = format!("{}.gz", file_path);
        std::fs::write(&gzip_path, &gzip_content)?;

        // Generate brotli version
        let brotli_content = compress_brotli(&content).map_err(|_| ReedError::CompressionError {
            algorithm: "brotli".to_string(),
            reason: "Brotli compression failed".to_string(),
        })?;
        let brotli_path = format!("{}.br", file_path);
        std::fs::write(&brotli_path, &brotli_content)?;

        stats.total_files += 1;
        stats.total_original_size += original_size;
        stats.total_gzip_size += gzip_content.len();
        stats.total_brotli_size += brotli_content.len();

        println!(
            "  ✓ {} ({} KB → gzip: {} KB, br: {} KB)",
            file_path,
            original_size / 1024,
            gzip_content.len() / 1024,
            brotli_content.len() / 1024
        );
    }

    let gzip_ratio = 100 - (stats.total_gzip_size * 100 / stats.total_original_size);
    let brotli_ratio = 100 - (stats.total_brotli_size * 100 / stats.total_original_size);

    println!("\n📊 Pre-compression Statistics:");
    println!("  Total files: {}", stats.total_files);
    println!("  Original size: {} KB", stats.total_original_size / 1024);
    println!("  Gzip size: {} KB (-{}%)", stats.total_gzip_size / 1024, gzip_ratio);
    println!("  Brotli size: {} KB (-{}%)", stats.total_brotli_size / 1024, brotli_ratio);

    Ok(stats)
}

/// Finds compressible files in directory.
fn find_compressible_files(dir: &str) -> ReedResult<Vec<String>> {
    let mut files = Vec::new();
    let compressible_extensions = vec!["css", "js", "json", "svg", "xml", "txt"];

    let entries = std::fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recurse into subdirectories
            files.extend(find_compressible_files(path.to_str().unwrap())?);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if compressible_extensions.contains(&ext) {
                files.push(path.display().to_string());
            }
        }
    }

    Ok(files)
}

/// Pre-compression statistics.
#[derive(Debug, Clone)]
pub struct PrecompressStats {
    pub total_files: usize,
    pub total_original_size: usize,
    pub total_gzip_size: usize,
    pub total_brotli_size: usize,
}
```

### Route Configuration (`src/reedcms/assets/server/routes.rs`)

```rust
/// Configures static asset routes for Actix-Web.
///
/// ## Routes
/// - /css/* → public/css/
/// - /js/* → public/js/
/// - /images/* → public/images/
/// - /fonts/* → public/fonts/
/// - /docs/* → public/docs/
/// - /favicon.ico → public/favicon.ico
pub fn configure_asset_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/css/{filename:.*}", web::get().to(serve_static_asset))
            .route("/js/{filename:.*}", web::get().to(serve_static_asset))
            .route("/images/{filename:.*}", web::get().to(serve_static_asset))
            .route("/fonts/{filename:.*}", web::get().to(serve_static_asset))
            .route("/docs/{filename:.*}", web::get().to(serve_static_asset))
            .route("/favicon.ico", web::get().to(serve_static_asset)),
    );
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/assets/server/static_server.rs` - Static asset server
- `src/reedcms/assets/server/compression.rs` - Compression utilities
- `src/reedcms/assets/server/precompress.rs` - Pre-compression
- `src/reedcms/assets/server/routes.rs` - Route configuration

### Test Files
- `src/reedcms/assets/server/static_server.test.rs`
- `src/reedcms/assets/server/compression.test.rs`
- `src/reedcms/assets/server/precompress.test.rs`
- `src/reedcms/assets/server/routes.test.rs`

## File Structure
```
src/reedcms/assets/server/
├── static_server.rs       # Static server
├── static_server.test.rs  # Server tests
├── compression.rs         # Compression
├── compression.test.rs    # Compression tests
├── precompress.rs         # Pre-compression
├── precompress.test.rs    # Pre-compress tests
├── routes.rs              # Route config
└── routes.test.rs         # Route tests
```

## Testing Requirements

### Unit Tests
- [ ] Test MIME type detection
- [ ] Test ETag generation
- [ ] Test cache TTL calculation
- [ ] Test gzip compression
- [ ] Test brotli compression
- [ ] Test path extraction
- [ ] Test security validation (directory traversal)

### Integration Tests
- [ ] Test asset serving with correct headers
- [ ] Test conditional requests with ETags
- [ ] Test compression negotiation
- [ ] Test pre-compressed file serving
- [ ] Test 404 for missing assets
- [ ] Test security header presence

### Performance Tests
- [ ] File serving: < 5ms
- [ ] ETag comparison: < 1ms
- [ ] Gzip compression: < 5ms per 100KB
- [ ] Brotli compression: < 8ms per 100KB
- [ ] Pre-compression: < 10s for 100 files

### Security Tests
- [ ] Test directory traversal prevention
- [ ] Test invalid path rejection
- [ ] Test security headers present
- [ ] Test MIME type sniffing prevention

## Acceptance Criteria
- [ ] Static assets served correctly
- [ ] MIME types detected automatically
- [ ] Cache headers set appropriately
- [ ] ETags generated for conditional requests
- [ ] Gzip compression working
- [ ] Brotli compression working
- [ ] Pre-compression implemented
- [ ] Security headers present
- [ ] Directory traversal prevented
- [ ] 404 handling for missing assets
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-06-01 (Server Foundation)

## Blocks
- None (final Asset Layer ticket)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1031-1033 in `project_summary.md`

## Notes
Static asset serving is critical for performance. Long cache TTLs (1 year for CSS/JS) combined with cache busting via filename changes enable optimal caching. ETags support conditional requests to reduce bandwidth. Compression (gzip/brotli) reduces file sizes by 70-75%. Pre-compression at build time eliminates runtime compression overhead. Security headers prevent XSS and clickjacking attacks. Directory traversal prevention protects file system.
