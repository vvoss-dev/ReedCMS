# Response Building

> HTML generation and HTTP headers

---

## Response Structure

```rust
HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .insert_header(("Cache-Control", "public, max-age=3600"))
    .insert_header(("Content-Language", lang))
    .body(html)
```

---

## Standard Headers

### Content-Type

```
Content-Type: text/html; charset=utf-8
```

**Always UTF-8** for proper character encoding.

### Cache-Control

```
Cache-Control: public, max-age=3600
```

**Static content:** Long cache (1 hour - 1 year)  
**Dynamic content:** Short cache (5 minutes) or no-cache

### Content-Language

```
Content-Language: de
```

Based on URL language segment.

### Content-Encoding

```
Content-Encoding: br
```

**Automatic:** Set by Compress middleware (gzip/brotli)

---

## Compression

**Middleware handles automatically:**
- Brotli (best compression)
- Gzip (universal support)
- Deflate (fallback)

**Selection:** Based on `Accept-Encoding` request header

**Savings:** 70-80% size reduction

---

## Status Codes

```rust
HttpResponse::Ok()                    // 200 OK
HttpResponse::MovedPermanently()      // 301 Redirect
HttpResponse::NotFound()              // 404 Not Found
HttpResponse::InternalServerError()   // 500 Error
```

---

## Performance

| Step | Time |
|------|------|
| Template render | < 1ms |
| Header building | < 100μs |
| Compression | < 2ms |
| **Total** | < 5ms |

---

**See also:**
- [Template Layer](../05-template-layer/) - HTML generation
- [Actix-Web Integration](actix-web-integration.md) - Response API
