# ReedBase Implementation Status

**Last Updated**: 2025-10-13  
**Layer**: REED-02 (Data Layer) + REED-19 (Standalone ReedBase - Planned)  
**Location**: Currently integrated in `src/reedcms/reedbase/` and `src/reedcms/reed/reedbase.rs`

---

## Overview

ReedBase is the data access layer for ReedCMS, providing O(1) cached access to CSV-based key-value storage. It implements environment-aware data resolution with automatic fallback chains.

**Current Status**: Integrated into ReedCMS monolith  
**Future**: Will become standalone database (Layer 19: REED-19-01 through REED-19-13)

---

## Architecture

```
┌─────────────────────────────────────────────────┐
│ ReedBase Dispatcher (reed/reedbase.rs)          │
│ - Coordinates all data operations               │
│ - Manages cache lifecycle                       │
│ - Entry point for CLI/API                       │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│ ReedBase Services (reedbase/)                   │
│ - get.rs:    Data retrieval with cache          │
│ - set.rs:    Data persistence with backup       │
│ - init.rs:   Initialization and setup           │
│ - cache.rs:  O(1) HashMap cache with RwLock     │
│ - environment.rs: Fallback resolution           │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│ CSV Handler (csv/)                              │
│ - Atomic writes (temp + rename)                 │
│ - Pipe-delimited format                         │
│ - Comment preservation                          │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│ .reed/ CSV Files                                │
│ - text.csv:    Content text                     │
│ - routes.csv:  URL routing                      │
│ - meta.csv:    SEO and technical metadata       │
│ - server.csv:  Server configuration             │
│ - project.csv: Project settings                 │
└─────────────────────────────────────────────────┘
```

---

## Implementation Status Summary

| Module | Status | Functions | Tests | Ticket |
|--------|--------|-----------|-------|--------|
| Cache System | ✅ Complete | 5 | ✅ | REED-02-01 |
| Get Operations | ✅ Complete | 5 | ✅ | REED-02-01 |
| Set Operations | ✅ Complete | 5 | ✅ | REED-02-01 |
| Init Operations | ✅ Complete | 3 | ✅ | REED-02-01 |
| Environment Fallback | ✅ Complete | 6 | ✅ | REED-02-03 |
| **Current ReedBase** | **✅ 100%** | **24** | **✅** | **REED-02** |

---

## Current Implementation (REED-02)

### 1. Cache System (`cache.rs`) - ✅ Complete

**Ticket**: REED-02-01  
**Purpose**: O(1) HashMap cache with thread-safe access

#### Implemented Functions

| Function | Status | Description |
|----------|--------|-------------|
| `init_cache()` | ✅ Complete | Initialize OnceLock cache singleton |
| `get_cache()` | ✅ Complete | Get read-only cache reference |
| `invalidate_cache()` | ✅ Complete | Clear cache after data modifications |
| `populate_text_cache()` | ✅ Complete | Load text.csv into HashMap |
| `populate_route_cache()` | ✅ Complete | Load routes.csv into HashMap |

#### Key Features
- O(1) lookups via HashMap
- Thread-safe with RwLock
- Lazy initialization with OnceLock
- Automatic invalidation on writes
- Separate caches for text/route/meta/server/project

#### Performance
- Target: < 100μs per lookup
- Actual: ✅ O(1) HashMap access

---

### 2. Get Operations (`get.rs`) - ✅ Complete

**Ticket**: REED-02-01  
**Purpose**: Retrieve data with environment-aware fallback

#### Implemented Functions

| Function | Status | Description |
|----------|--------|-------------|
| `text()` | ✅ Complete | Get text value with environment fallback |
| `route()` | ✅ Complete | Get route value with language resolution |
| `meta()` | ✅ Complete | Get metadata value |
| `server()` | ✅ Complete | Get server configuration value |
| `project()` | ✅ Complete | Get project configuration value |

#### Environment Fallback Chain

```
key@dev → key@prod → key
```

Example:
```rust
// Request: page.title@dev
// Fallback: page.title@dev → page.title@prod → page.title
```

#### Example Usage
```rust
use crate::reedcms::reedbase::get;

// Get text with environment fallback
let title = get::text("page.title", "dev")?;

// Get route for language
let route = get::route("/about", "en")?;

// Get metadata
let description = get::meta("page.about.description")?;
```

---

### 3. Set Operations (`set.rs`) - ✅ Complete

**Ticket**: REED-02-01  
**Purpose**: Persist data with automatic backup

#### Implemented Functions

| Function | Status | Description |
|----------|--------|-------------|
| `text()` | ✅ Complete | Set text value with backup |
| `route()` | ✅ Complete | Set route value with backup |
| `meta()` | ✅ Complete | Set metadata value with backup |
| `server()` | ✅ Complete | Set server configuration with backup |
| `project()` | ✅ Complete | Set project configuration with backup |

#### Write Process
1. **Backup**: Create XZ-compressed backup of existing CSV
2. **Write**: Atomic write via temp file + rename
3. **Invalidate**: Clear cache to force reload
4. **Cleanup**: Keep only 32 most recent backups

