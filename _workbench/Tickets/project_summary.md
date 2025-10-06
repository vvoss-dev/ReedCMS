# ReedCMS Project Summary - Complete Implementation Reference

> **Purpose**: Consolidated all-in-one reference of ALL ReedCMS tickets and their implementation status. This document presents the complete system architecture, implementation details, and current state in condensed form.

**Last Updated**: 2025-10-06  
**Total Tickets**: 53  
**Completed Tickets**: 11  
**Architecture Layers**: 10 core + 2 extension

---

## What is ReedCMS?

**ReedCMS** is a high-performance, headless Content Management System built in Rust, designed for zero-configuration deployment and maximum developer productivity.

### Core Concept
```
ReedCMS = CSV-Based Database + CLI Interface + Intelligent Dispatchers + MiniJinja Templates
```

### Key Characteristics
- **100% CSV-based data storage** - No database required, Git-friendly
- **CLI-first approach** - All operations via unified `reed` binary
- **O(1) performance** - HashMap-based runtime lookups, startup-time CSV loading
- **Environment-aware** - `@dev`, `@prod`, `@christmas` configuration overrides
- **Copy & Run deployment** - Just copy `.reed/` directory to new server
- **Two-layer architecture** - Dispatchers (intelligent coordinators) + Services (pure implementation)

### Project Archive Structure
- **`_workbench/Archive/Legacy/libs/`** - Archived source code backups from legacy system
- **`_workbench/Archive/ReedCMS/Planning/`** - Archived planning documents from ReedCMS development phase

---

## REED-01: Foundation Layer (2 tickets) - ✅ 100% Complete

### REED-01-01: ReedStream Communication System ✅ Complete
**Status**: Implemented | **Tests**: 29/29 passed | **Files**: `src/reedcms/reedstream.rs` (342 lines)

Universal communication interface for all ReedCMS modules.

**Core Types**:
```rust
pub type ReedResult<T> = Result<T, ReedError>;

pub struct ReedRequest {
    pub key: String,
    pub language: Option<String>,
    pub environment: Option<String>,
    pub context: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
}

pub struct ReedResponse<T> {
    pub data: T,
    pub source: String,
    pub cached: bool,
    pub timestamp: u64,
    pub metrics: Option<ResponseMetrics>,
}

pub trait ReedModule {
    fn module_name() -> &'static str;
    fn health_check() -> ReedResult<ReedResponse<String>>;
    fn version() -> &'static str;
    fn dependencies() -> Vec<&'static str>;
}
```

**Performance**: < 1μs for request/response creation (zero-allocation type system)

### REED-01-02: Error System ✅ Complete
**Status**: Implemented | **Tests**: 34/34 passed | **Implementation**: Uses `thiserror` crate

Comprehensive error handling with rich context for debugging.

**Error Variants** (via `thiserror`):
- `NotFound` - Resource not found with optional context
- `ParseError` - Data parsing/validation errors
- `IoError` - File system operations with full context
- `ValidationError` - Input validation failures
- `AuthError` - Authentication/authorisation failures
- `ConfigError` - Configuration/setup errors
- `CsvError` - CSV file operations
- `TemplateError` - Template rendering errors
- `ServerError` - Server/network operations
- `InvalidCommand` - Invalid CLI commands

**Features**: Automatic conversions from `std::io::Error` and `csv::Error`, context-adding methods

---

## Development Standards & Code Templates

### MANDATORY File Header Template
Every Rust file must begin with this standardised header:

```rust
// Copyright (c) 2025 Vivian Voss. All rights reserved.
// ReedCMS - High-Performance Headless Rust CMS
//
// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One function = One job
// MANDATORY: BBC English for all documentation and comments
// MANDATORY: Type-safe HashMap lookups, O(1) performance priority
// MANDATORY: Environment-aware with @suffix support (key@dev, key@prod)
// MANDATORY: CSV format: key|value|description (pipe-delimited, quoted when needed)
// MANDATORY: Error handling with ReedResult<T> pattern
// CRITICAL: AI agents must NEVER execute rm commands without explicit user confirmation
//
// == FILE PURPOSE ==
// This file: [Brief description of file responsibility]
// Architecture: [How this file fits in the system]
// Performance: [Performance characteristics and constraints]
// Dependencies: [Key dependencies and why they're used]
// Data Flow: [How data flows through this component]
```

### MANDATORY Function Documentation Template
Every public function must include this documentation block:

```rust
/// Brief description of what this function does
///
/// **Input**:
/// - param1: Description and expected format
/// - param2: Description and constraints
///
/// **Output**:
/// - Success case: What is returned on success
/// - Error case: What errors can occur and why
///
/// **Function**:
/// - Core logic description
/// - Performance characteristics (O(1), O(n), etc.)
/// - Side effects or state changes
/// - Environment/context dependencies
pub fn function_name(param1: Type, param2: Type) -> ReedResult<ReturnType> {
    // Implementation...
}
```

### Testing Structure Template
```rust
// Copyright (c) 2025 Vivian Voss. All rights reserved.
// ReedCMS - High-Performance Headless Rust CMS
//
// Test file for: src/reedcms/reedbase/get.rs

use super::*;
use crate::reedstream::{ReedRequest, ReedResult};

#[test]
fn test_get_text_existing_key() {
    // Test successful retrieval
}

#[test]
fn test_get_text_missing_key() {
    // Test NotFound error handling
}
```

---

## CSV Data Architecture & Key Nomenclature

### Key Nomenclature Rules

**MANDATORY Key Naming Standards** for all ReedCMS data files:

#### 1. Dot-Notation (Not Underscores)
```csv
# ✅ CORRECT - Dot notation
knowledge.page.title@de|Wissen|German page title
page.header.logo.title@de|vvoss|Logo title
landing.hero.headline@de|Entwickler|Hero headline

# ❌ WRONG - Underscore notation
KNOWLEDGE_PAGE_TITLE@DE
PAGE_HEADER_LOGO_TITLE@DE
```

#### 2. Sub-Layouts: Flat Structure (No Hierarchical Parent References)
```csv
# ✅ CORRECT - Flat, independent keys
agility.title@de|Agilität|Agility page title
actix-web.title@de|Actix-Web Framework|Sub-page title

# ❌ WRONG - Hierarchical parent reference
knowledge.agility.title@de
knowledge.actix-web.title@de
```
**Rationale**: Syntactic stringency, taxonomy-based associations, no unnecessary dependencies.

#### 3. Routes: Only in `.reed/routes.csv` (Central Aggregation)
```csv
# .reed/routes.csv
knowledge@de|wissen|German knowledge route
portfolio@de|portfolio|German portfolio route
agility@de|agilitaet|German agility route

# ❌ NEVER in component text.csv files:
# page-header.url.knowledge@de
```
**Rationale**: Type-separated files for performance, central route management.

#### 4. SEO Meta: In `.reed/meta.csv` (Separated from Content)
```csv
# .reed/meta.csv
landing.title@de|Vivian Voss - Principal Software Architect|SEO page title
landing.description@de|Enterprise-Architektur-Lösungen|SEO meta description
agility.title@de|Agilität: Prozess-Theater|SEO page title
agility.description@de|Kritische Analyse|SEO meta description

# ❌ NOT in text.csv:
# landing.meta.title@de
```
**Rationale**: SEO separated from content and routes.

#### 5. Global Components: With Component Namespace
```csv
# ✅ CORRECT - Component name as namespace
page.header.logo.title@de|vvoss|Logo title
page.header.menu.knowledge@de|Wissen|Menu text
page.footer.copyright@de|© 2025 Vivian Voss|Footer copyright

# ❌ WRONG - No namespace
logo.title@de
menu.knowledge@de
```
**Rationale**: Clear origin identification, collision prevention.

