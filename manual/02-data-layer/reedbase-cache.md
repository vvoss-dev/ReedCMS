# ReedBase Cache System

> O(1) HashMap cache with environment-aware fallback

---

## Overview

ReedBase implements three independent HashMap caches for text, routes, and metadata with thread-safe concurrent access and intelligent fallback resolution.

---

## Cache Architecture

```
┌──────────────────────────────────────────────────┐
│              Application Request                 │
│   get_text("page.title", "en", "dev")           │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│           Fallback Chain Resolution              │
│                                                  │
│   1. page.title@en@dev  ────┐                   │
│   2. page.title@en ─────────┤                   │
│   3. page.title@dev ────────┤                   │
│   4. page.title ────────────┘                   │
│                              │                   │
└──────────────────────────────┼───────────────────┘
                               │ First match wins
                               ▼
┌──────────────────────────────────────────────────┐
│            Text Cache (HashMap)                  │
│  ┌────────────────────────────────────────────┐  │
│  │  RwLock<HashMap<String, String>>           │  │
│  │                                            │  │
│  │  "page.title@en"     → "Welcome"          │  │
│  │  "page.title@de"     → "Willkommen"       │  │
│  │  "page.title@en@dev" → "Welcome [DEV]"    │  │
│  └────────────────────────────────────────────┘  │
└───────────────────┬──────────────────────────────┘
                    │ Cache miss?
                    ▼
┌──────────────────────────────────────────────────┐
│              Load from CSV                       │
│   .reed/text.csv → Parse → Insert to cache      │
└──────────────────────────────────────────────────┘
```

---

## Cache Structure

### Three Independent Caches

**Text Cache:**
```rust
text_cache: RwLock<HashMap<String, String>>
```
- Keys: `page.title@en`, `page.header.logo.alt@de`
- Values: Text content strings

**Route Cache:**
```rust
route_cache: RwLock<HashMap<String, String>>
```
- Keys: `knowledge@en`, `home@de`
- Values: URL paths (`knowledge`, `wissen`, `""`)

**Meta Cache:**
```rust
meta_cache: RwLock<HashMap<String, String>>
```
- Keys: `site.title`, `cache.ttl`, `og.image`
- Values: Metadata strings

### Thread Safety

**RwLock (Read-Write Lock):**
```rust
use std::sync::RwLock;

// Multiple concurrent readers
let value = cache.read().unwrap().get(key);

// Single writer (blocks all)
cache.write().unwrap().insert(key, value);
```

**Benefits:**
- Multiple threads can read simultaneously
- Write operations exclusive (one at a time)
- Optimised for read-heavy workloads (95% reads)

**Performance:**
- Read lock: ~10ns overhead
- Write lock: ~50ns overhead
- No contention on reads

---

## Environment-Aware Fallback

### Fallback Chain

Request: `get_text("page.title", "en", "dev")`

**4-Step Resolution:**
```
1. page.title@en@dev   (key + language + environment)
   ↓ Not found
2. page.title@en       (key + language)
   ↓ Not found
3. page.title@dev      (key + environment)
   ↓ Not found
4. page.title          (base key)
   ↓ Not found
   → ReedError::NotFound
```

**First match wins** - stops searching once value found.

### Use Cases

**Development-specific content:**
```csv
page.banner@dev|🚧 Development Mode|Dev banner
page.banner@prod|Welcome|Production banner
```

Request in dev: `get_text("page.banner", "en", "dev")`
1. Check `page.banner@en@dev` → Not found
2. Check `page.banner@en` → Not found
3. Check `page.banner@dev` → **Found!** → "🚧 Development Mode"

**Language-specific content:**
```csv
page.title@en|Welcome|English
page.title@de|Willkommen|German
```

Request: `get_text("page.title", "de", "prod")`
1. Check `page.title@de@prod` → Not found
2. Check `page.title@de` → **Found!** → "Willkommen"

**Seasonal variants:**
```csv
page.logo@christmas|logo-christmas.svg|Christmas logo
page.logo|logo.svg|Default logo
```

Request: `get_text("page.logo", "en", "christmas")`
1. Check `page.logo@en@christmas` → Not found
2. Check `page.logo@en` → Not found
3. Check `page.logo@christmas` → **Found!** → "logo-christmas.svg"

**Universal fallback:**
```csv
error.404|Page not found|Generic 404 message
```

Request: `get_text("error.404", "jp", "prod")`
1. Check `error.404@jp@prod` → Not found
2. Check `error.404@jp` → Not found
3. Check `error.404@prod` → Not found
4. Check `error.404` → **Found!** → "Page not found"

---

## Cache Operations

### Initialisation

**Eager loading (recommended):**
```rust
let reedbase = ReedBase::new(
    ".reed/text.csv",
    ".reed/routes.csv",
    ".reed/meta.csv",
);

// Load all CSV files into caches at startup
reedbase.init()?;
```

**Benefits:**
- Warm cache from start
- Predictable startup time
- All keys available immediately

**Performance:** < 30ms for 3,000 records

**Lazy loading (automatic):**
```rust
// First access to empty cache triggers CSV load
let text = reedbase.get(request)?;
```

**Benefits:**
- Faster startup (no init needed)
- Only loads when needed

**Drawback:**
- First request slower (< 10ms delay)

### Lookup (Get)