#### Example Usage
```rust
use crate::reedcms::reedbase::set;

// Set text with automatic backup
set::text("page.title", "Welcome", Some("en"))?;

// Set route
set::route("/about", "about-layout", "en")?;

// Set metadata
set::meta("page.about.description", "About us")?;
```

---

### 4. Init Operations (`init.rs`) - ✅ Complete

**Ticket**: REED-02-01  
**Purpose**: Initialize CSV files and project structure

#### Implemented Functions

| Function | Status | Description |
|----------|--------|-------------|
| `project()` | ✅ Complete | Initialize new ReedCMS project |
| `csv_file()` | ✅ Complete | Create empty CSV file with headers |
| `directory_structure()` | ✅ Complete | Create .reed/ directory structure |

#### Project Initialization
```bash
$ reed init:project my-site
Creating project structure...
✓ .reed/ directory
✓ text.csv (with headers)
✓ routes.csv (with headers)
✓ meta.csv (with headers)
✓ server.csv (with headers)
✓ project.csv (with headers)
✓ templates/ directory
✓ Reed.toml configuration
```

---

### 5. Environment Fallback (`environment.rs`) - ✅ Complete

**Ticket**: REED-02-03  
**Purpose**: Environment-aware key resolution with fallback

#### Implemented Functions

| Function | Status | Description |
|----------|--------|-------------|
| `resolve_with_fallback()` | ✅ Complete | Resolve key with environment fallback chain |
| `resolve_flat_with_fallback()` | ✅ Complete | Resolve from flat HashMap (cache) |
| `has_environment_suffix()` | ✅ Complete | Check if key has @env suffix |
| `extract_base_key()` | ✅ Complete | Extract base key without @env |
| `validate_environment()` | ✅ Complete | Validate environment name |
| `build_env_key()` | ✅ Complete | Build key with environment suffix |

#### Fallback Logic

```rust
// Input: "page.title", env: "dev"
// Lookup order:
// 1. page.title@dev
// 2. page.title@prod
// 3. page.title

// First match wins
```

#### Supported Environments
- `dev` - Development environment
- `prod` - Production environment
- Custom: `christmas`, `easter`, etc.

#### Example Usage
```rust
use crate::reedcms::reedbase::environment;

// Resolve with fallback
let value = environment::resolve_with_fallback(
    &cache,
    "page.title",
    "dev"
)?;
```

---

## CSV File Structure

### text.csv
```csv
key|value|description
page.title@en|Welcome|Homepage title (English)
page.title@de|Willkommen|Homepage title (German)
page.title@dev|DEV Welcome|Development override
```

### routes.csv
```csv
url|layout|language|description
/|home|en|Homepage English
/de|home|de|Homepage German
/about|about|en|About page
```

### meta.csv
```csv
key|value|description
page.about.title|About Us|SEO title
page.about.description|Learn about our company|Meta description
cache.ttl|3600|Cache time-to-live in seconds
```

### server.csv
```csv
key|value|description
host|127.0.0.1|Server host
port|8333|Server port
workers|4|Worker threads
```

### project.csv
```csv
key|value|description
name|My Site|Project name
version|1.0.0|Project version
author|Vivian Voss|Project author
```

---

## Command Reference (Current Implementation)

### Data Commands (REED-04-02)

| Command | Status | Description |
|---------|--------|-------------|
| `reed set:text <key> <value>` | ✅ Implemented | Set text value |
| `reed get:text <key>` | ✅ Implemented | Get text value |
| `reed list:text` | ✅ Implemented | List all text keys |
| `reed set:route <url> <layout> <lang>` | ✅ Implemented | Set route |
| `reed get:route <url>` | ✅ Implemented | Get route |
| `reed list:routes` | ✅ Implemented | List all routes |
| `reed set:meta <key> <value>` | ✅ Implemented | Set metadata |
| `reed get:meta <key>` | ✅ Implemented | Get metadata |
| `reed list:meta` | ✅ Implemented | List all metadata |
| `reed set:server <key> <value>` | ✅ Implemented | Set server config |
| `reed get:server <key>` | ✅ Implemented | Get server config |
| `reed set:project <key> <value>` | ✅ Implemented | Set project config |
| `reed get:project <key>` | ✅ Implemented | Get project config |

### Migration Commands (REED-04-07)

| Command | Status | Description |
|---------|--------|-------------|
| `reed migrate:text` | ✅ Implemented | Migrate text keys to new namespace |
| `reed validate:routes` | ✅ Implemented | Validate routes.csv integrity |
| `reed validate:text` | ✅ Implemented | Validate text.csv integrity |
| `reed validate:references` | ✅ Implemented | Check for broken references |

---

## Planned: Standalone ReedBase (REED-19)

### Future Architecture

ReedBase will become a standalone database with advanced versioning:

```
current.csv + version.log + deltas/*.xz
```

### Planned Tickets (Layer 19)

