# Routing

> URL parsing and route resolution

---

## URL Structure

```
/{language}/{path}
```

**Examples:**
- `/de/wissen` → German knowledge page
- `/en/knowledge` → English knowledge page
- `/de/` → German homepage
- `/en/blog/article` → English blog article

---

## Route Resolution

### Process

```
URL: /de/wissen
    ↓
1. Parse: lang="de", path="wissen"
    ↓
2. Lookup .reed/routes.csv: wissen@de → knowledge
    ↓
3. Load template: knowledge.mouse.jinja
```

### routes.csv Format

```csv
layout@language|path|description
knowledge@de|wissen|German knowledge page
knowledge@en|knowledge|English knowledge page
home@de||German homepage (empty = root)
blog@en|blog|English blog
```

---

## Implementation

```rust
pub fn resolve_route(path: &str, lang: &str) -> ReedResult<String> {
    let key = format!("{}@{}", path, lang);
    
    // Lookup in routes.csv via ReedBase
    let layout = get_route(&key)?;
    
    Ok(layout)
}
```

**Performance:** < 100μs (O(1) HashMap lookup)

---

## Special Routes

### Root Redirect

```
/ → /en/ or /de/
```

Based on `Accept-Language` header.

### 404 Handling

```
/invalid/path → 404 Not Found
```

Renders `404.mouse.jinja` template.

---

**See also:**
- [Data Layer - ReedBase Cache](../02-data-layer/reedbase-cache.md) - Route storage
- [CLI Data Commands](../04-cli-layer/data-commands.md#route-operations) - Route management
