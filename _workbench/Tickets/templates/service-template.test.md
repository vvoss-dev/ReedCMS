# ReedCMS Service Test Implementation Template

> **Purpose**: Complete guide for separate test file structure and service interaction patterns.

## Table of Contents
1. [Test File Structure](#test-file-structure)
2. [Service Test Template](#service-test-template)
3. [Test Categories](#test-categories)
4. [Service Interaction Patterns](#service-interaction-patterns)
5. [Performance Testing](#performance-testing)
6. [Integration Testing](#integration-testing)

---

## Test File Structure

### MANDATORY: Separate Test Files

```
src/reedcms/reedbase/
├── set.rs              # Service implementation
├── set.test.rs         # Service tests
├── get.rs              # Service implementation
├── get.test.rs         # Service tests
└── validate.rs         # Service implementation
└── validate.test.rs    # Service tests
```

### File Naming Convention
- **Implementation**: `{service_name}.rs`
- **Tests**: `{service_name}.test.rs`
- **Never**: Inline `#[cfg(test)]` modules

---

## Service Test Template

### Complete Test File: `set.test.rs`

```rust
// Copyright (c) 2025 Vivian Voss. All rights reserved.
// ReedCMS - High-Performance Headless Rust CMS
//
// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One test = One assertion
// MANDATORY: BBC English for all test names and documentation
// MANDATORY: Test all error paths explicitly
// MANDATORY: Performance assertions for all operations
// MANDATORY: Cleanup after each test
//
// == FILE PURPOSE ==
// This file: Tests for set.rs text content setting operations exclusively
// Architecture: Separate test file following KISS principle
// Performance: All tests must complete within defined time limits
// Dependencies: tokio::test, tempfile for test isolation
// Test Scope: Unit tests, integration tests, performance tests

use tokio::test;
use tempfile::TempDir;
use std::time::Instant;

// Import the service being tested
use crate::reedcms::reedbase::set::{set_text, validate_text_input, resolve_environment_key};
use crate::reedcms::reedstream::{ReedRequest, ReedResponse, ReedResult, ReedError};

// == UNIT TESTS ==

#[test]
fn test_validate_text_input_success() {
    let req = create_valid_request();
    let result = validate_text_input(&req);
    assert!(result.is_ok());
}

#[test]
fn test_validate_text_input_empty_key() {
    let req = ReedRequest {
        key: "".to_string(),  // Invalid: empty key
        ..create_valid_request()
    };

    let result = validate_text_input(&req);
    assert!(matches!(result, Err(ReedError::InvalidKey { .. })));
}

#[test]
fn test_validate_text_input_invalid_key_characters() {
    let req = ReedRequest {
        key: "invalid key with spaces".to_string(),  // Invalid: contains spaces
        ..create_valid_request()
    };

    let result = validate_text_input(&req);
    assert!(matches!(result, Err(ReedError::InvalidKey { .. })));
}

#[test]
fn test_validate_text_input_invalid_language() {
    let req = ReedRequest {
        lang: Some("eng".to_string()),  // Invalid: 3 characters instead of 2
        ..create_valid_request()
    };

    let result = validate_text_input(&req);
    assert!(matches!(result, Err(ReedError::InvalidLanguage { .. })));
}

#[test]
fn test_validate_text_input_empty_value() {
    let req = ReedRequest {
        value: "".to_string(),  // Invalid: empty value
        ..create_valid_request()
    };

    let result = validate_text_input(&req);
    assert!(matches!(result, Err(ReedError::EmptyValue { .. })));
}

#[test]
fn test_validate_text_input_short_description() {
    let req = ReedRequest {
        description: "Short".to_string(),  // Invalid: less than 10 characters
        ..create_valid_request()
    };

    let result = validate_text_input(&req);
    assert!(matches!(result, Err(ReedError::MissingDescription { .. })));
}

#[test]
fn test_resolve_environment_key_with_dev_environment() {
    let result = resolve_environment_key("base.key", &Some("dev".to_string()));
    assert_eq!(result.unwrap(), "base.key@dev");
}

#[test]
fn test_resolve_environment_key_with_prod_environment() {
    let result = resolve_environment_key("base.key", &Some("prod".to_string()));
    assert_eq!(result.unwrap(), "base.key@prod");
}

#[test]
fn test_resolve_environment_key_no_environment() {
    let result = resolve_environment_key("base.key", &None);
    assert_eq!(result.unwrap(), "base.key");
}

#[test]
fn test_resolve_environment_key_existing_environment_consistent() {
    let result = resolve_environment_key("base.key@prod", &None);
    assert_eq!(result.unwrap(), "base.key@prod");
}

#[test]
fn test_resolve_environment_key_environment_mismatch() {
    let result = resolve_environment_key("base.key@dev", &Some("prod".to_string()));
    assert!(matches!(result, Err(ReedError::EnvironmentMismatch { .. })));
}

// == INTEGRATION TESTS ==

#[tokio::test]
async fn test_set_text_complete_workflow() {
    let _temp_dir = setup_test_environment().await;

    let req = create_valid_request();
    let result = set_text(req).await;

    assert!(result.is_ok());

    // Verify CSV file was created and contains expected data
    verify_csv_contains("test.key", "en", "Test Value", "Valid test description").await;

    // Verify cache was updated
    verify_cache_contains("test.key", "en", "Test Value").await;
}

#[tokio::test]
async fn test_set_text_overwrites_existing_entry() {
    let _temp_dir = setup_test_environment().await;

    // Set initial value
    let req1 = create_request_with_value("Initial Value");
    set_text(req1).await.unwrap();

    // Overwrite with new value
    let req2 = create_request_with_value("Updated Value");
    let result = set_text(req2).await;

    assert!(result.is_ok());
    verify_csv_contains("test.key", "en", "Updated Value", "Valid test description").await;
    verify_cache_contains("test.key", "en", "Updated Value").await;
}

#[tokio::test]
async fn test_set_text_multiple_languages() {
    let _temp_dir = setup_test_environment().await;

    // Set English version
    let req_en = create_request_with_language("en", "English Value");
    set_text(req_en).await.unwrap();

    // Set German version
    let req_de = create_request_with_language("de", "German Value");
    let result = set_text(req_de).await;

    assert!(result.is_ok());
    verify_csv_contains("test.key", "en", "English Value", "Valid test description").await;
    verify_csv_contains("test.key", "de", "German Value", "Valid test description").await;
}

#[tokio::test]
async fn test_set_text_with_environment_override() {
    let _temp_dir = setup_test_environment().await;

    let req = ReedRequest {
        environment: Some("dev".to_string()),
        ..create_valid_request()
    };

    let result = set_text(req).await;
    assert!(result.is_ok());

    // Verify environment-specific key was used
    verify_csv_contains("test.key@dev", "en", "Test Value", "Valid test description").await;
}

#[tokio::test]
async fn test_set_text_backup_created() {
    let _temp_dir = setup_test_environment().await;

    // Create initial CSV file
    create_initial_csv_file().await;

    let req = create_valid_request();
    let result = set_text(req).await;

    assert!(result.is_ok());

    // Verify backup was created
    verify_backup_exists().await;
}

// == ERROR HANDLING TESTS ==

#[tokio::test]
async fn test_set_text_validation_failure() {
    let req = ReedRequest {
        key: "".to_string(),  // Invalid key
        ..create_valid_request()
    };

    let result = set_text(req).await;
    assert!(matches!(result, Err(ReedError::InvalidKey { .. })));
}

#[tokio::test]
async fn test_set_text_csv_write_failure() {
    // Set up read-only directory to trigger write failure
    let temp_dir = setup_readonly_test_environment().await;

    let req = create_valid_request();
    let result = set_text(req).await;

    assert!(matches!(result, Err(ReedError::CsvWriteFailed { .. })));
}

// == PERFORMANCE TESTS ==

#[tokio::test]
async fn test_set_text_performance_under_7ms() {
    let _temp_dir = setup_test_environment().await;

    let req = create_valid_request();

    let start = Instant::now();
    let result = set_text(req).await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration.as_millis() < 7, "set_text took {}ms, expected <7ms", duration.as_millis());
}

#[tokio::test]
async fn test_validate_text_input_performance_under_1ms() {
    let req = create_valid_request();

    let start = Instant::now();
    let result = validate_text_input(&req);
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration.as_micros() < 1000, "validate_text_input took {}μs, expected <1000μs", duration.as_micros());
}

#[tokio::test]
async fn test_resolve_environment_key_performance_under_50_microseconds() {
    let start = Instant::now();
    let result = resolve_environment_key("test.key", &Some("dev".to_string()));
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration.as_micros() < 50, "resolve_environment_key took {}μs, expected <50μs", duration.as_micros());
}

// == STRESS TESTS ==

#[tokio::test]
async fn test_set_text_multiple_concurrent_operations() {
    let _temp_dir = setup_test_environment().await;

    let mut handles = Vec::new();

    // Launch 10 concurrent set_text operations
    for i in 0..10 {
        let req = create_request_with_key(&format!("concurrent.key.{}", i));
        let handle = tokio::spawn(async move {
            set_text(req).await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Verify all operations succeeded
    for result in results {
        assert!(result.unwrap().is_ok());
    }
}

// == TEST HELPERS ==

fn create_valid_request() -> ReedRequest {
    ReedRequest {
        key: "test.key".to_string(),
        lang: Some("en".to_string()),
        value: "Test Value".to_string(),
        description: "Valid test description to verify functionality properly".to_string(),
        environment: None,
    }
}

fn create_request_with_value(value: &str) -> ReedRequest {
    ReedRequest {
        value: value.to_string(),
        ..create_valid_request()
    }
}

fn create_request_with_language(lang: &str, value: &str) -> ReedRequest {
    ReedRequest {
        lang: Some(lang.to_string()),
        value: value.to_string(),
        ..create_valid_request()
    }
}

fn create_request_with_key(key: &str) -> ReedRequest {
    ReedRequest {
        key: key.to_string(),
        ..create_valid_request()
    }
}

async fn setup_test_environment() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create .reed directory structure
    let reed_dir = temp_dir.path().join(".reed");
    tokio::fs::create_dir_all(&reed_dir).await.unwrap();

    // Set environment variable to use test directory
    std::env::set_var("REED_DATA_DIR", temp_dir.path());

    temp_dir
}

async fn setup_readonly_test_environment() -> TempDir {
    let temp_dir = setup_test_environment().await;

    // Make directory read-only to trigger write failures
    let reed_dir = temp_dir.path().join(".reed");
    let mut perms = tokio::fs::metadata(&reed_dir).await.unwrap().permissions();
    perms.set_readonly(true);
    tokio::fs::set_permissions(&reed_dir, perms).await.unwrap();

    temp_dir
}

async fn create_initial_csv_file() {
    use tokio::fs;

    let csv_content = "key;language;value;description\nexisting.key;en;Existing Value;Test entry\n";
    fs::write(".reed/text.csv", csv_content).await.unwrap();
}

async fn verify_csv_contains(key: &str, lang: &str, value: &str, description: &str) {
    use tokio::fs;

    let content = fs::read_to_string(".reed/text.csv").await.unwrap();
    let expected_line = format!("{};{};{};{}", key, lang, value, description);
    assert!(content.contains(&expected_line),
        "CSV does not contain expected line: {}\nActual content:\n{}", expected_line, content);
}

async fn verify_cache_contains(key: &str, lang: &str, expected_value: &str) {
    use crate::reedcms::reedbase::text_cache::TEXT_CACHE;

    let cache = TEXT_CACHE.read().await;
    let value = cache.get(key)
        .and_then(|lang_map| lang_map.get(lang))
        .expect(&format!("Cache should contain key '{}' with language '{}'", key, lang));

    assert_eq!(value, expected_value);
}

async fn verify_backup_exists() {
    use tokio::fs;
    use std::path::Path;

    let reed_dir = Path::new(".reed");
    let mut entries = fs::read_dir(reed_dir).await.unwrap();

    let mut backup_found = false;
    while let Some(entry) = entries.next_entry().await.unwrap() {
        let path = entry.path();
        if path.is_dir() && path.file_name().unwrap().to_str().unwrap().chars().all(|c| c.is_ascii_digit()) {
            // Found timestamped backup directory
            let backup_file = path.join("text.csv.xz");
            if backup_file.exists() {
                backup_found = true;
                break;
            }
        }
    }

    assert!(backup_found, "No backup file found in .reed/ timestamped directories");
}
```

---

## Test Categories

### 1. Unit Tests
- **Input validation** (all error paths)
- **Key resolution** (environment handling)
- **Pure functions** (no side effects)

### 2. Integration Tests
- **Complete workflows** (end-to-end)
- **File system operations** (CSV read/write)
- **Cache interactions** (HashMap updates)

### 3. Error Handling Tests
- **Validation failures** (invalid input)
- **File system errors** (permission denied)
- **Environment mismatches** (conflicting settings)

### 4. Performance Tests
- **Timing assertions** (<7ms total, <1ms validation)
- **Memory usage** (no unnecessary allocations)
- **Concurrency** (multiple operations)

### 5. Stress Tests
- **Concurrent operations** (race conditions)
- **Large datasets** (performance degradation)
- **Resource exhaustion** (file handles, memory)

---

## Service Interaction Patterns

### Testing Service Dependencies

```rust
// Service A depends on Service B
use crate::reedcms::reedbase::get::get_text;
use crate::reedcms::reedbase::set::set_text;

#[tokio::test]
async fn test_set_then_get_consistency() {
    let _temp_dir = setup_test_environment().await;

    // Set a value
    let set_req = create_valid_request();
    set_text(set_req).await.unwrap();

    // Retrieve the same value
    let get_req = ReedRequest {
        key: "test.key".to_string(),
        lang: Some("en".to_string()),
        ..ReedRequest::default()
    };

    let result = get_text(get_req).await.unwrap();
    assert_eq!(result.data, "Test Value");
}
```

### Testing Dispatcher-Service Interaction

```rust
// Test that dispatcher calls service correctly
use crate::reedcms::reedbase::reedbase_dispatcher;

#[tokio::test]
async fn test_dispatcher_routes_to_set_service() {
    let _temp_dir = setup_test_environment().await;

    let request = ReedRequest {
        operation: "set_text".to_string(),
        key: "test.key".to_string(),
        lang: Some("en".to_string()),
        value: "Test Value".to_string(),
        description: "Test description".to_string(),
        environment: None,
    };

    let result = reedbase_dispatcher::handle_request(request).await;
    assert!(result.is_ok());

    // Verify service was called correctly
    verify_csv_contains("test.key", "en", "Test Value", "Test description").await;
}
```

---

## Performance Testing

### Timing Benchmarks

```rust
// Performance test template
#[tokio::test]
async fn test_operation_performance() {
    let iterations = 100;
    let mut total_duration = std::time::Duration::new(0, 0);

    for _ in 0..iterations {
        let start = Instant::now();

        // Operation under test
        let result = expensive_operation().await;

        let duration = start.elapsed();
        total_duration += duration;

        assert!(result.is_ok());
    }

    let average_duration = total_duration / iterations;
    assert!(average_duration.as_millis() < 5,
        "Average operation took {}ms, expected <5ms",
        average_duration.as_millis());
}
```

### Memory Usage Testing

```rust
#[tokio::test]
async fn test_memory_efficiency() {
    let initial_memory = get_memory_usage();

    // Perform memory-intensive operation
    let _result = process_large_dataset().await;

    let final_memory = get_memory_usage();
    let memory_increase = final_memory - initial_memory;

    // Should not increase memory by more than 10MB
    assert!(memory_increase < 10_000_000,
        "Memory increased by {} bytes, expected <10MB", memory_increase);
}
```

---

## Integration Testing

### Multi-Service Workflows

```rust
#[tokio::test]
async fn test_complete_content_lifecycle() {
    let _temp_dir = setup_test_environment().await;

    // 1. Create content
    let create_req = create_valid_request();
    set_text(create_req).await.unwrap();

    // 2. Retrieve content
    let get_req = create_get_request("test.key", "en");
    let retrieved = get_text(get_req).await.unwrap();
    assert_eq!(retrieved.data, "Test Value");

    // 3. Update content
    let update_req = create_request_with_value("Updated Value");
    set_text(update_req).await.unwrap();

    // 4. Verify update
    let final_get = create_get_request("test.key", "en");
    let final_result = get_text(final_get).await.unwrap();
    assert_eq!(final_result.data, "Updated Value");

    // 5. Verify backup system
    verify_backup_exists().await;
}
```

### Environment Override Testing

```rust
#[tokio::test]
async fn test_environment_isolation() {
    let _temp_dir = setup_test_environment().await;

    // Set production value
    let prod_req = ReedRequest {
        key: "config.title".to_string(),
        value: "Production Title".to_string(),
        environment: None, // Standard environment
        ..create_valid_request()
    };
    set_text(prod_req).await.unwrap();

    // Set development override
    let dev_req = ReedRequest {
        key: "config.title".to_string(),
        value: "Development Title".to_string(),
        environment: Some("dev".to_string()),
        ..create_valid_request()
    };
    set_text(dev_req).await.unwrap();

    // Verify both values exist independently
    verify_csv_contains("config.title", "en", "Production Title", "Valid test description").await;
    verify_csv_contains("config.title@dev", "en", "Development Title", "Valid test description").await;
}
```

---

**Remember**: Every service MUST have a corresponding `{name}.test.rs` file following this template exactly. Tests are not optional - they are mandatory documentation of expected behaviour and performance characteristics.