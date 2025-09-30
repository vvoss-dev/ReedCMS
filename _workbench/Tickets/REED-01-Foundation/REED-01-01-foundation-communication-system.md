# REED-01-01: Foundation Communication System

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
- **ID**: REED-01-01
- **Title**: Foundation Communication System
- **Layer**: Foundation (REED-01)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: None (Foundation Layer)

## Summary Reference
- **Section**: Communication System (ReedStream)
- **Lines**: 589-731 in project_summary.md
- **Key Concepts**: Universal module communication interface

## Objective
Implement the ReedStream communication system as the universal interface for all ReedCMS modules. This is the foundation that all other components depend on.

## Requirements

### 1. Core Type Definitions
Implement the following types in `src/reedcms/reedstream.rs`:

#### ReedResult Type
```rust
pub type ReedResult<T> = Result<T, ReedError>;
```

#### ReedRequest Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedRequest {
    pub key: String,
    pub language: Option<String>,
    pub environment: Option<String>,
    pub context: Option<String>,
}
```

#### ReedResponse Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedResponse<T> {
    pub data: T,
    pub source: String,
    pub cached: bool,
    pub timestamp: u64,
    pub metrics: Option<ResponseMetrics>,
}
```

#### ResponseMetrics Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetrics {
    pub processing_time_us: u64,
    pub memory_allocated: Option<u64>,
    pub csv_files_accessed: u8,
    pub cache_info: Option<CacheInfo>,
}
```

### 2. ReedModule Trait
Implement standard module trait that ALL modules must implement:

```rust
pub trait ReedModule {
    fn module_name() -> &'static str;
    fn health_check() -> ReedResult<ReedResponse<String>>;
    fn version() -> &'static str { "1.0.0" }
    fn dependencies() -> Vec<&'static str> { Vec::new() }
}
```

### 3. Convenience Functions
Implement helper functions for common error creation:

```rust
pub fn not_found(resource: impl Into<String>) -> ReedError;
pub fn validation_error(field: impl Into<String>, value: impl Into<String>, constraint: impl Into<String>) -> ReedError;
pub fn csv_error(file_type: impl Into<String>, operation: impl Into<String>, reason: impl Into<String>) -> ReedError;
```

## Implementation Files

### Primary Implementation
- `src/reedcms/reedstream.rs` - Main module file

### Test Files
- `src/reedcms/reedstream.test.rs` - Comprehensive tests

## File Structure
```
src/reedcms/
├── reedstream.rs       # Main implementation
└── reedstream.test.rs  # Test suite
```

## Testing Requirements

### Unit Tests
- [ ] Test ReedRequest creation and field access
- [ ] Test ReedResponse creation with different data types
- [ ] Test ResponseMetrics calculation
- [ ] Test convenience functions (not_found, validation_error, csv_error)

### Integration Tests
- [ ] Test ReedModule trait implementation
- [ ] Test module health_check functionality
- [ ] Test version and dependencies reporting

### Performance Tests
- [ ] ReedRequest creation: < 1μs
- [ ] ReedResponse creation: < 1μs
- [ ] Metric calculation: < 10μs

## Standards Compliance

### Mandatory File Header
```rust
// Copyright (c) 2025 Vivian Voss. All rights reserved.
// ReedCMS - High-Performance Headless Rust CMS
//
// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One function = One job
// MANDATORY: BBC English for all documentation and comments
// MANDATORY: Type-safe HashMap lookups, O(1) performance priority
// MANDATORY: Environment-aware with @suffix support (key@dev, key@prod)
// MANDATORY: CSV format: key;value;description (semicolon-separated, quoted when needed)
// MANDATORY: Error handling with Result<T, ReedError> pattern
//
// == FILE PURPOSE ==
// This file: Universal communication interface for all ReedCMS modules
// Architecture: Foundation layer - no dependencies on other ReedCMS modules
// Performance: Zero-allocation type system, <1μs creation times
// Dependencies: serde for serialization, no ReedCMS dependencies
// Data Flow: All modules communicate through ReedRequest → ReedResponse pattern
```

### Documentation Standard
Every public function must follow the mandatory documentation template from service-template.md.

## Acceptance Criteria
- [ ] All type definitions compile without errors
- [ ] ReedModule trait can be implemented by test modules
- [ ] Convenience functions create correct error types
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks meet targets
- [ ] Documentation follows BBC English standards
- [ ] File header present and complete

## Dependencies
None - This is the foundation layer.

## Blocks
This ticket blocks:
- REED-01-02 (Error System needs ReedResult)
- REED-02-01 (ReedBase needs ReedRequest/ReedResponse)
- All other implementation tickets

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 589-731 in `project_summary.md`

## Notes
This is the most critical ticket in the entire project. All other modules depend on this communication interface. Ensure type definitions are stable before proceeding to other tickets.
