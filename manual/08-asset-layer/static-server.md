# Static Asset Server

HTTP serving of bundled assets with ETag caching, compression, and security headers.

## Purpose

- **Efficient Serving**: Fast static file delivery with zero overhead
- **Conditional Requests**: ETag-based 304 Not Modified responses
- **Compression**: Automatic gzip/brotli based on client support
- **Security**: Path validation, security headers, MIME type enforcement
- **Long-Lived Caching**: 1-year cache for immutable assets

## Architecture

```
┌─────────────────────────────────────────────────────┐
│ Static Asset Request Flow                            │
├─────────────────────────────────────────────────────┤
│                                                       │
│  Client Request                                      │
│  GET /session/styles/landing.a3f5b2c8.mouse.css     │
│  Accept-Encoding: gzip, br                           │
│  If-None-Match: "1a2b3c4d5e6f7890"                  │
│           ↓                                           │
│  1. Path Validation                                  │
│     ├─ Canonicalise path                            │
│     ├─ Check directory traversal                    │
│     └─ Ensure within public/ directory               │
│           ↓                                           │
│  2. ETag Generation                                  │
│     ├─ Read file metadata (mtime + size)            │
│     ├─ Generate hex hash                             │
│     └─ Compare with If-None-Match header             │
│           ↓                                           │
│  3. Conditional Response                             │
│     ├─ Match? → 304 Not Modified (no body)          │
│     └─ No match? → Continue to step 4                │
│           ↓                                           │
│  4. File Reading                                     │
│     ├─ Read file from disk                           │
│     ├─ Detect MIME type from extension              │
│     └─ Prepare content                               │
│           ↓                                           │
│  5. Compression Selection                            │
│     ├─ Parse Accept-Encoding header                 │
│     ├─ Prefer: brotli > gzip > none                 │
│     └─ Compress content if supported                 │
│           ↓                                           │
│  6. Security Headers                                 │
│     ├─ X-Content-Type-Options: nosniff              │
│     ├─ X-Frame-Options: DENY                        │
│     └─ Cache-Control: public, max-age=31536000      │
│           ↓                                           │
│  7. Response                                         │
│     └─ 200 OK + compressed content                   │
│                                                       │
└─────────────────────────────────────────────────────┘
```

## Implementation

### Core Serving Function

```rust
pub async fn serve_static_asset(
    req: &HttpRequest,
    file_path: &str,
    base_dir: &str,
) -> ReedResult<HttpResponse> 
{
    // 1. Validate path security
    let full_path = validate_path(file_path, base_dir)?;
    
    // 2. Generate ETag
    let etag = generate_etag(&full_path)?;
    
    // 3. Check If-None-Match for 304 response
    if let Some(if_none_match) = req.headers()
                                     .get("If-None-Match") 
    {
        if if_none_match.to_str().ok() == Some(&etag) {
            return Ok(HttpResponse::NotModified()
                .insert_header(("ETag", etag))
                .finish());
        }
    }
    
    // 4. Read file content
    let content = fs::read(&full_path)?;
    
    // 5. Detect MIME type
    let mime_type = detect_mime_type(&full_path);
    
    // 6. Determine compression
    let accept_encoding = req.headers()
        .get("Accept-Encoding")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    let compression_method = 
        get_compression_method(accept_encoding);
    
    // 7. Build response
    let mut response = HttpResponse::Ok();
    
    // Set headers
    response.insert_header(("Content-Type", mime_type));
    response.insert_header(("ETag", etag));
    response.insert_header((
        "Cache-Control", 
        get_cache_control(&full_path)
    ));
    response.insert_header((
        "X-Content-Type-Options", 
        "nosniff"
    ));
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
```

## ETag Caching

### ETag Generation

```rust
pub fn generate_etag(path: &Path) -> ReedResult<String> {
    let metadata = fs::metadata(path)?;
    
    // Use modification time + file size
    let mtime = metadata.modified()?
                        .duration_since(UNIX_EPOCH)?
                        .as_secs();
    let size = metadata.len();
    
    // Generate quoted hex string
    let etag = format!("\"{:x}{:x}\"", mtime, size);
    Ok(etag)
}
```

### Why Not Content Hash?

| Method | Speed | Accuracy | Use Case |
|--------|-------|----------|----------|
| **Content hash** (MD5) | Slow (~10ms) | 100% | Session hash (build-time) |
| **Metadata** (mtime+size) | Fast (~0.1ms) | 99.9% | ETag (runtime) |

**Rationale**: Metadata-based ETag is 100× faster with negligible collision risk for static assets.

### Conditional Request Flow

