// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0
//
// Test file for: src/reedcms/reedstream.rs

use crate::reedcms::reedstream::*;

// == UNIT TESTS: ReedRequest ==

#[test]
fn test_reed_request_creation() {
    let req = ReedRequest {
        key: "test.key".to_string(),
        language: Some("en".to_string()),
        environment: None,
        context: None,
        value: Some("test value".to_string()),
        description: Some("test description".to_string()),
    };

    assert_eq!(req.key, "test.key");
    assert_eq!(req.language, Some("en".to_string()));
    assert_eq!(req.value, Some("test value".to_string()));
}

#[test]
fn test_reed_request_minimal() {
    let req = ReedRequest {
        key: "minimal.key".to_string(),
        language: None,
        environment: None,
        context: None,
        value: None,
        description: None,
    };

    assert_eq!(req.key, "minimal.key");
    assert!(req.language.is_none());
    assert!(req.value.is_none());
}

// == UNIT TESTS: ReedResponse ==

#[test]
fn test_reed_response_new() {
    let response = ReedResponse::new("test data", "test_module");

    assert_eq!(response.data, "test data");
    assert_eq!(response.source, "test_module");
    assert!(!response.cached);
    assert!(response.timestamp > 0);
    assert!(response.metrics.is_none());
}

#[test]
fn test_reed_response_with_metrics() {
    let metrics = ResponseMetrics {
        processing_time_us: 150,
        memory_allocated: Some(1024),
        csv_files_accessed: 2,
        cache_info: None,
    };

    let response = ReedResponse::new(42, "test_module").with_metrics(metrics.clone());

    assert_eq!(response.data, 42);
    assert!(response.metrics.is_some());
    assert_eq!(response.metrics.unwrap().processing_time_us, 150);
}

#[test]
fn test_reed_response_with_different_types() {
    let str_response = ReedResponse::new("string", "module");
    let int_response = ReedResponse::new(123, "module");
    let bool_response = ReedResponse::new(true, "module");

    assert_eq!(str_response.data, "string");
    assert_eq!(int_response.data, 123);
    assert_eq!(bool_response.data, true);
}

// == UNIT TESTS: ResponseMetrics ==

#[test]
fn test_response_metrics_creation() {
    let cache_info = CacheInfo {
        hit: true,
        ttl_remaining_s: Some(3600),
        cache_key: "test.key".to_string(),
        cache_layer: "text".to_string(),
    };

    let metrics = ResponseMetrics {
        processing_time_us: 100,
        memory_allocated: None,
        csv_files_accessed: 1,
        cache_info: Some(cache_info),
    };

    assert_eq!(metrics.processing_time_us, 100);
    assert_eq!(metrics.csv_files_accessed, 1);
    assert!(metrics.cache_info.is_some());
}

// == UNIT TESTS: CacheInfo ==

#[test]
fn test_cache_info_hit() {
    let cache_info = CacheInfo {
        hit: true,
        ttl_remaining_s: Some(1800),
        cache_key: "knowledge.title".to_string(),
        cache_layer: "text".to_string(),
    };

    assert!(cache_info.hit);
    assert_eq!(cache_info.ttl_remaining_s, Some(1800));
    assert_eq!(cache_info.cache_layer, "text");
}

#[test]
fn test_cache_info_miss() {
    let cache_info = CacheInfo {
        hit: false,
        ttl_remaining_s: None,
        cache_key: "missing.key".to_string(),
        cache_layer: "route".to_string(),
    };

    assert!(!cache_info.hit);
    assert!(cache_info.ttl_remaining_s.is_none());
}

// == UNIT TESTS: ReedError ==

