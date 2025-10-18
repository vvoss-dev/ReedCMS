# REED-19-08: Schema Validation

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-19-08
- **Title**: Schema Validation
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-19-02 (Universal Table API)
- **Estimated Time**: 4 days

## Objective

Implement comprehensive schema validation for ReedBase:
1. **Key validation** (RBKS v2) - Enforce key structure for reliable indices
2. **Column schemas** - Define and enforce column types and constraints

This ensures data quality and enables high-performance index-based queries.

## Requirements

### Part 1: Key Validation (RBKS v2)

#### ReedBase Key Specification v2 (RBKS v2)

**Canonical Format:**
```
<namespace>.<hierarchy>.<type>[<modifier,modifier,...>]
```

**Why Key Validation?**
- ✅ Enables reliable index-based queries (namespace, language, hierarchy)
- ✅ Guarantees O(1) lookups for common patterns (`page.%`, `%<de>`)
- ✅ Prevents developer errors at write-time
- ✅ Self-documenting keys with clear structure

#### Key Structure Rules

1. **Lowercase only**: `page.header.title` ✅, `Page.Header.Title` ❌
2. **Dots for hierarchy**: `page.header.title` ✅, `page-header-title` ❌
3. **Angle brackets for modifiers**: `<de,prod>` ✅, `@de#prod` ❌
4. **Modifiers comma-separated**: `<de,prod,christmas>` ✅, `<de prod>` ❌
5. **Modifiers order-independent**: `<de,prod>` = `<prod,de>` ✅
6. **No empty segments**: `page.title` ✅, `page..title` ❌
7. **No leading/trailing dots**: `page.title` ✅, `.page.title` ❌
8. **Depth 2-8 levels**: `page.title` (2) ✅, `a.b.c.d.e.f.g.h.i` (9) ❌
9. **No empty modifiers**: `<de>` ✅, `<>` ❌

#### Modifier Categories

**Language** (ISO 639-1, 2-letter):
- `de`, `en`, `fr`, `es`, `it`, `pt`, `nl`, `pl`, `ru`, `ja`, `zh`, `ar`
- Max 1 per key
- Example: `page.title<de>`

**Environment**:
- `dev`, `prod`, `staging`, `test`
- Max 1 per key
- Example: `page.title<prod>`

**Season**:
- `christmas`, `easter`, `summer`, `winter`
- Max 1 per key
- Example: `landing.hero<christmas>`

**Variant** (device type):
- `mobile`, `desktop`, `tablet`
- Max 1 per key
- Example: `page.header<mobile>`

**Custom**:
- Any lowercase identifier not in above categories
- Multiple allowed
- Example: `component.widget<custom1,custom2>`

#### Key Examples

```rust
// Valid keys
"page.title"                              // Base key
"page.title<de>"                          // German only
"page.title<prod>"                        // Production only
"page.title<de,prod>"                     // German + Production
"page.title<de,prod,christmas>"           // German + Prod + Christmas
"landing.hero<en,mobile,summer>"          // Multi-modifier
"blog.post.headline<fr,staging>"          // Complex hierarchy

// Invalid keys
"Page.Title"                              // ❌ Uppercase
"page-title"                              // ❌ Hyphen instead of dot
"page.title<DE>"                          // ❌ Uppercase modifier
"page.title<de prod>"                     // ❌ Space instead of comma
"page.title<>"                            // ❌ Empty modifiers
"page.title<de,>"                         // ❌ Trailing comma
"page..title"                             // ❌ Empty segment
".page.title"                             // ❌ Leading dot
"page.title<de,en>"                       // ❌ Multiple languages
"a.b.c.d.e.f.g.h.i"                      // ❌ Too deep (9 levels)
```

#### Regex Pattern

```rust
pub const RBKS_V2_PATTERN: &str = 
    r"^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*){1,7}(<[a-z]+(,[a-z]+)*>)?$";
```

#### Modifier Classification

