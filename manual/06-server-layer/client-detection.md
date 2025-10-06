# Client Detection

> Device and variant detection from User-Agent

---

## Purpose

Automatically select template variant (mouse/touch/reader) based on client device.

---

## Detection Logic

```rust
pub fn detect_variant(req: &HttpRequest) -> &str {
    let ua = req.headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if is_mobile(ua) || is_tablet(ua) {
        "touch"
    } else if is_reader_mode(ua) {
        "reader"
    } else {
        "mouse"
    }
}
```

---

## Device Detection

### Mobile

**Patterns:**
- `Mobile`
- `Android`
- `iPhone`
- `iPod`

**Result:** `touch` variant

### Tablet

**Patterns:**
- `iPad`
- `Tablet`
- `Android` (with specific sizes)

**Result:** `touch` variant

### Desktop

**Default:** Everything else

**Result:** `mouse` variant

---

## User-Agent Examples

```
Mobile:
Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) ...
→ touch variant

Tablet:
Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X) ...
→ touch variant

Desktop:
Mozilla/5.0 (Windows NT 10.0; Win64; x64) ...
→ mouse variant

Bot:
Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)
→ mouse variant (default)
```

---

## Performance

**User-Agent parsing:** < 1ms

**Cached decision:** No caching (stateless)

---

## Screen Detection (JavaScript)

**Fallback for accurate detection:**

```javascript
// Injected on first visit
const width = window.innerWidth;
document.cookie = `screen_width=${width}`;
location.reload();
```

**Server reads cookie on second request** for precise variant selection.

---

**See also:**
- [Template Layer - Variants](../05-template-layer/atomic-design.md#variants) - Variant types
- [Request Handling](request-handling.md) - Complete lifecycle
