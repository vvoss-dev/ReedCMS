# REED-07-03: API Testing and Test Coverage

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function (CONFIG and SYSTEM use identical logic)
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Test Functions**: Move all test-only functions to `{name}.test.rs` - active code files must not contain test utilities
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs` - every topic (auth, login,...) has its own scoped rs service file
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-07-03
- **Title**: API Testing and Test Coverage
- **Layer**: API Layer (REED-07)
- **Priority**: High
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-07-01 (API must be implemented first)
- **Process Logs**: 251013-P03 (discovered missing tests during warning cleanup)

## Summary Reference
- **Section**: API Testing, Quality Assurance
- **Key Concepts**: Unit tests, integration tests, API endpoint testing, 100% coverage target

## Objective
Implement comprehensive test coverage for all API endpoints, handlers, and response types. Ensure all API functionality is properly tested with unit tests, integration tests, and end-to-end tests following ReedCMS testing standards.

## Problem Statement

During process 251013-P03 (API server integration), it was discovered that the entire API layer has **ZERO test coverage**:

### Missing Test Files
- ❌ `batch_handlers.test.rs` - Batch GET/SET operations untested
- ❌ `get_handlers.test.rs` - GET endpoints untested
- ❌ `set_handlers.test.rs` - SET endpoints untested
- ❌ `list_handlers.test.rs` - LIST operations untested
- ❌ `responses.test.rs` - Response types untested
- ❌ `routes.test.rs` - Route configuration untested

### Existing Test Files (Security Layer Only)
- ✅ `security/api_keys.test.rs` - API key management tested
- ✅ `security/matrix.test.rs` - Security matrix tested
- ✅ `security/rate_limit.test.rs` - Rate limiting tested

### Impact
- No verification that API endpoints work correctly
- No validation of request/response JSON formats
- No testing of error handling paths
- No integration tests for complete API workflows
- REED-07-01 marked "Complete" despite 0% handler test coverage

## Requirements

### Test Files to Create

#### 1. `batch_handlers.test.rs`
```rust
// Test batch GET operations
- test_batch_get_valid_keys()
- test_batch_get_empty_request()
- test_batch_get_exceeds_max_size()
- test_batch_get_invalid_cache_type()
- test_batch_get_with_language_env()

// Test batch SET operations
- test_batch_set_valid_items()
- test_batch_set_empty_request()
- test_batch_set_exceeds_max_size()
- test_batch_set_invalid_cache_type()
- test_batch_set_with_language_env()

// Test helper functions
- test_fetch_single_key()
- test_build_storage_key()
- test_set_single_key()
```

#### 2. `get_handlers.test.rs`
```rust
// Test GET endpoints
- test_get_text_valid_key()
- test_get_text_missing_key()
- test_get_route_valid()
- test_get_meta_valid()
- test_get_config_valid()

// Test language/environment handling
- test_get_with_language_suffix()
- test_get_with_environment_fallback()

// Test error cases
- test_get_invalid_cache_type()
- test_get_malformed_request()
```

#### 3. `set_handlers.test.rs`
```rust
// Test SET endpoints
- test_set_text_valid()
- test_set_route_valid()
- test_set_meta_valid()
- test_set_config_valid()

// Test language/environment handling
- test_set_with_language_suffix()
- test_set_with_environment()

// Test validation
- test_set_empty_key()
- test_set_empty_value()
- test_set_invalid_cache_type()

// Test CSV operations
- test_set_creates_backup()
- test_set_atomic_write()
```

#### 4. `list_handlers.test.rs`
```rust
// Test LIST operations
- test_list_text_keys()
- test_list_routes()
- test_list_layouts()

// Test filtering
- test_list_with_language_filter()
- test_list_with_pattern()

// Test error cases
- test_list_invalid_type()
```

#### 5. `responses.test.rs`
```rust
// Test response types
- test_api_response_success()
- test_api_response_error()
- test_api_batch_response()
- test_api_batch_result()

// Test JSON serialization
- test_response_json_format()
- test_error_json_format()
```

#### 6. `routes.test.rs`
```rust
// Test route configuration
- test_configure_api_routes()
- test_all_routes_require_auth()
- test_all_routes_have_security_middleware()