#### 6. Nesting Depth: Optimal 4, Maximum 8 Levels
```csv
# ✅ OPTIMAL (4 levels)
landing.hero.badge.audience@de|Enterprise|Badge text

# ✅ ACCEPTABLE (up to 8 levels when necessary)
component.section.subsection.element.variant.state.detail.info@de|Value|Comment

# ⚠️  AVOID if possible - Keep it simple (KISS principle)
```
**Rationale**: Readability, maintainability, KISS principle.

#### 7. Component Names: MANDATORY Dot-Notation
```csv
# ✅ CORRECT - Dots everywhere, no exceptions
page.header.logo.title@de
landing.hero.headline@de
knowledge.intro.title@de
page.footer.copyright.text@de

# ❌ WRONG - No hyphens in keys
page-header.logo.title@de
landing-hero.headline@de
```
**Rationale**: Unified dot-notation everywhere. Directory names use hyphens (filesystem), but CSV keys use only dots (logical structure).

#### 8. Environment Suffixes: After Complete Key
```csv
# ✅ CORRECT - Environment suffix at end
knowledge.page.title@de|Wissen|German title
knowledge.page.title@christmas|🎄 Festive Knowledge|Christmas theme
landing.hero.headline@dev|[DEV] Headline|Development version

# ✅ CORRECT - Fallback chain
key@christmas → key@de → key (if not found)
```

#### 9. CSV File Types and Their Content

**`.reed/text.csv`** - All content text:
```csv
page.header.logo.title@de|vvoss|Logo title
landing.hero.headline@de|Entwickler|Hero headline
agility.description@de|Kritische Analyse|Page content
```

**`.reed/routes.csv`** - All URL routing:
```csv
knowledge@de|wissen|German knowledge route
portfolio@en|portfolio|English portfolio route
```

**`.reed/meta.csv`** - All SEO metadata:
```csv
landing.title@de|Vivian Voss - Principal Software Architect|SEO title
landing.description@de|Enterprise-Architektur-Lösungen|SEO description
landing.cache.ttl|3600|Cache seconds (technical meta)
landing.access.level|public|Access control (technical meta)
```

#### 10. Migration from Legacy Format

**Old Format (vvoss.dev legacy):**
```csv
PAGE_HEADER_LOGO_TITLE@DE|vvoss|Logo title
LANDING_HERO_HEADLINE@DE|Entwickler|Hero headline
AGILITY_META_TITLE@DE|Agilität|Page title
```

**New Format (ReedCMS):**
```csv
# In .reed/text.csv
page.header.logo.title@de|vvoss|Logo title
landing.hero.headline@de|Entwickler|Hero headline

# In .reed/meta.csv
agility.title@de|Agilität|SEO page title
```

**Conversion Rules:**
1. Convert `UPPERCASE_WITH_UNDERSCORES` to `lowercase.with.dots`
2. Move `*_URL_*` keys to `.reed/routes.csv`
3. Move `*_META_TITLE` and `*_META_DESCRIPTION` to `.reed/meta.csv`
4. Keep all other content in `.reed/text.csv`
5. Preserve `@DE/@EN` language suffixes as `@de/@en` (lowercase)

### CSV Format Examples

#### Main Data Files
```csv
# .reed/text.csv
key|value|comment
knowledge.page.title@de|Wissen|German page title
knowledge.page.title@en|Knowledge|English page title
knowledge.page.title@christmas|🎄 Festive Knowledge|Christmas theme
knowledge.navigation.title@de|Hauptmenü|German menu title
portfolio.hero.subtitle@de|Meine Arbeiten|German subtitle

# .reed/routes.csv
key|value|comment
knowledge@de|wissen|German knowledge route
knowledge@en|knowledge|English knowledge route
knowledge@dev|test-route|Development route override
portfolio@de|portfolio|German portfolio route
portfolio@en|portfolio|English portfolio route

# .reed/meta.csv
key|value|comment
knowledge.cache.ttl|3600|Default cache seconds
knowledge.cache.ttl@dev|0|No cache in development
knowledge.access.level|public|Default access control
portfolio.author|vivian|Portfolio author
```

#### Registry Data Files
```csv
# .reed/registry.csv
layout|path|status|created|cli_version|last_validated
knowledge|templates/layouts/knowledge|active|2025-01-15|1.0.0|2025-01-15

# .reed/i18n.csv
language|active|comment
de|true|German (Standard)
en|true|English (Standard)
fr|false|French

# .reed/a11y.csv
feature|active|comment
screen_reader|true|Enhanced screen reader support
high_contrast|false|High contrast mode
keyboard_nav|true|Keyboard navigation

# .reed/presets.csv
name|languages|variants|cache_ttl|access_level|copyright|description|created
default|de,en|mouse,touch,reader|3600|public|ReedCMS|Default project preset|2025-01-15
docs|de,en,fr|mouse,reader|7200|public|ReedCMS|Documentation pages|2025-01-15
```

### Environment Override System
- **Base keys**: `knowledge.title` (applies to all environments)
- **Environment-specific**: `knowledge.title@dev` (overrides base in DEV)
- **Seasonal themes**: `knowledge.title@christmas` (special themes)
- **Fallback logic**: `key@env` → `key` if env-specific not found

---

## REED-02: Data Layer (6 tickets) - 🔄 In Progress

### REED-02-01: ReedBase Core Services 🔄 In Progress
**Files**: `src/reedcms/reedbase/{get.rs, set.rs, init.rs, cache.rs, mod.rs}`

Central data aggregation engine with O(1) HashMap performance.

**Services**:
- **get.rs**: `text()`, `route()`, `meta()`, `server()`, `project()`, `list_text()` - < 100μs per operation
- **set.rs**: Write operations with automatic backup - < 10ms total (5ms backup + 2ms write + 0.1ms cache update)
- **init.rs**: Startup-time CSV loading - < 200μs for 17 layouts
- **cache.rs**: Runtime HashMap caches with RwLock, granular invalidation, < 50ms refresh

**Cache System**:
- `TEXT_CACHE: Arc<RwLock<HashMap<String, HashMap<String, String>>>>` - Language-nested text cache
- `ROUTE_CACHE: Arc<RwLock<HashMap<String, HashMap<String, String>>>>` - Language-nested route cache
- `META_CACHE: Arc<RwLock<HashMap<String, String>>>` - Flat meta cache
- Invalidation: Per-key, per-cache-type, or global
- Integration: `set.rs` calls `invalidate_*_key()` after CSV writes

**Environment Fallback**: `key@dev` → `key` if not found (handled by `environment.rs`)

### REED-02-02: CSV Handler System 🔄 In Progress
**Files**: `src/reedcms/csv/{reader.rs, writer.rs, entry.rs, comments.rs, mod.rs}`

Universal CSV reader/writer for all `.reed/` files.

**Format**: Pipe-delimited `key|value|comment`

**Operations**:
- **reader.rs**: `read_csv()`, `get()`, `list_keys()` - Streaming read, < 5ms for 1000 entries
- **writer.rs**: `write_csv()`, `set()`, `update()` - Atomic write via temp file + rename, < 2ms
- **entry.rs**: `CsvEntry` struct with `from_line()` and `to_line()` parsing
- **comments.rs**: Comment preservation (min 10 chars), `get_existing_comment()`, `validate_comment()`

**Atomic Write Process**:
1. Write to `.reed/{file}.csv.tmp`
2. Validate CSV structure
3. Atomic rename to final file (< 1ms, prevents corruption)

### REED-02-03: Environment Fallback System 🔄 In Progress
**Files**: `src/reedcms/reedbase/environment.rs`

Environment-aware key resolution with fallback chain.

**Resolution Order**:
1. Try `key@environment` (e.g., `title@dev`)
2. Fall back to base `key` (e.g., `title`)

**Supported Environments**: `@dev`, `@prod`, `@christmas`, `@easter`

**Example**: `knowledge.title@dev` → `knowledge.title` if dev-specific not found