#[test]
fn test_reed_error_not_found() {
    let err = ReedError::NotFound {
        resource: "test.key".to_string(),
        context: Some("CSV lookup".to_string()),
    };

    match err {
        ReedError::NotFound { resource, context } => {
            assert_eq!(resource, "test.key");
            assert_eq!(context, Some("CSV lookup".to_string()));
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_reed_error_validation() {
    let err = ReedError::ValidationError {
        field: "key".to_string(),
        value: "invalid key".to_string(),
        constraint: "No spaces allowed".to_string(),
    };

    match err {
        ReedError::ValidationError {
            field,
            value,
            constraint,
        } => {
            assert_eq!(field, "key");
            assert_eq!(value, "invalid key");
            assert_eq!(constraint, "No spaces allowed");
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_reed_error_csv() {
    let err = ReedError::CsvError {
        file_type: "text".to_string(),
        operation: "read".to_string(),
        reason: "File not found".to_string(),
    };

    match err {
        ReedError::CsvError {
            file_type,
            operation,
            reason,
        } => {
            assert_eq!(file_type, "text");
            assert_eq!(operation, "read");
            assert_eq!(reason, "File not found");
        }
        _ => panic!("Wrong error variant"),
    }
}

// == UNIT TESTS: Convenience Functions ==

#[test]
fn test_not_found_convenience() {
    let err = not_found("missing.key");

    match err {
        ReedError::NotFound { resource, context } => {
            assert_eq!(resource, "missing.key");
            assert!(context.is_none());
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_validation_error_convenience() {
    let err = validation_error("email", "invalid", "Must contain @");

    match err {
        ReedError::ValidationError {
            field,
            value,
            constraint,
        } => {
            assert_eq!(field, "email");
            assert_eq!(value, "invalid");
            assert_eq!(constraint, "Must contain @");
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_csv_error_convenience() {
    let err = csv_error("text", "write", "Permission denied");

    match err {
        ReedError::CsvError {
            file_type,
            operation,
            reason,
        } => {
            assert_eq!(file_type, "text");
            assert_eq!(operation, "write");
            assert_eq!(reason, "Permission denied");
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_io_error_convenience() {
    let err = io_error("read", "/tmp/test.csv", "File not found");

    match err {
        ReedError::IoError {
            operation,
            path,
            reason,
        } => {
            assert_eq!(operation, "read");
            assert_eq!(path, "/tmp/test.csv");
            assert_eq!(reason, "File not found");
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_auth_error_convenience() {
    let err = auth_error(Some("admin".to_string()), "login", "Invalid password");

    match err {
        ReedError::AuthError {
            user,
            action,
            reason,
        } => {
            assert_eq!(user, Some("admin".to_string()));
            assert_eq!(action, "login");
            assert_eq!(reason, "Invalid password");
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_config_error_convenience() {
    let err = config_error("server", "Missing port configuration");

    match err {
        ReedError::ConfigError { component, reason } => {
            assert_eq!(component, "server");
            assert_eq!(reason, "Missing port configuration");
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_template_error_convenience() {
    let err = template_error("page.jinja", "Template not found");

    match err {
        ReedError::TemplateError { template, reason } => {
            assert_eq!(template, "page.jinja");
            assert_eq!(reason, "Template not found");
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_server_error_convenience() {
    let err = server_error("actix", "Port already in use");

    match err {
        ReedError::ServerError { component, reason } => {
            assert_eq!(component, "actix");
            assert_eq!(reason, "Port already in use");
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_invalid_command_convenience() {
    let err = invalid_command("get:invalid", "Unknown command");

    match err {
        ReedError::InvalidCommand { command, reason } => {
            assert_eq!(command, "get:invalid");
            assert_eq!(reason, "Unknown command");
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_parse_error_convenience() {
    let err = parse_error("malformed json", "Unexpected token");

    match err {
        ReedError::ParseError { input, reason } => {
            assert_eq!(input, "malformed json");
            assert_eq!(reason, "Unexpected token");
        }
        _ => panic!("Wrong error variant"),
    }
}

// == UNIT TESTS: Helper Functions ==

#[test]
fn test_current_timestamp() {
    let ts1 = current_timestamp();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let ts2 = current_timestamp();

    assert!(ts1 > 0);
    assert!(ts2 >= ts1);
}

#[test]
fn test_current_timestamp_performance() {
    use std::time::Instant;

    let start = Instant::now();
    for _ in 0..1000 {
        let _ = current_timestamp();
    }
    let duration = start.elapsed();

    // 1000 calls should complete in < 1ms
    assert!(duration.as_millis() < 1);
}

// == INTEGRATION TESTS: ReedModule Trait ==

struct TestModule;

impl ReedModule for TestModule {
    fn module_name() -> &'static str {
        "test_module"
    }

    fn health_check() -> ReedResult<ReedResponse<String>> {
        Ok(ReedResponse::new("healthy".to_string(), "test_module"))
    }

    fn version() -> &'static str {
        "1.2.3"
    }

    fn dependencies() -> Vec<&'static str> {
        vec!["dep1", "dep2"]
    }
}

#[test]
fn test_reed_module_implementation() {
    assert_eq!(TestModule::module_name(), "test_module");
    assert_eq!(TestModule::version(), "1.2.3");
    assert_eq!(TestModule::dependencies(), vec!["dep1", "dep2"]);

    let health = TestModule::health_check().unwrap();
    assert_eq!(health.data, "healthy");
    assert_eq!(health.source, "test_module");
}

// == PERFORMANCE TESTS ==

#[test]
fn test_reed_request_creation_performance() {
    use std::time::Instant;

    let start = Instant::now();
    for _ in 0..10000 {
        let _ = ReedRequest {
            key: "test.key".to_string(),
            language: Some("en".to_string()),
            environment: None,
            context: None,
            value: None,
            description: None,
        };
    }
    let duration = start.elapsed();

    // 10000 creations should complete in < 10ms (< 1μs per creation)
    assert!(duration.as_millis() < 10);
}

#[test]
fn test_reed_response_creation_performance() {
    use std::time::Instant;

    let start = Instant::now();
    for _ in 0..10000 {
        let _ = ReedResponse::new("data", "module");
    }
    let duration = start.elapsed();

    // 10000 creations should complete in < 10ms (< 1μs per creation)
    assert!(duration.as_millis() < 10);
}

#[test]
fn test_response_metrics_calculation_performance() {
    use std::time::Instant;

    let start = Instant::now();
    for _ in 0..10000 {
        let metrics = ResponseMetrics {
            processing_time_us: 100,
            memory_allocated: Some(1024),
            csv_files_accessed: 1,
            cache_info: None,
        };
        let _ = ReedResponse::new("data", "module").with_metrics(metrics);
    }
    let duration = start.elapsed();

    // 10000 operations should complete in < 100ms (< 10μs per operation)
    assert!(duration.as_millis() < 100);
}

// == ERROR TESTS ==

#[test]
fn test_reed_error_display() {
    let err = not_found("test.key");
    let display = format!("{}", err);
    assert!(display.contains("Resource not found"));
    assert!(display.contains("test.key"));
}

#[test]
fn test_reed_error_clone() {
    let err1 = validation_error("field", "value", "constraint");
    let err2 = err1.clone();

    match (err1, err2) {
        (
            ReedError::ValidationError {
                field: f1,
                value: v1,
                constraint: c1,
            },
            ReedError::ValidationError {
                field: f2,
                value: v2,
                constraint: c2,
            },
        ) => {
            assert_eq!(f1, f2);
            assert_eq!(v1, v2);
            assert_eq!(c1, c2);
        }
        _ => panic!("Clone failed"),
    }
}

// == REED-01-02: ERROR SYSTEM TESTS ==

#[test]
fn test_with_context_on_not_found() {
    let err = not_found("missing.key").with_context("CSV lookup failed");

    match err {
        ReedError::NotFound { resource, context } => {
            assert_eq!(resource, "missing.key");
            assert_eq!(context, Some("CSV lookup failed".to_string()));
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_with_context_on_other_error_ignored() {
    let err = validation_error("field", "value", "constraint").with_context("ignored");

    // with_context only affects NotFound, should remain unchanged
    match err {
        ReedError::ValidationError { .. } => {
            // Success - error unchanged
        }
        _ => panic!("Error variant changed unexpectedly"),
    }
}

#[test]
fn test_from_io_error() {
    use std::io;

    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let reed_err: ReedError = io_err.into();

    match reed_err {
        ReedError::IoError {
            operation,
            path,
            reason,
        } => {
            assert_eq!(operation, "io");
            assert_eq!(path, "unknown");
            assert!(reason.contains("file not found"));
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_from_csv_error() {
    use csv::ReaderBuilder;
    use std::io::Cursor;

    // Create invalid CSV to trigger csv::Error
    let invalid_csv = "header1|header2\nvalue1";
    let cursor = Cursor::new(invalid_csv);
    let mut reader = ReaderBuilder::new().delimiter(b'|').from_reader(cursor);

    let csv_err = reader.records().next().unwrap().unwrap_err();
    let reed_err: ReedError = csv_err.into();

    match reed_err {
        ReedError::CsvError {
            file_type,
            operation,
            reason,
        } => {
            assert_eq!(file_type, "unknown");
            assert_eq!(operation, "csv");
            assert!(!reason.is_empty());
        }
        _ => panic!("Wrong error variant"),
    }
}

#[test]
fn test_question_mark_operator_with_io_error() {
    use std::fs::File;

    fn read_file() -> ReedResult<String> {
        let _file = File::open("/nonexistent/path/file.txt")?; // io::Error auto-converts
        Ok("success".to_string())
    }

    let result = read_file();
    assert!(result.is_err());

    match result.unwrap_err() {
        ReedError::IoError { .. } => {
            // Success - automatic conversion worked
        }
        _ => panic!("Wrong error variant"),
    }
}