// Test route paths
- test_text_routes_configured()
- test_batch_routes_configured()
- test_list_routes_configured()
```

### Integration Tests

Create `tests/api_integration.rs`:
```rust
// End-to-end API tests
- test_complete_workflow_get_set()
- test_batch_operations_end_to_end()
- test_authentication_required()
- test_permission_checking()
- test_rate_limiting()
- test_concurrent_requests()
```

## Implementation Strategy

### Phase 1: Unit Tests for Handlers (Priority)
1. Create test files for each handler module
2. Implement tests for happy path scenarios
3. Add tests for error conditions
4. Test edge cases (empty inputs, max sizes, etc.)

### Phase 2: Response Type Tests
1. Test all response struct creation
2. Test JSON serialization/deserialization
3. Validate response formats match API documentation

### Phase 3: Route Configuration Tests
1. Test route registration
2. Verify middleware chain
3. Validate path mappings

### Phase 4: Integration Tests
1. Test complete API workflows
2. Test authentication integration
3. Test permission system integration
4. Test rate limiting integration

### Phase 5: Performance Tests
1. Benchmark GET operations (< 10ms target)
2. Benchmark SET operations (< 20ms target)
3. Benchmark batch operations (< 50ms for 100 items)
4. Load testing (1000 requests/sec)

## Testing Standards

### Test Structure (Per ReedCMS Standards)
- **Separate test files**: `{name}.test.rs` alongside `{name}.rs`
- **Test organization**: Arrange-Act-Assert pattern
- **Test naming**: `test_{function_name}_{scenario}()` format
- **Coverage target**: 100% line coverage for all handlers

### Mock Requirements
- Mock ReedBase services (get/set operations)
- Mock authentication/authorization
- Mock CSV file operations
- Use in-memory data for tests (no file I/O)

### Test Data
- Create test fixtures in `tests/fixtures/api/`
- Use realistic but minimal test data
- Include edge cases (empty, max size, unicode, special chars)

## Acceptance Criteria

### Unit Tests
- [x] batch_handlers.test.rs created with 100% coverage
- [ ] get_handlers.test.rs created with 100% coverage
- [ ] set_handlers.test.rs created with 100% coverage
- [ ] list_handlers.test.rs created with 100% coverage
- [ ] responses.test.rs created with 100% coverage
- [ ] routes.test.rs created with 100% coverage

### Integration Tests
- [ ] API workflow tests (auth → get → set)
- [ ] Batch operations end-to-end tests
- [ ] Authentication integration tests
- [ ] Permission checking tests
- [ ] Rate limiting tests

### Test Quality
- [ ] All tests pass with `cargo test`
- [ ] 100% line coverage for API handlers
- [ ] All edge cases covered
- [ ] Error paths tested
- [ ] Performance benchmarks met

### Documentation
- [ ] Test documentation in each test file
- [ ] Test fixtures documented
- [ ] Testing guide in README.md
- [ ] BBC English throughout

## Test Examples

### Example: batch_handlers.test.rs Structure
```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for batch operation handlers.

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    
    #[actix_web::test]
    async fn test_batch_get_valid_keys() {
        // Arrange
        let req = test::TestRequest::post()
            .uri("/api/v1/batch/get")
            .set_json(&BatchGetRequest {
                keys: vec!["page.title@en".to_string()],
                cache_type: "text".to_string(),
                language: Some("en".to_string()),
                environment: None,
            })
            .to_request();
        
        // Act
        let resp = batch_get(req, web::Json(body)).await;
        
        // Assert
        assert_eq!(resp.status(), 200);
        // ... more assertions
    }
    
    #[actix_web::test]
    async fn test_batch_get_exceeds_max_size() {
        // Test MAX_BATCH_SIZE limit
    }
    
    // ... more tests
}
```

## Dependencies
- **Requires**: REED-07-01 (API implementation must exist)
- **Blocks**: API can be considered production-ready only after tests pass
- **Related**: REED-90-03 (Warning cleanup - discovered this gap)

## Notes

### Why This Was Missed
- REED-07-01 was marked "Complete" in TICKET-STATUS.md without test verification
- Acceptance criteria "All tests pass with 100% coverage" was unchecked
- API handlers were implemented without corresponding test files
- Only security layer has tests (api_keys, matrix, rate_limit)

### Process Improvement
- This ticket was created during 251013-P03 process log
- Demonstrates value of systematic warning analysis
- Future tickets must verify test coverage before marking "Complete"

### Testing Priority
High priority because:
1. API endpoints are now connected to HttpServer (251013-P03)
2. API is accessible via HTTP (security risk without tests)
3. No validation that endpoints work correctly
4. Zero coverage is unacceptable for production code

## References
- Process Log: 251013-P03 (API server integration)
- Parent Ticket: REED-07-01 (API implementation)
- Testing Standards: `_workbench/Tickets/templates/service-template.test.md`
- ReedCMS Testing Guidelines: CLAUDE.md
