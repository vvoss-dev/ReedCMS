# Error Codes Reference

Complete listing of all `ReedError` variants used throughout ReedCMS.

## Error Type: ReedError

All ReedCMS functions return `ReedResult<T>` which is `Result<T, ReedError>`.

## Error Variants

### NotFound

**Description**: Resource not found (key, file, template, etc.)

**Fields**:
- `resource: String` - What was not found
- `context: Option<String>` - Additional context

**Examples**:
```rust
ReedError::NotFound {
    resource: "knowledge.title@de".to_string(),
    context: Some("text.csv".to_string()),
}

ReedError::NotFound {
    resource: "template: knowledge.mouse.jinja".to_string(),
    context: None,
}
```

**Common Causes**:
- Key not in CSV file
- Template file missing
- Route not defined
- User not found

---

### ValidationError

**Description**: Data validation failure

**Fields**:
- `field: String` - Field name
- `value: String` - Invalid value
- `constraint: String` - Validation rule violated

**Examples**:
```rust
ReedError::ValidationError {
    field: "email".to_string(),
    value: "invalid-email".to_string(),
    constraint: "Must be valid email format".to_string(),
}

ReedError::ValidationError {
    field: "password".to_string(),
    value: "***".to_string(),
    constraint: "Minimum 8 characters".to_string(),
}
```

**Common Causes**:
- Invalid email format
- Password too short
- Invalid key format
- Out of range values

---

### IoError

**Description**: File system or I/O operation failure

**Fields**:
- `operation: String` - Operation attempted
- `path: String` - File path
- `reason: String` - Error details

**Examples**:
```rust
ReedError::IoError {
    operation: "read".to_string(),
    path: ".reed/text.csv".to_string(),
    reason: "Permission denied".to_string(),
}

ReedError::IoError {
    operation: "write".to_string(),
    path: "public/bundle.css".to_string(),
    reason: "Disk full".to_string(),
}
```

**Common Causes**:
- Permission denied
- Disk full
- File locked by another process
- Network mount unavailable

---

### CsvError

**Description**: CSV file operation error

**Fields**:
- `file_type: String` - CSV file type (text, routes, meta)
- `operation: String` - Operation attempted
- `reason: String` - Error details

**Examples**:
```rust
ReedError::CsvError {
    file_type: "text".to_string(),
    operation: "parse".to_string(),
    reason: "Malformed CSV at line 42".to_string(),
}

ReedError::CsvError {
    file_type: "routes".to_string(),
    operation: "write".to_string(),
    reason: "Duplicate key: knowledge@en".to_string(),
}
```

**Common Causes**:
- Malformed CSV syntax
- Duplicate keys
- Invalid delimiters
- Character encoding issues

---

### AuthError

**Description**: Authentication or authorisation failure

**Fields**:
- `user: Option<String>` - Username (if known)
- `action: String` - Action attempted
- `reason: String` - Why auth failed

**Examples**:
```rust
ReedError::AuthError {
    user: Some("john@example.com".to_string()),
    action: "login".to_string(),
    reason: "Invalid password".to_string(),
}

ReedError::AuthError {
    user: Some("editor".to_string()),
    action: "delete_user".to_string(),
    reason: "Insufficient permissions".to_string(),
}
```

**Common Causes**:
- Wrong password
- Expired session
- Insufficient permissions
- Account locked

---

### ConfigError

**Description**: Configuration or setup error

**Fields**:
- `component: String` - Component name
- `reason: String` - Error details

**Examples**:
```rust
ReedError::ConfigError {
    component: "server".to_string(),
    reason: "Invalid port number: 99999".to_string(),
}

ReedError::ConfigError {
    component: "database".to_string(),
    reason: "Missing required field: connection_string".to_string(),
}
```

**Common Causes**:
- Invalid configuration values
- Missing required fields
- Conflicting settings
- Environment not set

---

### TemplateError

**Description**: Template rendering failure

**Fields**:
- `template: String` - Template name
- `reason: String` - Error details

**Examples**:
```rust
ReedError::TemplateError {
    template: "knowledge.mouse.jinja".to_string(),
    reason: "Undefined variable: page_title".to_string(),
}

ReedError::TemplateError {
    template: "blog.touch.jinja".to_string(),
    reason: "Syntax error at line 24".to_string(),
}
```

**Common Causes**:
- Undefined variables
- Template syntax errors
- Filter not found
- Include file missing

---

### ServerError

**Description**: Server or network operation error

**Fields**:
- `component: String` - Server component
- `reason: String` - Error details

**Examples**:
```rust
ReedError::ServerError {
    component: "http_server".to_string(),
    reason: "Failed to bind to 0.0.0.0:3000".to_string(),
}

ReedError::ServerError {
    component: "request_handler".to_string(),
    reason: "Request timeout after 30s".to_string(),
}
```

**Common Causes**:
- Port already in use
- Request timeout
- Connection refused
- Network unreachable

---

### InvalidCommand

**Description**: Invalid CLI command or parameters

**Fields**:
- `command: String` - Command attempted
- `reason: String` - Why invalid

**Examples**:
```rust
ReedError::InvalidCommand {
    command: "reed data:get".to_string(),
    reason: "Missing required argument: key".to_string(),
}

ReedError::InvalidCommand {
    command: "reed server:start --port=abc".to_string(),
    reason: "Invalid port number".to_string(),
}
```