**Read operation:**
```rust
pub fn get(&self, request: ReedRequest) -> ReedResult<ReedResponse<String>> {
    let key = format!("{}@{}", request.key, request.language.unwrap_or_default());
    
    // Try cache first
    let cache = self.text_cache.read().unwrap();
    if let Some(value) = cache.get(&key) {
        return Ok(ReedResponse::new(value.clone(), "reedbase::cache"));
    }
    
    // Cache miss - load from CSV (not shown)
}
```

**Performance:** < 100μs

### Insert (Set)

**Write operation:**
```rust
pub fn set(&self, request: ReedRequest) -> ReedResult<ReedResponse<String>> {
    let key = format!("{}@{}", request.key, request.language.unwrap_or_default());
    let value = request.value.unwrap();
    
    // Update cache
    let mut cache = self.text_cache.write().unwrap();
    cache.insert(key.clone(), value.clone());
    
    // Write to CSV (not shown)
    // Create backup (not shown)
    
    Ok(ReedResponse::new(value, "reedbase::set"))
}
```

**Performance:** < 50ms (includes CSV write and backup)

### Cache Invalidation

**On write:**
- Old value automatically replaced by HashMap insert
- No manual invalidation needed

**On external CSV edit:**
- Restart server/CLI to reload
- Future: File watcher for auto-reload

---

## Memory Management

### Memory Usage

**Per entry:** ~100 bytes
- Key: ~40 bytes (average)
- Value: ~50 bytes (average)
- HashMap overhead: ~10 bytes

**Example datasets:**
```
1,000 entries  ≈ 100 KB
3,000 entries  ≈ 300 KB
10,000 entries ≈ 1 MB
```

**Total ReedBase memory:**
```
3 caches × 3,000 entries = 9,000 entries ≈ 900 KB
```

### Scaling Considerations

**Optimal:** < 10,000 entries per cache

**Beyond 10,000:**
- Consider SQLite migration
- Or implement LRU cache eviction
- Or split into multiple CSV files

**Current limits:**
- No automatic eviction
- No size limits
- All keys cached indefinitely

---

## Performance Characteristics

### Lookup Performance

| Scenario | Time | Notes |
|----------|------|-------|
| Cache hit | < 100μs | O(1) HashMap |
| Cache miss | < 10ms | Load entire CSV |
| Fallback chain (4 steps) | < 400μs | 4× HashMap lookups |

**99th percentile:** < 200μs (cached)

### Concurrency Performance

**Read-heavy workload (95% reads):**
```
Concurrent readers: No contention
Throughput: ~10,000 req/s per core
```

**Write operations:**
```
Writers block readers: ~50ms per write
Max write throughput: ~20 writes/s
```

**Recommendation:** Batch writes when possible

---

## Integration Examples

### CLI Commands

```bash
# Automatically uses cache
reed text:get page.title@en
# → Cache hit: < 100μs

# First access after startup (lazy load)
reed text:get page.new@en
# → Cache miss: < 10ms (loads CSV)

# Subsequent access
reed text:get page.new@en
# → Cache hit: < 100μs
```

### Template Filters

```jinja
{# Uses ReedBase cache #}
<h1>{{ "page.title" | text("en") }}</h1>
<!-- Rendered in < 100μs -->

{# Fallback chain example #}
<p>{{ "page.subtitle" | text("de", "dev") }}</p>
<!-- Tries: @de@dev → @de → @dev → base -->
```

### Server Initialization

```rust
// Startup: Warm all caches
let reedbase = ReedBase::new(/* ... */);
reedbase.init()?;
println!("Cache initialized in 28ms");

// Request handling: Fast lookups
let title = reedbase.get(text_request)?;
// < 100μs per request
```

---

## Cache Strategies

### Warm Cache at Startup

**Recommended for production:**
```rust
// src/main.rs
fn main() -> ReedResult<()> {
    let reedbase = ReedBase::new(/* ... */);
    
    // Warm cache
    reedbase.init()?;
    
    // Start server with warm cache
    start_server(reedbase)?;
    
    Ok(())
}
```

### Lazy Load on Demand

**Suitable for development:**
```rust
// No init() call - cache empty at startup
let reedbase = ReedBase::new(/* ... */);

// First get() triggers CSV load
let text = reedbase.get(request)?; // < 10ms
let text2 = reedbase.get(request2)?; // < 100μs
```

### Hybrid Strategy

**Best of both worlds:**
```rust
// Warm most-used caches
reedbase.init_text()?;  // Warm text cache
// Leave route/meta for lazy load

// Or: Pre-load specific keys
for key in critical_keys {
    reedbase.get_text(key, "en")?;
}
```

---

## Best Practices

**Warm cache at startup (production):**
```rust
// ✅ Predictable performance
reedbase.init()?;
```

**Use fallback chains:**
```csv
# ✅ Environment-specific with fallback
page.debug@dev|Debug: ON|Dev mode indicator
page.debug|Debug: OFF|Production default
```

**Batch writes:**
```rust
// ✅ Minimize write lock contention
for (key, value) in updates {
    reedbase.set(key, value)?;
}
```

**Monitor cache size:**
```rust
// Check memory usage
let text_size = reedbase.text_cache_size();
if text_size > 10000 {
    warn!("Text cache large: {} entries", text_size);
}
```

---

**See also:**
- [CSV Architecture](csv-architecture.md) - File format and structure
- [Backup System](backup-system.md) - Automatic backups on write
- [Data Operations](data-operations.md) - Complete API reference
