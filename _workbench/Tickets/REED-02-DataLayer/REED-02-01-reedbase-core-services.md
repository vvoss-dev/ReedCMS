# REED-02-01: ReedBase Core Services

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
- **ID**: REED-02-01
- **Title**: ReedBase Core Services
- **Layer**: Data Layer (REED-02)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-01-01, REED-01-02, REED-02-02, REED-02-04

## Summary Reference
- **Section**: ReedBase Dispatcher
- **Lines**: 746-749 in project_summary.md
- **Key Concepts**: O(1) HashMap performance, environment fallback logic, cache management

## Objective
Implement ReedBase as the central data aggregation engine with intelligent environment-aware data access and O(1) performance guarantees.

## Requirements

### 1. Core Service Files
Create separate service files following KISS principle:

#### Get Operations (`src/reedcms/reedbase/get.rs`)
```rust
/// Retrieves text content with environment fallback.
///
/// ## Input
/// - `req.key`: Text identifier
/// - `req.language`: Language code (ISO 639-1)
/// - `req.environment`: Optional environment override
///
/// ## Output
/// - `ReedResult<ReedResponse<String>>`: Text content or NotFound error
///
/// ## Performance
/// - O(1) HashMap lookup
/// - Execution time: < 100μs
pub fn text(req: &ReedRequest) -> ReedResult<ReedResponse<String>>

pub fn route(req: &ReedRequest) -> ReedResult<ReedResponse<String>>
pub fn meta(req: &ReedRequest) -> ReedResult<ReedResponse<String>>
pub fn server(req: &ReedRequest) -> ReedResult<ReedResponse<String>>
pub fn project(req: &ReedRequest) -> ReedResult<ReedResponse<String>>
pub fn list_text(req: &ReedRequest) -> ReedResult<ReedResponse<Vec<String>>>
```

#### Set Operations (`src/reedcms/reedbase/set.rs`)
```rust
/// Sets text content with automatic backup.
///
/// ## Input
/// - `req.key`: Text identifier
/// - `req.language`: Language code
/// - `req.value`: Content to store
/// - `req.description`: Mandatory comment (min 10 chars)
///
/// ## Performance
/// - Backup: ~5ms (XZ compression)
/// - CSV write: ~2ms
/// - Cache update: ~0.1ms
/// - Total: < 10ms
pub fn text(req: &ReedRequest) -> ReedResult<ReedResponse<()>>

pub fn route(req: &ReedRequest) -> ReedResult<ReedResponse<()>>
pub fn meta(req: &ReedRequest) -> ReedResult<ReedResponse<()>>
pub fn server(req: &ReedRequest) -> ReedResult<ReedResponse<()>>
pub fn project(req: &ReedRequest) -> ReedResult<ReedResponse<()>>
```

#### Initialisation (`src/reedcms/reedbase/init.rs`)
```rust
/// Initialises ReedBase by loading all CSV files into memory.
///
/// ## Performance
/// - Startup-time loading only
/// - Target: < 200μs for 17 layouts
pub fn init() -> ReedResult<ReedResponse<InitStats>>

/// Struct containing initialisation statistics
pub struct InitStats {
    pub text_entries: usize,
    pub route_entries: usize,
    pub meta_entries: usize,
    pub load_time_ms: u64,
}
```

### 2. Cache System (`src/reedcms/reedbase/cache.rs`)
Implement runtime HashMap caches:

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

pub type TextCache = Arc<RwLock<HashMap<String, HashMap<String, String>>>>;
pub type RouteCache = Arc<RwLock<HashMap<String, HashMap<String, String>>>>;
pub type MetaCache = Arc<RwLock<HashMap<String, String>>>;

lazy_static! {
    pub static ref TEXT_CACHE: TextCache = Arc::new(RwLock::new(HashMap::new()));
    pub static ref ROUTE_CACHE: RouteCache = Arc::new(RwLock::new(HashMap::new()));
    pub static ref META_CACHE: MetaCache = Arc::new(RwLock::new(HashMap::new()));
}

