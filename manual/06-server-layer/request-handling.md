# Request Handling

> Complete HTTP request lifecycle

**See [README.md](README.md#request-flow) for overview.**

---

## Request Lifecycle

```
1. TCP Connection → Actix-Web
2. HTTP Parsing → Headers + Body
3. Middleware Pipeline → Logger, Compress, Auth
4. Route Matching → Handler selection
5. Handler Execution → Business logic
6. Response Building → HTML + Headers
7. Middleware Pipeline (reverse) → Compression
8. TCP Response → Client
```

**Total time:** < 10ms (production, cached)

---

## Handler Implementation

### Basic Handler

```rust
async fn handle_request(req: HttpRequest) -> Result<HttpResponse, ReedError> {
    // 1. Parse URL
    let url = parse_url(req.path())?;
    
    // 2. Resolve route
    let layout = resolve_route(&url.path, &url.lang)?;
    
    // 3. Detect variant
    let variant = detect_variant(&req)?;
    
    // 4. Render template
    let html = render_template(&layout, &variant, &url.lang)?;
    
    // 5. Build response
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
```

---

## URL Parsing

### Structure

```
/{language}/{path}

Examples:
/de/wissen → lang="de", path="wissen"
/en/knowledge → lang="en", path="knowledge"
/de/ → lang="de", path=""
```

### Implementation

```rust
pub fn parse_url(path: &str) -> ReedResult<UrlParts> {
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    
    match parts.as_slice() {
        [lang, rest @ ..] => Ok(UrlParts {
            lang: lang.to_string(),
            path: rest.join("/"),
        }),
        [] => Err(ReedError::InvalidUrl { /* ... */ }),
    }
}
```

---

## Performance

| Step | Time |
|------|------|
| TCP connection | ~1ms |
| HTTP parsing | < 1ms |
| Middleware | < 2ms |
| Route resolution | < 100μs |
| Template render | < 1ms |
| Response building | < 1ms |
| Compression | < 2ms |
| **Total** | < 10ms |

---

**See also:**
- [Routing](routing.md) - Route resolution
- [Client Detection](client-detection.md) - Variant selection
- [Response Building](response-building.md) - HTML generation