**Common Causes**:
- Missing arguments
- Invalid argument values
- Unknown subcommand
- Conflicting flags

---

### ParseError

**Description**: Data parsing error

**Fields**:
- `input: String` - Input that failed to parse
- `reason: String` - Parsing error details

**Examples**:
```rust
ReedError::ParseError {
    input: "2025-13-45".to_string(),
    reason: "Invalid date format".to_string(),
}

ReedError::ParseError {
    input: "true_false".to_string(),
    reason: "Expected boolean".to_string(),
}
```

**Common Causes**:
- Invalid date format
- Invalid number format
- Invalid boolean value
- Invalid JSON/TOML

---

### FileNotFound

**Description**: Specific file not found

**Fields**:
- `path: String` - File path
- `reason: String` - Additional context

**Examples**:
```rust
ReedError::FileNotFound {
    path: "templates/layouts/knowledge/knowledge.mouse.jinja".to_string(),
    reason: "Template file does not exist".to_string(),
}
```

---

### DirectoryNotFound

**Description**: Directory not found

**Fields**:
- `path: String` - Directory path
- `reason: String` - Additional context

**Examples**:
```rust
ReedError::DirectoryNotFound {
    path: "public/session/styles".to_string(),
    reason: "Asset directory missing".to_string(),
}
```

---

### WriteError

**Description**: Write operation failure

**Fields**:
- `path: String` - File path
- `reason: String` - Write error details

**Examples**:
```rust
ReedError::WriteError {
    path: ".reed/text.csv".to_string(),
    reason: "Disk quota exceeded".to_string(),
}
```

---

### CompressionFailed

**Description**: Compression operation failure

**Fields**:
- `reason: String` - Error details

**Examples**:
```rust
ReedError::CompressionFailed {
    reason: "Gzip compression failed: out of memory".to_string(),
}
```

---

### SecurityViolation

**Description**: Security policy violation

**Fields**:
- `reason: String` - Violation details

**Examples**:
```rust
ReedError::SecurityViolation {
    reason: "Path traversal attempt: ../../etc/passwd".to_string(),
}

ReedError::SecurityViolation {
    reason: "Rate limit exceeded: 1000 req/min".to_string(),
}
```

**Common Causes**:
- Path traversal attempts
- Rate limit exceeded
- Invalid CSRF token
- Malicious input detected

---

### InvalidMetadata

**Description**: Invalid file metadata

**Fields**:
- `reason: String` - Metadata error details

**Examples**:
```rust
ReedError::InvalidMetadata {
    reason: "Cannot read file modification time".to_string(),
}
```

---

### BuildError

**Description**: Build operation failure

**Fields**:
- `component: String` - Build component
- `reason: String` - Build error details

**Examples**:
```rust
ReedError::BuildError {
    component: "css_bundler".to_string(),
    reason: "CSS minification failed".to_string(),
}

ReedError::BuildError {
    component: "cargo_build".to_string(),
    reason: "Compilation failed with 3 errors".to_string(),
}
```

---

### WatcherError

**Description**: File watcher error

**Fields**:
- `reason: String` - Watcher error details

**Examples**:
```rust
ReedError::WatcherError {
    reason: "inotify watch limit exceeded".to_string(),
}
```

---

## Error Handling Patterns

### Basic Pattern

```rust
match operation() {
    Ok(result) => println!("Success: {:?}", result),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Specific Error Handling

```rust
match operation() {
    Ok(result) => Ok(result),
    Err(ReedError::NotFound { resource, .. }) => {
        eprintln!("Not found: {}", resource);
        Err(ReedError::NotFound { resource, context: None })
    }
    Err(ReedError::AuthError { user, action, reason }) => {
        eprintln!("Auth failed for {:?} attempting {}: {}", 
                  user, action, reason);
        Err(ReedError::AuthError { user, action, reason })
    }
    Err(e) => Err(e),
}
```

### Map Error with Context

```rust
std::fs::read("file.txt")
    .map_err(|e| ReedError::IoError {
        operation: "read".to_string(),
        path: "file.txt".to_string(),
        reason: e.to_string(),
    })
```

### Early Return Pattern

```rust
pub fn process() -> ReedResult<String> {
    let data = read_data()?;  // Returns early on error
    let validated = validate(data)?;
    let result = transform(validated)?;
    Ok(result)
}
```

## CLI Error Messages

All CLI commands display user-friendly error messages:

```bash
$ reed data:get missing.key@en
Error: Resource not found: missing.key@en, context: Some("text.csv")
Suggestion: Check key exists with: reed data:list --search=missing

$ reed server:start --port=99999
Error: Configuration error in component 'server': Invalid port number: 99999
Valid range: 1024-65535

$ reed user:create test@example.com --password=short
Error: Validation error in field 'password': value '***' does not meet constraint 'Minimum 8 characters'
```

## HTTP Status Code Mapping

| ReedError | HTTP Status | Use Case |
|-----------|-------------|----------|
| NotFound | 404 | Resource not found |
| ValidationError | 400 | Bad request |
| AuthError | 401/403 | Unauthorized/Forbidden |
| ServerError | 500 | Internal server error |
| TemplateError | 500 | Rendering error |
| SecurityViolation | 403 | Forbidden access |
| ParseError | 400 | Invalid input |

## See Also

- [Foundation Layer](../01-foundation-layer/error-handling.md) - Error handling patterns
- [ReedStream](../01-foundation-layer/reedstream.md) - Universal types
