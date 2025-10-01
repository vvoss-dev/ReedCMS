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
D017,Taxonomy-based navigation,"Drupal-style flexibility, multiple menu locations",2025-01-30,Active
D018,Session hash asset bundling,"MD5-based cache-busting, on-demand generation",2025-01-30,Active
D019,Client detection via cookie,"Server-side responsive rendering, no JS needed",2025-01-30,Active
D020,SVG icon molecule wrapper,"Atomic Design compliance, accessibility support",2025-01-30,Active
D021,Full namespace text keys,"No auto-prefixing, explicit key format validation",2025-01-30,Active
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

### 17. Service Template Standardisation
**Status**: ✅ Complete - All templates aligned with REED-01-01 specification

**Changes**:
- **CSV Delimiter**: Changed from semicolon (`;`) to pipe (`|`) in all file headers and examples
- **ReedRequest Structure**: Updated field names (`lang` → `language`) and optionality (`value`, `description` now `Option<String>`)
- **ReedResponse Structure**: Aligned with REED-01-01 (removed `success`, `message`; added `source`, `cached`, `timestamp`, `metrics`)
- **ReedError Variants**: Reduced to 10 base variants matching REED-01-01:
  - `NotFound`, `ValidationError`, `ParseError`, `IoError`
  - `CsvError`, `AuthError`, `ConfigError`, `TemplateError`
  - `ServerError`, `InvalidCommand`
- **Error Messages**: Changed to simple format strings with `{field:?}` syntax (Option A)
- **Response Constructors**: Added helper functions (`new()`, `with_metrics()`)
- **Timestamp Helper**: Added central `current_timestamp()` function
- **Convenience Functions**: Specified all 10 ReedError convenience constructors

**Affected Files**:
- `_workbench/Tickets/templates/service-template.md` (primary template)
- `_workbench/Tickets/templates/service-template.test.md` (unchanged, already consistent)

**Benefits**:
- Single source of truth for all service implementations
- Consistent error handling across all modules
- Clear separation: base errors (REED-01-01) vs. extended errors (REED-01-02)
- Ergonomic helper functions for common patterns
- Zero confusion about pipe vs. semicolon delimiter

**Implementation Decisions**:
- Error format: Option A (simple `{field:?}` strings)
- Response helpers: Yes (ergonomic API)
- Timestamp function: Central in `reedstream.rs`
- Convenience functions: All 10 variants
- Cargo.toml: Self-managed, extended as needed
- Directory structure: `src/reedcms/` created from scratch

**Files**: `_workbench/Tickets/templates/service-template.md`

---

**Note**: This document complements `project_summary.md` which contains the main system design. This file focuses exclusively on decisions and optimisations made during the planning phase.

---

## Template Integration Decisions (2025-01-30)

### 16. Template System Integration Analysis - COMPLETED

**Status**: ✅ All 9 questions resolved  
**Duration**: 2025-01-30 (single day)  
**Result**: Existing templates 100% compatible with ReedCMS architecture

#### Decision D017: Taxonomy-Based Navigation (Drupal-Style)
**Problem**: Templates had hardcoded navigation arrays. How to make dynamic?

**Solution**: Use REED-03-03 taxonomy system with Matrix Type 4 syntax:
```csv
# .reed/entity_taxonomy.matrix.csv
entity_id|term_id|properties|desc
knowledge|navigation|weight[10],enabled[true]|Main navigation entry
portfolio|navigation|weight[20],enabled[true]|Main navigation entry
impressum|footer-legal|weight[10],enabled[true]|Footer link
```

**Template Usage**:
```jinja
{% for item in taxonomy('navigation') %}
  <a href="/{{ client.lang }}/{{ item.entity_id | route('auto') }}/">
    {{ item.entity_id | text('auto') }}
  </a>
{% endfor %}
```

**Benefits**:
- Multiple menu locations (navigation, footer-legal, sidebar, etc.)
- Weight-based ordering
- Enable/disable per item
- Hierarchical support with `parent[term_id]`
- CLI management: `reed taxonomy:assign knowledge navigation weight[10],enabled[true]`

**Files**: REED-03-03, REED-05-03

---

#### Decision D018: Session Hash Asset Bundling
**Problem**: How to bundle CSS/JS per layout with cache-busting?

**Solution**: MD5 session hash over all CSS/JS files:
- Generate hash: `MD5(all_css_files + all_js_files)` → 8-char hex
- Store in `.reed/project.csv`: `project.session_hash|a3f5b2c8`
- Bundle naming: `{layout}.{session_hash}.{variant}.css`
- On-demand generation on first request
- Template variables: `{{ asset_css }}`, `{{ asset_js }}`

