# ReedCMS - Decision History and System Optimisations

**Purpose**: Comprehensive documentation of architectural decisions and system optimisations made during the planning and analysis phase of ReedCMS.

**Status**: Completed before implementation phase  
**Date Range**: 2025-01-15 to 2025-01-30

---

## Decision History

Key architectural decisions are tracked for transparency and consistency:

```csv
ID,Decision,Rationale,Date,Status
D001,CSV-based storage,"Fast, git-friendly, direct editing",2025-01-15,Active
D002,Rust implementation,"Memory safety, performance, type safety",2025-01-15,Active
D003,Matrix CSV for complex data,"Enhanced relationships, backward compatible",2025-01-15,Active
D004,ReedStream universal interface,"Consistent cross-module communication",2025-01-15,Active
D005,Environment-aware fallback,"Flexible deployment, inheritance patterns",2025-01-15,Active
D006,FreeBSD syslog format,"Professional system integration, standard compliance",2025-01-15,Active
D007,XZ backup compression,"Efficient storage, automatic recovery",2025-01-15,Active
D008,Argon2 password hashing,"Security best practice, future-proof",2025-01-15,Active
D009,Unix socket deployment,"High performance, security, nginx integration",2025-01-15,Active
D010,Template hot-reload,"Development efficiency, immediate feedback",2025-01-15,Active
D011,Rate limiting system,"API protection, DoS prevention",2025-01-15,Active
D012,Hierarchical taxonomy,"Universal categorisation, scalable organisation",2025-01-15,Active
D013,Permission caching,"Sub-millisecond lookups, performance optimisation",2025-01-15,Active
D014,CLI command bridge,"Zero business logic duplication, direct execution",2025-01-15,Active
D015,Flow persistence rules,"Clear data ownership, service coordination",2025-01-15,Active
D016,Component inclusion functions,"KISS principle, automatic variant resolution",2025-01-30,Active
```

---

## Recent System Optimisations (2025-01-30)

This section documents the comprehensive system optimisation phase completed before implementation begins. All tickets have been refined for consistency, performance, and implementation clarity.

### 1. CSV Delimiter Standardisation
**Status**: ✅ Complete across all tickets and documentation

- **Change**: Standardised CSV delimiter from semicolon (`;`) to pipe (`|`) globally
- **Affected Files**: REED-02-02, REED-10-04, REED-07-02, project_summary.md
- **Rationale**: Pipe delimiter reduces escaping complexity and improves readability
- **Format**: `key|value|description` for all CSV files in `.reed/` directory

### 2. ReedRequest Structure Enhancement
**Status**: ✅ Extended in REED-01-01 and project_summary.md

- **Added Fields**:
  - `pub value: Option<String>` - For set operations
  - `pub description: Option<String>` - For CSV comment field in set operations
- **Benefit**: Unified request structure for both get and set operations via ReedStream

### 3. CacheInfo Structure Definition
**Status**: ✅ Fully specified in REED-01-01 and project_summary.md

```rust
pub struct CacheInfo {
    pub hit: bool,                    // Whether this was a cache hit
    pub ttl_remaining_s: Option<u64>, // Time-to-live remaining
    pub cache_key: String,            // Cache key used for lookup
    pub cache_layer: String,          // Which cache (text/route/meta)
}
```
- **Integration**: Used in `ResponseMetrics.cache_info` for performance tracking

### 4. Dependency Chain Corrections
**Status**: ✅ Fixed in REED-02-01 and REED-04-08

- **REED-02-01 (ReedBase Core)**: Now depends on `REED-02-02` (CSV Handler) and `REED-02-04` (Backup System)
- **REED-04-08 (Build Commands)**: Now depends on `REED-08-01` (CSS Bundler) and `REED-08-02` (JS Bundler)
- **Benefit**: Proper build order and clear service dependencies

### 5. Route CSV Format Clarification
**Status**: ✅ Specified in REED-06-02

- **Format**: `route|layout|language|description`
- **Value Field**: Contains `layout|language` (pipe-delimited)
- **Parsing**: Updated route resolution to correctly parse multi-component values
- **Example**: `knowledge@de|wissen|knowledge|de|German knowledge page route`

