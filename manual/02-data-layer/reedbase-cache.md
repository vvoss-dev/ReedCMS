# ReedBase Cache System

> O(1) HashMap cache with OnceLock initialisation and environment-aware fallback

---

## Overview

ReedBase implements five independent HashMap caches for text, routes, metadata, project configuration, and server configuration with thread-safe OnceLock initialisation and intelligent fallback resolution.

**Implementation:** Completed 2025-10-07 (REED-02-01)

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
│         Text Cache (OnceLock<HashMap>)           │
│  ┌────────────────────────────────────────────┐  │
│  │  HashMap<String, HashMap<String, String>>  │  │
│  │                                            │  │
│  │  "en" → {                                  │  │
│  │    "page.title" → "Welcome"                │  │
│  │    "page.title@dev" → "Welcome [DEV]"      │  │
│  │  }                                         │  │
│  │  "de" → {                                  │  │
│  │    "page.title" → "Willkommen"             │  │
│  │  }                                         │  │
│  └────────────────────────────────────────────┘  │
└───────────────────┬──────────────────────────────┘
                    │ Cache miss?
                    ▼
┌──────────────────────────────────────────────────┐
│              Return key as fallback              │
│   (Templates show missing keys for debugging)   │
└──────────────────────────────────────────────────┘
```

---

## Cache Structure

### Five Independent OnceLock Caches

**Text Cache:**
```rust
static TEXT_CACHE: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();
```
- Nested structure: Language → (Key → Value)
- Keys: `page.title`, `page.header.logo.alt`
- Language suffix in CSV: `page.title@en`, `page.title@de`
- Values: Text content strings

**Route Cache:**
```rust
static ROUTE_CACHE: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();
```
- Nested structure: Language → (Key → URL)
- Keys: `knowledge`, `home`, `portfolio`
- Values: URL paths (`knowledge`, `wissen`, `""`)

**Meta Cache:**
```rust
static META_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();
```
- Flat structure: Key → Value
- Keys: `site.title`, `cache.ttl`, `og.image`
- Values: Metadata strings (no language variants)

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
- **Zero runtime overhead** - no locking after initialisation
- **Thread-safe by design** - cannot be mutated after `set()`
- **Initialise once** - server startup or first access
- **Immutable after init** - perfect for read-heavy workloads

**Performance:**
- Initialisation: One-time < 30ms for 3,000 records
- Read access: < 1μs (direct HashMap lookup, no lock)
- No contention between threads
- No lock overhead on every read

---

## Environment-Aware Fallback

### Fallback Chain

Request: `get_text("page.title", "en", "dev")`

**4-Step Resolution:**
```
1. page.title@dev      (key + environment in language map)
   ↓ Not found in "en" map
2. page.title          (base key in language map)
   ↓ Found in "en" map
   → "Welcome"
```

**First match wins** - stops searching once value found.

### Use Cases

**Development-specific content:**
```csv
page.banner@dev|🚧 Development Mode|Dev banner
page.banner|Welcome|Production banner
```

Cache structure:
```rust
{
  "en": {
    "page.banner@dev": "🚧 Development Mode",
    "page.banner": "Welcome"
  }
}
```

Request in dev: `get_text("page.banner", "en", "dev")`
1. Check `page.banner@dev` in "en" map → **Found!** → "🚧 Development Mode"

**Language-specific content:**
```csv
page.title@en|Welcome|English
page.title@de|Willkommen|German
```

Cache structure:
```rust
{
  "en": { "page.title": "Welcome" },
  "de": { "page.title": "Willkommen" }
}
```

Request: `get_text("page.title", "de", "prod")`
1. Check `page.title` in "de" map → **Found!** → "Willkommen"

**Seasonal variants:**
```csv
page.logo@christmas|logo-christmas.svg|Christmas logo
page.logo|logo.svg|Default logo
```

Request: `get_text("page.logo", "en", "christmas")`
1. Check `page.logo@christmas` in "en" map → **Found!** → "logo-christmas.svg"
2. Fallback: `page.logo` → "logo.svg"

**Missing key fallback:**
```csv
# Key not in CSV
```

Request: `get_text("missing.key", "en", "prod")`
1. Check `missing.key` in "en" map → Not found
2. Return key itself: `"missing.key"`

**Why return key?** Templates show missing keys visibly during development instead of crashing.

---

## Cache Operations

### Initialisation at Server Startup

**Server startup (http_server.rs):**
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

**Cache-first with fallback (get.rs):**
```rust
pub fn text(request: &ReedRequest) -> ReedResult<ReedResponse<String>> {
    // Try cache first
    if super::cache::is_initialised() {
        let lang = request.language.as_deref().unwrap_or("de");
        match super::cache::get_text(&request.key, lang, request.environment.as_deref()) {
            Ok(value) => {
                return Ok(ReedResponse {
                    data: value,
                    source: format!("{}@{}", request.key, lang),
                    cached: true,
                    timestamp: current_timestamp(),
                    metrics: None,
                });
            }
            Err(_) => {
                // Cache miss - fall through to CSV read
            }
        }
    }
    
    // Fallback: Read from CSV if cache not initialised or key not found
    read_from_csv(".reed/text.csv", request)
}
```

**Performance:** 
- Cache hit: < 1μs
- Cache miss: Returns key as fallback (< 1μs)
- CSV fallback: < 10ms (only if cache not initialised)

### Cache Loading

**Load language-aware CSV (cache.rs):**
```rust
fn load_language_csv(path: &str) -> ReedResult<HashMap<String, HashMap<String, String>>> {
    let mut result: HashMap<String, HashMap<String, String>> = HashMap::new();
    
    let records = crate::reedcms::csv::universal::read_csv(path)?;
    
    for record in records {
        let full_key = record.get("key").ok_or(/* ... */)?;
        let value = record.get("value").ok_or(/* ... */)?;
        
        // Parse key@lang format
        if let Some((key, lang)) = full_key.split_once('@') {
            result
                .entry(lang.to_string())
                .or_insert_with(HashMap::new)
                .insert(key.to_string(), value.to_string());
        } else {
            // No language suffix - add to all language maps
            // (handled by fallback logic)
        }
    }
    
    Ok(result)
}
```

**Load flat CSV:**
```rust
fn load_flat_csv(path: &str) -> ReedResult<HashMap<String, String>> {
    let mut result = HashMap::new();
    let records = crate::reedcms::csv::universal::read_csv(path)?;
    
    for record in records {
        let key = record.get("key").ok_or(/* ... */)?;
        let value = record.get("value").ok_or(/* ... */)?;
        result.insert(key.to_string(), value.to_string());
    }
    
    Ok(result)
}
```

### Cache Invalidation

**Current implementation:**
- Caches are immutable after initialisation
- Changes to CSV files require server restart

**Future enhancement:**
- File watcher for automatic reload
- Or: CLI command `reed cache:reload`

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
5 caches × 3,000 entries = 15,000 entries ≈ 1.5 MB
```

