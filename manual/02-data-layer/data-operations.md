# Data Operations API Reference

> Complete reference for get, set, and init operations

---

## Overview

ReedBase provides three core operations for data management. All operations return `ReedResult<ReedResponse<T>>` following the ReedStream pattern.

**See:** [Foundation Layer - ReedStream](../01-foundation-layer/README.md) for communication types.

---

## Get Operation

### Function Signature

```rust
pub fn get(request: ReedRequest) -> ReedResult<ReedResponse<String>>
```

**File:** `src/reedcms/reedbase/get.rs`

### Input (ReedRequest)

```rust
ReedRequest {
    key: String,              // Required: Base key (e.g., "page.title")
    language: Option<String>, // Optional: Language code ("en", "de")
    environment: Option<String>, // Optional: Environment ("dev", "prod")
    context: Option<String>,  // Required: Data type ("text", "route", "meta")
    value: Option<String>,    // Unused for get
    description: Option<String>, // Unused for get
}
```

### Output (ReedResponse)

```rust
ReedResponse {
    data: String,            // Retrieved value
    source: String,          // "reedbase::get"
    cached: bool,            // true if from cache
    timestamp: u64,          // Unix timestamp
    metrics: Option<Metrics>, // Performance data
}
```

### Examples

**Get text with language:**
```rust
let request = ReedRequest {
    key: "page.title".to_string(),
    language: Some("en".to_string()),
    environment: None,
    context: Some("text".to_string()),
    value: None,
    description: None,
};

let response = get(request)?;
println!("{}", response.data); // "Welcome"
```

**Get with fallback chain:**
```rust
let request = ReedRequest {
    key: "page.banner".to_string(),
    language: Some("de".to_string()),
    environment: Some("dev".to_string()),
    context: Some("text".to_string()),
    ..Default::default()
};

// Tries: page.banner@de@dev → page.banner@de → page.banner@dev → page.banner
let response = get(request)?;
```

**Get route:**
```rust
let request = ReedRequest {
    key: "knowledge".to_string(),
    language: Some("de".to_string()),
    context: Some("route".to_string()),
    ..Default::default()
};

let response = get(request)?;
println!("{}", response.data); // "wissen"
```

**Get metadata:**
```rust
let request = ReedRequest {
    key: "site.title".to_string(),
    context: Some("meta".to_string()),
    ..Default::default()
};

let response = get(request)?;
println!("{}", response.data); // "ReedCMS"
```

### Performance

- **Cache hit:** < 100μs
- **Cache miss:** < 10ms (loads CSV)
- **Fallback chain (4 steps):** < 400μs

### Error Conditions

```rust
ReedError::NotFound {
    key: "page.title@en".to_string(),
    context: "text".to_string(),
}
```

**Causes:**
- Key doesn't exist in CSV
- All fallback attempts failed
- CSV file not found

---

## Set Operation

### Function Signature

```rust
pub fn set(request: ReedRequest) -> ReedResult<ReedResponse<String>>
```

**File:** `src/reedcms/reedbase/set.rs`

### Input (ReedRequest)

```rust
ReedRequest {
    key: String,              // Required: Base key
    language: Option<String>, // Optional: Language suffix
    environment: Option<String>, // Optional: Environment suffix
    context: Option<String>,  // Required: Data type
    value: Option<String>,    // Required: Value to store
    description: Option<String>, // Optional: Documentation
}
```

### Output (ReedResponse)

```rust
ReedResponse {
    data: String,            // Stored value (echo)
    source: String,          // "reedbase::set"
    cached: false,           // Always false (write operation)
    timestamp: u64,          // Unix timestamp
    metrics: Option<Metrics>, // Performance data
}
```

### Examples

**Set text with description:**
```rust
let request = ReedRequest {
    key: "page.title".to_string(),
    language: Some("en".to_string()),
    value: Some("Welcome to ReedCMS".to_string()),
    description: Some("Homepage title".to_string()),
    context: Some("text".to_string()),
    ..Default::default()
};

let response = set(request)?;
```

**Set route:**
```rust
let request = ReedRequest {
    key: "knowledge".to_string(),
    language: Some("de".to_string()),
    value: Some("wissen".to_string()),
    description: Some("German knowledge page URL".to_string()),
    context: Some("route".to_string()),
    ..Default::default()
};

let response = set(request)?;
```