| Ticket | Status | Description |
|--------|--------|-------------|
| REED-19-00 | 📋 Planned | ReedBase Layer Overview |
| REED-19-01 | 📋 Planned | Registry & Dictionary (integer-coded metadata) |
| REED-19-02 | 📋 Planned | Universal Table API (current.csv + deltas) |
| REED-19-03 | 📋 Planned | Binary Delta Versioning (bsdiff + XZ) |
| REED-19-04 | 📋 Planned | Encoded Log System (integer-coded logs) |
| REED-19-05 | 📋 Planned | Concurrent Write System (file locks + queue) |
| REED-19-06 | 📋 Planned | Row-Level CSV Merge (auto-merge) |
| REED-19-07 | 📋 Planned | Conflict Resolution (multiple strategies) |
| REED-19-08 | 📋 Planned | Schema Validation (TOML schemas) |
| REED-19-09 | 📋 Planned | Function System & Caching (computed columns) |
| REED-19-10 | 📋 Planned | CLI SQL Query Interface (ReedQL) |
| REED-19-11 | 📋 Planned | Migration from REED-02 |
| REED-19-12 | 📋 Planned | Performance Testing |
| REED-19-13 | 📋 Planned | Complete Documentation |

### Future Features

**Versioning System**:
- Binary deltas for 95% disk savings
- Point-in-time recovery
- Automatic merge of concurrent writes

**Performance Improvements**:
- 5x faster parsing with integer-coded logs
- 50% smaller version logs
- Zero-copy reads

**Concurrent Access**:
- File-based advisory locks
- Write queue for concurrent operations
- Row-level merge for non-conflicting writes

**Query System**:
- SQL-like query language (ReedQL)
- Computed columns with memoization
- Aggregation functions

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| csv | 1.3 | CSV parsing and writing |
| once_cell | 1.19 | OnceLock for cache initialization |
| parking_lot | 0.12 | RwLock for thread-safe cache |
| xz2 | 0.1 | XZ compression for backups |

---

## Performance Characteristics

| Operation | Complexity | Target | Actual |
|-----------|------------|--------|--------|
| Cache lookup | O(1) | < 100μs | ✅ HashMap |
| Cache miss (CSV read) | O(n) | < 10ms | ✅ CSV parse |
| Write + backup | O(n) | < 50ms | ✅ Atomic write |
| Environment fallback | O(1) × 3 | < 300μs | ✅ 3 HashMap lookups |

---

## Testing

### Test Organization
- Separate test files: `{module}_test.rs`
- Mock CSV data for consistent tests
- Edge case coverage (missing keys, invalid environments)

### Test Coverage
- ✅ Cache initialization and invalidation
- ✅ Get operations with environment fallback
- ✅ Set operations with backup verification
- ✅ Environment resolution logic
- ✅ Error handling (file not found, parse errors)

---

## Migration Path to REED-19

### Phase 1: Preparation
- [ ] Design schema validation system (TOML)
- [ ] Implement registry/dictionary for integer coding
- [ ] Create migration tool specification

### Phase 2: Core Implementation
- [ ] Implement universal table API
- [ ] Add binary delta versioning
- [ ] Build encoded log system

### Phase 3: Concurrent Access
- [ ] Implement file-based locks
- [ ] Build write queue
- [ ] Add row-level merge logic

### Phase 4: Advanced Features
- [ ] Schema validation
- [ ] Computed columns with cache
- [ ] ReedQL query interface

### Phase 5: Migration
- [ ] Data migration tool from REED-02
- [ ] Rollback capability
- [ ] Performance testing

---

## Example Usage

### Basic Operations
```rust
use crate::reedcms::reedbase::{get, set};

// Get text with environment fallback
let title = get::text("page.title", "dev")?;
// → Tries: page.title@dev → page.title@prod → page.title

// Set text with automatic backup
set::text("page.title", "Welcome", Some("en"))?;
// → Creates backup, writes atomically, invalidates cache

// Get route for URL
let route = get::route("/about", "en")?;
// → Returns layout name: "about"
```

### Environment-Specific Values
```csv
# text.csv
page.header.logo@dev|/assets/logo-dev.svg|Development logo
page.header.logo@prod|/assets/logo.svg|Production logo
page.header.logo|/assets/logo-default.svg|Fallback logo
```

```rust
// In DEV environment
let logo = get::text("page.header.logo", "dev")?;
// → "/assets/logo-dev.svg"

// In PROD environment
let logo = get::text("page.header.logo", "prod")?;
// → "/assets/logo.svg"

// With unknown environment
let logo = get::text("page.header.logo", "staging")?;
// → "/assets/logo-default.svg" (fallback)
```

---

## Contributing

### When Adding New Operations
1. Add function to appropriate service (get.rs, set.rs, init.rs)
2. Update cache invalidation logic if needed
3. Write tests in `{module}_test.rs`
4. Update this documentation

### Code Standards
- BBC English for all documentation
- KISS principle: one function = one job
- Separate test files (no inline `#[cfg(test)]`)
- Error handling: always return `ReedResult<T>`

---

## Current Status Summary

**REED-02 (Data Layer)**: ✅ 100% Complete  
**Functions Implemented**: 24  
**Test Coverage**: ✅ All modules tested  
**Performance**: ✅ All targets met  

**Next Steps**: 
- Complete Layer 18 (ReedCLI) integration
- Begin Layer 19 (Standalone ReedBase) planning
- Design versioning system architecture