/// Cache invalidation functions.
///
/// ## Invalidation Triggers
/// - After CSV file writes (set operations)
/// - On external CSV file changes (file watcher)
/// - Manual invalidation via CLI command
/// - Periodic refresh (optional)
///
/// ## Granularity
/// - Invalidate specific key
/// - Invalidate entire cache type (text/route/meta)
/// - Invalidate all caches

/// Invalidates specific text key.
pub async fn invalidate_text_key(key: &str) {
    let mut cache = TEXT_CACHE.write().await;
    cache.retain(|lang_map_key, _| lang_map_key != key);
}

/// Invalidates entire text cache.
pub async fn invalidate_text_cache() {
    let mut cache = TEXT_CACHE.write().await;
    cache.clear();
}

/// Invalidates specific route key.
pub async fn invalidate_route_key(key: &str) {
    let mut cache = ROUTE_CACHE.write().await;
    cache.retain(|route_key, _| route_key != key);
}

/// Invalidates entire route cache.
pub async fn invalidate_route_cache() {
    let mut cache = ROUTE_CACHE.write().await;
    cache.clear();
}

/// Invalidates specific meta key.
pub async fn invalidate_meta_key(key: &str) {
    let mut cache = META_CACHE.write().await;
    cache.remove(key);
}

/// Invalidates entire meta cache.
pub async fn invalidate_meta_cache() {
    let mut cache = META_CACHE.write().await;
    cache.clear();
}

/// Invalidates all caches (text, route, meta).
///
/// ## Use Cases
/// - System restart
/// - Major data migration
/// - CLI command: `reed cache:clear`
pub async fn invalidate_all_caches() {
    invalidate_text_cache().await;
    invalidate_route_cache().await;
    invalidate_meta_cache().await;
}

/// Refreshes cache from CSV files.
///
/// ## Process
/// 1. Clear existing cache
/// 2. Reload from CSV files
/// 3. Rebuild HashMap structures
///
/// ## Performance
/// - < 50ms (same as init operation: loads 5 CSV files + builds HashMap caches)
pub async fn refresh_cache() -> ReedResult<()> {
    invalidate_all_caches().await;
    init::initialize()
}

/// Gets cache statistics.
pub fn get_cache_stats() -> ReedResult<CacheStats> {
    // Implementation returns entry counts
    Ok(CacheStats {
        text_entries: 0,  // Count TEXT_CACHE entries
        route_entries: 0, // Count ROUTE_CACHE entries
        meta_entries: 0,  // Count META_CACHE entries
        total_entries: 0,
    })
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub text_entries: usize,
    pub route_entries: usize,
    pub meta_entries: usize,
    pub total_entries: usize,
}
```

**Integration with set.rs**:
After each set operation in `set.rs`, call the appropriate cache invalidation function:
- `set::text()` → call `cache::invalidate_text_key(&req.key).await`
- `set::route()` → call `cache::invalidate_route_key(&req.key).await`
- `set::meta()` → call `cache::invalidate_meta_key(&req.key).await`

This ensures cache consistency after CSV writes.

### 3. Environment Fallback (`src/reedcms/reedbase/environment.rs`)
Implement key@env resolution logic:

```rust
/// Resolves environment-aware keys with fallback.
///
/// ## Resolution Order
/// 1. Try key@environment (e.g., "title@dev")
/// 2. Fall back to base key (e.g., "title")
///
/// ## Example
/// - Input: "knowledge.title@dev"
/// - Lookup: "knowledge.title@dev" → "knowledge.title" (if not found)
pub fn resolve_key(base_key: &str, environment: &Option<String>) -> String
```

### ReedModule Trait Implementation (`src/reedcms/reedbase/mod.rs`)

```rust
use crate::reedstream::{ReedModule, ReedResult};

/// ReedBase module implementation.
pub struct ReedBaseModule;