### REED-02-04: Backup System 🔄 In Progress
**Files**: `src/reedcms/backup/{create.rs, restore.rs, cleanup.rs, mod.rs}`

Automatic XZ compression backups before CSV modifications.

**Backup Strategy**:
- Trigger: Before any `.reed/*.csv` write
- Compression: XZ for efficient storage
- Location: `.reed/backups/{filename}.{timestamp}.csv.xz`
- Retention: Keep latest 32 backups per file
- Performance: ~5ms per backup operation

**CLI Commands**:
```bash
reed debug:backup list text.csv        # Show available backups
reed debug:backup restore text.csv 3   # Restore 3 steps back
reed debug:backup cleanup              # Manual cleanup old backups
```

### REED-02-05: Matrix CSV Handler 🔄 In Progress
**Files**: `src/reedcms/matrix/{reader.rs, writer.rs, types.rs, mod.rs}`

Complex CSV relationships with 4-type value system.

**Matrix CSV Types** (used in `*.matrix.csv` files):
- **Type 1 (Single)**: `username|admin|desc` - Single values
- **Type 2 (List)**: `username|role1,role2,role3|desc` - Comma-separated lists
- **Type 3 (Single+Modifiers)**: `asset|minify[prod]|desc` - Single value with modifiers
- **Type 4 (List+Modifiers)**: `role|perm1[rwx],perm2[rw-]|desc` - List with modifiers

**Used By**: `users.matrix.csv`, `roles.matrix.csv`, `taxonomie.matrix.csv`, `entity_taxonomy.matrix.csv`

### REED-02-06: Taxonomy System 🔄 In Progress
**Files**: `src/reedcms/taxonomy/{terms.rs, entities.rs, hierarchy.rs, mod.rs}`  
**Status**: ✅ Implemented (58/58 tests passing, 100% coverage)

Universal hierarchical taxonomy for entity tagging.

**Term Management** (terms.rs):
- CRUD: `create_term()`, `get_term()`, `list_terms()`, `search_terms()`, `update_term()`, `delete_term()`
- Format: `{category}:{term}` (e.g., `Programming:Rust`)
- Validation: 2-64 chars, alphanumeric + spaces/hyphens/underscores
- Features: Parent-child hierarchy, active/inactive status, usage tracking

**Entity Tagging** (entities.rs):
- **8 Entity Types**: User, Content, Template, Route, Site, Project, Asset, Role
- Operations: `assign_terms()`, `get_entity_terms()`, `list_entities_by_term()`, `unassign_terms()`
- Entity Key Format: `{entity_type}:{entity_id}` (e.g., `content:post-123`)
- Automatic usage count increment/decrement

**Hierarchy Navigation** (hierarchy.rs):
- `get_children(term_id, recursive)` - Direct children or all descendants
- `get_ancestors(term_id)` - Full ancestry path from root
- `get_path(term_id, separator)` - Formatted path string (e.g., "Programming > Rust")
- `get_depth(term_id)` - Depth level (0 = root)
- `has_circular_reference(term_id, new_parent_id)` - Cycle detection
- `get_tree(category)` - Complete tree with nested children

**Performance**:
- Term creation: < 10ms for < 1000 terms
- Search: < 50ms for 10,000+ terms
- Hierarchy traversal: < 5ms for depth < 10

**CSV Files**:
```csv
# .reed/taxonomie.matrix.csv
term_id|term|category|parent_id|description|color|icon|status|created_by|usage_count|created_at|updated_at
Programming:Rust|Rust|Programming||Systems programming|#FF6600|rust-logo|active|admin|0|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z

# .reed/entity_taxonomy.matrix.csv (Matrix CSV Type 2: Lists)
entity_key|entity_type|entity_id|term_ids|assigned_by|assigned_at|updated_at
content:post-123|content|post-123|Programming:Rust,Topics:Systems|admin|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
```

---

## REED-03: Security Layer (2 tickets) - 🔄 In Progress

### REED-03-01: User Management System 🔄 In Progress
**Files**: `src/reedcms/users/{crud.rs, validation.rs, profile.rs, mod.rs}`

Extended user management with Argon2 password hashing and social profiles.

**CSV File**: `.reed/users.matrix.csv` (Matrix CSV Type 2)
```csv
username|password|roles|firstname|lastname|street|city|postcode|region|country|email|mobile|twitter|facebook|tiktok|insta|youtube|whatsapp|desc|created_at|updated_at|last_login|is_active
admin|$argon2id$hash|admin|Admin|User|Main St 1|London|SW1A 1AA|London|UK|admin@example.com|+44123456789|||||||System Admin|1640995200|1640995200||true
```

**Features**:
- Argon2id password hashing (~100ms intentional slowdown)
- Extended profile: Full contact information + social media profiles
- Account status tracking: Login history, activation, timestamps
- Password validation: Uppercase, lowercase, digit, special character requirements
- Email/username uniqueness enforcement

**Operations**: `create_user()`, `get_user()`, `update_user()`, `delete_user()`, `verify_password()`, `change_password()`

### REED-03-02: Role Permission System 🔄 In Progress
**Files**: `src/reedcms/roles/{crud.rs, permissions.rs, inheritance.rs, mod.rs}`

Unix-style permissions with role inheritance and hierarchical resource matching.

**CSV File**: `.reed/roles.matrix.csv` (Matrix CSV Type 4: Lists with Modifiers)
```csv
rolename|permissions|inherits|desc|created_at|updated_at|is_active
editor|text[rwx],route[rw-],content[rw-]||Content Editor|1640995200|1640995200|true
admin|*[rwx]|editor|Full Admin|1640995200|1640995200|true
```

**Permission Syntax**: `resource[rwx]` where r=read, w=write, x=execute

**Features**:
- Hierarchical resource matching: `content/blog/*` applies to `content/blog/article_1`
- Role inheritance: `admin` inherits `editor` permissions, can override
- Wildcard support: `*[rwx]` grants full access
- Circular dependency detection
- Sub-millisecond cached lookups with automatic invalidation

**Permission Resolution**:
1. Check user's direct permissions
2. Check role permissions
3. Check inherited role permissions (recursively)
4. Apply hierarchical resource matching
5. Cache result for subsequent lookups

---

## REED-04: CLI Layer (12 tickets) - 🔄 In Progress

### REED-04-01: CLI Command Foundation 🔄 In Progress
**Files**: `src/reedcms/cli/{mod.rs, parser.rs, executor.rs}`

Unified `reed` binary with command parsing and execution.

**Command Structure**: `reed <command>:<subcommand> [args] [flags]`

**Examples**:
```bash
reed set:text knowledge.title@de "Wissen" --desc "Page title"
reed server:io --port 8333
reed build:watch
```

**Features**: Argument parsing, flag handling, help text generation, error messages

### REED-04-02: CLI Data Commands ✅ Complete
**Files**: `src/reedcms/cli/data_commands.rs`

CRUD operations for text, route, meta, server, project data.

**Commands**:
```bash
reed set:text key@lang "value" --desc "comment"
reed get:text key@lang
reed list:text pattern
reed set:route layout@lang "path"
reed get:route layout@lang
reed set:meta key "value" --desc "comment"
reed get:meta key
reed set:server key "value"
reed set:project key "value"
```

**Validation**: Min 10 chars comment, key format validation

### REED-04-03: CLI Layout Commands ✅ Complete
**Files**: `src/reedcms/cli/layout_commands.rs`

Layout creation, registration, and management.

**Commands**:
```bash
reed init:layout knowledge --languages de,en --variants mouse,touch,reader
reed list:layouts
reed validate:layout knowledge
reed remove:layout knowledge
```

**Process**:
1. Create directory: `templates/layouts/{layout}/`
2. Generate templates: `{layout}.{variant}.jinja`
3. Generate CSS: `{layout}.{variant}.css`
4. Create text file: `{layout}.text.csv`
5. Register in `.reed/registry.csv`