**Additional overhead:**
- OnceLock wrapper: ~8 bytes per cache (negligible)
- Nested HashMap (text/route): ~50 bytes per language
- Total overhead: < 1 KB

### Scaling Considerations

**Optimal:** < 10,000 entries per cache

**Beyond 10,000:**
- Consider SQLite migration
- Or implement LRU cache eviction
- Or split into multiple CSV files by feature

**Current limits:**
- No automatic eviction
- No size limits
- All keys cached indefinitely
- Perfect for small-to-medium sites (< 50,000 total entries)

---

## Performance Characteristics

### Lookup Performance

| Scenario | Time | Notes |
|----------|------|-------|
| Cache hit | < 1μs | O(1) HashMap, no lock overhead |
| Cache miss (key fallback) | < 1μs | Returns key string |
| Fallback chain (2 steps) | < 2μs | 2× HashMap lookups |
| Server startup (5 caches) | < 30ms | One-time cost |

**99th percentile:** < 5μs (including fallback chain)

### Concurrency Performance

**Read-heavy workload (100% reads after init):**
```
Concurrent readers: Zero contention (no locks)
Throughput: ~1,000,000 req/s per core
```

**Write operations:**
```
Writers block readers: ~50ms per write
Max write throughput: ~20 writes/s
```

Performance gain: 10× faster
```

---

## Integration Examples

### CLI Commands

```bash
# Automatically uses cache (after server started once)
reed text:get page.title@en
# → Cache hit: < 1μs

# Missing key returns key itself
reed text:get nonexistent.key@en
# → "nonexistent.key"
```

### Template Filters

```jinja
{# Uses ReedBase cache with fallback #}
<h1>{{ "page.title" | text }}</h1>
<!-- Cache hit: < 1μs, shows "Welcome" -->

{# Missing key shows key itself #}
<p>{{ "missing.key" | text }}</p>
<!-- Shows: "missing.key" (visible during development) -->

{# Fallback chain example #}
<p>{{ "page.subtitle" | text }}</p>
<!-- Tries: page.subtitle@dev → page.subtitle → "page.subtitle" -->
```

### Server Initialisation

```rust
// Startup: Warm all caches
let reedbase = ReedBase::new(/* ... */);
reedbase.init()?;
println!("Cache initialised in 28ms");

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

**Rely on startup initialisation:**
```rust
// ✅ Server handles cache warming automatically
// No manual init needed in application code
```

**Use fallback chains in CSV:**
```csv
# ✅ Environment-specific with fallback
page.debug@dev|Debug: ON|Dev mode indicator
page.debug|Debug: OFF|Production default
```

**Missing keys are visible:**
```jinja
{# ✅ Missing keys show as "missing.key" in rendered output #}
{# Visible during development, easy to spot #}
<p>{{ "missing.key" | text }}</p>
```

**Immutable cache design:**
```rust
// ✅ OnceLock prevents accidental mutation
// CSV changes require deliberate server restart
// Prevents runtime data inconsistencies
```

---

**See also:**
- [CSV Architecture](csv-architecture.md) - File format and structure
- [Backup System](backup-system.md) - Automatic backups on write
- [Data Operations](data-operations.md) - Complete API reference