**Set metadata:**
```rust
let request = ReedRequest {
    key: "site.title".to_string(),
    value: Some("ReedCMS".to_string()),
    description: Some("Site title for meta tags".to_string()),
    context: Some("meta".to_string()),
    ..Default::default()
};

let response = set(request)?;
```

### Process Flow

```
1. Validate request (key, value, context)
2. Read current CSV file
3. Create XZ backup → .reed/backups/{file}.{timestamp}.xz
4. Update HashMap cache
5. Write to temp CSV file
6. Atomic rename: temp.csv → {file}.csv
7. Return success response
```

### Performance

- **Complete operation:** < 50ms
  - CSV read: ~5ms
  - XZ compression: ~20ms
  - CSV write: ~10ms
  - Cache update: < 1ms

### Error Conditions

```rust
// Missing value
ReedError::ValidationError {
    field: "value".to_string(),
    value: "".to_string(),
    constraint: "Value required".to_string(),
}

// Duplicate key (if enforce_unique = true)
ReedError::DuplicateKey {
    key: "page.title@en".to_string(),
    context: "text".to_string(),
}

// Write failed
ReedError::WriteError {
    path: ".reed/text.csv".to_string(),
    reason: "Permission denied".to_string(),
}
```

---

## Init Operation

### Function Signature

```rust
pub fn init(request: ReedRequest) -> ReedResult<ReedResponse<HashMap<String, String>>>
```

**File:** `src/reedcms/reedbase/init.rs`

### Input (ReedRequest)

```rust
ReedRequest {
    key: String,              // Unused
    language: Option<String>, // Unused
    environment: Option<String>, // Unused
    context: Option<String>,  // Required: Data type
    value: Option<String>,    // Required: CSV file path
    description: Option<String>, // Unused
}
```

### Output (ReedResponse)

```rust
ReedResponse {
    data: HashMap<String, String>, // Complete cache
    source: String,                // "reedbase::init"
    cached: false,                 // Always false (load operation)
    timestamp: u64,                // Unix timestamp
    metrics: Option<Metrics>,      // Performance data
}
```

### Examples

**Initialise text cache:**
```rust
let request = ReedRequest {
    context: Some("text".to_string()),
    value: Some(".reed/text.csv".to_string()),
    ..Default::default()
};

let response = init(request)?;
let cache: HashMap<String, String> = response.data;

println!("Loaded {} text entries", cache.len());
```

**Initialise all caches:**
```rust
// Text
let text_cache = init(ReedRequest {
    context: Some("text".to_string()),
    value: Some(".reed/text.csv".to_string()),
    ..Default::default()
})?.data;

// Routes
let route_cache = init(ReedRequest {
    context: Some("route".to_string()),
    value: Some(".reed/routes.csv".to_string()),
    ..Default::default()
})?.data;

// Meta
let meta_cache = init(ReedRequest {
    context: Some("meta".to_string()),
    value: Some(".reed/meta.csv".to_string()),
    ..Default::default()
})?.data;
```

### Process Flow

```
1. Open CSV file
2. Validate header row
3. Parse each line:
   - Skip empty lines
   - Skip comments (starting with #)
   - Split by pipe delimiter
   - Insert into HashMap
4. Return populated cache
```

### Performance

| Records | Time | Memory |
|---------|------|--------|
| 100 | < 1ms | ~10 KB |
| 1,000 | < 10ms | ~100 KB |
| 3,000 | < 30ms | ~300 KB |
| 10,000 | < 100ms | ~1 MB |

### Error Conditions

```rust
// File not found
ReedError::FileNotFound {
    path: ".reed/text.csv".to_string(),
    reason: "File does not exist".to_string(),
}

// Invalid CSV format
ReedError::ParseError {
    line: 42,
    reason: "Expected 3 columns, found 2".to_string(),
}

// Invalid header
ReedError::ParseError {
    line: 1,
    reason: "Expected 'key|value|description'".to_string(),
}
```

---

## Common Patterns

### Check if Key Exists

```rust
match get(request) {
    Ok(response) => println!("Value: {}", response.data),
    Err(ReedError::NotFound { .. }) => println!("Key not found"),
    Err(e) => return Err(e),
}
```

### Update or Insert