### REED-04-04: CLI User Commands ✅ Complete
**Files**: `src/reedcms/cli/user_commands.rs`

User account management via CLI.

**Commands**:
```bash
reed create:user username email password --firstname "John" --lastname "Doe"
reed list:users [pattern]
reed get:user username
reed update:user username --email new@example.com
reed delete:user username
reed reset:password username new_password
```

**Features**: Argon2 password hashing, profile management, role assignment

### REED-04-05: CLI Role Commands ✅ Complete
**Files**: `src/reedcms/cli/role_commands.rs`

Role and permission management via CLI.

**Commands**:
```bash
reed create:role editor --permissions "text[rwx],content[rw-]"
reed list:roles
reed get:role editor
reed update:role editor --add-permission "route[rw-]"
reed delete:role editor
reed assign:role username editor
```

**Features**: Permission syntax validation, inheritance checking, circular reference detection

### REED-04-06: CLI Taxonomy Commands 🔄 In Progress
**Files**: `src/reedcms/cli/taxonomy_commands.rs`

Taxonomy term and entity management via CLI.

**Commands**:
```bash
reed create:term Programming:Rust --parent Programming --color "#FF6600"
reed list:terms [category]
reed search:terms "rust"
reed assign:term content:post-123 Programming:Rust
reed list:entities Programming:Rust
```

### REED-04-07: CLI Migration Commands 🔄 In Progress
**Files**: `src/reedcms/cli/migration_commands.rs`

Data migration and validation utilities.

**Commands**:
```bash
reed migrate:from-legacy path/to/old/text.csv
reed validate:all
reed validate:keys [pattern]
reed validate:routes
reed fix:duplicates
```

### REED-04-08: CLI Build Commands 🔄 In Progress
**Files**: `src/reedcms/cli/build_commands.rs`

Asset building and compilation.

**Commands**:
```bash
reed build:css [layout]
reed build:js [layout]
reed build:all
reed build:watch
reed build:release
```

### REED-04-09: CLI Server Commands 🔄 In Progress
**Files**: `src/reedcms/cli/server_commands.rs`

Server lifecycle management.

**Commands**:
```bash
reed server:io --port 8333
reed server:io --socket /tmp/reed.sock
reed server:start  # Background daemon
reed server:stop
reed server:restart
reed server:status
reed server:logs --tail 50
```

### REED-04-10: CLI Agent Commands 🔄 In Progress
**Files**: `src/reedcms/cli/agent_commands.rs`

MCP (Model Context Protocol) integration foundation for AI-assisted CMS management.

**Commands**:
```bash
reed agent:init  # Initialise MCP server
reed agent:status
reed agent:capabilities
```

**Purpose**: Enable AI assistants to interact with ReedCMS via standardised MCP protocol

### REED-04-11: CLI Man Page 🔄 In Progress
**Files**: `src/reedcms/cli/man.rs`, `docs/reed.1`

Unix man page documentation for `reed` command.

**Generation**: `reed help --man > /usr/local/share/man/man1/reed.1`

**Sections**: NAME, SYNOPSIS, DESCRIPTION, COMMANDS, OPTIONS, EXAMPLES, FILES, SEE ALSO

### REED-04-12: Reed.toml Configuration ✅ Complete
**Status**: Implemented | **Tests**: All commands tested | **Files**: `Reed.toml`, `Reed.toml.example`, `.env`, `src/reedcms/config/` (3 modules)

Bootstrap configuration system with bidirectional sync between TOML and CSV.

**CRITICAL ARCHITECTURE**: CSV files are the single source of truth at runtime!

**Configuration Files**:
```
Reed.toml.example  →  Reed.toml  →  config:sync --force  →  .reed/*.csv
   (Reference)      (Bootstrap)                            (Runtime Truth)
                         ↑                                        ↓
                         └──────── config:export ─────────────────┘
```

**File Structure**:
- `Reed.toml.example` - Complete reference with ALL options (documentation)
- `Reed.toml` - Minimal bootstrap config (project-specific overrides)
- `.reed/project.csv` - Runtime project configuration (CSV = truth)
- `.reed/server.csv` - Runtime server configuration (CSV = truth)
- `.env` - Environment control (ENVIRONMENT=dev|prod)

**Commands**:
| Command | Direction | Purpose | Safety |
|---------|-----------|---------|--------|
| `config:sync` | TOML → CSV | Bootstrap from Reed.toml | ⚠️ Overwrites CSV! |
| `config:export` | CSV → TOML | Backup CSV to Reed.toml | ✅ Safe |
| `config:show` | Read CSV | Display current config | ✅ Read-only |
| `config:validate` | Read TOML | Validate TOML syntax | ✅ Read-only |
| `config:init` | Template | Create new Reed.toml | ✅ Safe |

**Important Rules**:
1. Server startup reads ONLY from `.reed/*.csv`, NEVER from Reed.toml
2. CLI commands (`set:project`, `set:server`) write ONLY to CSV
3. `config:sync` is DESTRUCTIVE - requires `--force` flag
4. `config:export` backs up CSV → TOML for version control
5. `.env` controls server binding: dev=localhost:8333, prod=socket

**Environment Control** (`.env`):
- `ENVIRONMENT=dev` → Server binds to `localhost:8333` (HTTP)
- `ENVIRONMENT=prod` → Server binds to `/tmp/reed.sock` (Unix socket)

**Workflow**:
```bash
# Initial setup
reed config:sync --force          # TOML → CSV (bootstrap)

# Runtime changes (recommended)
reed set:project name "New Name"  # Direct CSV write

# Backup configuration
reed config:export --force        # CSV → TOML (for git)
```

---

### REED-04-13: System Setup Scripts ✅ Complete
**Status**: Implemented | **Files**: `scripts/*.sh` (6 scripts + README)

System integration scripts for binary and man page installation.

**Available Scripts**:

| Script | Mode | Requires Sudo | Purpose | File Operations |
|--------|------|---------------|---------|-----------------|
| `setup-dev.sh` | Development | Yes | Creates symlinks for live development | Symlinks only |
| `install-system.sh` | Production | Yes | System-wide installation (all users) | File copies |
| `install-user.sh` | User-local | No | User-only installation | File copies |
| `uninstall-system.sh` | Uninstall | Yes | Remove system installation | File removal |
| `uninstall-user.sh` | Uninstall | No | Remove user installation | File removal |
| `build-man-pages.sh` | Build | No | Compile `.ronn` → `.1` man pages | File generation |

**Installation Modes**:

1. **Development Mode** (`setup-dev.sh`):
   - Binary: `/usr/local/bin/reed` → `target/release/reed` (symlink)
   - Man pages: `/usr/local/share/man/man1/*.1` → `man/*.1` (symlinks)
   - Auto-updates when `cargo build --release` runs
   - Requires sudo for `/usr/local/bin` access

2. **System Installation** (`install-system.sh`):
   - Binary: `/usr/local/bin/reed` (copy, 755 permissions)
   - Man pages: `/usr/local/share/man/man1/*.1` (copies, 644 permissions)
   - Updates man database with `mandb`
   - Production-ready installation

3. **User Installation** (`install-user.sh`):
   - Binary: `~/.local/bin/reed` (copy)
   - Man pages: `~/.local/share/man/man1/*.1` (copies)
   - No sudo required
   - Requires `~/.local/bin` in PATH and `~/.local/share/man` in MANPATH

**Usage After Installation**:
```bash
# From anywhere in the system
reed data:get knowledge.title@en

# Access man pages
man reed
man reed-data
```

**Verification Commands**:
```bash
which reed                  # Shows binary location
reed --version              # Tests binary execution
man reed                    # Displays man page
man -w reed                 # Shows man page location
```