impl ReedModule for ReedBaseModule {
    fn module_name(&self) -> &'static str {
        "reedbase"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn health_check(&self) -> ReedResult<String> {
        // Check CSV files accessible
        let csv_paths = [
            ".reed/text.csv",
            ".reed/routes.csv",
            ".reed/meta.csv",
            ".reed/server.csv",
            ".reed/project.csv",
        ];

        for path in &csv_paths {
            if !std::path::Path::new(path).exists() {
                return Err(ReedError::ConfigError {
                    component: "reedbase".to_string(),
                    reason: format!("Required CSV file not found: {}", path),
                });
            }
        }

        // Check cache status
        let cache_stats = cache::get_cache_stats()?;
        
        Ok(format!(
            "ReedBase healthy: {} entries cached, {} CSV files loaded",
            cache_stats.total_entries,
            csv_paths.len()
        ))
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["csv_handler", "backup_system"]
    }
}

/// Returns ReedBase module instance.
pub fn module() -> ReedBaseModule {
    ReedBaseModule
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/reedbase/get.rs` - Get operations
- `src/reedcms/reedbase/set.rs` - Set operations
- `src/reedcms/reedbase/init.rs` - Initialisation
- `src/reedcms/reedbase/cache.rs` - Cache management

**Note**: `environment.rs` is owned by REED-02-03 (Environment Resolution Service) and is referenced by this module.

### Test Files
- `src/reedcms/reedbase/get.test.rs`
- `src/reedcms/reedbase/set.test.rs`
- `src/reedcms/reedbase/init.test.rs`
- `src/reedcms/reedbase/cache.test.rs`

**Note**: `environment.test.rs` is owned by REED-02-03.

## File Structure
```
src/reedcms/reedbase/
├── get.rs              # Get operations
├── get.test.rs         # Get tests
├── set.rs              # Set operations
├── set.test.rs         # Set tests
├── init.rs             # Initialisation
├── init.test.rs        # Init tests
├── cache.rs            # Cache system
├── cache.test.rs       # Cache tests
├── environment.rs      # Environment resolution
└── environment.test.rs # Environment tests
```

## Testing Requirements

### Unit Tests (per file)
- [ ] Test successful get operations
- [ ] Test NotFound error handling
- [ ] Test environment fallback (key@dev → key)
- [ ] Test set operations with validation
- [ ] Test cache updates after set
- [ ] Test initialisation with test CSV files

### Integration Tests
- [ ] Test set followed by get (round-trip)
- [ ] Test multiple languages
- [ ] Test environment override isolation
- [ ] Test concurrent cache access

### Performance Tests
- [ ] Get operations: p95 < 100μs
- [ ] Set operations: p95 < 10ms
- [ ] Init operation: < 50ms (loads 5 CSV files + builds HashMap caches)
- [ ] Cache lookups: O(1) verified

## Standards Compliance

### Mandatory File Headers
Each file must include the standard header with specific file purpose.

### Documentation Standard
Every public function must follow the template from `service-template.md`.

### KISS Principle
- One file = One responsibility
- `get.rs` only handles get operations
- `set.rs` only handles set operations
- No shared "handler.rs" or "utils.rs" files

## Acceptance Criteria
- [ ] All get/set/init functions implemented
- [ ] Cache system working with RwLock
- [ ] Environment fallback logic correct
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks meet targets (< 100μs get, < 10ms set)
- [ ] File headers and documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-01-01 (ReedStream), REED-01-02 (ReedError)

## Blocks
This ticket blocks:
- REED-02-03 (Environment Fallback relies on these services)
- REED-04-02 (CLI Data Commands need ReedBase)
- REED-05-01 (Template Filters need ReedBase)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 746-749, 1236-1248 in `project_summary.md`

## Notes
This is the heart of ReedCMS data access. Prioritise O(1) performance and correct environment fallback logic. The cache system must be thread-safe with RwLock for concurrent access.