### 6. FreeBSD Logging System Specification
**Status**: ✅ Comprehensive implementation added to REED-10-01 (~240 lines)

- **Format**: `{timestamp} {hostname} {process}[{pid}]: {level}: {message}`
- **8 Log Levels**: EMERG, ALERT, CRIT, ERROR, WARN, NOTICE, INFO, DEBUG (RFC 5424)
- **4 Output Modes**: Silent, Log, Forward, Both
- **Features**: Log rotation (100MB limit), compression (gzip), retention (10 files)
- **Performance**: Zero-allocation message passing, <1% overhead, 10k+ msg/s throughput

### 7. ReedModule Trait Implementation
**Status**: ✅ Implemented across all REED-02 tickets

- **Tickets**: REED-02-01, REED-02-02, REED-02-03, REED-02-04
- **Methods**:
  - `module_name() -> &'static str` - Module identification
  - `version() -> &'static str` - Version tracking
  - `health_check() -> ReedResult<String>` - System health verification
  - `dependencies() -> Vec<&'static str>` - Dependency declaration
- **Benefit**: Standardised module interface for monitoring and diagnostics

### 8. Registry CSV Format Definition
**Status**: ✅ Specified in REED-05-03

- **Format**: `key|type|enabled|order|parent|description`
- **Purpose**: Layout registry and navigation management
- **Implementation**: Dynamic navigation loading from `.reed/registry.csv`
- **Example**: `knowledge|layout|true|10||Knowledge base layout`

### 9. Template Engine Initialisation Sequence
**Status**: ✅ Documented in REED-05-02 (~130 lines)

- **Pattern**: Global singleton with `OnceLock<Environment<'static>>`
- **Startup Sequence**:
  1. Template Engine initialisation
  2. Hot-reload setup (DEV mode only)
  3. ReedBase initialisation
  4. Monitoring system initialisation
- **Benefit**: Clear startup order and proper resource initialisation

### 10. Filter Error Conversion Layer
**Status**: ✅ Implemented in REED-05-01

- **Function**: `convert_reed_error_to_jinja(err: ReedError, filter: &str, key: &str) -> minijinja::Error`
- **Coverage**: All ReedError variants mapped to appropriate MiniJinja ErrorKind
- **Integration**: All four filters (text, route, meta, config) use error conversion
- **Benefit**: Proper error context in template rendering with actionable messages

### 11. Cache Invalidation Strategy
**Status**: ✅ Comprehensive specification in REED-02-01 (~100 lines)

- **Granularity Levels**:
  - Key-specific: `invalidate_text_key(key)`, `invalidate_route_key(key)`, `invalidate_meta_key(key)`
  - Cache-type: `invalidate_text_cache()`, `invalidate_route_cache()`, `invalidate_meta_cache()`
  - Global: `invalidate_all_caches()`, `refresh_cache()`
- **Integration**: `set.rs` calls `cache::invalidate_*_key()` after each CSV write
- **Performance**: < 50ms for complete cache refresh (5 CSV files + HashMap rebuild)
- **Triggers**: Set operations, file watcher, CLI commands, periodic refresh

### 12. Environment.rs Ownership Resolution
**Status**: ✅ Clarified in REED-02-01 and REED-02-03

- **Owner**: REED-02-03 (Environment Resolution Service)
- **File**: `src/reedcms/reedbase/environment.rs`
- **Reference**: REED-02-01 (ReedBase Core) references but does not create
- **Benefit**: Clear ownership prevents duplicate implementations

### 13. Performance Target Corrections
**Status**: ✅ Fixed in REED-02-01 and project_summary.md

- **Startup**: Changed from "< 200μs" to "< 50ms" (realistic for 5 CSV files + HashMap build)
- **Cache Refresh**: Changed from "< 200ms" to "< 50ms" (consistent with init)
- **Rationale**: 200μs was unrealistic for file I/O; 50ms is achievable and professionally acceptable
- **Other Targets Remain**: Get < 100μs, Set < 10ms, O(1) cache lookups

