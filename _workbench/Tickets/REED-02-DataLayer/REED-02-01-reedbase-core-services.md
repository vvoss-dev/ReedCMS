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
- **Dependencies**: REED-01-01, REED-01-02

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
```

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

## Implementation Files

### Primary Implementation
- `src/reedcms/reedbase/get.rs` - Get operations
- `src/reedcms/reedbase/set.rs` - Set operations
- `src/reedcms/reedbase/init.rs` - Initialisation
- `src/reedcms/reedbase/cache.rs` - Cache management
- `src/reedcms/reedbase/environment.rs` - Environment resolution

### Test Files
- `src/reedcms/reedbase/get.test.rs`
- `src/reedcms/reedbase/set.test.rs`
- `src/reedcms/reedbase/init.test.rs`
- `src/reedcms/reedbase/cache.test.rs`
- `src/reedcms/reedbase/environment.test.rs`

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
- [ ] Init operation: < 200μs for 17 layouts
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