```rust
/// Parsed modifiers from <lang,env,...> syntax.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Modifiers {
    pub language: Option<String>,       // Some("de")
    pub environment: Option<String>,    // Some("prod")
    pub season: Option<String>,         // Some("christmas")
    pub variant: Option<String>,        // Some("mobile")
    pub custom: Vec<String>,            // ["other", "modifiers"]
}

/// Classify raw modifiers into categories.
///
/// ## Rules
/// - Language: 2-letter ISO 639-1 codes (max 1)
/// - Environment: dev/prod/staging/test (max 1)
/// - Season: christmas/easter/summer/winter (max 1)
/// - Variant: mobile/desktop/tablet (max 1)
/// - Custom: anything else (multiple allowed)
///
/// ## Error Conditions
/// - Multiple languages: "Multiple languages not allowed"
/// - Multiple environments: "Multiple environments not allowed"
/// - Multiple seasons: "Multiple seasons not allowed"
/// - Multiple variants: "Multiple variants not allowed"
fn classify_modifiers(raw: &[String]) -> ReedResult<Modifiers>;
```

#### Fallback Chain Resolution

```rust
/// Fallback chain for modifier resolution.
///
/// ## Priority (highest to lowest)
/// 1. Exact match: page.title<de,prod,christmas>
/// 2. Without season: page.title<de,prod>
/// 3. Without environment: page.title<de,christmas>
/// 4. Language only: page.title<de>
/// 5. Environment + season: page.title<prod,christmas>
/// 6. Environment only: page.title<prod>
/// 7. Season only: page.title<christmas>
/// 8. Base key: page.title
///
/// ## Performance
/// - Max 8 lookups (power set of 3 modifier types)
/// - < 100μs typical (early exit on match)
///
/// ## Example
/// ```rust
/// // Query: get("page.title<de,prod,christmas>")
/// // Tries in order:
/// get_exact("page.title<de,prod,christmas>")      // Full match
///   .or_else(|| get_exact("page.title<de,prod>"))  // Without season
///   .or_else(|| get_exact("page.title<de,christmas>")) // Without env
///   .or_else(|| get_exact("page.title<de>"))       // Language only
///   .or_else(|| get_exact("page.title<prod,christmas>")) // Without lang
///   .or_else(|| get_exact("page.title<prod>"))     // Env only
///   .or_else(|| get_exact("page.title<christmas>")) // Season only
///   .or_else(|| get_exact("page.title"))           // Base fallback
/// ```
pub fn get_with_fallback(
    base_key: &str,
    modifiers: &Modifiers,
) -> ReedResult<Option<String>>;
```

#### Normalization

```rust
/// Normalize key to canonical format.
///
/// ## Operations
/// - Convert to lowercase
/// - Sort modifiers alphabetically
/// - Remove duplicate modifiers
/// - Trim whitespace
/// - Remove duplicate dots
/// - Remove leading/trailing dots
///
/// ## Performance
/// - O(n + m log m) where n = key length, m = modifiers count
/// - < 15μs typical
///
/// ## Example
/// ```rust
/// let normalized = normalize_key("Page.Header..Title<PROD,DE,prod>")?;
/// assert_eq!(normalized, "page.header.title<de,prod>");
/// ```
pub fn normalize_key(raw: &str) -> ReedResult<String>;
```

#### CLI Integration

```bash
# Set with validation
reed set:text page.title<de,prod> "Titel"
# ✅ Valid - accepted

reed set:text "Page.Title<DE>" "Test"
# ❌ Error: Key validation failed: Must be lowercase
# Hint: Did you mean "page.title<de>"?

reed set:text page-title<de> "Test"
# ❌ Error: Key validation failed: Use dots (.) for hierarchy
# Hint: Did you mean "page.title<de>"?

reed set:text page.title<> "Test"
# ❌ Error: Key validation failed: Empty modifiers <> not allowed

reed set:text page.title<de,en> "Test"
# ❌ Error: Key validation failed: Multiple languages not allowed

# Auto-normalize (optional flag)
reed set:text "Page.Title<PROD,DE>" "Test" --normalize
# ⚠️  Warning: Key normalized: page.title<de,prod>
# ✅ Set: page.title<de,prod> = "Test"