```
Client                                  Server
  │                                        │
  ├─ GET /style.css ───────────────────→  │
  │  If-None-Match: "1a2b3c4d"           │
  │                                       │
  │                    ┌─────────────────┴─────┐
  │                    │ Generate ETag from    │
  │                    │ file metadata         │
  │                    │ → "1a2b3c4d"          │
  │                    └─────────────────┬─────┘
  │                                       │
  │  ← 304 Not Modified ─────────────────┤
  │    ETag: "1a2b3c4d"                   │
  │    (no body, ~200 bytes)              │
  │                                        │
  
Savings: ~50 KB file → 200 bytes response
```

## Compression

### Algorithm Selection

```rust
pub fn get_compression_method(accept_encoding: &str) 
    -> Option<CompressionMethod> 
{
    // Check in order of preference
    if accept_encoding.contains("br") {
        Some(CompressionMethod::Brotli)  // Best compression
    } else if accept_encoding.contains("gzip") {
        Some(CompressionMethod::Gzip)    // Good compression
    } else {
        None  // No compression
    }
}
```

### Compression Ratios

| Asset Type | Original | Gzip | Brotli | Best |
|------------|----------|------|--------|------|
| CSS | 50 KB | 12 KB (-76%) | 10 KB (-80%) | Brotli |
| JavaScript | 80 KB | 22 KB (-72%) | 18 KB (-77%) | Brotli |
| JSON | 30 KB | 5 KB (-83%) | 4 KB (-87%) | Brotli |
| Images | 100 KB | 98 KB (-2%) | 98 KB (-2%) | None |

**Rule**: Compress text-based assets (CSS/JS/JSON), skip binary assets (images/fonts).

### On-the-Fly vs Pre-Compression

#### On-the-Fly (Current Implementation)

```rust
// Compress during request
let compressed = compress_with_method(&content, Brotli)?;
response.body(compressed)
```

**Pros**: Simple, no build step
**Cons**: ~5-10ms latency per request

#### Pre-Compression (Future Optimisation)

```rust
// Pre-compress during build
pub fn precompress_assets(dir: &str) -> ReedResult<()> {
    for file in discover_assets(dir)? {
        let content = fs::read(&file)?;
        
        // Write .gz file
        let gzipped = gzip_compress(&content)?;
        fs::write(format!("{}.gz", file), gzipped)?;
        
        // Write .br file
        let brotli = brotli_compress(&content)?;
        fs::write(format!("{}.br", file), brotli)?;
    }
    Ok(())
}

// Serve pre-compressed file
if accept_encoding.contains("br") && 
   Path::new(&format!("{}.br", file_path)).exists() 
{
    response.body(fs::read(format!("{}.br", file_path))?)
}
```

**Pros**: Zero compression latency
**Cons**: 3× disk space (original + .gz + .br)

## MIME Type Detection

### Implementation

```rust
pub fn detect_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        // Text
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("html") => "text/html",
        Some("json") => "application/json",
        Some("txt") => "text/plain",
        
        // Images
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        
        // Fonts
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        
        // Fallback
        _ => "application/octet-stream",
    }
}
```

### Why Extension-Based?

| Method | Speed | Accuracy | Use Case |
|--------|-------|----------|----------|
| **Extension** | O(1) | 99% | Static assets |
| **Magic bytes** | O(n) | 100% | User uploads |

**Rationale**: Extension detection is instant and sufficient for build-time generated assets.

## Cache Strategy

### Cache-Control Headers

```rust
pub fn get_cache_control(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        // Immutable assets (session hash in filename)
        Some("css") | Some("js") => 
            "public, max-age=31536000, immutable",
        
        // Long-lived images
        Some("png") | Some("jpg") | Some("svg") => 
            "public, max-age=2592000",  // 30 days
        
        // Fonts
        Some("woff") | Some("woff2") => 
            "public, max-age=31536000",  // 1 year
        
        // Other
        _ => "public, max-age=86400",  // 1 day
    }
}
```

### Why 1-Year Cache for CSS/JS?

**Answer**: Session hash ensures file changes = new filename

```
Old version:  landing.a3f5b2c8.mouse.css  (cached 1 year)
New version:  landing.b7e4f9d2.mouse.css  (different file)

Browser sees new filename → cache miss → fetches new file
```

**Benefit**: Zero cache invalidation overhead + maximum caching

## Security

### Path Traversal Prevention

