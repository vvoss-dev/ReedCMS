# Data Layer (Layer 02)

> CSV-based storage with O(1) HashMap cache and automatic backups

**Status:** ✅ Complete  
**Implementation:** REED-02-01 to REED-02-04

---

## Overview

The Data Layer provides persistent storage through pipe-delimited CSV files with O(1) in-memory HashMap caching. ReedBase acts as the intelligent dispatcher coordinating all data operations.

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│              Application Layer                   │
│  (CLI, Server, API)                              │
└───────────────────┬──────────────────────────────┘
                    │ get_text("page.title", "en")
                    ▼
┌──────────────────────────────────────────────────┐
│           ReedBase Dispatcher                    │
│  ┌────────────────────────────────────────────┐  │
│  │  Text Cache    (HashMap + RwLock)          │  │
│  │  Route Cache   (HashMap + RwLock)          │  │
│  │  Meta Cache    (HashMap + RwLock)          │  │
│  └────────────────────────────────────────────┘  │
└───────────────────┬──────────────────────────────┘
                    │ Cache miss? Load from CSV
                    ▼
┌──────────────────────────────────────────────────┐
│              CSV Storage Layer                   │
│  .reed/text.csv                                  │
│  .reed/routes.csv                                │
│  .reed/meta.csv                                  │
│  .reed/users.matrix.csv                          │
│  .reed/roles.matrix.csv                          │
└───────────────────┬──────────────────────────────┘
                    │ Write operations trigger backup
                    ▼
