# Response Building

> HTML generation and HTTP headers with per-request template environments

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

## Template Rendering

### Per-Request Environment Pattern

**ReedCMS creates a NEW MiniJinja Environment for each request** with language-specific filters and variant-specific functions.

**Why per-request?**
- Filters capture request language (`text("en")` vs `text("de")`)
- Functions capture interaction variant (`organism()` with "mouse" vs "touch")
- Template functions must exist **before** template parsing (for `{% extends %}`)

**Implementation (response/builder.rs):**
```rust
fn render_template(
    template_name: &str,
    context: &HashMap<String, serde_json::Value>,
) -> Result<String, ReedError> {
    let lang = context
        .get("client")
        .and_then(|c| c.get("lang"))
        .and_then(|l| l.as_str())
        .unwrap_or("en");

    let variant = context
        .get("client")
        .and_then(|c| c.get("interaction_mode"))
        .and_then(|v| v.as_str())
        .unwrap_or("mouse");

    // Create NEW environment per request
    let mut env = Environment::new();
    env.set_loader(template_loader);
    env.set_auto_escape_callback(|name| {
        if name.ends_with(".jinja") || name.ends_with(".html") {
            AutoEscape::Html
        } else {
            AutoEscape::None
        }
    });
    env.set_undefined_behavior(UndefinedBehavior::Strict);

    // Add filters with request language
    env.add_filter("text", filters::text::make_text_filter(lang.to_string()));
    env.add_filter("route", filters::route::make_route_filter(lang.to_string()));
    env.add_filter("meta", filters::meta::make_meta_filter());
    env.add_filter("config", filters::config::make_config_filter());

    // Add functions with request variant
    env.add_function("organism", functions::make_organism_function(variant.to_string()));
    env.add_function("molecule", functions::make_molecule_function(variant.to_string()));
    env.add_function("atom", functions::make_atom_function(variant.to_string()));
    env.add_function("layout", functions::make_layout_function(variant.to_string()));

    // Parse template (functions available at parse-time)
    let template = env.get_template(template_name)?;
    
    // Render with context
    template.render(context)
}
```

**Performance:**
- Environment creation: < 50μs
- Filter/function registration: < 10μs
- Template parsing: < 500μs (cached by MiniJinja internally)
- Template rendering: < 1ms
- **Total:** < 2ms per request

**Why this is fast:**
- MiniJinja caches parsed templates internally
- Environment creation is lightweight (just metadata)
- O(1) ReedBase cache lookups during render

### Context Variables

**Critical: Avoid naming conflicts with template functions**

```rust
// ❌ WRONG: context variable "layout" conflicts with layout() function
ctx.insert("layout".to_string(), serde_json::json!(layout));

// ✅ CORRECT: use "pagekey" instead
ctx.insert("pagekey".to_string(), serde_json::json!(layout));
```

**Standard context structure:**
```rust
{
    "client": {
        "lang": "de",
        "interaction_mode": "mouse",
        "device": "desktop"
    },
    "pagekey": "knowledge",
    "page": {
        "latest_update": "2025-10-07"
    },
    // ... additional page-specific data
}
```

**Reserved names (avoid in context):**
- `organism` - Template function
- `molecule` - Template function
- `atom` - Template function
- `layout` - Template function
- `text` - Filter name
- `route` - Filter name
- `meta` - Filter name
- `config` - Filter name

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