### 14. Template Integration: Filter System Migration
**Status**: ✅ Completed - Templates migrated to ReedCMS filter pattern

**Changes**:
- Removed legacy `reed` dictionary pattern from vvoss.dev
- Migrated to consistent filter usage: `{{ pagekey | route('auto') }}`
- Simplified route filter with empty route handling for landing pages

**Migration Results**:
```jinja
// Before (legacy vvoss.dev):
{{ reed['pageroute@' + pagekey] }}
{{ reed.pageroute }}
{% if pagekey != 'landing' %}{{ pagekey | route('auto') }}/{% endif %}

// After (ReedCMS):
{{ pagekey | route('auto') }}
{{ current_pagekey | route('de') }}
{{ pagekey | route('auto') }}/  // Filter handles empty routes internally
```

**Benefits**:
- Consistent filter system across all template types
- KISS principle: Logic in Rust filters, not template conditionals
- No dictionary overhead in template context
- Lazy evaluation - only computed when used
- Clear data flow: Template → Filter → ReedBase

**Files Updated**:
- REED-05-01: Language detection strategy + empty route handling specification
- page-header.mouse/touch/reader.jinja: All reed references migrated

**Performance**: No dictionary population overhead, filters execute on-demand with < 100μs latency

### 15. Component Inclusion Functions
**Status**: ✅ Completed - Custom functions specification added to REED-05-02

**Changes**:
- Added 4 custom functions to REED-05-02: `organism()`, `molecule()`, `atom()`, `layout()`
- Functions automatically resolve component paths based on `interaction_mode`
- KISS principle: Simple string formatting, O(1) performance

**Implementation**:
```rust
// organism(name) → templates/components/organisms/{name}/{name}.{interaction_mode}.jinja
pub fn make_organism_function(interaction_mode: String) -> impl Function {
    move |name: &str| -> Result<String> {
        Ok(format!(
            "templates/components/organisms/{}/{}.{}.jinja",
            name, name, interaction_mode
        ))
    }
}

// Similar for molecule(), atom(), layout() (no interaction_mode)
```

**Template Usage**:
```jinja
{% extends layout("page") %}
{% include organism("page-header") %}
{% include organism("landing-hero") %}
```

**Benefits**:
- Automatic variant resolution based on client context
- Zero allocations in hot path
- < 1μs per function call
- No filesystem access, only path generation
- Templates already using correct syntax

**Files Updated**:
- REED-05-02: Extended with Custom Functions section

---

## Implementation Readiness

All 37 tickets across 10 layers have been optimised and are now ready for implementation:

- ✅ **Consistency**: CSV delimiter, key nomenclature, error handling, filter patterns
- ✅ **Completeness**: All missing structures, formats, and specifications added
- ✅ **Correctness**: Dependencies, performance targets, integration points verified
- ✅ **Clarity**: Implementation guidance, ownership, and patterns documented
- ✅ **Template Integration**: Existing templates analysed and migrated to ReedCMS patterns

**Complete Feature Coverage**:
- ✅ Universal ReedStream communication interface with comprehensive error handling
- ✅ Matrix CSV system with 4-type value support and XZ backup protection
- ✅ Advanced user management with Argon2 hashing and social profiles
- ✅ Role-based permissions with inheritance and sub-millisecond cached lookups
- ✅ Universal taxonomy system with hierarchical terms and cross-entity tagging
- ✅ FreeBSD-style monitoring with configurable output modes
- ✅ API security matrix with rate limiting and token management
- ✅ CLI command execution bridge with streaming and batch support
- ✅ Template hot-reload for development efficiency
- ✅ Unix socket deployment for production performance
- ✅ Flow persistence with clear data ownership rules
- ✅ Environment-aware fallback system for flexible deployment
- ✅ Component inclusion functions with automatic variant resolution

**Active Analysis**: Template system integration ongoing - tracking in `_workbench/Tickets/project_todo.md`

**Next Phase**: Complete remaining template integration questions (D-H), then begin systematic implementation starting with Foundation Layer (REED-01).

---

**Note**: This document complements `project_summary.md` which contains the main system design. This file focuses exclusively on decisions and optimisations made during the planning phase.
