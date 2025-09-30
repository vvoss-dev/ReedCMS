# ReedCMS Service Implementation Template

> **Purpose**: Exemplary structure for all ReedCMS services with concrete code examples, rules and standards.

## Table of Contents
1. [Mandatory File Header](#mandatory-file-header)
2. [Service Structure Template](#service-structure-template)
3. [Function Documentation Standard](#function-documentation-standard)
4. [Input/Output Patterns](#inputoutput-patterns)
5. [Error Handling Implementation](#error-handling-implementation)
6. [Performance Guidelines](#performance-guidelines)
7. [Testing Template](#testing-template)

---

## Mandatory File Header

**EVERY Rust file MUST begin with this exact header:**

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
// This file: [Brief description of file responsibility]
// Architecture: [How this file fits in the system]
// Performance: [Performance characteristics and constraints]
// Dependencies: [Key dependencies and why they're used]
// Data Flow: [How data flows through this component]
```

---

## Service Structure Template

### Complete Service Example: `src/reedcms/reedbase/set.rs`

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
// This file: Text content setting operations for ReedCMS
// Architecture: ReedBase service called by ReedBase dispatcher
// Performance: O(1) HashMap update + atomic CSV write, automatic backup
// Dependencies: csv::Writer, xz2 compression, std::collections::HashMap
// Data Flow: ReedRequest → validation → backup → CSV write → HashMap update → ReedResponse

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use csv::Writer;
use xz2::write::XzEncoder;

use crate::reedcms::reedstream::{ReedRequest, ReedResponse, ReedResult, ReedError};
use crate::reedcms::reedbase::backup::create_backup;

/// Sets text content in the ReedCMS text storage system.
///
/// ## Input
/// - `req.key`: Text identifier (e.g., "welcome.title")
/// - `req.lang`: Language code (e.g., "en", "de")
/// - `req.value`: Text content to store
/// - `req.description`: Mandatory description for documentation
/// - `req.environment`: Optional environment override (@dev, @prod)
///
/// ## Output
/// - `ReedResult<ReedResponse<()>>`: Success confirmation or detailed error
///
/// ## Performance
/// - **Backup creation**: ~5ms for XZ compression
/// - **CSV write**: ~2ms atomic file operation
/// - **HashMap update**: ~0.1ms O(1) operation
/// - **Total**: ~7ms per text setting operation
///
/// ## Error Conditions
/// - `ReedError::InvalidKey`: Key contains invalid characters
/// - `ReedError::MissingDescription`: Description field empty
/// - `ReedError::BackupFailed`: Cannot create backup before modification
/// - `ReedError::CsvWriteFailed`: Cannot write to CSV file
/// - `ReedError::HashMapUpdateFailed`: Runtime cache update failed
///
/// ## Example Usage
/// ```rust
/// let req = ReedRequest {
///     key: "welcome.title".to_string(),
///     lang: Some("en".to_string()),
///     value: "Welcome to ReedCMS".to_string(),
///     description: "Landing page main title".to_string(),
///     environment: None,
/// };
/// let result = set_text(req).await?;
/// ```
pub async fn set_text(req: ReedRequest) -> ReedResult<ReedResponse<()>> {
    // == STEP 1: Input Validation ==
    validate_text_input(&req)?;

    // == STEP 2: Environment Key Resolution ==
    let final_key = resolve_environment_key(&req.key, &req.environment)?;

    // == STEP 3: Automatic Backup Creation ==
    create_backup(".reed/text.csv").await?;

    // == STEP 4: CSV File Update ==
    write_to_csv(&final_key, &req.lang, &req.value, &req.description).await?;

    // == STEP 5: Runtime HashMap Update ==
    update_runtime_cache(&final_key, &req.lang, &req.value).await?;

    // == STEP 6: Success Response ==
    Ok(ReedResponse::success(()))
}

/// Validates text input according to ReedCMS standards.
///
/// ## Input Validation Rules
/// - Key: Must match pattern `^[a-zA-Z0-9._-]+$` (no spaces or special chars)
/// - Language: Must be valid ISO 639-1 code ("en", "de", etc.)
/// - Value: Must not be empty after trim
/// - Description: Mandatory, minimum 10 characters
///
/// ## Performance
/// - **Execution time**: ~0.1ms (regex validation)
/// - **Memory usage**: Zero allocations (borrows input)
///
/// ## Error Handling
/// Returns specific `ReedError` for each validation failure with context.
fn validate_text_input(req: &ReedRequest) -> ReedResult<()> {
    use regex::Regex;

    // Key validation
    let key_pattern = Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap();
    if !key_pattern.is_match(&req.key) {
        return Err(ReedError::InvalidKey {
            key: req.key.clone(),
            reason: "Key must contain only alphanumeric, dots, underscores and hyphens".to_string(),
        });
    }

    // Language validation
    if let Some(ref lang) = req.lang {
        if lang.len() != 2 || !lang.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(ReedError::InvalidLanguage {
                language: lang.clone(),
                reason: "Language must be 2-character ISO 639-1 code (e.g., 'en', 'de')".to_string(),
            });
        }
    }

    // Value validation
    if req.value.trim().is_empty() {
        return Err(ReedError::EmptyValue {
            key: req.key.clone(),
            reason: "Text content cannot be empty".to_string(),
        });
    }

    // Description validation (mandatory for all text content)
    if req.description.trim().len() < 10 {
        return Err(ReedError::MissingDescription {
            key: req.key.clone(),
            current_length: req.description.len(),
            minimum_required: 10,
            reason: "Description must be at least 10 characters for documentation purposes".to_string(),
        });
    }

    Ok(())
}

/// Resolves environment-specific keys with @suffix support.
///
/// ## Environment Key Resolution
/// - `welcome.title` + `@dev` → `welcome.title@dev`
/// - `welcome.title@prod` + `None` → `welcome.title@prod` (preserves existing)
/// - `welcome.title` + `None` → `welcome.title` (no environment)
///
/// ## Performance
/// - **Execution time**: ~0.05ms (string concatenation)
/// - **Memory usage**: Single allocation for environment keys only
fn resolve_environment_key(base_key: &str, environment: &Option<String>) -> ReedResult<String> {
    match environment {
        Some(env) if !env.is_empty() => {
            if base_key.contains('@') {
                // Key already has environment suffix, validate consistency
                let parts: Vec<&str> = base_key.split('@').collect();
                if parts.len() == 2 && parts[1] == env {
                    Ok(base_key.to_string())
                } else {
                    Err(ReedError::EnvironmentMismatch {
                        key: base_key.to_string(),
                        requested_env: env.clone(),
                        existing_env: parts.get(1).unwrap_or(&"").to_string(),
                    })
                }
            } else {
                // Add environment suffix
                Ok(format!("{}@{}", base_key, env))
            }
        }
        _ => Ok(base_key.to_string()),
    }
}

/// Writes text data to CSV file with atomic operation.
///
/// ## CSV Format
/// ```csv
/// key;language;value;description
/// welcome.title;en;"Welcome to ReedCMS";"Landing page main title"
/// welcome.title;de;"Willkommen bei ReedCMS";"Haupttitel der Startseite"
/// ```
///
/// ## Atomic Write Process
/// 1. Write to temporary file: `.reed/text.csv.tmp`
/// 2. Validate CSV structure
/// 3. Atomic rename to final file: `.reed/text.csv`
///
/// ## Performance
/// - **Write time**: ~2ms for typical CSV files (100-1000 entries)
/// - **Memory usage**: Streams data, no full file loading
/// - **Atomic guarantee**: No corruption on system crash/interruption
async fn write_to_csv(key: &str, lang: &Option<String>, value: &str, description: &str) -> ReedResult<()> {
    use std::path::Path;
    use tokio::fs;

    let csv_path = ".reed/text.csv";
    let temp_path = ".reed/text.csv.tmp";

    // Read existing CSV content
    let mut existing_entries = Vec::new();
    if Path::new(csv_path).exists() {
        existing_entries = read_existing_csv_entries(csv_path).await?;
    }

    // Update or add entry
    let lang_key = lang.as_deref().unwrap_or("default");
    let entry_key = format!("{}:{}", key, lang_key);

    // Remove existing entry with same key+lang combination
    existing_entries.retain(|entry| entry.composite_key != entry_key);

    // Add new entry
    existing_entries.push(CsvEntry {
        composite_key: entry_key,
        key: key.to_string(),
        language: lang_key.to_string(),
        value: value.to_string(),
        description: description.to_string(),
    });

    // Write to temporary file
    write_csv_entries(temp_path, &existing_entries).await?;

    // Atomic rename
    fs::rename(temp_path, csv_path).await.map_err(|e| ReedError::CsvWriteFailed {
        path: csv_path.to_string(),
        reason: format!("Atomic rename failed: {}", e),
    })?;

    Ok(())
}

/// Updates runtime HashMap cache for O(1) lookups.
///
/// ## Cache Structure
/// ```rust
/// HashMap<String, HashMap<String, String>>
/// // Key structure: text_cache["welcome.title"]["en"] = "Welcome to ReedCMS"
/// ```
///
/// ## Thread Safety
/// Uses `tokio::sync::RwLock` for concurrent read/write access.
///
/// ## Performance
/// - **Update time**: ~0.1ms O(1) operation
/// - **Memory impact**: ~50 bytes per text entry
/// - **Concurrency**: Multiple readers, single writer at a time
async fn update_runtime_cache(key: &str, lang: &Option<String>, value: &str) -> ReedResult<()> {
    use crate::reedcms::reedbase::cache::TEXT_CACHE;

    let lang_key = lang.as_deref().unwrap_or("default");

    // Acquire write lock
    let mut cache = TEXT_CACHE.write().await;

    // Get or create language map for this key
    let lang_map = cache.entry(key.to_string()).or_insert_with(HashMap::new);

    // Update value
    lang_map.insert(lang_key.to_string(), value.to_string());

    // Cache updated successfully
    Ok(())
}

// == INTERNAL DATA STRUCTURES ==

#[derive(Debug, Clone)]
struct CsvEntry {
    composite_key: String,  // For deduplication: "key:lang"
    key: String,
    language: String,
    value: String,
    description: String,
}

// == HELPER FUNCTIONS ==

async fn read_existing_csv_entries(path: &str) -> ReedResult<Vec<CsvEntry>> {
    // Implementation details...
    todo!("Implement CSV reading logic")
}

async fn write_csv_entries(path: &str, entries: &[CsvEntry]) -> ReedResult<()> {
    // Implementation details...
    todo!("Implement CSV writing logic")
}

// Tests are in separate set.test.rs file - see Testing Template section
```

---

## Function Documentation Standard

### MANDATORY Documentation Template

```rust
/// Brief one-line description of what this function does.
///
/// ## Input
/// - `param1`: Description of first parameter
/// - `param2`: Description of second parameter
/// - `param3`: Optional parameter (if applicable)
///
/// ## Output
/// - `ReedResult<ReedResponse<T>>`: Success type or specific error conditions
///
/// ## Performance
/// - **Execution time**: Specific timing (e.g., ~2ms, ~0.1ms)
/// - **Memory usage**: Memory characteristics (allocations, borrowing)
/// - **Complexity**: Big O notation if applicable
///
/// ## Error Conditions
/// - `ReedError::SpecificError`: When this error occurs
/// - `ReedError::AnotherError`: When this other error occurs
///
/// ## Example Usage
/// ```rust
/// let result = function_name(param1, param2).await?;
/// assert_eq!(result.data, expected_value);
/// ```
pub async fn function_name(param1: Type1, param2: Type2) -> ReedResult<ReedResponse<ReturnType>> {
    // Implementation
}
```

---

## Input/Output Patterns

### Standard Request/Response Types

```rust
// == REEDSTREAM COMMUNICATION PATTERNS ==

// Standard request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedRequest {
    pub key: String,                    // Primary identifier
    pub lang: Option<String>,           // Language code (ISO 639-1)
    pub value: String,                  // Content/data to store
    pub description: String,            // Mandatory documentation
    pub environment: Option<String>,    // Environment override (@dev, @prod)
    pub metadata: Option<HashMap<String, String>>, // Additional context
}

// Standard response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedResponse<T> {
    pub success: bool,                  // Operation success flag
    pub data: T,                        // Response payload
    pub message: Option<String>,        // Human-readable message
    pub execution_time_ms: u64,         // Performance tracking
    pub cache_hit: bool,                // Performance indicator
}

// Result type alias
pub type ReedResult<T> = Result<T, ReedError>;

// Standard response constructors
impl<T> ReedResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            message: None,
            execution_time_ms: 0,
            cache_hit: false,
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data,
            message: Some(message),
            execution_time_ms: 0,
            cache_hit: false,
        }
    }
}
```

### Input Validation Pattern

```rust
/// Standard input validation function pattern.
///
/// ## Validation Rules
/// Every service MUST validate all inputs before processing.
///
/// ## Performance
/// - **Execution time**: Must be < 1ms for typical inputs
/// - **Memory usage**: Zero allocations preferred (use borrowing)
fn validate_input(req: &ReedRequest) -> ReedResult<()> {
    // Key validation (mandatory for all services)
    if req.key.trim().is_empty() {
        return Err(ReedError::EmptyKey {
            reason: "Key cannot be empty".to_string(),
        });
    }

    // Service-specific validations
    // ... additional validation logic

    Ok(())
}
```

---

## Error Handling Implementation

### Comprehensive Error Types

```rust
// == REED ERROR TYPES ==
// All services MUST use these standardised error types

#[derive(Debug, Clone, thiserror::Error)]
pub enum ReedError {
    // Input validation errors
    #[error("Invalid key '{key}': {reason}")]
    InvalidKey { key: String, reason: String },

    #[error("Invalid language '{language}': {reason}")]
    InvalidLanguage { language: String, reason: String },

    #[error("Empty value for key '{key}': {reason}")]
    EmptyValue { key: String, reason: String },

    #[error("Missing description for key '{key}': current length {current_length}, minimum required {minimum_required}. {reason}")]
    MissingDescription {
        key: String,
        current_length: usize,
        minimum_required: usize,
        reason: String,
    },

    // Environment errors
    #[error("Environment mismatch for key '{key}': requested '{requested_env}', existing '{existing_env}'")]
    EnvironmentMismatch {
        key: String,
        requested_env: String,
        existing_env: String,
    },

    // File system errors
    #[error("CSV write failed for path '{path}': {reason}")]
    CsvWriteFailed { path: String, reason: String },

    #[error("Backup creation failed for path '{path}': {reason}")]
    BackupFailed { path: String, reason: String },

    #[error("Backup not found: cannot restore {steps_back} steps back")]
    BackupNotFound { steps_back: u32 },

    // Cache errors
    #[error("HashMap update failed for key '{key}': {reason}")]
    HashMapUpdateFailed { key: String, reason: String },

    #[error("Cache corruption detected for key '{key}': {reason}")]
    CacheCorruption { key: String, reason: String },

    // Server errors
    #[error("Server start failed: {reason}")]
    ServerStartFailed { reason: String },

    #[error("Unix socket error for path '{socket_path}': {reason}")]
    UnixSocketError { socket_path: String, reason: String },

    // Template errors
    #[error("Template rendering failed for '{template}': {reason}")]
    TemplateRenderingFailed { template: String, reason: String },

    #[error("Template not found: '{template}'")]
    TemplateNotFound { template: String },

    // System errors
    #[error("Configuration error: {reason}")]
    ConfigurationError { reason: String },

    #[error("IO error: {reason}")]
    IoError { reason: String },
}

// Error conversion patterns
impl From<std::io::Error> for ReedError {
    fn from(err: std::io::Error) -> Self {
        ReedError::IoError {
            reason: err.to_string(),
        }
    }
}

impl From<csv::Error> for ReedError {
    fn from(err: csv::Error) -> Self {
        ReedError::CsvWriteFailed {
            path: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}
```

### Error Handling Pattern in Services

```rust
/// Standard error handling pattern for all services.
///
/// ## Error Context
/// Always provide rich context for debugging and user feedback.
async fn service_function(req: ReedRequest) -> ReedResult<ReedResponse<T>> {
    // Wrap operations with context
    let result = risky_operation(&req)
        .await
        .map_err(|e| ReedError::SpecificError {
            key: req.key.clone(),
            reason: format!("Operation failed in service_function: {}", e),
        })?;

    // Always return structured response
    Ok(ReedResponse::success(result))
}

/// Error logging pattern (optional, for debugging)
fn log_error(error: &ReedError, context: &str) {
    eprintln!("[ReedCMS Error] {}: {}", context, error);

    // Optional: Send to centralized logging system
    // logger::error("reedcms", error, context);
}
```

---

## Performance Guidelines

### Timing Targets

```rust
// == PERFORMANCE REQUIREMENTS ==

// Service-level performance targets
const VALIDATION_MAX_TIME_MS: u64 = 1;      // Input validation
const CSV_WRITE_MAX_TIME_MS: u64 = 5;       // File operations
const CACHE_UPDATE_MAX_TIME_MS: u64 = 1;    // HashMap operations
const BACKUP_MAX_TIME_MS: u64 = 10;         // Backup creation (XZ compression)

// Measurement pattern
use std::time::Instant;

async fn timed_operation<T, F>(operation: F, operation_name: &str) -> ReedResult<T>
where
    F: std::future::Future<Output = ReedResult<T>>,
{
    let start = Instant::now();
    let result = operation.await;
    let duration = start.elapsed();

    // Log performance warning if too slow
    if duration.as_millis() > 10 {
        eprintln!("[Performance Warning] {} took {}ms", operation_name, duration.as_millis());
    }

    result
}

// Usage in service functions
pub async fn optimised_service_function(req: ReedRequest) -> ReedResult<ReedResponse<()>> {
    timed_operation(
        validate_input(&req),
        "input_validation"
    ).await?;

    timed_operation(
        write_to_csv(&req.key, &req.value),
        "csv_write"
    ).await?;

    Ok(ReedResponse::success(()))
}
```

### Memory Management

```rust
// == MEMORY EFFICIENCY PATTERNS ==

// Prefer borrowing over cloning
fn efficient_function(req: &ReedRequest) -> ReedResult<String> {
    // Good: Borrow data
    let processed = process_string(&req.key);
    Ok(processed)
}

// Avoid unnecessary allocations
fn avoid_allocations(input: &str) -> &str {
    // Good: Return slice without allocation
    input.trim()
}

fn with_allocations(input: &str) -> String {
    // Use only when necessary
    format!("processed_{}", input)
}

// Use Cow for conditional cloning
use std::borrow::Cow;

fn conditional_clone(input: &str, modify: bool) -> Cow<str> {
    if modify {
        Cow::Owned(format!("modified_{}", input))
    } else {
        Cow::Borrowed(input)
    }
}
```

---

## Testing Template

### MANDATORY: Separate Test Files

All tests MUST be in separate `{name}.test.rs` files, never inline `#[cfg(test)]` modules.

**See complete test implementation guide:** `00_service-test-template.md`

**Example structure:**
```
src/reedcms/reedbase/
├── set.rs              # Implementation
└── set.test.rs         # Tests (see 00_service-test-template.md)
```

### Test File Integration

Each service implementation file should reference its test counterpart:

```rust
// Tests are in separate set.test.rs file - see 00_service-test-template.md

    // == UNIT TESTS ==

    #[test]
    fn test_input_validation_success() {
        let req = create_valid_request();
        let result = validate_input(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_input_validation_invalid_key() {
        let req = ReedRequest {
            key: "".to_string(),  // Invalid: empty key
            ..create_valid_request()
        };

        let result = validate_input(&req);
        assert!(matches!(result, Err(ReedError::EmptyKey { .. })));
    }

    #[test]
    fn test_environment_key_resolution() {
        // Test various environment scenarios
        assert_eq!(
            resolve_environment_key("base.key", &Some("dev".to_string())).unwrap(),
            "base.key@dev"
        );

        assert_eq!(
            resolve_environment_key("base.key@prod", &None).unwrap(),
            "base.key@prod"
        );
    }

    // == INTEGRATION TESTS ==

    #[tokio::test]
    async fn test_complete_service_workflow() {
        // Setup test environment
        setup_test_csv_files().await;

        let req = create_valid_request();

        // Test complete workflow
        let result = service_function(req).await;
        assert!(result.is_ok());

        // Verify side effects
        verify_csv_updated("test.key", "en", "Test Value").await;
        verify_cache_updated("test.key", "en", "Test Value").await;

        // Cleanup
        cleanup_test_files().await;
    }

    #[tokio::test]
    async fn test_error_recovery() {
        // Test backup and recovery mechanism
        let original_content = read_csv_content(".reed/text.csv").await;

        // Simulate error scenario
        let bad_req = create_invalid_request();
        let result = service_function(bad_req).await;
        assert!(result.is_err());

        // Verify original content preserved
        let current_content = read_csv_content(".reed/text.csv").await;
        assert_eq!(original_content, current_content);
    }

    // == PERFORMANCE TESTS ==

    #[tokio::test]
    async fn test_performance_requirements() {
        use std::time::Instant;

        let req = create_valid_request();

        let start = Instant::now();
        let result = service_function(req).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_millis() < 20); // Total operation < 20ms
    }

    // == TEST HELPERS ==

    fn create_valid_request() -> ReedRequest {
        ReedRequest {
            key: "test.key".to_string(),
            lang: Some("en".to_string()),
            value: "Test Value".to_string(),
            description: "Valid test description for unit testing".to_string(),
            environment: None,
        }
    }

    fn create_invalid_request() -> ReedRequest {
        ReedRequest {
            key: "".to_string(),  // Invalid key
            lang: Some("en".to_string()),
            value: "Test Value".to_string(),
            description: "".to_string(),  // Invalid description
            environment: None,
        }
    }

    async fn setup_test_csv_files() {
        // Create test directory structure
        tokio::fs::create_dir_all(".reed/test").await.unwrap();
        // Initialize test CSV files
    }

    async fn cleanup_test_files() {
        // Remove test files
        let _ = tokio::fs::remove_dir_all(".reed/test").await;
    }

    async fn verify_csv_updated(key: &str, lang: &str, expected_value: &str) {
        // Read CSV and verify content
        let content = read_csv_content(".reed/text.csv").await;
        assert!(content.contains(&format!("{};{};{}", key, lang, expected_value)));
    }

    async fn verify_cache_updated(key: &str, lang: &str, expected_value: &str) {
        // Check runtime cache
        use crate::reedcms::reedbase::cache::TEXT_CACHE;
        let cache = TEXT_CACHE.read().await;
        let value = cache.get(key).and_then(|lang_map| lang_map.get(lang));
        assert_eq!(value, Some(&expected_value.to_string()));
    }

    async fn read_csv_content(path: &str) -> String {
        tokio::fs::read_to_string(path).await.unwrap_or_default()
    }
}
```

---

## Implementation Checklist

### Before Implementing Any Service

- [ ] Copy mandatory file header with specific file purpose
- [ ] Define clear Input/Output types using ReedRequest/ReedResponse
- [ ] Implement comprehensive input validation
- [ ] Add proper error handling with specific ReedError types
- [ ] Include performance measurements and targets
- [ ] Write unit tests for all public functions
- [ ] Write integration tests for complete workflows
- [ ] Add performance tests with timing assertions
- [ ] Document all functions with standard template
- [ ] Verify BBC English in all documentation

### Code Review Checklist

- [ ] File header present and complete
- [ ] Function documentation follows template
- [ ] Input validation comprehensive
- [ ] Error handling specific and contextual
- [ ] Performance requirements met
- [ ] Test coverage complete (unit + integration + performance)
- [ ] No generic error messages
- [ ] No allocation-heavy patterns
- [ ] Follows KISS principle (one function = one job)
- [ ] BBC English throughout

---

**Remember**: This template is the gold standard for ALL ReedCMS services. Every service implementation MUST follow these patterns exactly for consistency, maintainability, and performance.