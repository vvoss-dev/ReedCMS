# REED-02-03: Environment Fallback System

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
- **ID**: REED-02-03
- **Title**: Environment Fallback System
- **Layer**: Data Layer (REED-02)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-02-01

## Summary Reference
- **Section**: Environment Override System
- **Lines**: 537-542 in project_summary.md
- **Key Concepts**: key@env fallback logic, seasonal themes

## Objective
Implement intelligent environment-aware key resolution with fallback from environment-specific to base keys.

## Requirements

### Fallback Logic
```
Lookup order:
1. key@environment (e.g., "title@dev", "title@christmas")
2. key (base key without environment)

Examples:
- "knowledge.title@dev" → try "knowledge.title@dev" → fallback to "knowledge.title"
- "knowledge.title@christmas" → try "@christmas" → fallback to base
```

### Implementation (`src/reedcms/reedbase/environment.rs`)

```rust
/// Resolves environment-specific key with fallback.
///
/// ## Input
/// - `key`: Base key or key with @environment suffix
/// - `environment`: Optional environment override
/// - `cache`: HashMap cache to lookup from
///
/// ## Output
/// - Resolved value from cache
/// - Falls back to base key if environment-specific not found
///
/// ## Performance
/// - O(1) lookup for environment key
/// - O(1) fallback lookup for base key
/// - Total: < 10μs
pub fn resolve_with_fallback(
    key: &str,
    environment: &Option<String>,
    cache: &HashMap<String, String>
) -> ReedResult<String>

/// Tests if key has environment suffix.
///
/// ## Examples
/// - "title@dev" → true
/// - "title" → false
pub fn has_environment_suffix(key: &str) -> bool

/// Extracts base key from environment-specific key.
///
/// ## Examples
/// - "knowledge.title@dev" → "knowledge.title"
/// - "knowledge.title" → "knowledge.title"
pub fn extract_base_key(key: &str) -> String

/// Validates environment name.
///
/// ## Valid Environments
/// - dev, prod, staging
/// - christmas, easter (seasonal themes)
/// - Custom environment names (alphanumeric + underscore)
pub fn validate_environment(env: &str) -> ReedResult<()>

/// Builds environment-specific key.
///
/// ## Example
/// - base: "title", env: "dev" → "title@dev"
pub fn build_env_key(base: &str, environment: &str) -> String
```

## Implementation Files

### Primary Implementation
- `src/reedcms/reedbase/environment.rs` - Environment resolution logic

### Test Files
- `src/reedcms/reedbase/environment.test.rs` - Comprehensive tests

## File Structure
```
src/reedcms/reedbase/
├── environment.rs      # Environment resolution
└── environment.test.rs # Tests
```

## Testing Requirements

### Unit Tests
- [ ] Test environment suffix detection
- [ ] Test base key extraction
- [ ] Test environment validation
- [ ] Test env key building
- [ ] Test fallback logic (env → base)

### Integration Tests
- [ ] Test with actual cache HashMap
- [ ] Test seasonal theme fallback (@christmas → base)
- [ ] Test multiple environment levels
- [ ] Test invalid environment names

### Performance Tests
- [ ] Environment resolution: < 10μs
- [ ] Base key extraction: < 1μs
- [ ] Fallback lookup: < 5μs

## Acceptance Criteria
- [ ] Fallback logic correct (@dev → base)
- [ ] Seasonal themes supported (@christmas, @easter)
- [ ] Cache lookups optimized
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-02-01 (ReedBase cache system)

## Blocks
- REED-04-02 (CLI needs environment-aware commands)
- REED-05-01 (Template filters need environment resolution)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 537-542 in `project_summary.md`

## Notes
The environment fallback system is critical for deployment flexibility. The @dev → @prod → base fallback chain allows developers to override production values for testing without modifying production data. Seasonal themes (@christmas, @easter) enable temporary content changes without data duplication.