**Shell Configuration** (for user installation):
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="${HOME}/.local/bin:${PATH}"
export MANPATH="${HOME}/.local/share/man:${MANPATH}"
```

**Files**:
- `scripts/setup-dev.sh` - Development symlink setup (128 lines)
- `scripts/install-system.sh` - System installation (103 lines)
- `scripts/install-user.sh` - User installation (147 lines)
- `scripts/uninstall-system.sh` - System uninstall (72 lines)
- `scripts/uninstall-user.sh` - User uninstall (64 lines)
- `scripts/build-man-pages.sh` - Man page builder (exists)
- `scripts/README.md` - Complete installation documentation

---

## CLI Command Reference

### Complete CLI Command Table

| CLI Command | Template Access | ReedStream API | Input | Output | File |
|-------------|----------------|---------------|-------|--------|------|
| **Data Management** | | | | | |
| `reed set:text key@lang "value" --desc "..."` | `{{ "key" \| text("lang") }}` | `reedbase::set::text(req)` | `key: String, lang: String, value: String, desc: String` | `ReedResult<ReedResponse<()>>` | `set.rs` |
| `reed set:route key@lang "route" --desc "..."` | `{{ "key" \| route("lang") }}` | `reedbase::set::route(req)` | `key: String, lang: String, route: String, desc: String` | `ReedResult<ReedResponse<()>>` | `set.rs` |
| `reed set:meta key "value" --desc "..."` | `{{ "key" \| meta }}` | `reedbase::set::meta(req)` | `key: String, value: String, desc: String` | `ReedResult<ReedResponse<()>>` | `set.rs` |
| `reed get:text key@lang` | `{{ "key" \| text("lang") }}` | `reedbase::get::text(req)` | `key: String, lang: String` | `ReedResult<ReedResponse<String>>` | `get.rs` |
| `reed get:route key@lang` | `{{ "key" \| route("lang") }}` | `reedbase::get::route(req)` | `key: String, lang: String` | `ReedResult<ReedResponse<String>>` | `get.rs` |
| `reed list:text pattern.*` | - | `reedbase::get::list_text(req)` | `pattern: String` | `ReedResult<ReedResponse<Vec<String>>>` | `get.rs` |
| **Layout Management** | | | | | |
| `reed init:layout name` | - | `layouts::create(req)` | `name: String` | `ReedResult<ReedResponse<LayoutInfo>>` | `init.rs` |
| `reed list:layouts` | - | `layouts::list(req)` | - | `ReedResult<ReedResponse<Vec<String>>>` | `init.rs` |
| **User Management** | | | | | |
| `reed user:create username --email "..." --password "..."` | - | `user::create(req)` | `username: String, email: String, password: String` | `ReedResult<ReedResponse<UserInfo>>` | `user.rs` |
| `reed user:list` | - | `user::list(req)` | - | `ReedResult<ReedResponse<Vec<UserInfo>>>` | `user.rs` |
| `reed user:show username` | - | `user::get(req)` | `username: String` | `ReedResult<ReedResponse<UserInfo>>` | `user.rs` |
| **Role Management** | | | | | |
| `reed role:create rolename --permissions "text[rwx],route[rw-]"` | - | `role::create(req)` | `rolename: String, permissions: String` | `ReedResult<ReedResponse<RoleInfo>>` | `role.rs` |
| `reed role:list` | - | `role::list(req)` | - | `ReedResult<ReedResponse<Vec<RoleInfo>>>` | `role.rs` |
| **Taxonomy Management** | | | | | |
| `reed taxonomy:create "term" --category "tag"` | - | `taxonomy::create(req)` | `term: String, category: String` | `ReedResult<ReedResponse<TaxonomyTerm>>` | `taxonomy.rs` |
| `reed taxonomy:list` | - | `taxonomy::list(req)` | - | `ReedResult<ReedResponse<Vec<TaxonomyTerm>>>` | `taxonomy.rs` |
| `reed taxonomy:assign entity:id --terms "1,2,3"` | - | `taxonomy::assign(req)` | `entity: String, term_ids: Vec<u32>` | `ReedResult<ReedResponse<()>>` | `taxonomy.rs` |
| **Migration & Validation** | | | | | |
| `reed migrate:text path/` | - | `migration::text(req)` | `path: String` | `ReedResult<ReedResponse<MigrationResult>>` | `migrate.rs` |
| `reed validate:routes` | - | `validation::routes(req)` | - | `ReedResult<ReedResponse<ValidationReport>>` | `validate.rs` |
| **Server Management** | | | | | |
| `reed server:io --port 8333` | - | `server::start_http(req)` | `port: u16` | `ReedResult<ReedResponse<ServerInfo>>` | `server.rs` |
| `reed server:io --socket "/tmp/reed.sock"` | - | `server::start_socket(req)` | `socket_path: String` | `ReedResult<ReedResponse<ServerInfo>>` | `server.rs` |
| `reed server:stop` | - | `server::stop(req)` | - | `ReedResult<ReedResponse<()>>` | `server.rs` |
| `reed server:status` | - | `server::status(req)` | - | `ReedResult<ReedResponse<ServerStatus>>` | `server.rs` |
| **Configuration Management** | | | | | |
| `reed config:sync --force` | - | `config::sync_toml_to_csv(cfg)` | `config: ReedConfig` | `ReedResult<ReedResponse<Vec<String>>>` | `toml_to_csv.rs` |
| `reed config:export --force` | - | `config::export_csv_to_toml(path)` | `output_path: String` | `ReedResult<ReedResponse<String>>` | `config_commands.rs` |
| `reed config:show` | - | `config::parse_reed_toml(path)` | `toml_path: String` | `ReedResult<ReedResponse<String>>` | `config_commands.rs` |
| `reed config:validate` | - | `config::validate_config(cfg)` | `config: ReedConfig` | `ReedResult<ReedResponse<String>>` | `toml_parser.rs` |
| `reed config:init` | - | `config::create_template(path)` | `output_path: String` | `ReedResult<ReedResponse<String>>` | `config_commands.rs` |
| `reed set:project key value --desc "..."` | - | `reedbase::set::project(req)` | `key: String, value: String, desc: String` | `ReedResult<ReedResponse<()>>` | `data_commands.rs` |
| `reed set:server key value --desc "..."` | - | `reedbase::set::server(req)` | `key: String, value: String, desc: String` | `ReedResult<ReedResponse<()>>` | `data_commands.rs` |
| **Debug & Recovery** | | | | | |
| `reed debug:backup list text.csv` | - | `debug::backup::list(req)` | `csv_file: String` | `ReedResult<ReedResponse<Vec<BackupInfo>>>` | `backup.rs` |
| `reed debug:backup restore text.csv 3` | - | `debug::backup::restore(req)` | `csv_file: String, steps_back: u32` | `ReedResult<ReedResponse<()>>` | `backup.rs` |

### CLI Usage Examples

```bash
# Data Management
reed set:text knowledge.page.title@de "Wissen" --desc "Page title"
reed set:route knowledge@de "wissen" --desc "German route"
reed get:text knowledge.page.title@de
reed list:text knowledge.*

# Layout Creation
reed init:layout knowledge --languages de,en --variants mouse,touch,reader
reed list:layouts

# User Management
reed user:create admin --email admin@example.com --password secret123
reed user:list --format table
reed user:show admin

# Migration
reed migrate:text organisms/page-header/
reed validate:routes

