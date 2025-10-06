# Foundation Layer

> Universal communication types for all ReedCMS modules

## Status: ✅ 100% Complete

**Implementation:** REED-01-01, REED-01-02  
**Files:** `src/reedcms/reedstream.rs` (342 lines)  
**Tests:** 63/63 passed (29 + 34)

## Overview

The Foundation Layer provides the universal communication interface used by all other ReedCMS layers. Every function in the system returns `ReedResult<ReedResponse<T>>` for consistency and rich error handling.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  All ReedCMS Modules                    │
│   CLI │ Data │ Security │ Server │ Template │ ...      │
└───────────────────┬─────────────────────────────────────┘
                    │ Uses ReedStream types
                    ▼
┌─────────────────────────────────────────────────────────┐
│              Foundation Layer (Layer 01)                │
│                                                          │
│  ReedStream  │  ReedError  │  ReedResponse  │  Module  │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Core Components

### 1. ReedStream Communication

**File:** `src/reedcms/reedstream.rs`

Universal request/response types for inter-module communication.

#### ReedRequest

```rust
pub struct ReedRequest {
    pub key: String,
    pub language: Option<String>,
    pub environment: Option<String>,
    pub context: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
}
```

**Purpose:** Standardised request format for all operations.

**Example:**
```rust
let request = ReedRequest {
    key: "page.title".to_string(),
    language: Some("en".to_string()),
    environment: Some("dev".to_string()),
    context: None,
    value: None,
    description: None,
};
```

#### ReedResponse<T>

```rust
pub struct ReedResponse<T> {
    pub data: T,
    pub source: String,
    pub cached: bool,
    pub timestamp: u64,
    pub metrics: Option<ResponseMetrics>,
}
```

**Purpose:** Standardised response wrapper with metadata.

**Fields:**
- `data: T` - The actual response data (generic type)
- `source: String` - Where the data came from (e.g., "ReedBase cache")
- `cached: bool` - Whether response came from cache
- `timestamp: u64` - Unix timestamp of response
- `metrics: Option<ResponseMetrics>` - Optional performance metrics

**Example:**
```rust
let response = ReedResponse {
    data: "Welcome".to_string(),
    source: "ReedBase cache".to_string(),
    cached: true,
    timestamp: 1704067200,
    metrics: None,
};
```

#### ReedResult<T>

```rust
pub type ReedResult<T> = Result<T, ReedError>;
```

**Purpose:** Type alias for consistent error handling.

**Usage:** Every ReedCMS function returns `ReedResult<ReedResponse<T>>`.

**Example:**
```rust
pub fn get_text(key: &str, lang: &str) -> ReedResult<ReedResponse<String>> {
    // Implementation
}
```

#### ResponseMetrics

```rust
pub struct ResponseMetrics {
    pub duration_us: u64,
    pub cache_hit: bool,
    pub operations: usize,
}
```

**Purpose:** Optional performance tracking.

**Fields:**
- `duration_us` - Operation duration in microseconds
- `cache_hit` - Whether cache was hit
- `operations` - Number of operations performed

### 2. ReedError System

**Implementation:** Uses `thiserror` crate for clean error definitions.

#### Error Variants

```rust
#[derive(Error, Debug)]
pub enum ReedError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("CSV error: {0}")]
    CsvError(String),
    
    #[error("Template error: {0}")]
    TemplateError(String),
    
    #[error("Server error: {0}")]
    ServerError(String),
    
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
}
```

**Usage:**
```rust
// Return specific error
return Err(ReedError::NotFound("Key not found".to_string()));

// Automatic conversion from std::io::Error
let file = File::open("test.csv")?; // Auto-converts to ReedError::IoError

// Add context
.map_err(|e| ReedError::ConfigError(format!("Failed to parse: {}", e)))?
```

### 3. ReedModule Trait

```rust
pub trait ReedModule {
    fn module_name() -> &'static str;
    fn health_check() -> ReedResult<ReedResponse<String>>;
    fn version() -> &'static str;
    fn dependencies() -> Vec<&'static str>;
}
```

**Purpose:** Every ReedCMS module implements this trait for consistent module interface.

**Example Implementation:**
```rust
impl ReedModule for ReedBase {
    fn module_name() -> &'static str {
        "ReedBase"
    }
    
    fn health_check() -> ReedResult<ReedResponse<String>> {
        Ok(ReedResponse {
            data: "ReedBase operational".to_string(),
            source: "ReedBase".to_string(),
            cached: false,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metrics: None,
        })
    }
    
    fn version() -> &'static str {
        "1.0.0"
    }
    
    fn dependencies() -> Vec<&'static str> {
        vec!["csv", "serde"]
    }
}
```

## Communication Flow