```rust
// Try to get existing value
let existing = get(request.clone()).ok();

// Set new value
set(request)?;

if existing.is_some() {
    println!("Updated existing key");
} else {
    println!("Created new key");
}
```

### Batch Operations

```rust
// Batch get
let keys = vec!["page.title", "page.subtitle", "page.hero"];
for key in keys {
    let request = ReedRequest {
        key: key.to_string(),
        language: Some("en".to_string()),
        context: Some("text".to_string()),
        ..Default::default()
    };
    
    match get(request) {
        Ok(r) => println!("{}: {}", key, r.data),
        Err(e) => eprintln!("Error getting {}: {:?}", key, e),
    }
}

// Batch set
let updates = vec![
    ("page.title", "Welcome"),
    ("page.subtitle", "High-performance CMS"),
];

for (key, value) in updates {
    set(ReedRequest {
        key: key.to_string(),
        language: Some("en".to_string()),
        value: Some(value.to_string()),
        context: Some("text".to_string()),
        ..Default::default()
    })?;
}
```

### Environment-Specific Values

```rust
// Set dev-specific value
set(ReedRequest {
    key: "api.endpoint".to_string(),
    environment: Some("dev".to_string()),
    value: Some("http://localhost:8333".to_string()),
    context: Some("meta".to_string()),
    ..Default::default()
})?;

// Set prod-specific value
set(ReedRequest {
    key: "api.endpoint".to_string(),
    environment: Some("prod".to_string()),
    value: Some("https://api.example.com".to_string()),
    context: Some("meta".to_string()),
    ..Default::default()
})?;

// Get with environment
let endpoint = get(ReedRequest {
    key: "api.endpoint".to_string(),
    environment: Some(current_env),
    context: Some("meta".to_string()),
    ..Default::default()
})?;
```

---

## Integration with Dispatcher

### ReedBase Dispatcher Usage

```rust
use crate::reedcms::reed::reedbase::ReedBase;

// Create dispatcher
let reedbase = ReedBase::new(
    ".reed/text.csv",
    ".reed/routes.csv",
    ".reed/meta.csv",
);

// Initialise caches
reedbase.init()?;

// Get text
let response = reedbase.get(ReedRequest {
    key: "page.title".to_string(),
    language: Some("en".to_string()),
    context: Some("text".to_string()),
    ..Default::default()
})?;

// Set text
reedbase.set(ReedRequest {
    key: "page.title".to_string(),
    language: Some("en".to_string()),
    value: Some("New Title".to_string()),
    context: Some("text".to_string()),
    ..Default::default()
})?;
```

---

## CLI Integration

**CLI commands map directly to operations:**

```bash
reed text:get page.title@en
→ get(ReedRequest { key: "page.title", language: "en", context: "text" })

reed text:set page.title@en "Welcome"
→ set(ReedRequest { key: "page.title", language: "en", value: "Welcome", context: "text" })

reed route:get knowledge@de
→ get(ReedRequest { key: "knowledge", language: "de", context: "route" })
```

**See:** [CLI Commands - Data Commands](../04-cli-layer/data-commands.md)

---

## Best Practices

**Always check errors:**
```rust
// ✅ Good
match get(request) {
    Ok(response) => println!("{}", response.data),
    Err(e) => handle_error(e),
}

// ❌ Bad
let response = get(request).unwrap();
```

**Use environment fallback:**
```rust
// ✅ Good - environment-aware
get(ReedRequest {
    key: "config.value".to_string(),
    environment: Some(current_env()),
    context: Some("meta".to_string()),
    ..Default::default()
})

// ❌ Bad - hardcoded
get(ReedRequest {
    key: "config.value@prod".to_string(),
    context: Some("meta".to_string()),
    ..Default::default()
})
```

**Batch related operations:**
```rust
// ✅ Good - batch updates
for (key, value) in updates {
    set(/* ... */)?;
}

// ❌ Bad - individual operations with delays
for (key, value) in updates {
    set(/* ... */)?;
    thread::sleep(Duration::from_millis(100)); // Why?
}
```

---

**See also:**
- [ReedBase Cache](reedbase-cache.md) - Cache implementation
- [CSV Architecture](csv-architecture.md) - File format details
- [CLI Commands](../04-cli-layer/data-commands.md) - Command reference