# Server
reed server:io --port 8333
reed server:status
reed server:stop
```

---

## REED-05: Template Layer (3 tickets) - 🔄 In Progress

### REED-05-01: Template Filter System 🔄 In Progress
**Files**: `src/reedcms/filters/{text.rs, route.rs, meta.rs, mod.rs}`

MiniJinja custom filters for ReedBase data access in templates.

**Filters**:
- `{{ "key" | text("de") }}` - Get text content
- `{{ pagekey | route("de") }}` - Get route path
- `{{ "knowledge.title" | meta }}` - Get metadata
- `{{ pagekey | url(lang) }}` - Build full URL: `/de/wissen/`

**Implementation**: Calls ReedBase `get::text()`, `get::route()`, `get::meta()` services

**Performance**: < 100μs per filter call (O(1) HashMap lookup)

### REED-05-02: Template Engine Setup 🔄 In Progress
**Files**: `src/reedcms/template/{engine.rs, loader.rs, mod.rs}`

MiniJinja environment configuration with hot-reload support.

**Features**:
- Template directory: `templates/layouts/`
- Component inclusion: `templates/components/{atoms,molecules,organisms}/`
- Auto-reload in DEV mode: File watcher detects changes
- Production mode: Templates compiled at startup
- Error handling: Graceful template rendering failures

**Setup**:
```rust
let env = minijinja::Environment::new();
env.add_filter("text", filters::text);
env.add_filter("route", filters::route);
env.add_filter("meta", filters::meta);
env.set_loader(template_loader);
```

### REED-05-03: Template Context Builder 🔄 In Progress
**Files**: `src/reedcms/template/context.rs`

Build template rendering context from request data.

**Context Structure**:
```rust
{
    "pagekey": "knowledge",
    "lang": "de",
    "client": {
        "variant": "mouse",
        "screen_width": 1920,
        "screen_height": 1080,
        "pixel_ratio": 2.0
    },
    "params": {},  // URL parameters
}
```

**Process**:
1. Extract layout and language from route resolution
2. Parse client detection cookie (`screen_info`)
3. Add URL parameters
4. Build context HashMap

---

## REED-06: Server Layer (6 tickets) - 🔄 In Progress

### REED-06-01: Server Foundation ✅ Complete
**Files**: `src/reedcms/server/{http_server.rs, socket_server.rs, config.rs, shutdown.rs}`

Actix-Web 4.9 HTTP/Unix socket server with graceful shutdown.

**Features**:
- HTTP server: Default port 8333, CPU core detection for workers
- Unix socket server: 20-30% faster for nginx/apache reverse proxy
- Middleware: Logger, Compression (gzip/brotli)
- Graceful shutdown: 30s timeout, SIGTERM/SIGINT handling

**CLI Integration**:
```bash
reed server:io --port 8333
reed server:io --socket /tmp/reed.sock
reed server:io --workers 8
```

**Performance**: HTTP startup < 50ms, Unix socket startup < 30ms

### REED-06-02: Routing System ✅ Complete
**Files**: `src/reedcms/routing/{resolver.rs, language.rs, patterns.rs}`

URL → Layout + Language resolution with 404 handling.

**Process**:
1. Extract language prefix from URL: `/de/wissen` → `lang="de", path="wissen"`
2. Lookup route in `.reed/routes.csv`: `knowledge@de|wissen`
3. Return `RouteInfo { layout: "knowledge", language: "de", params: {} }`

**Route Format**: `layout@lang|path` (e.g., `knowledge@de|wissen`)

**Language Detection** (language.rs):
1. URL path prefix: `/en/page` → `en`
2. Accept-Language header: Browser preference
3. Default language: `project.default_language` from config
4. Fallback: `"de"`

**Pattern Matching** (patterns.rs):
- `:param` - Named parameter extraction
- `*` - Wildcard segment
- Literal - Exact match
- Example: `/blog/:slug` matches `/blog/my-post` → `{ slug: "my-post" }`

**Performance**: < 5ms per URL resolution (O(n) linear scan)

### REED-06-03: Authentication Middleware ✅ Complete
**Files**: `src/reedcms/auth/{middleware.rs, credentials.rs, verification.rs, rate_limit.rs, errors.rs, site_protection.rs}`

HTTP authentication with Basic Auth and progressive rate limiting.

**Authentication Methods**:
1. **HTTP Basic Auth**: `Authorization: Basic base64(username:password)`
2. **Bearer Token**: `Authorization: Bearer {token}` (reserved for sessions)

**Rate Limiting** (Progressive Lockout):
- 3 failed attempts → 1min lockout
- 5 failed attempts → 5min lockout
- 10 failed attempts → 30min lockout
- Reset on successful login

**Site-Wide Protection** (site_protection.rs):
- Simple htaccess-style authentication for entire site
- Config: `.reed/server.csv` → `server.auth.enabled|true`
- Bypass: Specific routes can be excluded

**Password Verification**: Argon2id (~100ms intentional slowdown against brute-force)

**Performance**: < 200ms per auth check (including Argon2 verification)

### REED-06-04: Response Builder ✅ Complete
**Files**: `src/reedcms/server/response_builder.rs`

HTTP response building with template rendering orchestration.

**Process**:
1. Route resolution → Layout + Language
2. Client detection → Variant selection (mouse/touch/reader)
3. Context building → Template variables
4. Template rendering → HTML output
5. HTTP response → Headers + Body

**Template Selection**: `{layout}.{variant}.jinja` (e.g., `knowledge.mouse.jinja`)

**Performance**: < 50ms for complete response cycle (route → template → HTML)

### REED-06-05: Client Detection Service ✅ Complete
**Files**: `src/reedcms/server/client_detection.rs`

JavaScript-based screen measurement with cookie storage.

**Process**:
1. First visit: No `screen_info` cookie → Serve detection HTML + JS
2. JS measures: Screen width/height, pixel ratio, touch support
3. Cookie set: `screen_info={width}x{height}@{ratio}@{touch}`
4. Reload: Cookie present → Parse variant, render template

**Variant Selection**:
- `touch` if `touch_support == true`
- `mouse` if `touch_support == false && width >= 768`
- `reader` if `width < 768` (mobile portrait)

**Cookie Format**: `1920x1080@2.0@false` → `{width}x{height}@{ratio}@{touch}`

**Performance**: Detection page < 1ms, variant parsing < 0.1ms

### REED-06-06: Language System Fix ✅ Complete
**Files**: `.reed/routes.csv`, `.reed/project.csv`, `src/reedcms/routing/resolver.rs`

Fixed default language configuration and URL prefix routing.

**Problems Solved**:
1. Missing `project.default_language` → Added `project.default_language|de`
2. Routes stored full URLs (`de/wissen`) instead of path segments (`wissen`)

**Solution**:
- **routes.csv format**: `layout@lang|path_segment` (e.g., `knowledge@de|wissen`)
- **Route lookup**: Filters by language parameter to prevent cross-language matching
- **URL building**: Templates construct `/{{ lang }}/{{ route }}/` correctly
- **Root redirect**: `/` → `/de/` or `/en/` based on Accept-Language (301 for SEO)

**Route Resolution Flow**:
```
URL: /de/wissen
→ extract_language_prefix("de/wissen") → (lang="de", path="wissen")
→ lookup_exact_route("wissen", "de") → finds knowledge@de|wissen
→ Returns RouteInfo{layout: "knowledge", language: "de"}
```

**Benefits**: Language switcher works, proper SEO structure, clean URLs

---

## REED-07: API Layer (2 tickets) - 🔄 In Progress

### REED-07-01: ReedAPI HTTP Interface 🔄 In Progress
**Files**: `src/reedcms/api/{mod.rs, handlers.rs, routes.rs}`

RESTful HTTP API for external integrations.

**Endpoints**:
```
GET    /api/v1/text/{key}?lang=de
POST   /api/v1/text
PUT    /api/v1/text/{key}
DELETE /api/v1/text/{key}
GET    /api/v1/routes
POST   /api/v1/users
```

**Authentication**: Bearer token via `Authorization: Bearer {token}`

**Response Format**: JSON with metadata
```json
{
  "data": "Wissen",
  "source": "reedbase::get::text",
  "cached": true,
  "timestamp": 1640995200
}
```

### REED-07-02: API Security Matrix 🔄 In Progress
**Files**: `src/reedcms/api/security.rs`

Permission-based API access control using role system.

**Access Control**:
- API endpoints mapped to resources: `/api/v1/text` → `text[r--]`
- User roles checked: `editor` has `text[rwx]` → allowed
- Operations: GET=read, POST=write, PUT=write, DELETE=execute

**Rate Limiting**: Per-user API rate limits (separate from auth rate limiting)

---

## REED-08: Asset Layer (3 tickets) - 🔄 In Progress

### REED-08-01: CSS Bundler 🔄 In Progress
**Files**: `src/reedcms/assets/css_bundler.rs`

CSS compilation, bundling, and minification.

**Process**:
1. Collect all `{layout}.{variant}.css` files
2. Collect component CSS: `atoms/*.css`, `molecules/*.css`, `organisms/*.css`
3. Concatenate in dependency order
4. Minify (remove whitespace, comments) in `@prod`
5. Output: `public/{layout}.{variant}.min.css`

**Performance**: < 10s for complete build (all layouts + components)

### REED-08-02: JS Bundler 🔄 In Progress
**Files**: `src/reedcms/assets/js_bundler.rs`

JavaScript bundling and minification.

**Process**:
1. Collect component JS files
2. Concatenate in dependency order
3. Minify with terser/swc in `@prod`
4. Output: `public/{bundle}.min.js`

### REED-08-03: Static Asset Server 🔄 In Progress
**Files**: `src/reedcms/server/static_server.rs`

Static file serving with caching headers.

**Features**:
- Serve `public/` directory
- Cache headers: `Cache-Control: max-age=3600` (configurable)
- Compression: Pre-compressed `.gz` and `.br` variants
- Performance: < 10ms per static file response

---

## REED-09: Build Layer (3 tickets) - 🔄 In Progress

### REED-09-01: Binary Compiler 🔄 In Progress
**Files**: `build.rs`, `src/reedcms/build/compiler.rs`

Rust binary compilation and release system.

**Build Targets**:
- Development: `cargo build` → `target/debug/reed`
- Release: `cargo build --release` → `target/release/reed` (optimised)
- Platform-specific: Linux, macOS, Windows binaries

**Optimisation Flags**: LTO, codegen-units=1, opt-level=3

### REED-09-02: Asset Pipeline 🔄 In Progress
**Files**: `src/reedcms/build/pipeline.rs`

Orchestrates CSS/JS bundling during build process.

**Build Phases**:
1. Compile Rust binary
2. Bundle CSS assets
3. Bundle JS assets
4. Copy static files
5. Generate manifest

**CLI Integration**: `reed build:all`, `reed build:release`

### REED-09-03: File Watcher System 🔄 In Progress
**Files**: `src/reedcms/build/watcher.rs`

Auto-rebuild on file changes during development.

**Watched Directories**:
- `src/` → Recompile binary
- `templates/` → Reload templates
- `templates/**/*.css` → Rebuild CSS
- `.reed/*.csv` → Invalidate caches

**CLI Integration**: `reed build:watch`

**Performance**: < 500ms rebuild time for template changes

---

## REED-10: Monitor Layer (4 tickets) - 🔄 In Progress

### REED-10-01: ReedMonitor Foundation 🔄 In Progress
**Files**: `src/reedcms/monitor/{mod.rs, metrics.rs, logging.rs}`

Universal monitoring and FreeBSD-style system logging.

**Metrics Types**:
- **Counter**: Operation counts (`csv_operations`, `template_renders`)
- **Timer**: Performance metrics (`csv_read_time`, `request_duration`)
- **Gauge**: Current values (`active_connections`, `memory_usage`)
- **Event**: Custom events with metadata (`user_login`, `error_occurred`)

**Log Format** (FreeBSD syslog):
```
{timestamp} {hostname} {process}[{pid}]: {level}: {message}
Dec 15 14:23:01 server reedcms[1234]: INFO: METRIC[counter] csv_operations: 42
```

**Output Modes**:
- **Silent**: No monitoring (performance mode)
- **Log**: Write to system logs and optional log file
- **Forward**: Send to external monitoring systems
- **Both**: Log locally and forward

**Performance**: < 1% overhead, 10k+ messages/second, < 10MB memory

### REED-10-02: Performance Profiler 🔄 In Progress
**Files**: `src/reedcms/monitor/profiler.rs`

Automatic benchmark collection and performance analysis.

**Benchmark Targets**:
- CSV operations: p95 < 500μs, p99 < 1ms
- HashMap lookups: p95 < 10μs, p99 < 50μs
- Template rendering: p95 < 50ms, p99 < 100ms
- Full request cycle: p95 < 100ms, p99 < 200ms
- Startup time: < 200μs for 17 layouts

**Usage**:
```rust
let _timer = benchmark("csv_read_text", BenchmarkConfig {
    target_p50: 100,  // μs
    target_p95: 500,  // μs
    target_p99: 1000, // μs
});
// Automatic timing on drop
```

**Results**: Logged to `.reed/flow/reeddebug.csv`

**CLI Commands**:
```bash
reed debug:benchmark csv           # CSV operation benchmarks
reed debug:benchmark templates     # Template rendering benchmarks
reed debug:benchmark report        # Full performance report
reed debug:benchmark compare v1 v2 # Compare versions
```

### REED-10-03: Debug Tools 🔄 In Progress
**Files**: `src/reedcms/monitor/debug.rs`

Development debugging utilities.

**Features**:
- Template variable inspection: Live template context debugging
- Data flow tracing: Request → Response pipeline visualisation
- Error context: Rich error messages with full ReedCMS state
- Hot reload: Live change detection and reload coordination

**CLI Commands**:
```bash
reed debug:template knowledge.mouse.jinja
reed debug:trace /de/wissen
reed debug:context knowledge de
```

### REED-10-04: Backup Recovery CLI 🔄 In Progress
**Files**: `src/reedcms/cli/backup_commands.rs`

Backup management and recovery utilities.

**Commands**:
```bash
reed backup:list text.csv
reed backup:restore text.csv 3
reed backup:cleanup
reed backup:export backups.tar.xz
reed backup:import backups.tar.xz
```

**Features**: XZ-compressed backups, keep latest 32, restore by steps back

---

## REED-11: Extension Layer (4 tickets) - 🔄 Planning

### REED-11-01: Hook System 🔄 Planning
Trigger-based automation for CMS events.

**Events**: `on_text_set`, `on_user_create`, `on_template_render`, `on_server_start`

**Actions**: Execute shell commands, HTTP webhooks, custom Rust functions

### REED-11-02: Workflow Engine 🔄 Planning
Multi-step automation workflows.

**Example**: Content approval workflow (draft → review → publish)

### REED-11-03: External API Bridges 🔄 Planning
Social media integration (Twitter, Facebook, etc.)

**Features**: Post syndication, cross-posting, analytics integration

### REED-11-04: Scheduled Tasks 🔄 Planning
Cron-style task scheduling.

**Examples**: Daily backups, weekly reports, periodic cache cleanup

---

## REED-20: Third-Party Integration (4 tickets) - 🔄 Planning

### REED-20-01: MCP Server Library 🔄 Planning
Model Context Protocol server library for AI assistant integration.

**Purpose**: Enable AI assistants (Claude, GPT) to interact with ReedCMS

**Features**: Standardised MCP protocol, reed command exposure, context sharing

### REED-20-02: VSCode Extension 🔄 Planning
Visual Studio Code integration for ReedCMS development.

### REED-20-03: Zed Extension 🔄 Planning
Zed editor integration for ReedCMS development.

### REED-20-04: JetBrains Extension 🔄 Planning
IntelliJ/RustRover integration for ReedCMS development.

---

## REED-90: Quality Layer (2 tickets) - 🔄 Planning

### REED-90-01: Quality Standards Restoration 🔄 Planning
Comprehensive code quality audit and standards enforcement.

### REED-90-02: Comprehensive Documentation 🔄 Planning
Complete user manual, API reference, and developer guides.

---

## Project Status Summary

### Implementation Progress
- **REED-01 Foundation**: ✅ 100% (2/2 tickets complete)
- **REED-02 Data Layer**: 🔄 16% (1/6 complete - taxonomy implemented)
- **REED-03 Security Layer**: 🔄 0% (0/2 complete)
- **REED-04 CLI Layer**: 🔄 33% (4/12 complete)
- **REED-05 Template Layer**: 🔄 0% (0/3 complete)
- **REED-06 Server Layer**: ✅ 100% (6/6 tickets complete)
- **REED-07 API Layer**: 🔄 0% (0/2 complete)
- **REED-08 Asset Layer**: 🔄 0% (0/3 complete)
- **REED-09 Build Layer**: 🔄 0% (0/3 complete)
- **REED-10 Monitor Layer**: 🔄 0% (0/4 complete)
- **REED-11 Extension Layer**: 🔄 0% (0/4 planning)
- **REED-20 Third-Party**: 🔄 0% (0/4 planning)
- **REED-90 Quality Layer**: 🔄 0% (0/2 planning)

**Overall**: 11/53 tickets complete (20.8%)

### Critical Path
1. ✅ REED-01: Foundation (ReedStream, Error System)
2. 🔄 REED-02: Data Layer (ReedBase, CSV Handler) - **Current Focus**
3. 🔄 REED-05: Template Layer (Filters, Engine, Context)
4. ✅ REED-06: Server Layer (HTTP, Routing, Auth, Response)
5. 🔄 REED-08: Asset Layer (CSS/JS Bundling)
6. 🔄 REED-09: Build Layer (Compiler, Pipeline, Watcher)

### Performance Achievements
- ReedStream: < 1μs request/response creation
- Taxonomy: 58/58 tests passing, 100% coverage
- Server Foundation: < 50ms HTTP startup
- URL Routing: < 5ms per resolution
- Language System: Proper URL structure, working language switcher

### Key Technical Decisions
See `project_optimisations.md` for complete decision log (50+ architectural decisions documented).

---

## File Structure Reference

```
ReedCMS/
├── .reed/                          # CSV Database (single source of truth)
│   ├── text.csv                    # Content text (pipe-delimited)
│   ├── routes.csv                  # URL routing definitions
│   ├── meta.csv                    # SEO and technical metadata
│   ├── server.csv                  # Server configuration
│   ├── project.csv                 # Project settings
│   ├── registry.csv                # Layout registry
│   ├── i18n.csv                    # Activated languages
│   ├── a11y.csv                    # Accessibility config
│   ├── users.matrix.csv            # User management (Matrix CSV)
│   ├── roles.matrix.csv            # Roles and permissions (Matrix CSV)
│   ├── taxonomie.matrix.csv        # Taxonomy terms (Matrix CSV)
│   ├── entity_taxonomy.matrix.csv  # Entity-term assignments (Matrix CSV)
│   ├── backups/                    # XZ-compressed CSV backups
│   └── flow/                       # Dispatcher working data
│       ├── reedbase.csv            # ReedBase cache indices
│       ├── reedcli.csv             # CLI transaction states
│       ├── reedserver.csv          # Server process tracking
│       └── reeddebug.csv           # Debug sessions and benchmarks
│
├── templates/                      # MiniJinja templates + components
│   ├── components/
│   │   ├── atoms/                  # Icon, Button, Badge
│   │   ├── molecules/              # Card, Menu, Form
│   │   └── organisms/              # PageHeader, PageFooter
│   │       └── {name}/
│   │           ├── {name}.mouse.jinja
│   │           ├── {name}.touch.jinja
│   │           ├── {name}.reader.jinja
│   │           ├── {name}.mouse.css
│   │           └── {name}.text.csv  # Component-local text
│   └── layouts/                    # Page layouts
│       └── {layout}/
│           ├── {layout}.mouse.jinja
│           ├── {layout}.touch.jinja
│           ├── {layout}.reader.jinja
│           ├── {layout}.mouse.css
│           └── {layout}.text.csv
│
├── src/reedcms/                    # Rust implementation
│   ├── reedstream.rs               # Universal communication (REED-01-01)
│   ├── reed/                       # Dispatchers (intelligent coordinators)
│   │   ├── reedbase.rs             # Data dispatcher
│   │   ├── reedcli.rs              # CLI dispatcher
│   │   ├── reedserver.rs           # Server dispatcher
│   │   └── reeddebug.rs            # Debug dispatcher
│   ├── reedbase/                   # ReedBase services (REED-02-01)
│   │   ├── get.rs                  # Get operations
│   │   ├── set.rs                  # Set operations
│   │   ├── init.rs                 # Initialisation
│   │   ├── cache.rs                # Cache management
│   │   └── environment.rs          # Environment fallback
│   ├── csv/                        # CSV handler (REED-02-02)
│   │   ├── reader.rs               # CSV reading
│   │   ├── writer.rs               # Atomic CSV writing
│   │   ├── entry.rs                # Entry structure
│   │   └── comments.rs             # Comment preservation
│   ├── matrix/                     # Matrix CSV handler (REED-02-05)
│   │   ├── reader.rs               # Matrix CSV reading
│   │   ├── writer.rs               # Matrix CSV writing
│   │   └── types.rs                # Matrix value types
│   ├── taxonomy/                   # Taxonomy system (REED-02-06)
│   │   ├── terms.rs                # Term CRUD operations
│   │   ├── entities.rs             # Entity tagging
│   │   └── hierarchy.rs            # Hierarchy navigation
│   ├── users/                      # User management (REED-03-01)
│   ├── roles/                      # Role permissions (REED-03-02)
│   ├── cli/                        # CLI commands (REED-04-*)
│   ├── filters/                    # MiniJinja filters (REED-05-01)
│   ├── template/                   # Template engine (REED-05-02/03)
│   ├── server/                     # Server services (REED-06-*)
│   ├── routing/                    # URL routing (REED-06-02)
│   ├── auth/                       # Authentication (REED-06-03)
│   ├── api/                        # HTTP API (REED-07-*)
│   ├── assets/                     # Asset bundling (REED-08-*)
│   ├── build/                      # Build system (REED-09-*)
│   └── monitor/                    # Monitoring (REED-10-*)
│
└── _workbench/                     # Development resources
    └── Tickets/                    # Implementation tickets
        ├── REED-01-Foundation/
        ├── REED-02-DataLayer/
        ├── REED-03-SecurityLayer/
        ├── REED-04-CLILayer/
        ├── REED-05-TemplateLayer/
        ├── REED-06-ServerLayer/
        ├── REED-07-APILayer/
        ├── REED-08-AssetLayer/
        ├── REED-09-BuildLayer/
        ├── REED-10-MonitorLayer/
        ├── REED-11-ExtensionLayer/
        ├── REED-20-ThirdParty/
        ├── REED-90-QualityLayer/
        ├── project_summary.md          # This file
        ├── project_optimisations.md    # Architectural decisions
        └── project_functions.csv       # Function registry
```

---

## Development Standards

All implementations must follow these mandatory standards:

- **Language**: BBC English for all documentation and code comments
- **Principle**: KISS (Keep It Simple, Stupid)
- **File Naming**: One file = One clear responsibility
- **Functions**: One function = One distinctive job
- **Testing**: Separate `.test.rs` files, never inline `#[cfg(test)]`
- **Avoid**: Swiss Army knife functions, generic names like `handler.rs` or `utils.rs`
- **License Header**: Every code file starts with copyright and SPDX identifier
- **Documentation**: Mandatory sections (Input, Output, Performance, Error Conditions, Examples)

See `CLAUDE.md` for complete development guidelines.

---

**End of Project Summary**
