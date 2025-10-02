    # ReedCMS - Complete System Design and Implementation Guide

> **Purpose**: Comprehensive, didactically structured guide to ReedCMS - from basic concepts to complete implementation. This document contains ALL planning information in logical learning order.

## Table of Contents
1. [What is ReedCMS?](#what-is-reedcms)
2. [Core Philosophy & Target Audiences](#core-philosophy--target-audiences)
3. [Mandatory Development Standards](#mandatory-development-standards)
4. [System Architecture Foundation](#system-architecture-foundation)
5. [Data Storage & CSV Architecture](#data-storage--csv-architecture)
6. [Communication System (ReedStream)](#communication-system-reedstream)
7. [Complete Component Architecture](#complete-component-architecture)
8. [CLI Interface & Commands](#cli-interface--commands)
9. [Template System Integration](#template-system-integration)
10. [Performance & Technical Specifications](#performance--technical-specifications)
11. [Implementation Guidelines](#implementation-guidelines)
12. [Migration & Integration](#migration--integration)

---

## What is ReedCMS?

**ReedCMS** is a high-performance, headless Content Management System built in Rust, designed for zero-configuration deployment and maximum developer productivity.

### Core Concept
```
ReedCMS = Enhanced Key:Value CSV Database + CLI Interface + Intelligent Dispatchers
```

### Key Characteristics
- **100% CSV-based data storage** - No database required, Git-friendly
- **CLI-first approach** - All operations via unified `reed` binary
- **O(1) performance** - HashMap-based runtime lookups, startup-time CSV loading
- **Environment-aware** - `@dev`, `@prod`, `@christmas` configuration overrides
- **Copy & Run deployment** - Just copy `.reed/` directory to new server
- **Professional CMS features** - Bulk operations, theme switching, asset bundling

### Zero-Configuration Deployment
```bash
# New server setup - instantly running CMS
git clone my-project.git
cd my-project/
reed server:io --port 80    # Instantly running CMS
```

---

## Core Philosophy & Target Audiences

### Universal KISS Principle
> **Kernprinzip**: KISS für den Kopf des Entwicklers, des Admins und auch des Content Managers

### Target Audiences & Their Simplicity Needs

#### 1. **Developer (Kopf des Entwicklers)**
- Clean `{{ "key" | text("de") }}` template syntax
- One function = One job Rust architecture
- Type-separated CSV files (text.csv, routes.csv, meta.csv)
- Predictable file structure and naming conventions

#### 2. **Admin (Kopf des Admins)**
- CLI-first approach: `reed set:text knowledge.title@de "Wissen" --desc "Page title"`
- Wildcard operations: `reed set:meta *.cache.ttl "3600"`
- Git-friendly CSV format for version control and backup
- Environment-aware configuration with @environment syntax

#### 3. **Content Manager (Kopf des Content Managers)**
- Future: Intuitive admin panel built on same CLI foundation
- Human-readable CSV structure (key;value;comment)
- Self-documenting data with mandatory comment fields
- No technical complexity - just content and context

### Database Philosophy
- **CSV Format**: Human-readable, Git-friendly, universally accessible
- **Key:Value++**: Enhanced with mandatory comment field for context
- **Central Storage**: `.reed/` directory - single source of truth
- **Type Separation**: Logical data organisation (text/routes/meta)

---

## Mandatory Development Standards

### MANDATORY Development Guidelines
- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal and professional
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function (CONFIG and SYSTEM use identical logic)
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules (see Testing Guidelines below)
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs`
- **CRITICAL**: AI agents must NEVER execute `rm` commands without explicit user confirmation

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

### Benefits of Mandatory Documentation
- ✅ **AI-Optimised**: Future AI tools understand coding standards immediately
- ✅ **Self-Documenting**: Code explains itself without external documentation
- ✅ **Team Onboarding**: New developers understand patterns instantly
- ✅ **Maintainability**: Clear function contracts prevent breaking changes
- ✅ **Professional Standards**: Enterprise-grade code quality

### MANDATORY Anti-Hallucination Guidelines
- **Question First**: Ask before implementing anything not directly derivable from existing src/libs/ code
- **Point-by-Point Approval**: Propose solutions step-by-step, wait for approval per point
- **Breaking Change Warnings**: Alert if proposed changes would break existing functionality
- **Alternative Proposals**: Suggest alternatives if original ideas cause issues

### Testing Guidelines

**File Organisation**: One test file per source file
```
/src/reedcms/reedbase/get.rs
/src/reedcms/reedbase/get.test.rs
```

**Test Structure** (`{name}.test.rs` files):
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

#[test]
fn test_get_text_environment_fallback() {
    // Test key@dev → key fallback logic
}
```

**Coverage Requirements**:
- ✅ All public functions must have tests
- ✅ Test success cases and all error paths
- ✅ Test environment override logic (@dev, @prod fallbacks)
- ✅ Test ReedResult<T> error variants
- ✅ Performance tests for O(1) guarantees
- ✅ Integration tests for dispatcher coordination

### Versioning Strategy

**Version Format**: `MAJOR.MINOR.{unixtime}`

**Development Phase** (Current):
- `0.1.{unixtime}` - Initial development
- Version bumped on every `git push` with current Unix timestamp
- Example: `0.1.1735574400` (pushed at 2025-01-30 12:00:00 UTC)

**Testing Phase**:
- `0.8.{unixtime}` - Product ready for testing
- All core features implemented and functional
- CLI commands operational
- Basic documentation complete

**Release Candidate Phase**:
- `0.9.{unixtime}` - Release candidates
- Feature-complete
- Bug fixing and stabilisation
- Performance optimisation

**Production Release**:
- `1.0` - First stable release
- All features tested and documented
- Production-ready performance
- Complete CLI command set

**Post-Release**:
- `1.{unixtime}` - Patch releases and minor features
- Unix timestamp for automatic versioning
- Follows same pattern as development phase

**Deprecation Strategy**:
- Breaking changes only in MAJOR version bumps
- Deprecated features marked 6 months before removal
- CLI warnings for deprecated commands
- Migration guides in documentation

---

## System Architecture Foundation

### Two-Layer Architecture Principle

ReedCMS follows a strict two-layer architecture separating intelligence from implementation:

#### Reed Core Layer (`src/reedcms/reed/`) - **Dispatchers**
Intelligent coordinators with business logic and persistence rights

#### Implementation Layer - **Services**
Pure implementation without persistence rights

### Architecture Types

#### **Dispatchers** (5 total)
- Have **intelligence** - business logic, coordination, decisions
- May **persist working data** to `.reed/flow/`
- Coordinate multiple services
- Handle complex scenarios (bulk operations, error recovery, caching)

#### **Services** (20+ total)
- Pure **implementation** - no business decisions
- **NO persistence rights** - communicate via ReedStream to dispatchers
- One file = One specific technical task
- Clear input/output contracts

### Source Code Organisation Pattern
```
/src/reedcms/{scope}/{function}.rs
```

**Examples:**
- `/src/reedcms/reedbase/get.rs` - Get operations for ReedBase
- `/src/reedcms/reedbase/set.rs` - Set operations for ReedBase
- `/src/reedcms/cli/init.rs` - Layout initialisation
- `/src/reedcms/assets/css_bundler.rs` - CSS compilation service

---

## Data Storage & CSV Architecture

### .reed/ Directory Structure
```
.reed/
├── server.csv               # Infrastructure & Authentication
├── project.csv              # Project & CMS Settings
├── text.csv                 # Content text data
├── routes.csv               # URL routing definitions
├── meta.csv                 # Page metadata
├── registry.csv             # Layout registry and management
├── presets.csv              # CLI presets for layout creation
├── i18n.csv                # Activated languages per project
├── a11y.csv                # Accessibility configuration
├── users.matrix.csv         # User management with roles
├── roles.matrix.csv         # Role-based permissions with inheritance
├── taxonomie.matrix.csv     # Universal taxonomy terms
├── entity_taxonomy.matrix.csv # Entity-term assignments
└── flow/                   # Dispatcher working data (persistent state)
    ├── reedbase.csv         # Cache indices, performance metrics
    ├── reedcli.csv          # Transaction states, bulk operation progress
    ├── reedserver.csv       # Process IDs, port mappings, health status
    └── reeddebug.csv        # Debug sessions, trace states, profiling data
```

### Matrix CSV System for Complex Relations

ReedCMS uses two types of CSV files:
- **Simple CSV**: `*.csv` for basic key-value data
- **Matrix CSV**: `*.matrix.csv` for complex relationships and hierarchical data

#### Automatic XZ Backup System

**Backup Strategy**: All CSV modifications are protected by automatic XZ compression backups:
```rust
// Before any write operation to .reed/ files
fn create_backup(csv_path: &str) -> ReedResult<()> {
    let timestamp = current_timestamp();
    let backup_path = format!(".reed/backups/{}.{}.csv.xz", filename, timestamp);

    // Compress original with XZ for efficient storage
    let original = std::fs::read(csv_path)?;
    let compressed = xz_compress(&original)?;
    std::fs::write(backup_path, compressed)?;

    // Keep latest 32 backups, cleanup older
    cleanup_old_backups(".reed/backups/", 32)?;
    Ok(())
}
```

**Recovery Commands**:
```bash
reed debug:backup list text.csv        # Show available backups
reed debug:backup restore text.csv 3   # Restore 3 steps back
reed debug:backup cleanup              # Manual cleanup old backups
```

#### Matrix CSV 4-Type Value System
```csv
# Type 1: Single values
username|status|desc
admin|active|System Administrator

# Type 2: Lists (comma-separated)
username|roles|desc
jane|editor,author|Multi-role user

# Type 3: Single values with modifiers
asset|optimization|desc
main.css|minify[prod]|Main stylesheet

# Type 4: Lists with modifiers (currently used in roles.matrix.csv)
rolename|permissions|desc
editor|text[rwx],route[rw-],project[r--]|Standard Editor
admin|users[rwx],content[rwx],system[rw-]|Full Administrator
```

#### Security & Permission System

ReedCMS implements a comprehensive Unix-style permission system with role inheritance:

```csv
# .reed/users.matrix.csv - Extended user management with social profiles
username|password|roles|firstname|lastname|street|city|postcode|region|country|email|mobile|twitter|facebook|tiktok|insta|youtube|whatsapp|desc|created_at|updated_at|last_login|is_active
admin|$argon2id$hash|admin|Admin|User|Main St 1|London|SW1A 1AA|London|UK|admin@example.com|+44123456789|||||||System Administrator|1640995200|1640995200||true
editor|$argon2id$hash|editor|Jane|Doe|High St 42|Manchester|M1 1AA|Manchester|UK|jane@example.com|+44987654321|@jane_editor|jane.doe||||Content Editor|1640995200|1640995200|1640999800|true

# .reed/roles.matrix.csv - Role-based permission system with inheritance
rolename|permissions|inherits|desc|created_at|updated_at|is_active
editor|text[rwx],route[rw-],content[rw-]||Standard Content Editor|1640995200|1640995200|true
admin|*[rwx]|editor|Full Admin with inheritance|1640995200|1640995200|true
author|text[rw-],content[r--]|editor|Content Author|1640995200|1640995200|true
viewer|*[r--]||Read-only access|1640995200|1640995200|true
```

**Enhanced User Management Features**:
- **Argon2 Password Hashing**: Secure password storage with salt
- **Extended Profile Data**: Full contact information and social media profiles
- **Account Status Tracking**: Login history, account activation, timestamps
- **Password Validation**: Strength requirements with uppercase, lowercase, digit, special character
- **Email/Username Uniqueness**: Enforced validation and conflict prevention

**Permission Architecture**:
- **Unix-Style Syntax**: `resource[rwx]` where r=read, w=write, x=execute
- **Hierarchical Inheritance**: `content/blog/*` applies to `content/blog/article_1`
- **Role Inheritance**: Roles inherit permissions via `inherits` field with circular detection
- **Wildcard Support**: `*[rwx]` grants full access to all resources
- **Sub-millisecond Lookups**: Cached permission checks for performance
- **Permission Override**: Child roles can override inherited permissions
- **Safe Role Deletion**: Dependency checking prevents orphaned inheritance

**Permission Resolution**:
1. Check user's direct permissions
2. Check user's role permissions
3. Check inherited role permissions recursively
4. Apply hierarchical resource matching
5. Cache result for subsequent lookups
6. Automatic cache invalidation on permission changes

**Performance**: Sub-millisecond cached permission lookups with automatic cache invalidation on role/permission changes

#### Universal Taxonomy System

**Status**: ✅ Implemented in REED-02-06 (58/58 tests passing, 100% coverage)

ReedCMS provides a comprehensive hierarchical taxonomy system for universal entity tagging using Matrix CSV Type 1 (Single) and Type 2 (List):

```csv
# .reed/taxonomie.matrix.csv - Hierarchical taxonomy term definitions (Matrix CSV Type 1)
term_id|term|category|parent_id|description|color|icon|status|created_by|usage_count|created_at|updated_at
Programming:Rust|Rust|Programming||Systems programming language|#FF6600|rust-logo|active|admin|0|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
Topics:Programming|Programming|Topics||Programming languages|#2563eb|code|active|admin|2|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
Topics:Rust|Rust|Topics|Topics:Programming|Rust sub-category|#ce422b|rust|active|admin|5|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z

# .reed/entity_taxonomy.matrix.csv - Entity-term assignments (Matrix CSV Type 2: Lists)
entity_key|entity_type|entity_id|term_ids|assigned_by|assigned_at|updated_at
content:post-123|content|post-123|Programming:Rust,Topics:Systems|admin|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
user:admin|user|admin|Topics:Programming|system|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
template:blog|template|blog.jinja|Programming:Rust|admin|2025-01-01T00:00:00Z|2025-01-01T00:00:00Z
```

**Implementation Details** (REED-02-06):
- **Module**: `src/reedcms/taxonomy/` with terms.rs, entities.rs, hierarchy.rs
- **Term ID Format**: `{category}:{term}` for uniqueness (e.g., `Programming:Rust`)
- **Matrix CSV Integration**: Uses MatrixValue::Single for single values, MatrixValue::List for term_ids
- **Field Access**: `record.fields.get("field_name")` with MatrixValue pattern matching
- **First Field Strategy**: term_id as first field in field_order for record identification

**Term Management** (terms.rs):
- **CRUD Operations**: create_term, get_term, list_terms, search_terms, update_term, delete_term
- **Validation**: 2-64 character names, alphanumeric + spaces/hyphens/underscores, hex color validation
- **Hierarchical Support**: Parent-child relationships with unlimited depth
- **Duplicate Prevention**: Category-scoped uniqueness (same term in different categories allowed)
- **Search**: Full-text search across term name, category, and description
- **Status Management**: Active/Inactive terms with soft deletion

**Entity Tagging** (entities.rs):
- **8 Universal Entity Types**: User, Content, Template, Route, Site, Project, Asset, Role
- **Operations**: assign_terms, get_entity_terms, list_entities_by_term, unassign_terms
- **Usage Tracking**: Automatic increment/decrement of term usage_count
- **Entity Key Format**: `{entity_type}:{entity_id}` (e.g., `content:post-123`)
- **Term IDs Storage**: MatrixValue::List for multiple terms per entity

**Hierarchy Navigation** (hierarchy.rs):
- **get_children(term_id, recursive)**: Direct children or all descendants
- **get_ancestors(term_id)**: Full ancestry path from root to term
- **get_path(term_id, separator)**: Formatted path string (e.g., "Programming > Rust > Async")
- **get_depth(term_id)**: Depth level in hierarchy (0 = root)
- **has_circular_reference(term_id, new_parent_id)**: Cycle detection before parent updates
- **get_tree(category)**: Complete tree structure with nested children

**Performance Characteristics**:
- **Term Creation**: <10ms for <1000 terms (O(n) uniqueness check)
- **Term Lookup**: <5ms for <1000 terms (O(n) linear search)
- **Search**: <50ms for 10,000+ terms (O(n) with text matching)
- **Hierarchy Traversal**: <5ms for depth <10 (O(d) where d = depth)
- **Tree Building**: <100ms for <1000 terms (O(n²) worst case)

**Circular Reference Protection**:
- BFS traversal to detect cycles before updates
- Validates term cannot be its own parent
- Checks descendants to prevent circular parent-child relationships
- Force delete option cascades to children

**Test Coverage**:
- 58/58 tests passing (100% coverage)
- Terms: 25 tests (CRUD, validation, search, performance)
- Entities: 18 tests (assignments, usage tracking, entity types)
- Hierarchy: 15 tests (traversal, cycles, tree building)

### Flow Persistence Rules
- **ONLY Dispatchers** may write to `.reed/flow/` directory
- **Services** communicate via ReedStream to their dispatcher for persistence needs
- **Automatic cleanup** of flow data on clean shutdown
- **Flow data** is working state, not permanent configuration

#### Flow Persistence Specification

**Purpose**: Dispatcher-exclusive write access for operational data and performance metrics

**Format**: CSV with standardised columns for operational tracking
```csv
# .reed/flow/reedbase.csv
timestamp,operation,key,execution_time_ms,environment
2025-01-15T10:00:00Z,get_text,knowledge.title,15,DEV
2025-01-15T10:00:01Z,set_text,knowledge.subtitle,45,DEV

# .reed/flow/reedcli.csv
timestamp,operation,transaction_id,status,affected_files
2025-01-15T10:05:00Z,bulk_migrate,tx_001,in_progress,"text.csv,routes.csv"
2025-01-15T10:05:30Z,bulk_migrate,tx_001,completed,"text.csv,routes.csv"

# .reed/flow/reedserver.csv
timestamp,process_id,port,status,bind_address
2025-01-15T10:10:00Z,1234,8080,running,127.0.0.1
2025-01-15T10:15:00Z,1234,8080,stopped,127.0.0.1
```

**Architecture Rules**:
- Foundation services are **read-only** - no persistence rights
- Dispatchers **coordinate** services and persist working data
- Services communicate via ReedStream to their dispatcher for persistence needs

### CSV Format Standards

#### Main Data Files Examples
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

### Data Principles
- ✅ Layout namespacing (knowledge.*, portfolio.*)
- ✅ @environment overrides for all configurations
- ✅ Type-separated files for performance
- ✅ Universal CSV handler with comment preservation
- ❌ NEVER CLI management data mixed with content

### Key Nomenclature Rules

**MANDATORY Key Naming Standards** for all ReedCMS data files:

#### 1. Dot-Notation (Not Underscores)
```csv
# ✅ CORRECT - Dot notation
knowledge.page.title@de|Wissen|German page title
page-header.logo.title@de|vvoss|Logo title
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
page-header.logo.title@de|vvoss|Logo title
page-header.menu.knowledge@de|Wissen|Menu text
page-footer.copyright@de|© 2025 Vivian Voss|Footer copyright

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

#### 7. Component Names: Hyphens Allowed
```csv
# ✅ CORRECT - Hyphens in component names
page-header.logo.title@de
landing-hero.headline@de
knowledge-intro.title@de

# ✅ ALSO CORRECT - Without hyphens if simpler
pageheader.logo.title@de
```
**Rationale**: Compatibility with file system naming conventions.

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
page-header.logo.title@de|vvoss|Logo title
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

#### 10. Migration from Underscore Legacy Format

**Old Format (vvoss.dev legacy):**
```csv
PAGE_HEADER_LOGO_TITLE@DE|vvoss|Logo title
LANDING_HERO_HEADLINE@DE|Entwickler|Hero headline
AGILITY_META_TITLE@DE|Agilität|Page title
```

**New Format (ReedCMS):**
```csv
# In .reed/text.csv
page-header.logo.title@de|vvoss|Logo title
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

### Initial Configuration Structure

**ReedCMS is a ground-up reimplementation** - inspired by vvoss.dev concepts but built from scratch with clean architecture.

**Relationship to vvoss.dev**:
- ✅ vvoss.dev serves as conceptual template and inspiration
- ✅ ReedCMS implements the same core ideas with professional architecture
- ✅ Complete code rewrite - no legacy code dependencies
- ✅ Clean `.reed/` structure from the beginning
- ✅ CLI-first approach without backward compatibility constraints

#### Server Configuration Keys (.reed/server.csv)
```csv
key|value|comment
server.auth.enabled|true|Enable HTTP Basic Authentication
server.auth.username|admin|Authentication username
server.auth.password|$argon2id$hash|Hashed authentication password
server.endpoint|127.0.0.1:8080|Server bind address and port
server.workers|4|Number of worker threads
server.logging.level|info|Logging verbosity level
```

#### Project Configuration Keys (.reed/project.csv)
```csv
key|value|comment
project.languages|de,en|Active languages for project
project.template.path|templates/|Template directory path
project.layout.path|templates/layouts/|Layout directory path
project.atoms.path|templates/components/atoms/|Atoms directory path
project.molecules.path|templates/components/molecules/|Molecules directory path
project.organisms.path|templates/components/organisms/|Organisms directory path
project.index|landing|Default landing page layout
project.template.cache|true|Enable template caching
project.public.path|public/|Static files directory
project.public.cache.max_age|3600|Cache max age in seconds
```

---

## Communication System (ReedStream)

### ReedStream - Universal Module Communication Interface

> **Note**: Complete implementation specification in REED-01-01. Service template standardisation completed 2025-01-30 (see `project_optimisations.md` §17).

All ReedCMS modules must use the standardised ReedStream communication protocol:

```rust
// src/reedcms/reedstream.rs - MANDATORY for all modules

/// Standard Result type for all ReedCMS operations
pub type ReedResult<T> = Result<T, ReedError>;

/// Standard Error types across all modules - comprehensive coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReedError {
    /// Resource not found
    NotFound { resource: String, context: Option<String> },
    /// Data parsing or validation error
    ParseError { input: String, reason: String },
    /// File system or I/O operation error
    IoError { operation: String, path: String, reason: String },
    /// Input validation failed
    ValidationError { field: String, value: String, constraint: String },
    /// Authentication or authorization failure
    AuthError { user: Option<String>, action: String, reason: String },
    /// Configuration or setup error
    ConfigError { component: String, reason: String },
    /// CSV file operation error
    CsvError { file_type: String, operation: String, reason: String },
    /// Template rendering error
    TemplateError { template: String, reason: String },
    /// Server or network operation error
    ServerError { component: String, reason: String },
    /// Invalid CLI command or parameters
    InvalidCommand { command: String, reason: String },
}

/// Standard Request structure for all module communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedRequest {
    pub key: String,
    pub language: Option<String>,
    pub environment: Option<String>,
    pub context: Option<String>,
    pub value: Option<String>,        // For set operations
    pub description: Option<String>,  // For set operations (comment field in CSV)
}

/// Standard Response structure with performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedResponse<T> {
    pub data: T,
    pub source: String,                    // Which module provided this response
    pub cached: bool,                      // Whether this was served from cache
    pub timestamp: u64,                    // When this response was generated
    pub metrics: Option<ResponseMetrics>,  // Optional performance metrics
}

/// Performance metrics for responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetrics {
    pub processing_time_us: u64,           // Processing time in microseconds
    pub memory_allocated: Option<u64>,     // Memory allocated during processing
    pub csv_files_accessed: u8,            // Number of CSV files accessed
    pub cache_info: Option<CacheInfo>,     // Cache hit/miss information
}

/// Cache information for performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub hit: bool,                         // Whether this was a cache hit
    pub ttl_remaining_s: Option<u64>,      // Time-to-live remaining in seconds
    pub cache_key: String,                 // The cache key used for lookup
    pub cache_layer: String,               // Which cache layer (text/route/meta)
}

/// Standard Module trait - ALL modules must implement
pub trait ReedModule {
    fn module_name() -> &'static str;
    fn health_check() -> ReedResult<ReedResponse<String>>;
    fn version() -> &'static str { "1.0.0" }
    fn dependencies() -> Vec<&'static str> { Vec::new() }
}

/// Convenience functions for common error creation
pub fn not_found(resource: impl Into<String>) -> ReedError;
pub fn validation_error(field: impl Into<String>, value: impl Into<String>, constraint: impl Into<String>) -> ReedError;
pub fn csv_error(file_type: impl Into<String>, operation: impl Into<String>, reason: impl Into<String>) -> ReedError;
```

#### Interface Contract System

**Universal Implementation Pattern**: Every public function across ALL modules must follow this contract:

```rust
// Example from reedbase/get.rs
pub fn text(req: &ReedRequest) -> ReedResult<ReedResponse<String>> {
    let start_time = std::time::Instant::now();
    let data = lookup_text(&req.key, &req.language)?;

    let metrics = ResponseMetrics {
        processing_time_us: start_time.elapsed().as_micros() as u64,
        memory_allocated: None,
        csv_files_accessed: 1,
        cache_info: Some(CacheInfo { hit: false, key: req.key.clone(), age_seconds: None }),
    };

    Ok(ReedResponse::new(data, "reedbase::get::text").with_metrics(metrics))
}
```

**Contract Benefits**:
- ✅ **Universal Interface**: Every module speaks the same language
- ✅ **Performance Tracking**: Built-in metrics collection across all operations
- ✅ **Error Standardisation**: Consistent error handling with rich context
- ✅ **Source Tracking**: Full traceability of where responses originate
- ✅ **Type Safety**: Rust compiler enforces consistent communication patterns

### Mandatory Implementation Pattern

**Every public function in every module must follow this pattern:**

```rust
// Example from reedbase/get.rs
pub fn text(req: &ReedRequest) -> ReedResult<ReedResponse<String>> {
    let data = lookup_text(&req.key, &req.language)?;
    Ok(ReedResponse {
        data,
        source: "reedbase::get".to_string(),
        cached: false,
        timestamp: current_timestamp(),
    })
}

// Example from routing/resolver.rs
pub fn resolve_url(req: &ReedRequest) -> ReedResult<ReedResponse<RouteInfo>> {
    let route_info = resolve_internal(&req.key)?;
    Ok(ReedResponse {
        data: route_info,
        source: "routing::resolver".to_string(),
        cached: true,
        timestamp: current_timestamp(),
    })
}
```

### ReedStream Benefits
- ✅ **Uniform Interface**: Every module speaks the same language
- ✅ **Type Safety**: Rust compiler enforces consistent communication
- ✅ **Error Handling**: Standardised error types across all modules
- ✅ **Debugging**: Source tracking for all responses
- ✅ **Performance Monitoring**: Built-in caching and timing information
- ✅ **Future-Proof**: Easy to extend with new fields without breaking changes

---

## Complete Component Architecture

### Reed Core Layer (`src/reedcms/reed/`) - **Dispatchers**

#### 1. **ReedStream** - `reedstream.rs` - Universal service communication interface
- Type definitions: ReedRequest, ReedResponse, ReedError
- Service trait definitions for consistent interfaces
- NO intelligence - pure type system

#### 2. **ReedBase Dispatcher** - `reedbase.rs` - Intelligent data engine dispatcher
- **Environment-Fallback Logic**: `key@dev` → `key` when not found
- **Cache Management**: O(1) HashMap performance with smart invalidation
- **Cache Invalidation Strategy**: 
  - Granular key-specific invalidation after set operations
  - Full cache type invalidation (text/route/meta) on demand
  - Global cache invalidation for migrations and system restarts
  - Automatic invalidation on CSV file writes via `set.rs` integration
  - Performance: < 50ms for complete cache refresh (5 CSV files + HashMap rebuild)
- **Error Recovery**: Retry-logic for CSV file operations
- **Data Consistency**: Atomic writes to .reed/ files
- **Batch Optimisation**: Group operations by CSV file type
- **Flow Persistence**: `.reed/flow/reedbase.csv` for cache indices and metrics

#### 3. **ReedCLI Dispatcher** - `reedcli.rs` - Intelligent command dispatcher for bulk operations
- **Bulk Performance**: Batch CSV operations (1× file open vs N×)
- **Transaction Management**: Atomic multi-file operations
- **Theme Switching**: Seasonal content switching (Christmas/default)
- **Migration Coordination**: Large-scale content imports
- **Dependency Resolution**: Multi-layout creation with assets
- **State Management**: Backup/restore for safe bulk changes
- **Flow Persistence**: `.reed/flow/reedcli.csv` for transaction states and bulk progress

#### 4. **ReedServer Dispatcher** - `reedserver.rs` - Intelligent server lifecycle dispatcher
- **Process Management**: Start/stop with PID tracking and cleanup
- **Zombie Cleanup**: Find and terminate hanging instances
- **Port Conflict Resolution**: Detect and resolve socket/port conflicts
- **Graceful Shutdown**: Safe shutdown with active request completion
- **Health Monitoring**: Server status and resource monitoring
- **Auto-Recovery**: Restart crashed instances with backoff logic
- **Monitoring Integration**: FreeBSD-style logging and metrics collection
- **Flow Persistence**: `.reed/flow/reedserver.csv` for PIDs and health status

#### 5. **ReedDebug Dispatcher** - `reeddebug.rs` - Intelligent debugging and diagnostics dispatcher
- **Template Debugging**: Live template variable inspection and rendering traces
- **Performance Profiling**: CSV lookup timing, cache hit/miss rates, template rendering performance
- **Data Flow Tracing**: Full request → response pipeline visualisation
- **Error Context**: Rich error messages with full ReedCMS state context
- **Hot Development**: Live reload coordination and change detection
- **Diagnostic Reports**: System health snapshots and performance bottleneck analysis
- **Flow Persistence**: `.reed/flow/reeddebug.csv` for debug sessions and trace states

#### 6. **ReedMonitor Foundation** - `monitor.rs` - Universal monitoring and logging system
- **Metrics Collection**: Counter, Timer, Gauge, and Event tracking across all services
- **FreeBSD Syslog Integration**: Professional system logging with standard Unix format
- **Performance Monitoring**: Zero-allocation message passing with configurable output modes
- **Multi-Mode Output**: Silent, Log, Forward, or Both modes for different environments
- **Automatic Timing**: RAII-based timer guards for performance measurement
- **System Integration**: Hostname, PID, and process tracking for server environments

#### FreeBSD-Style System Logging Specification

**Format**: Standard Unix syslog format for professional system integration
```
{timestamp} {hostname} {process}[{pid}]: {level}: {message}
```

**Example Output**:
```bash
Dec 15 14:23:01 server reedcms[1234]: INFO: METRIC[counter] csv_operations: 42
Dec 15 14:23:02 server reedcms[1234]: INFO: METRIC[timer] csv_read: 15ms
Dec 15 14:23:03 server reedcms[1234]: WARN: METRIC[timer] template_render: 1250ms
Dec 15 14:23:04 server reedcms[1234]: ERROR: EVENT user_login_failed {"username": "invalid"}
```

**Log Levels**:
- EMERG, ALERT, CRIT, ERROR, WARN, NOTICE, INFO, DEBUG

**Automatic Level Assignment**:
- Counter > 1000 → WARN
- Timer > 1000ms → WARN
- Event name contains "error" → ERROR
- Event name contains "warn" → WARN
- Default → INFO

**Performance Requirements**:
- Zero-allocation message passing
- Configurable overhead: <1% of monitored operations
- Support for 10k+ messages per second
- Memory footprint <10MB

**Output Modes**:
- **Silent**: No monitoring output (performance mode)
- **Log**: Write to system logs and optional log file
- **Forward**: Send metrics to external monitoring systems
- **Both**: Log locally and forward to external systems

**Advanced Monitoring Capabilities**:
- **RAII Timer Guards**: Automatic timing measurement with zero-allocation patterns
- **Metric Types**: Counter, Timer, Gauge, and Event tracking with rich metadata
  - **Counter**: Operation counts (csv_operations, template_renders)
  - **Timer**: Performance metrics (csv_read_time, request_duration)
  - **Gauge**: Current values (active_connections, memory_usage)
  - **Event**: Custom events with metadata (user_login, error_occurred)
- **Hostname Integration**: Automatic system identification in multi-server environments
- **Process Tracking**: PID and process name integration for system monitoring
- **Log File Management**: Optional file output with append-only writing
- **External Forwarding**: Message passing system for external monitoring integration
- **Performance Overhead**: <1% operational impact with configurable thresholds
- **Message Throughput**: Support for 10k+ messages per second
- **Memory Efficiency**: <10MB footprint for comprehensive monitoring

**Performance Benchmarking Integration**:
```rust
// Automatic benchmark collection via ReedMonitor
use reedcms::monitor::{benchmark, BenchmarkConfig};

// Benchmark CSV operations
let _timer = benchmark("csv_read_text", BenchmarkConfig {
    target_p50: 100,  // microseconds
    target_p95: 500,  // microseconds
    target_p99: 1000, // microseconds
});
// Automatic timing on drop

// Results logged to .reed/flow/reeddebug.csv
// operation,p50_us,p95_us,p99_us,samples,timestamp
// csv_read_text,85,450,950,10000,2025-01-30T12:00:00Z
```

**Benchmark Targets** (enforced via monitoring):
- CSV operations: p95 < 500μs, p99 < 1ms
- HashMap lookups: p95 < 10μs, p99 < 50μs
- Template rendering: p95 < 50ms, p99 < 100ms
- Full request cycle: p95 < 100ms, p99 < 200ms
- Startup time: < 200μs for 17 layouts

**CLI Performance Commands**:
```bash
reed debug:benchmark csv           # Show CSV operation benchmarks
reed debug:benchmark templates     # Show template rendering benchmarks
reed debug:benchmark report        # Generate full performance report
reed debug:benchmark compare v1 v2 # Compare two versions
```

---

## ReedAPI - HTTP Interface Layer

### Single-Endpoint API Architecture

ReedAPI provides a minimal HTTP wrapper around ReedCMS CLI commands, enabling web interfaces and external integrations while maintaining the CLI-first philosophy.

#### Core API Design
```
POST /api/reed
{
  "command": "user:list --format json",
  "auth": "bearer_token",
  "environment": "prod"
}
```

#### API Security Matrix Configuration

The API layer uses Matrix CSV files for comprehensive security control:

```csv
# .reed/api.matrix.csv - Command validation rules
rule_type|patterns|permissions|description|environments|rate_limit
whitelist|user,role,taxonomy,get,set|api[rwx]|Core API commands allowed|@prod,@dev|1000
blacklist|server,debug,build|api[---]|Server commands blocked from API|@prod,@dev|0
restricted|delete,remove,cleanup|api[rw-:confirm]|Destructive operations need confirmation|@prod|10
admin_only|flow,backup,migration|admin[rwx]|Admin-only dangerous operations|@prod,@dev|5
dev_only|debug,trace,profile|dev[rwx]|Development commands only|@dev|unlimited

# .reed/api_auth.matrix.csv - Token-based authentication
token_type|pattern|permissions|expires_in|rate_limit|description
bearer|api_*|user[r--],content[rw-]|3600|100|Standard API access
admin|admin_*|*[rwx]|7200|1000|Full admin access
readonly|ro_*|*[r--]||unlimited|Read-only access for monitoring
service|svc_*|get[rwx],set[rw-]|unlimited|10000|Service-to-service communication
```

**Advanced Security Features**:
- **Token Expiration Management**: Automatic token expiry with configurable lifetimes
- **Rate Limiting System**: Per-token rate limits with 1-minute sliding windows
- **Command Pattern Matching**: Flexible pattern-based command validation
- **Environment-Aware Rules**: Different security policies for @dev/@prod environments
- **Destructive Operation Protection**: Mandatory `--confirm` flag for dangerous commands
- **Admin Privilege Escalation**: Separate admin tokens for privileged operations
- **Real-time Rate Monitoring**: Active rate limit tracking with window resets

**CLI Command Execution Bridge**:
- **Output Format Detection**: Automatic JSON, CSV, and text parsing
- **Streaming Support**: Progress callbacks for long-running operations
- **Batch Execution**: Atomic transaction support for multiple commands
- **Binary Path Discovery**: Automatic reed binary location and validation
- **Health Checks**: Continuous monitoring of CLI command executor status
- **Timeout Management**: Configurable execution timeouts with proper cleanup
- **Error Translation**: Comprehensive error handling with context-aware messages

**Matrix Security Features**:
- **Permission Syntax**: `resource[rwx]` where r=read, w=write, x=execute
- **Environment Awareness**: Different rules for @dev/@prod environments
- **Rate Limiting**: Per-token and per-command rate limits with unlimited option
- **Token Patterns**: Wildcard matching for flexible token management
- **Command Validation**: Whitelist/blacklist patterns with confirmation requirements

#### Format Support
- **JSON (default)**: `--format json` for web applications
- **CSV**: `--format csv` for Excel exports and bulk operations
- **Table**: `--format table` for debugging and CLI-style output

#### Security Features
- **Command Validation**: Whitelist/blacklist via api.matrix.csv
- **Token-based Authentication**: Multiple token types with different permissions
- **Rate Limiting**: Per-token and per-command rate limits
- **Environment Awareness**: Different rules for @dev/@prod environments
- **Audit Logging**: All API calls logged via FreeBSD-style monitoring

#### Benefits of CLI-as-API Approach
- ✅ **Zero API Drift**: CLI commands automatically become API endpoints
- ✅ **Consistent Validation**: Same business logic for CLI and API
- ✅ **Single Maintenance Point**: No duplicate API implementations
- ✅ **Self-Documenting**: CLI help becomes API documentation
- ✅ **Format Flexibility**: JSON for apps, CSV for exports

### Implementation Services

> **Service Implementation Templates**: For comprehensive service implementation and testing standards, see:
> - [Service Template](_workbench/Tickets/templates/service-template.md) - Complete service implementation guide with mandatory standards
> - [Service Test Template](_workbench/Tickets/templates/service-template.test.md) - Separate test file structure and patterns

#### 6. **ReedBase Services** (`src/reedcms/reedbase/`)
- `get.rs`, `set.rs`, `init.rs`, `cache.rs` - O(1) HashMap CSV database implementation services
- **Note**: `environment.rs` is owned by REED-02-03 (Environment Resolution Service) and referenced by ReedBase

#### 7. **CLI Services** (`src/reedcms/cli/`)
- `init.rs`, `set.rs`, `get.rs`, `migrate.rs`, `validate.rs`, `preset.rs`, `build.rs`, `server.rs` - CLI command services

#### 8. **Server Services** (`src/reedcms/server/`)
- `actix.rs`, `socket.rs`, `http.rs`, `start.rs`, `stop.rs`, `restart.rs` - Server implementation services

#### 9. **Debug Services** (`src/reedcms/debug/`)
- `template_trace.rs`, `performance_profiler.rs`, `error_context.rs`, `hot_reload.rs` - Debug implementation services

#### 10. **Template Filter Services** (`src/reedcms/filters/`)
- `text.rs`, `route.rs`, `meta.rs`, `config.rs` - MiniJinja integration services

#### 11. **CSV Handler Services** (`src/reedcms/csv/`)
- `reader.rs`, `writer.rs` - Universal .reed/ file management services

#### Missing Pipeline Services

#### 12. **Authentication Services** (`src/reedcms/auth/`)
- `auth.rs` - HTTP Basic Auth validation and middleware service

#### 13. **Logging Services** (`src/reedcms/logging/`)
- `logging.rs` - Request logging and file output service

#### 14. **Error Handling Services** (`src/reedcms/errors/`)
- `errors.rs` - Error middleware and page generation service

#### 15. **Request Router Services** (`src/reedcms/routing/`)
- `resolver.rs` - URL → Layout + Language resolution service via .reed/routes.csv
- `matcher.rs` - Pattern matching service for dynamic routes
- `language.rs` - Language detection and routing service

#### 16. **Template Engine Services** (`src/reedcms/templates/`)
- `minijinja_setup.rs` - MiniJinja environment configuration service
- `template_loader.rs` - Template file discovery and loading service
- `context_builder.rs` - Template context building service with ReedBase data

#### 17. **Asset Bundler Services** (`src/reedcms/assets/`)
- `css_bundler.rs` - CSS compilation service per session hash
- `js_bundler.rs` - JavaScript bundling service
- `component_discovery.rs` - Component asset discovery service (atoms/molecules/organisms)
- `static_files.rs` - Static file serving service

#### 18. **Response Builder Services** (`src/reedcms/response/`)
- `html_generator.rs` - HTML response generation service
- `http_headers.rs` - HTTP headers management service
- `cache_control.rs` - Cache control headers service
- `security_headers.rs` - Security headers service
- `error_pages.rs` - Error page generation service

#### 19. **Build System Services** (`src/reedcms/build/`)
- `binary_compiler.rs` - ReedCMS binary compilation service
- `asset_pipeline.rs` - Asset pipeline integration service
- `file_watcher.rs` - File change detection service

#### 20. **Development Services** (`src/reedcms/dev/`)
- `template_hot_reload.rs` - Template hot reloading service
- `development_monitor.rs` - Development monitoring service

### Component Interaction Flow
```
1. server/actix.rs → Start HTTP server
2. auth/auth.rs → Validate request
3. routing/resolver.rs → URL → Layout via .reed/routes.csv
4. reedbase/get.rs → Load content via O(1) HashMap lookup
5. templates/template_loader.rs → Apply filters + render template
6. assets/css_bundler.rs → Serve session-specific CSS/JS
7. response/html_generator.rs → Generate final HTTP response
8. Browser receives complete page
```

---

## CLI Interface & Commands

### Implementation Status

**REED-04-01: CLI Command Foundation** ✅ Complete (2025-02-02)
- **Parser**: Colon notation (`reed namespace:action [args] [--flags]`)
- **Router**: HashMap-based O(1) command routing
- **Help System**: General help, command-specific help, version info
- **Test Coverage**: 44/44 tests (100%)
- **Files**: parser.rs, router.rs, help.rs + 3 test files

**REED-04-02: CLI Data Commands** ✅ Complete (2025-10-02)
- **Set Commands**: set:text, set:route, set:meta with mandatory --desc flag (min 10 chars)
- **Get Commands**: get:text, get:route, get:meta with ReedBase integration
- **List Commands**: list:text, list:route, list:meta with pattern matching (*, prefix.*, *.suffix)
- **Test Coverage**: 18/18 tests (100%)
- **Files**: data_commands.rs, data_commands_test.rs

**REED-04-03: CLI Layout Commands** ✅ Complete (2025-10-02)
- **Flag-Based**: No interactive prompts, all via --languages, --variants, --routes, --parent
- **Template Generation**: Automatic Jinja + CSS file creation for all variants
- **Registry Management**: Automatic updates to .reed/registry.csv
- **Default Content**: Auto-generated routes, text, meta entries
- **Multi-Layout**: Batch creation support (init:layout a b c)
- **Test Coverage**: 21/21 tests (100%)
- **Files**: layout_commands.rs (425 lines), layout_commands_test.rs

**REED-04-04: CLI User Commands** ✅ Complete (2025-10-02)
- **Security API Integration**: Uses ReedRequest/ReedResponse pattern
- **Commands**: user:create, user:list, user:show, user:update, user:delete, user:passwd, user:roles
- **Output Formats**: Table (default), JSON, CSV for list operations
- **Flag-Based**: All via --email, --password, --roles, --force (no prompts)
- **Role Management**: Show current, add, remove, or set roles
- **Test Coverage**: 28 tests (compilation successful)
- **Files**: user_commands.rs (472 lines), user_commands_test.rs

**REED-04-10: Man Page Documentation** 📋 Ticket Created (2025-02-02)
- **Decision**: Implement comprehensive Unix/Linux man page system
- **Format**: Markdown-based `.ronn` source compiled to `.1` groff
- **Structure**: Main `reed.1` + 7 subcommand pages (data, layout, user, role, taxonomy, server, build)
- **Build Tool**: `ronn-ng` gem for Markdown → groff compilation
- **Rationale**: Professional tool standard, offline access, system integration

### Command Syntax Standard
**All CLI commands use colon notation**: `reed command:action`

**Parser Features**:
- Boolean flags: `--help`, `--verbose`, `--dry-run`, `--json`, `--force`, `--watch`
- Short flags: `-h`, `-v` (single-character boolean)
- Value flags: `--email value`, `--desc "text"`, `--port 8333`
- Validation: Alphanumeric + underscore + hyphen for namespace/action
- Help interception: `--help` and `-h` show command help before execution

### Complete CLI Command Reference

| CLI Command | Template Access | ReedStream API | Input | Output | File |
|-------------|----------------|---------------|-------|--------|------|
| **Data Management** | | | | | |
| `reed set:text key@lang "value" --desc "..."` | `{{ "key" \| text("lang") }}` | `reedbase::set::text(req)` | `key: String, lang: String, value: String, desc: String` | `ReedResult<ReedResponse<()>>` | `set.rs` |
| `reed set:route key@lang "route" --desc "..."` | `{{ "key" \| route("lang") }}` | `reedbase::set::route(req)` | `key: String, lang: String, route: String, desc: String` | `ReedResult<ReedResponse<()>>` | `set.rs` |
| `reed set:meta key "value" --desc "..."` | `{{ "key" \| meta }}` | `reedbase::set::meta(req)` | `key: String, value: String, desc: String` | `ReedResult<ReedResponse<()>>` | `set.rs` |
| `reed set:server key "value" --desc "..."` | `{{ "server.key" \| config }}` | `reedbase::set::server(req)` | `key: String, value: String, desc: String` | `ReedResult<ReedResponse<()>>` | `set.rs` |
| `reed set:project key "value" --desc "..."` | `{{ "project.key" \| config }}` | `reedbase::set::project(req)` | `key: String, value: String, desc: String` | `ReedResult<ReedResponse<()>>` | `set.rs` |
| `reed get:text key@lang` | `{{ "key" \| text("lang") }}` | `reedbase::get::text(req)` | `key: String, lang: String` | `ReedResult<ReedResponse<String>>` | `get.rs` |
| `reed get:route key@lang` | `{{ "key" \| route("lang") }}` | `reedbase::get::route(req)` | `key: String, lang: String` | `ReedResult<ReedResponse<String>>` | `get.rs` |
| `reed get:server key` | `{{ "server.key" \| config }}` | `reedbase::get::server(req)` | `key: String` | `ReedResult<ReedResponse<String>>` | `get.rs` |
| `reed get:project key` | `{{ "project.key" \| config }}` | `reedbase::get::project(req)` | `key: String` | `ReedResult<ReedResponse<String>>` | `get.rs` |
| `reed list:text pattern.*` | - | `reedbase::get::list_text(req)` | `pattern: String` | `ReedResult<ReedResponse<Vec<String>>>` | `get.rs` |
| **Layout Management** | | | | | |
| `reed init:layout name [names...]` | - | `layouts::create(req)` | `names: Vec<String>` | `ReedResult<ReedResponse<Vec<LayoutInfo>>>` | `init.rs` |
| `reed init:layout name --parent parent` | - | `layouts::create_child(req)` | `name: String, parent: String` | `ReedResult<ReedResponse<LayoutInfo>>` | `init.rs` |
| `reed init:layout name --preset preset` | - | `layouts::create_preset(req)` | `name: String, preset: String` | `ReedResult<ReedResponse<LayoutInfo>>` | `init.rs` |
| **Migration & Maintenance** | | | | | |
| `reed migrate:text path/` | - | `migration::text(req)` | `path: String` | `ReedResult<ReedResponse<MigrationResult>>` | `migrate.rs` |
| `reed validate:routes` | - | `validation::routes(req)` | - | `ReedResult<ReedResponse<ValidationReport>>` | `validate.rs` |
| `reed validate:consistency` | - | `validation::consistency(req)` | - | `ReedResult<ReedResponse<ValidationReport>>` | `validate.rs` |
| **Preset Management** | | | | | |
| `reed create:preset name --languages x,y` | - | `presets::create(req)` | `name: String, languages: Vec<String>` | `ReedResult<ReedResponse<PresetInfo>>` | `preset.rs` |
| `reed list:presets` | - | `presets::list(req)` | - | `ReedResult<ReedResponse<Vec<PresetInfo>>>` | `preset.rs` |
| **Build System** | | | | | |
| `reed build:kernel` | - | `build::kernel(req)` | - | `ReedResult<ReedResponse<BuildResult>>` | `build.rs` |
| `reed build:public` | - | `build::assets(req)` | - | `ReedResult<ReedResponse<BuildResult>>` | `build.rs` |
| `reed build:complete --debug "log.txt"` | - | `build::complete(req)` | `debug_log: Option<String>` | `ReedResult<ReedResponse<BuildResult>>` | `build.rs` |
| `reed build:watch` | - | `build::watch(req)` | - | `ReedResult<ReedResponse<WatchResult>>` | `build.rs` |
| **Server Management** | | | | | |
| `reed server:io --port 8333` | - | `server::start_http(req)` | `port: u16` | `ReedResult<ReedResponse<ServerInfo>>` | `server.rs` |
| `reed server:io --socket "path"` | - | `server::start_socket(req)` | `socket_path: String` | `ReedResult<ReedResponse<ServerInfo>>` | `server.rs` |
| `reed server:start --environment DEV` | - | `server::start(req)` | `environment: String` | `ReedResult<ReedResponse<ServerInfo>>` | `server.rs` |
| `reed server:stop` | - | `server::stop(req)` | - | `ReedResult<ReedResponse<()>>` | `server.rs` |
| `reed server:restart` | - | `server::restart(req)` | - | `ReedResult<ReedResponse<ServerInfo>>` | `server.rs` |
| `reed server:status` | - | `server::status(req)` | - | `ReedResult<ReedResponse<ServerStatus>>` | `server.rs` |
| `reed server:logs --tail 50` | - | `server::logs(req)` | `tail: Option<usize>` | `ReedResult<ReedResponse<Vec<LogEntry>>>` | `server.rs` |
| **Debug & Recovery** | | | | | |
| `reed debug:backup list text.csv` | - | `debug::backup::list(req)` | `csv_file: String` | `ReedResult<ReedResponse<Vec<BackupInfo>>>` | `backup.rs` |
| `reed debug:backup restore text.csv 3` | - | `debug::backup::restore(req)` | `csv_file: String, steps_back: u32` | `ReedResult<ReedResponse<()>>` | `backup.rs` |
| `reed debug:backup cleanup` | - | `debug::backup::cleanup(req)` | - | `ReedResult<ReedResponse<CleanupResult>>` | `backup.rs` |
| `reed debug:template knowledge.jinja --trace-vars` | - | `debug::template::trace(req)` | `template: String, trace_vars: bool` | `ReedResult<ReedResponse<TraceResult>>` | `template_trace.rs` |
| `reed debug:performance --profile-csv-lookups` | - | `debug::performance::profile(req)` | `profile_csv: bool` | `ReedResult<ReedResponse<ProfileResult>>` | `performance_profiler.rs` |
| **User Management** | | | | | |
| `reed user:create username --roles "role1,role2"` | - | `user::create(req)` | `username: String, roles: Vec<String>` | `ReedResult<ReedResponse<UserInfo>>` | `user.rs` |
| `reed user:list --format table` | - | `user::list(req)` | `format: OutputFormat` | `ReedResult<ReedResponse<Vec<UserInfo>>>` | `user.rs` |
| `reed user:show username` | - | `user::get(req)` | `username: String` | `ReedResult<ReedResponse<UserInfo>>` | `user.rs` |
| `reed user:update username --email "new@email.com"` | - | `user::update(req)` | `username: String, updates: UserUpdate` | `ReedResult<ReedResponse<UserInfo>>` | `user.rs` |
| `reed user:delete username --confirm` | - | `user::delete(req)` | `username: String, confirm: bool` | `ReedResult<ReedResponse<()>>` | `user.rs` |
| `reed user:passwd username` | - | `user::change_password(req)` | `username: String` | `ReedResult<ReedResponse<()>>` | `user.rs` |
| `reed user:roles username --add "editor"` | - | `user::manage_roles(req)` | `username: String, action: RoleAction` | `ReedResult<ReedResponse<UserInfo>>` | `user.rs` |
| `reed user:search --role "editor"` | - | `user::search(req)` | `criteria: UserSearchCriteria` | `ReedResult<ReedResponse<Vec<UserInfo>>>` | `user.rs` |
| **Role Management** | | | | | |
| `reed role:create rolename --permissions "text[rwx],route[rw-]"` | - | `role::create(req)` | `rolename: String, permissions: String` | `ReedResult<ReedResponse<RoleInfo>>` | `role.rs` |
| `reed role:list --show-permissions` | - | `role::list(req)` | `show_permissions: bool` | `ReedResult<ReedResponse<Vec<RoleInfo>>>` | `role.rs` |
| `reed role:show rolename` | - | `role::get(req)` | `rolename: String` | `ReedResult<ReedResponse<RoleInfo>>` | `role.rs` |
| `reed role:update rolename --inherit "parent_role"` | - | `role::update(req)` | `rolename: String, updates: RoleUpdate` | `ReedResult<ReedResponse<RoleInfo>>` | `role.rs` |
| `reed role:delete rolename --confirm` | - | `role::delete(req)` | `rolename: String, confirm: bool` | `ReedResult<ReedResponse<()>>` | `role.rs` |
| `reed role:permissions rolename --add "cms[rw-]"` | - | `role::manage_permissions(req)` | `rolename: String, action: PermissionAction` | `ReedResult<ReedResponse<RoleInfo>>` | `role.rs` |
| `reed role:users rolename` | - | `role::list_users(req)` | `rolename: String` | `ReedResult<ReedResponse<Vec<UserInfo>>>` | `role.rs` |
| **Taxonomy Management** | | | | | |
| `reed taxonomy:create "rust" --category tag --parent 1` | - | `taxonomy::create(req)` | `term: String, category: String, parent_id: Option<u32>` | `ReedResult<ReedResponse<TaxonomyTerm>>` | `taxonomy.rs` |
| `reed taxonomy:list --tree` | - | `taxonomy::list(req)` | `tree_view: bool` | `ReedResult<ReedResponse<Vec<TaxonomyTerm>>>` | `taxonomy.rs` |
| `reed taxonomy:show term_id` | - | `taxonomy::get(req)` | `term_id: u32` | `ReedResult<ReedResponse<TaxonomyTerm>>` | `taxonomy.rs` |
| `reed taxonomy:search "technology"` | - | `taxonomy::search(req)` | `query: String` | `ReedResult<ReedResponse<Vec<TaxonomyTerm>>>` | `taxonomy.rs` |
| `reed taxonomy:assign user:admin --terms "1,4,6"` | - | `taxonomy::assign(req)` | `entity: String, term_ids: Vec<u32>` | `ReedResult<ReedResponse<()>>` | `taxonomy.rs` |
| `reed taxonomy:unassign user:admin --terms "4"` | - | `taxonomy::unassign(req)` | `entity: String, term_ids: Vec<u32>` | `ReedResult<ReedResponse<()>>` | `taxonomy.rs` |
| `reed taxonomy:entities term_id` | - | `taxonomy::list_entities(req)` | `term_id: u32` | `ReedResult<ReedResponse<Vec<EntityInfo>>>` | `taxonomy.rs` |
| `reed taxonomy:usage --stats` | - | `taxonomy::usage_stats(req)` | `detailed: bool` | `ReedResult<ReedResponse<UsageStats>>` | `taxonomy.rs` |
| `reed taxonomy:update term_id --status "deprecated"` | - | `taxonomy::update(req)` | `term_id: u32, updates: TaxonomyUpdate` | `ReedResult<ReedResponse<TaxonomyTerm>>` | `taxonomy.rs` |
| `reed taxonomy:delete term_id --confirm` | - | `taxonomy::delete(req)` | `term_id: u32, confirm: bool` | `ReedResult<ReedResponse<()>>` | `taxonomy.rs` |

### Interactive Layout Creation

```bash
reed init:layout knowledge

🎯 Creating new ReedCMS layout: knowledge

📍 Routes Configuration:
? Which languages should this layout support?
  ✓ de (German)
  ✓ en (English)

? Route for German (de): wissen
? Route for English (en): knowledge

📱 Template Variants:
? Which interaction modes?
  ✓ mouse (Desktop/laptop users)
  ✓ touch (Mobile/tablet users)
  ✓ reader (Bots/screen readers)
```

### CLI Data Management Examples
```bash
# Set content via CLI
reed set:text knowledge.page.title@de "Wissen" --desc "Page title"
reed set:route knowledge@de "wissen" --desc "German route"
reed set:meta knowledge.cache.ttl "3600" --desc "Cache seconds"

# Get content
reed get:text knowledge.page.title@de
reed get:route knowledge@en
reed list:text knowledge.*
```

### Bulk Operations
```bash
# Multiple layouts
reed init:layout landing knowledge portfolio

# Hierarchical layouts
reed init:layout knowledge-term --parent knowledge

# Preset-based creation
reed init:layout api-docs --preset docs

# Migration operations
reed migrate:text organisms/page-header/
reed migrate:text layouts/knowledge/

# Bulk text operations
reed set:text page-header.* --interactive
reed set:text landing-*.title@de --interactive
```

### Real-World CMS Scenarios
```bash
# Seasonal content switching - Christmas activation
reed switch:theme christmas
# → 10 normale Seiten offline
# → 10 Weihnachts-Layouts online
# → Alle in einer Transaction

# Emergency maintenance mode
reed maintenance:activate --message "System update in progress"

# A/B testing layout switch
reed switch:layout knowledge --variant experimental

# Performance debugging
reed debug:template knowledge.jinja --trace-vars
reed debug:performance --profile-csv-lookups
```

### Generated File Structure
```
# CLI creates template files and updates central data:
templates/layouts/knowledge/
├── knowledge.mouse.jinja
├── knowledge.touch.jinja
├── knowledge.reader.jinja
├── knowledge.mouse.css
├── knowledge.touch.css
└── knowledge.reader.css

# Central data automatically updated:
.reed/
├── registry.csv         # Layout registered
├── routes.csv          # Routes added
├── text.csv            # Default text added
└── meta.csv            # Default meta added
```

### CLI File Organisation
**Rule**: One CLI command namespace = One file

```
/src/reedcms/cli/
├── init.rs          # reed init:layout commands
├── set.rs           # reed set:text, set:route, set:meta, set:server, set:project
├── get.rs           # reed get:text, get:route, get:server, get:project, list:*
├── migrate.rs       # reed migrate:text, migrate:routes, migrate:config
├── validate.rs      # reed validate:routes, validate:consistency
├── preset.rs        # reed create:preset, list:presets
├── build.rs         # reed build:kernel, build:public, build:complete, build:watch
└── server.rs        # reed server:io, server:start, server:stop, server:logs
```

### Universal Reed Binary Benefits
- ✅ **Single CLI**: All CMS + Build + Server management in one binary
- ✅ **Consistent Syntax**: Unified `:` notation across all commands
- ✅ **Integrated Workflow**: From content management to deployment
- ✅ **Self-Contained**: No external shell scripts required
- ✅ Clear namespace separation
- ✅ Maintainable code organisation
- ✅ Isolated testing per command type
- ✅ Clean Git diffs per feature area

---

## Template System Integration

### ReedBase - Central Data Aggregation Engine

**Purpose**: Single source aggregation for all .reed/ data files

**Core Structure** (minimal planning-level code):
```rust
// /src/reedcms/reedbase/get.rs
pub fn get_text(key: &str, lang: &str) -> ReedResult<String>
pub fn get_route(key: &str, lang: &str) -> ReedResult<String>
pub fn get_meta(key: &str) -> ReedResult<String>

// /src/reedcms/reedbase/set.rs
pub fn set_text(key: &str, value: &str, comment: &str) -> ReedResult<()>
pub fn set_route(key: &str, value: &str, comment: &str) -> ReedResult<()>
pub fn set_meta(key: &str, value: &str, comment: &str) -> ReedResult<()>
```

**Performance Strategy**:
- Startup-time loading of .reed/ files only
- Runtime O(1) HashMap lookups
- Type-separated indices for text/routes/meta

### Template Filter System

**Purpose**: Type-specific filters for clean template access

**Template Access**:
```jinja
{# Type-specific filters - clear intent and data source #}
{{ "knowledge.page.title" | text("de") }}     {# .reed/text.csv #}
{{ "knowledge" | route("en") }}               {# .reed/routes.csv #}
{{ "knowledge.cache.ttl" | meta }}            {# .reed/meta.csv #}

{# Environment overrides #}
{{ "knowledge@dev" | route("de") }}           {# Environment-specific routes #}
{{ "knowledge.hero.title" | text("auto") }}   {# Auto-detect current language #}

{# Configuration access #}
{{ "project.languages" | config }}   {# Available languages #}
{{ "project.index" | config }}       {# Landing page layout #}
{{ "server.auth.enabled" | config }}  {# Authentication status #}
```

**Filter Implementation** (structural overview):
```rust
// /src/reedcms/filters/text.rs
pub fn make_text_filter() -> FilterFunction

// /src/reedcms/filters/route.rs
pub fn make_route_filter() -> FilterFunction

// /src/reedcms/filters/meta.rs
pub fn make_meta_filter() -> FilterFunction

// /src/reedcms/filters/config.rs
pub fn make_config_filter() -> FilterFunction  {# Auto-detects project./server. prefix #}
```

### Universal CSV Handler
```rust
// /src/reedcms/csv/reader.rs
pub fn get(file_type: &str, key: &str) -> ReedResult<String>
pub fn set(file_type: &str, key: &str, value: &str, comment: &str) -> ReedResult<()>
```

### Data Flow
1. **Startup**: Load .reed/ CSV files only
2. **Aggregation**: ReedBase builds type-separated indices
3. **Runtime**: O(1) lookups via template filters
4. **Templates**: Access via type-specific filters (text, route, meta)
5. **CLI**: Direct .reed/ file updates with validation

### Template Filter Examples
```jinja
{# Text content access #}
{{ "knowledge.page.title" | text("de") }}      {# German page title #}
{{ "knowledge.hero.subtitle" | text("auto") }} {# Auto-detect language #}

{# Routing access #}
{{ "knowledge" | route("en") }}                 {# English route path #}
{{ "knowledge@dev" | route("de") }}             {# Development override #}

{# Metadata access #}
{{ "knowledge.cache.ttl" | meta }}              {# Cache configuration #}
{{ "knowledge.access.level" | meta }}           {# Access control #}

{# Configuration access #}
{{ "project.languages" | config }}     {# Available languages #}
{{ "project.index" | config }}         {# Landing page layout #}
{{ "server.auth.enabled" | config }}    {# Authentication status #}
```

---

## Performance & Technical Specifications

### Performance Characteristics
- **Startup**: < 50ms for initialisation (loads 5 CSV files + builds HashMap caches)
- **Runtime**: O(1) HashMap lookups for all data access
- **Get Operations**: p95 < 100μs (cached HashMap lookups)
- **Set Operations**: p95 < 10ms (CSV write + cache invalidation)
- **Cache Refresh**: < 50ms (complete cache rebuild from CSV files)
- **Memory**: Efficient CSV-to-HashMap transformation at startup
- **Scalability**: Tested up to 100+ layouts

### Bulk Operation Performance
```
Naive Implementation: 500× einzelne CSV-Opens = 2-3 seconds
ReedCMS Smart Batch: 1× Batch-Operation = 50ms
```

### Error Prevention Mechanisms
- **Single Source of Truth**: No data duplication between .reed/ and template files
- **Validation**: CLI validates consistency before operations
- **Type Safety**: Rust structs prevent invalid configurations
- **Conflict Detection**: Route collision prevention
- **Flow Control**: Only dispatchers may persist working data

### CSV Corruption Handling & Recovery
**Automatic Backup System** before any ReedBase manipulation:
```rust
// src/reedcms/reedbase/backup.rs
use std::time::{SystemTime, UNIX_EPOCH};
use xz2::write::XzEncoder;

// Before any CSV modification (cli, set, build, etc.)
fn create_backup(csv_path: &str) -> ReedResult<()> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    let filename = Path::new(csv_path).file_name().unwrap();
    let backup_path = format!(".reed/{}/{}.csv.xz", timestamp, filename.to_str().unwrap());

    // Compress and store original
    let original = std::fs::read(csv_path)?;
    let compressed = xz_compress(&original)?;
    std::fs::write(backup_path, compressed)?;

    // Cleanup: Keep only latest 32 backups
    cleanup_old_backups(".reed/", 32)?;
    Ok(())
}

// Recovery mechanism
fn restore_from_backup(csv_path: &str, steps_back: u32) -> ReedResult<()> {
    let backups = get_sorted_backups(csv_path)?;
    let backup_to_restore = backups.get(steps_back as usize)
        .ok_or(ReedError::BackupNotFound)?;

    let compressed = std::fs::read(backup_to_restore)?;
    let original = xz_decompress(&compressed)?;
    std::fs::write(csv_path, original)?;
    Ok(())
}
```

**Recovery Commands**:
```bash
reed debug:backup list text.csv        # Show available backups
reed debug:backup restore text.csv 3   # Restore 3 steps back
reed debug:backup cleanup              # Manual cleanup old backups
```

### Development Hot-Reload Configuration
**Template Auto-Reload** (available via `minijinja-autoreload` crate):
```rust
// src/reedcms/debug/hot_reload.rs
use minijinja_autoreload::AutoReloader;
use minijinja::{path_loader, Environment};

// Development mode with auto-reload
if std::env::var("ENVIRONMENT").unwrap_or_default() == "DEV" {
    let reloader = AutoReloader::new(|notifier| {
        let mut env = Environment::new();
        notifier.watch_path("templates/", true);
        env.set_loader(path_loader("templates/"));
        // Register custom ReedCMS filters
        env.add_filter("text", make_text_filter());
        env.add_filter("config", make_config_filter());
        Ok(env)
    });

    // Auto-reload on file changes
    let env = reloader.acquire_env()?;
} else {
    // Production: Static environment, no file watching
    let mut env = Environment::new();
    env.set_loader(path_loader("templates/"));
}
```

### Production Deployment Configuration
**Unix Socket Setup** (proven from legacy system):
```rust
// src/reedcms/server/socket.rs
use std::os::unix::net::UnixListener;
use std::os::unix::fs::PermissionsExt;

// Remove existing socket file
if Path::new(socket_path).exists() {
    std::fs::remove_file(socket_path)?;
}

// Bind and set permissions for nginx access
let listener = UnixListener::bind(socket_path)?;
std::fs::set_permissions(
    socket_path,
    std::fs::Permissions::from_mode(0o666)
)?;
```

### Quality Standards for Each Component
- **Mandatory**: File headers with AI coding guidelines
- **Mandatory**: Function documentation with Input/Output/Performance specs
- **Performance**: O(1) runtime lookups, startup-only file I/O
- **Architecture**: KISS principle, one function = one job
- **Testing**: Unit tests for each public function

---

## Implementation Guidelines

### File Naming Conventions

#### Template Files
```
{layout}.{variant}.{extension}
knowledge.mouse.jinja
knowledge.touch.css
knowledge.reader.jinja
```

#### Source Code Files
```
/src/reedcms/{scope}/{function}.rs

Examples:
/src/reedcms/reedbase/get.rs      # Get operations
/src/reedcms/reedbase/set.rs      # Set operations
/src/reedcms/reedbase/init.rs     # Initialisation
/src/reedcms/reedbase/debug.rs    # Debug utilities
/src/reedcms/cli/init.rs          # Layout creation
/src/reedcms/cli/validate.rs      # Validation commands
/src/reedcms/templates/context_builder.rs # Template context building
```

#### Scope Organisation
- `reedbase/` - Central data aggregation (get.rs, set.rs, init.rs)
- `cli/` - Command-line interface (init.rs, set.rs, get.rs, migrate.rs, validate.rs, preset.rs)
- `filters/` - Template filters (text.rs, route.rs, meta.rs)
- `csv/` - Universal CSV handler (reader.rs, writer.rs)
- `validation/` - Data validation and consistency
- `cache/` - Performance caching

### Asset Integration
ReedCMS provides enhanced asset bundling while maintaining existing CSS structure:

```rust
// /src/reedcms/assets/css_bundler.rs - reimplemented for ReedCMS
pub fn bundle_component_css(component: &str, mode: &str) -> String
pub fn bundle_layout_css(layout: &str, mode: &str) -> String
pub fn bundle_session_css(session_hash: &str) -> String
```

### Wildcard CLI Operations
Manage existing components efficiently:
```bash
# Migrate all component text to centralised system
reed migrate:text organisms/page-header/
reed migrate:text layouts/knowledge/

# Bulk operations on existing structure
reed set:text page-header.* --interactive
reed set:text landing-*.title@de --interactive
reed list:text knowledge.*                    # Show all knowledge-related text
```

---

## Migration & Integration

### Project Root Structure
```
project-root/
├── .reed/                   # ReedCMS central configuration and data
│   ├── registry.csv         # Layout registry and management
│   ├── presets.csv          # CLI presets for layout creation
│   ├── i18n.csv            # Activated languages per project
│   ├── a11y.csv            # Accessibility configuration
│   ├── text.csv            # ALL text content (centralised)
│   ├── routes.csv          # ALL routing definitions (centralised)
│   └── meta.csv            # ALL metadata (centralised)
├── templates/layouts/
│   ├── knowledge/
│   │   ├── knowledge.mouse.jinja
│   │   ├── knowledge.touch.jinja
│   │   └── knowledge.reader.jinja
```

### Compatible Architecture
ReedCMS integrates seamlessly with existing Atomic Design and interaction mode patterns:

```
templates/
├── components/
│   ├── atoms/icons/          # Individual icon components
│   ├── molecules/           # Component molecules with .text.csv
│   └── organisms/           # Complex components with .text.csv
└── layouts/                 # Page layouts with .text.csv
    ├── knowledge/           # Layout with sub-pages
    ├── landing/             # Landing page layout
    └── portfolio/           # Portfolio layout
```

### What Remains Unchanged
- ✅ **Atomic Design Pattern**: atoms/ molecules/ organisms/ layouts/
- ✅ **Interaction Modes**: .mouse.jinja, .touch.jinja, .reader.jinja
- ✅ **CSS Bundling**: .mouse.css, .touch.css, .reader.css per component
- ✅ **Template Files**: All .jinja files remain exactly as-is
- ✅ **Component Structure**: File naming and organisation unchanged

### Migration Path
**From distributed component text**:
```
organisms/page-header/page-header.text.csv
organisms/landing-hero/landing-hero.text.csv
layouts/knowledge/knowledge.text.csv
layouts/knowledge/actix-web/actix-web.text.csv
```

**To centralised ReedCMS data**:
```csv
# .reed/text.csv
key|value|comment
page-header.logo.title@de|vvoss|Page header logo title
page-header.logo.title@en|vvoss|Page header logo title
landing-hero.title@de|Willkommen|Landing hero title
landing-hero.subtitle@de|Digitale Lösungen|Landing hero subtitle
knowledge.page.title@de|Wissen|Knowledge main page title
actix-web.page.title@de|Actix-Web Framework|Knowledge sub-page title
```

### Backward Compatibility
- Templates continue using `{{ "key" | text("de") }}` syntax
- CSS bundling maintains per-component file structure
- Component discovery works with existing directory patterns
- No breaking changes to template rendering logic

### Design Decisions

#### Centralised vs Distributed
- **Chosen**: Centralised .reed/ files (CLI-first approach)
- **Rejected**: Distributed template-local files (manual editing complexity)
- **Benefit**: Atomic operations, better Git diffs, simpler backup
- **Trade-off**: Less proximity to templates (acceptable with CLI)

#### Centralised Architecture Benefits
- **All Data**: Centralised in `.reed/` with layout namespacing
- **Performance**: Fewer file operations, type-separated loading
- **Developer UX**: CLI-first with manual editing fallback
- **Git**: Clean diffs, no template directory pollution

#### CLI Philosophy
- **Interactive by default**: Explicit configuration over assumptions
- **Validation first**: Prevent conflicts before they occur
- **Developer-friendly**: Copy-paste workflows, preset systems

#### Template Hot-Reload Development System

**Development Mode Features**: Enhanced development experience with automatic template reloading

```rust
// Template hot-reload system for development
pub struct TemplateWatcher {
    watch_paths: Vec<PathBuf>,
    reload_callback: Box<dyn Fn(&str) -> ReedResult<()>>,
    debounce_duration: Duration,
}

impl TemplateWatcher {
    pub fn watch_templates(&self, base_path: &str) -> ReedResult<()> {
        // File system watcher for .jinja template changes
        // Automatic cache invalidation and template recompilation
        // Debounced reload to prevent excessive rebuilds
    }
}
```

**Hot-Reload Capabilities**:
- **File System Monitoring**: Automatic detection of template file changes
- **Intelligent Debouncing**: Prevents excessive reloads during rapid file changes
- **Cache Invalidation**: Automatic template cache clearing on file modifications
- **Dependency Tracking**: Reload parent templates when included components change
- **Error Recovery**: Graceful handling of template syntax errors during development
- **Live Browser Updates**: Integration with browser refresh for immediate visual feedback

#### Unix Socket Deployment Configuration

**Production Deployment Options**: Professional server deployment with Unix domain sockets

```bash
# Unix socket configuration for nginx integration
server {
    listen 80;
    server_name example.com;

    location /api/ {
        proxy_pass http://unix:/var/run/reedcms/api.sock;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location / {
        proxy_pass http://unix:/var/run/reedcms/web.sock;
        proxy_buffering off;
    }
}

# ReedCMS server configuration
# .reed/server.csv
service|bind_type|bind_address|socket_path|permissions|user|group
api|unix|/var/run/reedcms/api.sock||660|reedcms|www-data
web|unix|/var/run/reedcms/web.sock||660|reedcms|www-data
monitor|tcp|127.0.0.1:9090|||reedcms|reedcms
```

**Unix Socket Features**:
- **High Performance**: Faster than TCP for local communication
- **Security**: File system permissions control access
- **nginx Integration**: Direct proxy_pass to Unix sockets
- **Process Isolation**: Separate sockets for API and web services
- **Permission Management**: Fine-grained socket access control
- **Automatic Cleanup**: Socket file management on service restart



---

## Template System Integration Status

**Status**: ✅ COMPLETED (2025-01-30)  
**Result**: Existing templates 100% compatible with planned ReedCMS tickets  
**Tracking**: See `_workbench/Tickets/project_todo.md` for complete analysis

**All 9 Integration Questions Resolved**:
- ✅ **A**: Text filter `text('auto')` language resolution - URL path is source of truth
- ✅ **B**: Remove reed dictionary - use filter system consistently
- ✅ **B.1**: Route filter empty route handling - return empty string for invalid keys
- ✅ **C**: Component inclusion functions - organism(), molecule(), atom(), layout()
- ✅ **D**: CSS bundling strategy - session hash (MD5) with on-demand generation
- ✅ **E**: Client context population - screen_info cookie with ClientInfo structure (REED-06-05)
- ✅ **F**: Icon rendering - svg-icon molecule wrapper (Atomic Design compliant)
- ✅ **G**: Navigation management - Drupal-style taxonomy with Matrix Type 4 (REED-03-03)
- ✅ **H**: Text migration - direct 1:1 copy with full namespace keys (REED-04-07)

**Tickets Created/Extended**:
- 🆕 **REED-06-05**: Client Detection Service (new ticket)
- 📝 **REED-05-01**: Text/Route/Meta filter specifications
- 📝 **REED-05-02**: Custom functions (organism, molecule, atom, layout)
- 📝 **REED-05-03**: Context builder with ClientInfo and asset paths
- 📝 **REED-03-03**: Taxonomy system with Matrix Type 4 and template filter
- 📝 **REED-08-01**: CSS bundler with session hash strategy
- 📝 **REED-04-07**: Text migration command specification

**Key Architectural Decisions**:
- **Navigation**: Taxonomy-based (Drupal-style) with multiple menu locations
- **Icons**: Molecule wrapper pattern (svg-icon) for accessibility and flexibility
- **Assets**: Session hash bundling prevents cache issues, on-demand generation
- **Client Detection**: Server-side responsive rendering from screen_info cookie
- **Migration**: No auto-prefixing - keys already have full namespace

**Next Phase**: Begin systematic implementation starting with Foundation Layer (REED-01-01, REED-01-02).

---

## Related Documentation

- **Architectural Decisions**: See `project_optimisations.md` for complete decision history and system optimisations
- **Implementation Templates**: See `templates/service-template.md` and `templates/service-template.test.md`
- **Ticket Index**: See `ticket-index.csv` for complete ticket overview

---

**This comprehensive guide contains all ReedCMS planning information in a logical, didactic structure. Every concept builds upon previous sections, ensuring complete understanding from basic philosophy to detailed implementation specifications.**

This architecture ensures ReedCMS delivers on its promise of **simplicity without compromise** - providing enterprise-grade functionality through an elegantly simple CSV-based foundation.