# Query with modifiers (uses indices!)
reed query text "SELECT * WHERE key LIKE 'page.%<de>'"
# → Uses NamespaceIndex + LanguageIndex → O(1) lookup!
```

#### Performance Requirements (Key Validation)

| Operation | Target | Notes |
|-----------|--------|-------|
| Parse key | < 15μs | Regex + modifier parsing |
| Validate key | < 20μs | Pattern match + classification |
| Normalize key | < 15μs | Lowercase + sort + dedup |
| Classify modifiers | < 10μs | Category matching |
| Generate fallback chain | < 50μs | Power set generation |
| **Total SET overhead** | **< 30μs** | +20% vs no validation |

---


## File Structure

```
reedbase/src/
├── schema/
│   ├── mod.rs              # Public API
│   ├── rbks.rs             # RBKS v2 key validation
│   └── rbks.test.rs        # Key validation tests
```

## Dependencies

**Internal**:
- `reedstream::ReedError` - Error handling

**External**:
- `regex` - Key pattern matching

## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| key_validation_latency | Histogram | μs | <20 | >100 | rbks.rs:validate_key() |
| key_parse_latency | Histogram | μs | <15 | >80 | rbks.rs:parse_key() |
| key_normalize_latency | Histogram | μs | <15 | >80 | rbks.rs:normalize_key() |
| validation_error_rate | Gauge | % | <5 | >20 | rbks.rs:validate_key() |
| invalid_modifier_count | Counter | count | <1% | >10% | rbks.rs:parse_modifiers() |

### Alert Rules

**CRITICAL Alerts:**
- `validation_error_rate > 20%` for 10 minutes → "High key validation failure rate - check key formats"
- `key_validation_latency p99 > 100μs` for 5 minutes → "Key validation critically slow"

**WARNING Alerts:**
- `validation_error_rate > 5%` for 15 minutes → "Elevated validation errors - review key generation"
- `invalid_modifier_count > 10%` for 10 minutes → "High invalid modifier rate - check modifier usage"

### Implementation

```rust
use crate::reedbase::metrics::global as metrics;
use std::time::Instant;

pub fn validate_key(key: &str) -> ReedResult<()> {
    let start = Instant::now();
    let result = validate_key_inner(key);
    
    metrics().record(Metric {
        name: "key_validation_latency".to_string(),
        value: start.elapsed().as_nanos() as f64 / 1000.0, // Convert to μs
        unit: MetricUnit::Microseconds,
        tags: hashmap!{ "valid" => result.is_ok().to_string() },
    });
    
    if result.is_err() {
        metrics().record(Metric {
            name: "validation_error_rate".to_string(),
            value: 1.0,
            unit: MetricUnit::Percent,
            tags: hashmap!{ "key" => key },
        });
    }
    
    result
}

pub fn parse_key(key: &str) -> ReedResult<ParsedKey> {
    let start = Instant::now();
    let parsed = parse_key_inner(key)?;
    
    metrics().record(Metric {
        name: "key_parse_latency".to_string(),
        value: start.elapsed().as_nanos() as f64 / 1000.0, // Convert to μs
        unit: MetricUnit::Microseconds,
        tags: hashmap!{ "depth" => parsed.depth().to_string() },
    });
    
    Ok(parsed)
}
```

### Collection Strategy

- **Sampling**: All operations
- **Aggregation**: 1-minute rolling window
- **Storage**: `.reedbase/metrics/schema.csv`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**key_validation_latency**: Write path performance
- Key validation happens on EVERY write operation
- Sub-microsecond performance critical for high throughput
- Slow validation directly impacts write latency

**validation_error_rate**: Data quality indicator
- Low error rate (<5%) = good key hygiene
- High rates indicate bugs in key generation or user errors
- Helps identify problematic code paths

**invalid_modifier_count**: Modifier usage health
- Tracks modifier parsing failures
- High rates indicate misuse of modifier syntax
- Guides documentation and error message improvements

**key_parse_latency**: Index performance
- Parsed keys used for index lookups
- Fast parsing critical for query performance
- Affects all indexed operations

## Acceptance Criteria

### Functional Requirements
- [x] Parse key with RBKS v2 regex pattern
- [x] Validate key structure (lowercase, dots, modifiers)
- [x] Parse modifiers from `<modifier,modifier>` syntax
- [x] Classify modifiers into categories (language, environment, season, variant, custom)
- [x] Reject multiple modifiers of same category (e.g., `<de,en>`)
- [x] Normalize keys (lowercase, sort modifiers, dedup)
- [x] Generate fallback chain for modifier resolution
- [x] Validate depth (2-8 levels)
- [x] Validate no empty segments or trailing commas
- [x] CLI `--normalize` flag for auto-normalization
- [x] Helpful error messages with suggestions ("Did you mean...?")

### Performance Requirements
- [x] Parse key: < 15μs
- [x] Validate key: < 20μs
- [x] Normalize key: < 15μs
- [x] Classify modifiers: < 10μs
- [x] Generate fallback chain: < 50μs
- [x] **Total SET overhead: < 30μs** (+20% vs no validation)

### Quality Requirements
- [x] 100% test coverage for RBKS v2 validation
- [x] Performance benchmarks for all operations
- [x] Integration tests with real keys
- [x] Separate test file: `rbks.test.rs`

### Documentation Requirements
- [x] Architecture documentation (this ticket)
- [x] API documentation for all validation functions
- [x] RBKS v2 specification document
- [x] CLI usage examples
- [x] Fallback chain algorithm documentation

## Implementation Notes

### RBKS v2 Philosophy

**Why Key Validation?**
- **Enables Indices**: Structured keys = O(1) lookups via NamespaceIndex, LanguageIndex (REED-19-11)
- **Self-Documenting**: `page.header.title<de,prod>` explains itself
- **Prevents Chaos**: Without validation, keys become inconsistent over time
- **Performance**: 100-1000x faster queries via index-based lookups

**Key Design Decisions:**
- **Angle-bracket modifiers `<>`**: Cleaner than `@#`, order-independent
- **Comma-separated**: `<de,prod>` is clear, no ambiguity
- **Lowercase enforced**: Consistency, no case-sensitivity issues
- **Dots for hierarchy**: Standard practice, works with existing tooling
- **Depth 2-8**: Optimal for readability and performance