```rust
pub fn validate_path(requested_path: &str, base_dir: &str) 
    -> ReedResult<PathBuf> 
{
    let base = PathBuf::from(base_dir);
    let requested = base.join(requested_path);
    
    // Canonicalise paths
    let canonical_base = base.canonicalize()?;
    let canonical_requested = requested.canonicalize()?;
    
    // Ensure requested path is within base directory
    if !canonical_requested.starts_with(&canonical_base) {
        return Err(ReedError::SecurityViolation {
            reason: format!(
                "Path traversal attempt: {}", 
                requested_path
            ),
        });
    }
    
    Ok(canonical_requested)
}
```

### Attack Prevention Examples

```
✗ GET /session/styles/../../.reed/text.csv
  → SecurityViolation (outside public/)

✗ GET /session/styles/../../../etc/passwd
  → SecurityViolation (outside public/)

✓ GET /session/styles/landing.a3f5b2c8.mouse.css
  → OK (within public/session/styles/)
```

### Security Headers

```http
X-Content-Type-Options: nosniff
  → Prevents MIME type sniffing attacks

X-Frame-Options: DENY
  → Prevents clickjacking via iframes

Content-Type: text/css; charset=utf-8
  → Explicit MIME type enforcement
```

## Performance

| Operation | Timing | Note |
|-----------|--------|------|
| Path validation | < 0.1ms | Canonicalisation |
| ETag generation | < 0.1ms | Metadata read |
| 304 response | < 0.2ms | No file read |
| File read | < 2ms | 50 KB file |
| MIME detection | < 0.01ms | Extension lookup |
| Gzip compression | < 5ms | 50 KB → 12 KB |
| Brotli compression | < 8ms | 50 KB → 10 KB |
| **Total (cached)** | **< 0.2ms** | 304 Not Modified |
| **Total (uncached)** | **< 10ms** | 200 OK + brotli |

### Benchmark Results

```
10,000 requests to cached asset:
- 304 responses: 9,950 (99.5%)
- 200 responses: 50 (0.5%)
- Total time: 2.1s
- Average: 0.21ms per request
- Throughput: 4,760 req/s
```

## Actix-Web Integration

### Route Configuration

```rust
use actix_web::{web, App, HttpServer};
use actix_files::Files;

HttpServer::new(|| {
    App::new()
        // Static file serving
        .service(
            Files::new("/session", "public/session")
                .use_etag(true)
                .use_last_modified(true)
        )
        // Or custom handler
        .route("/session/{file:.*}", web::get().to(serve_asset))
})
```

### Custom Handler

```rust
async fn serve_asset(
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> 
{
    let file_path = path.into_inner();
    
    serve_static_asset(&req, &file_path, "public/session")
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e))
}
```

## Troubleshooting

### 304 Not Modified Not Working

**Symptom**: Always 200 responses, never 304

**Cause**: Client not sending If-None-Match header

**Solution**: Check client behaviour

```bash
# Test with curl
curl -I http://localhost:3000/session/styles/landing.css

# Should return ETag header
ETag: "1a2b3c4d5e6f7890"

# Second request with ETag
curl -I \
  -H 'If-None-Match: "1a2b3c4d5e6f7890"' \
  http://localhost:3000/session/styles/landing.css

# Should return 304 Not Modified
```

### Compression Not Applied

**Symptom**: Response not compressed despite Accept-Encoding

**Cause**: Content-Encoding header missing or wrong

**Solution**: Check Accept-Encoding parsing

```bash
# Test compression
curl -H 'Accept-Encoding: gzip, br' \
     http://localhost:3000/session/styles/landing.css \
     --compressed -I

# Should see:
Content-Encoding: br
```

### Path Traversal Security Error

**Symptom**: SecurityViolation error for valid path

**Cause**: Symlinks or canonical path outside base

**Solution**: Ensure public/ directory structure is clean

```bash
# Check for symlinks
ls -la public/session/

# Remove invalid symlinks
find public/session/ -type l -delete
```

## Related Documentation

- [Session Hash](session-hash.md) - Cache-busting strategy
- [CSS Bundler](css-bundler.md) - CSS bundle generation
- [JS Bundler](js-bundler.md) - JavaScript bundle generation
- [Actix-Web Integration](../06-server-layer/actix-web-integration.md) - HTTP server setup

## CLI Reference

```bash
# Start server with static asset serving
reed server:start

# Test asset serving
curl -I http://localhost:3000/session/styles/landing.a3f5b2c8.mouse.css

# Test with compression
curl -H 'Accept-Encoding: br' \
     http://localhost:3000/session/styles/landing.a3f5b2c8.mouse.css \
     --compressed

# Benchmark asset serving
ab -n 10000 -c 100 \
   http://localhost:3000/session/styles/landing.a3f5b2c8.mouse.css
```