┌──────────────────────────────────────────────────┐
│              Backup System                       │
│  .reed/backups/*.csv.xz (XZ-compressed)          │
│  Retention: 32 most recent backups               │
└──────────────────────────────────────────────────┘
```

---

## Core Concepts

### CSV as Single Source of Truth

ReedCMS uses CSV files instead of traditional databases:

**Advantages:**
- Git-friendly (easy diffs, merge conflicts, version control)
- Human-readable and editable
- Zero-dependency storage (no database server)
- Fast I/O for small to medium datasets (< 10,000 records)
- Atomic operations via temp file + rename

**Format:**
```
key|value|description
page.title@en|Welcome|Homepage title
page.title@de|Willkommen|German title
```

**Delimiter:** Pipe (`|`) character (rarely used in content)

### HashMap Cache

**Performance:**
- Lookup: O(1) average case, < 100μs
- Insert: O(1) average case
- Memory: ~100 bytes per entry

**Thread Safety:**
- `RwLock` allows multiple concurrent readers
- Single writer blocks all access temporarily
- Optimised for read-heavy workloads (95% reads)

**Initialisation:**
- Lazy loading on first access OR
- Eager loading via `reedbase.init()` at startup

### Environment-Aware Fallback

ReedBase implements intelligent key resolution:

```
Request: page.title@en

Fallback chain:
1. page.title@en@dev   (key + language + environment)
2. page.title@en       (key + language)
3. page.title@dev      (key + environment)
4. page.title          (base key)
```

**Environment suffix:**
- `@dev` - Development environment
- `@prod` - Production environment
- `@christmas`, `@easter` - Seasonal variants

### Automatic Backups

Every write operation triggers backup:

**Backup Process:**
1. Read current CSV file
2. Compress with XZ (LZMA2 algorithm)
3. Save as `.reed/backups/{filename}.{timestamp}.csv.xz`
4. Delete oldest if > 32 backups exist

**Compression Ratio:** ~10:1 for text-heavy content

**Retention:** 32 most recent backups (configurable)

---

## Core Components

### ReedBase Dispatcher

**File:** `src/reedcms/reed/reedbase.rs`

**Responsibilities:**
- Manages three HashMap caches (text, route, meta)
- Coordinates get/set/init operations
- Provides thread-safe access via RwLock

**Key Methods:**
```rust
impl ReedBase {
    pub fn new(text_path: &str, route_path: &str, meta_path: &str) -> Self
    pub fn init(&self) -> ReedResult<()>
    pub fn get(&self, request: ReedRequest) -> ReedResult<ReedResponse<String>>
    pub fn set(&self, request: ReedRequest) -> ReedResult<ReedResponse<String>>
}
```

### Get Service

**File:** `src/reedcms/reedbase/get.rs`

**Function:** `get(request: ReedRequest) -> ReedResult<ReedResponse<String>>`

**Features:**
- O(1) HashMap lookup
- Environment-aware fallback (4-step chain)
- Cache-first strategy
- Lazy CSV loading on cache miss

**Performance:** < 100μs (cached), < 10ms (uncached)

### Set Service

**File:** `src/reedcms/reedbase/set.rs`

**Function:** `set(request: ReedRequest) -> ReedResult<ReedResponse<String>>`

**Features:**
- Atomic CSV write (temp file + rename)
- Automatic backup creation (XZ-compressed)
- Cache invalidation/update
- Duplicate key prevention

**Performance:** < 50ms (includes backup)

### Init Service

**File:** `src/reedcms/reedbase/init.rs`

**Function:** `init(request: ReedRequest) -> ReedResult<ReedResponse<HashMap<String, String>>>`

**Features:**
- Loads entire CSV file into HashMap
- Validates CSV format
- Skips empty lines and comments
- Returns populated cache

**Performance:** < 30ms for 3,000 records

---

## CSV File Types

### Type 1: Simple Key-Value

**Format:** `key|value|description`

**Files:**
- `.reed/text.csv` - All text content
- `.reed/routes.csv` - URL routing
- `.reed/meta.csv` - Metadata
- `.reed/server.csv` - Server config
- `.reed/project.csv` - Project settings

**Example:**
```csv
page.title@en|Welcome|Homepage title
page.title@de|Willkommen|German homepage title
page.header.logo.alt@en|ReedCMS Logo|Logo alt text
```

### Type 2: Matrix (Lists)

**Format:** `key|list,of,values|other|columns`

**Files:**
- `.reed/users.matrix.csv` - Users with role lists
- `.reed/roles.matrix.csv` - Roles with permission lists

**Example:**
```csv
username|password|roles|firstname|lastname|...
admin|$argon2id$...|admin,editor,viewer|John|Doe|...
```

**Note:** Comma-separated values in specific columns

---

## Data Operations

### Read Operations

**CLI:**
```bash
reed text:get page.title@en
reed route:get knowledge@de
reed meta:get site.title
```

**Rust:**
```rust
use crate::reedcms::reedbase::get::get;

let request = ReedRequest {
    key: "page.title".to_string(),
    language: Some("en".to_string()),
    environment: Some("dev".to_string()),
    context: Some("text".to_string()),
    ..Default::default()
};

let response = get(request)?;
println!("{}", response.data); // "Welcome"
```

### Write Operations

**CLI:**
```bash
reed text:set page.title@en "Welcome" --desc "Homepage title"
reed route:set knowledge@de "wissen" --desc "German knowledge page"
reed meta:set site.title "ReedCMS"
```

**Rust:**
```rust
use crate::reedcms::reedbase::set::set;

let request = ReedRequest {
    key: "page.title".to_string(),
    language: Some("en".to_string()),
    value: Some("Welcome".to_string()),
    description: Some("Homepage title".to_string()),
    context: Some("text".to_string()),
    ..Default::default()
};

let response = set(request)?;
```

### Bulk Operations

**CLI:**
```bash
# Import from CSV
reed text:import content.csv

# Export to CSV
reed text:export backup.csv

# List all keys
reed text:list
reed text:list "*@en"  # Filter
```

---

## Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| Cache lookup | < 100μs | O(1) HashMap |
| Cache miss (load CSV) | < 10ms | Sequential read |
| Write + backup | < 50ms | Atomic + XZ compress |
| Init 1,000 records | < 10ms | Bulk HashMap insert |
| Init 3,000 records | < 30ms | Bulk HashMap insert |

**Memory Usage:**
- ~100 bytes per cached entry
- 3,000 entries ≈ 300 KB memory

**Scalability:**
- Optimised for < 10,000 records per CSV
- Beyond 10,000: Consider SQLite migration

---

## Integration

### CLI Layer

CLI commands use ReedBase for all data operations:

```
reed text:set → CLI parser → ReedBase.set() → CSV write
reed text:get → CLI parser → ReedBase.get() → Cache/CSV read
```

See [CLI Commands](../04-cli-layer/) for details.

### Template Layer

MiniJinja filters access ReedBase:

```jinja
{{ "page.title" | text("en") }}       → ReedBase.get(text)
{{ "knowledge" | route("de") }}       → ReedBase.get(route)
{{ "site.title" | meta }}             → ReedBase.get(meta)
```

See [Template Filters](../05-template-layer/filters.md) for details.

### Server Layer

Server initialises ReedBase at startup:

```rust
let reedbase = ReedBase::new(
    ".reed/text.csv",
    ".reed/routes.csv",
    ".reed/meta.csv",
);
reedbase.init()?; // Warm cache
```

See [Server Layer](../06-server-layer/) for details.

---

## Documentation

- [CSV Architecture](csv-architecture.md) - Design philosophy and format specifications
- [ReedBase Cache](reedbase-cache.md) - Cache implementation and fallback system
- [Backup System](backup-system.md) - XZ compression and retention
- [Data Operations](data-operations.md) - Complete API reference

---

## Related Layers

- **Layer 01 - Foundation:** Provides ReedStream communication types
- **Layer 04 - CLI:** Exposes data operations via commands
- **Layer 05 - Template:** Consumes data via filters
- **Layer 06 - Server:** Initialises ReedBase and serves content

---

## Summary

The Data Layer provides:
- ✅ CSV-based storage (Git-friendly, human-readable)
- ✅ O(1) HashMap cache (< 100μs lookups)
- ✅ Thread-safe concurrent access (RwLock)
- ✅ Environment-aware fallback (4-step chain)
- ✅ Automatic XZ backups (32 retained)
- ✅ Atomic write operations (temp + rename)
- ✅ Multiple CSV types (simple, matrix)
- ✅ Complete CLI integration

All features production-ready and fully tested.