```
┌──────────────────────────────────────────────────────────┐
│ Caller (e.g., CLI Command)                               │
└────────────┬─────────────────────────────────────────────┘
             │
             │ Creates ReedRequest
             ▼
┌──────────────────────────────────────────────────────────┐
│ Service Function (e.g., get_text)                        │
│                                                           │
│  fn get_text(key, lang) -> ReedResult<ReedResponse<T>>  │
└────────────┬─────────────────────────────────────────────┘
             │
             │ Returns ReedResult
             ▼
┌──────────────────────────────────────────────────────────┐
│ Caller handles Result                                    │
│                                                           │
│  match result {                                          │
│      Ok(response) => use response.data                   │
│      Err(e) => handle ReedError                          │
│  }                                                        │
└──────────────────────────────────────────────────────────┘
```

## Usage Examples

### Basic Request/Response

```rust
// Create request
let request = ReedRequest::new("page.title", Some("en"));

// Call service
let result = get_text(&request)?;

// Use response
println!("Title: {}", result.data);
println!("Source: {}", result.source);
println!("Cached: {}", result.cached);
```

### Error Handling

```rust
match get_text("page.title", "en") {
    Ok(response) => {
        println!("Success: {}", response.data);
    }
    Err(ReedError::NotFound(msg)) => {
        eprintln!("Not found: {}", msg);
    }
    Err(ReedError::ValidationError(msg)) => {
        eprintln!("Invalid input: {}", msg);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

### With Metrics

```rust
let response = get_text("page.title", "en")?;

if let Some(metrics) = response.metrics {
    println!("Duration: {}μs", metrics.duration_us);
    println!("Cache hit: {}", metrics.cache_hit);
    println!("Operations: {}", metrics.operations);
}
```

## Performance

| Operation | Time | Allocations |
|-----------|------|-------------|
| ReedRequest creation | < 1μs | 1 |
| ReedResponse creation | < 1μs | 1 |
| Error creation | < 1μs | 1 |
| Type conversions | 0μs | 0 (zero-cost) |

**Design:** Zero-cost abstractions where possible. All types are stack-allocated when values fit.

## Testing

**Test Coverage:** 100%  
**Test Files:** `src/reedcms/reedstream.test.rs`  
**Test Count:** 63 tests (29 ReedStream + 34 Error)

**Example Tests:**
```rust
#[test]
fn test_reed_request_creation() {
    let req = ReedRequest::new("key", Some("en"));
    assert_eq!(req.key, "key");
    assert_eq!(req.language, Some("en".to_string()));
}

#[test]
fn test_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
    let reed_err: ReedError = io_err.into();
    match reed_err {
        ReedError::IoError(_) => {},
        _ => panic!("Wrong error type"),
    }
}
```

## Best Practices

### 1. Always Use ReedResult

```rust
// ✅ Good
pub fn my_function() -> ReedResult<ReedResponse<String>> {
    // ...
}

// ❌ Bad - Don't use raw Result
pub fn my_function() -> Result<String, Box<dyn Error>> {
    // ...
}
```

### 2. Include Context in Errors

```rust
// ✅ Good
.map_err(|e| ReedError::ConfigError(
    format!("Failed to parse config at line {}: {}", line, e)
))?

// ❌ Bad - No context
.map_err(|e| ReedError::ConfigError(e.to_string()))?
```

### 3. Set Meaningful Source

```rust
// ✅ Good
ReedResponse {
    data: value,
    source: "ReedBase cache (text.csv)".to_string(),
    // ...
}

// ❌ Bad - Generic source
ReedResponse {
    data: value,
    source: "cache".to_string(),
    // ...
}
```

### 4. Use Metrics for Performance-Critical Code

```rust
let start = Instant::now();
// ... operation ...
let duration = start.elapsed().as_micros() as u64;

Ok(ReedResponse {
    data: result,
    metrics: Some(ResponseMetrics {
        duration_us: duration,
        cache_hit: true,
        operations: 1,
    }),
    // ...
})
```

## Integration with Other Layers

**All layers use Foundation types:**

- **Layer 02 (Data):** Returns `ReedResult<ReedResponse<String>>` for all get/set operations
- **Layer 03 (Security):** Returns `ReedResult<ReedResponse<User>>` for user operations
- **Layer 04 (CLI):** Accepts `ReedResult` from all services, formats output
- **Layer 06 (Server):** Converts `ReedResult` to HTTP responses

**Example Chain:**
```
CLI Command → Data Service → ReedBase → Response
  (calls)       (returns)     (returns)  (uses)
               ReedResult    ReedResult  ReedResponse
```

## File Reference

```
src/reedcms/
├── reedstream.rs              # Core types (342 lines)
└── reedstream.test.rs         # Tests (63 tests)
```

## Related Documentation

- [Error Handling](error-handling.md) - Detailed error patterns
- [Response Types](response-types.md) - Deep dive into ReedResponse
- [Data Layer](../02-data-layer/README.md) - Uses Foundation types
- [CLI Layer](../04-cli-layer/README.md) - Converts to user output

## Summary

The Foundation Layer provides:
- ✅ Universal request/response types (`ReedRequest`, `ReedResponse`)
- ✅ Standardised error handling (`ReedError` with 10 variants)
- ✅ Type-safe results (`ReedResult<T>`)
- ✅ Module trait for consistency (`ReedModule`)
- ✅ Performance metrics support
- ✅ 100% test coverage

All 63 tests passing. Production-ready.