**Validation Strategy:**
- **Strict on write**: Reject invalid keys immediately
- **Normalization available**: `--normalize` flag auto-fixes common mistakes
- **Helpful errors**: "Did you mean...?" suggestions
- **< 30μs overhead**: Minimal performance impact

**Fallback Chain Benefits:**
- **Graceful degradation**: Falls back from specific to general
- **Environment-agnostic**: Same content, different environments
- **Seasonal themes**: Christmas theme falls back to default
- **Early exit**: Stops at first match (< 100μs typical)

### Integration with Smart Indices (REED-19-11)

**Key Validation enables:**
- **NamespaceIndex**: `page.%` → O(1) lookup via namespace="page"
- **LanguageIndex**: `%<de>` → O(1) lookup via language="de"
- **EnvironmentIndex**: `%<prod>` → O(1) lookup via environment="prod"
- **HierarchyTrie**: `page.header.%` → O(d) trie walk where d=depth

**Without key validation:**
- ❌ Index lookups unreliable (keys inconsistent)
- ❌ Queries fall back to O(n) full scans
- ❌ Performance degrades over time

### Future Enhancements

1. **Namespace restrictions**
   - Optional: Restrict allowed namespaces in schema
   - Example: Only allow `page`, `blog`, `landing`, `api`

2. **Custom modifier categories**
   - User-defined categories beyond language/environment/season/variant
   - Example: `region` (eu, us, asia), `brand` (premium, standard)

3. **Key templates**
   - Predefined patterns: `blog.post.{slug}.{field}<{lang}>`
   - Automatic validation of pattern compliance

4. **Key migration tools**
   - Batch normalize existing keys
   - Detect and fix inconsistencies

## References

- **REED-19-02**: Universal Table API (integration point)
- **REED-19-11**: Smart Indices (uses RBKS v2 for O(1) lookups)
- **REED-19-12**: ReedQL (uses RBKS v2 for query optimization)
- Service Template: `_workbench/Tickets/templates/service-template.md`

## Summary

RBKS v2 Key Validation provides **structured key format enforcement** for ReedBase. Keys follow the format `namespace.hierarchy<language,environment,season,variant>` with strict validation rules (lowercase, dots, angle-brackets, 2-8 depth). This enables **O(1) index-based queries** (REED-19-11) via NamespaceIndex, LanguageIndex, EnvironmentIndex, and HierarchyTrie. Validation adds **<30μs overhead** while unlocking **100-1000x query speedup** through Smart Indices.