**Example**:
```
/public/session/styles/landing.a3f5b2c8.mouse.css
/public/session/scripts/landing.a3f5b2c8.js
```

**Benefits**:
- Automatic cache invalidation when files change
- No manual versioning needed
- Per-layout bundles (optimal loading)
- On-demand generation (no build-time overhead)
- Component discovery via template parsing

**Files**: REED-08-01, REED-05-03

---

#### Decision D019: Client Detection via Screen Info Cookie
**Problem**: How to detect device type and interaction mode for server-side variant selection?

**Solution**: JavaScript screen info cookie on first visit:
```javascript
document.cookie='screen_info='+encodeURIComponent(JSON.stringify({
  width: screen.width,
  height: screen.height,
  dpr: window.devicePixelRatio,
  viewport_width: window.innerWidth,
  viewport_height: window.innerHeight,
  active_voices: window.speechSynthesis.getVoices().length
}));
```

**ClientInfo Structure**:
```rust
pub struct ClientInfo {
    pub lang: String,
    pub interaction_mode: String,      // mouse/touch/reader
    pub device_type: String,           // mobile/tablet/desktop/bot
    pub breakpoint: String,            // phone/tablet/screen/wide
    pub viewport_width: Option<u32>,
    pub is_bot: bool,
}
```

**Detection Logic**:
- `reader` mode: active_voices > 0 (screen reader detected)
- `touch` mode: mobile/tablet device
- `mouse` mode: desktop device
- Breakpoints: phone (≤559px), tablet (≤959px), screen (≤1259px), wide (>1259px)

**Benefits**:
- Server-side responsive rendering
- No client-side detection logic
- Accessibility support (reader mode)
- SEO optimised (bots get reader mode)
- One-time detection (cookie persists)

**Files**: REED-06-05 (new ticket), REED-05-03

---

#### Decision D020: SVG Icon Molecule Wrapper
**Problem**: Icon atoms contain only SVG fragments. How to add `<svg>` wrapper?

**Solution**: Use existing `svg-icon` molecule as wrapper:
```jinja
{% include molecule('svg-icon') with {
  icon: "arrow-right",
  size: "24",
  class: "nav-icon",
  alt: "Next page"
} %}
```

**Molecule Responsibilities**:
- Adds `<svg>` element with viewBox, stroke attributes
- Handles size (width/height)
- Adds CSS class for styling
- Provides accessibility (role="img", aria-label)
- Error handling with fallback icon
- Includes atom: `atoms/icons/{icon_name}.jinja`

**Benefits**:
- Atomic Design compliance (Molecule = Atom + Wrapper)
- Variant-specific CSS (`svg-icon.mouse.css` vs `svg-icon.touch.css`)
- Centralised accessibility attributes
- Template-based (no Rust function needed)
- Error handling with fallback

**Files**: Existing `templates/components/molecules/svg-icon/`

---

#### Decision D021: Full Namespace Text Keys (No Auto-Prefixing)
**Problem**: Do migration commands auto-prefix component keys?

**Solution**: NO auto-prefixing. Keys MUST have full namespace:
- Component keys: `page-header.logo.title@de` ✅
- Layout keys: `knowledge.intro.title@de` ✅
- Partial keys rejected: `logo.title@de` ❌
- No `@lang` suffix rejected: `page-header.logo.title` ❌

**Migration Process**:
1. Discover `.text.csv` files
2. Validate: key has `@lang` suffix
3. Validate: key has namespace (contains `.`)
4. Check duplicates in `.reed/text.csv`
5. XZ backup
6. Direct 1:1 append to `.reed/text.csv`

**Benefits**:
- No ambiguity about key format
- Simple validation: check for `@` and `.`
- No complex auto-prefixing logic
- Consistent with key nomenclature rules
- KISS principle: direct copy

**Files**: REED-04-07

---

### Integration Statistics

**Questions Resolved**: 9 (A, B, B.1, C, D, E, F, G, H)  
**Tickets Created**: 1 (REED-06-05)  
**Tickets Extended**: 6 (REED-05-01, REED-05-02, REED-05-03, REED-03-03, REED-08-01, REED-04-07)  
**Template Files Analyzed**: 31 `.text.csv` files, 14 `.jinja` files  
**New Decisions**: 5 (D017-D021)  
**Result**: Template integration ready for implementation

---
