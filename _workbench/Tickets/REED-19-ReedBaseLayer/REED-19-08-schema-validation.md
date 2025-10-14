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

### Part 2: Column Schema Validation

### Schema File Format

```
.reed/tables/{table_name}/
├── schema.toml          # Schema definition
├── current.csv
└── version.log
```

**schema.toml:**
```toml
[schema]
version = "2.0"  # v2 includes RBKS key validation
strict = true     # Reject writes that violate schema

# ===== PART 1: Key Validation (RBKS v2) =====

[key_validation]
enabled = true
pattern = "^[a-z][a-z0-9]*(\\.[a-z][a-z0-9]*){1,7}(<[a-z]+(,[a-z]+)*>)?$"
min_depth = 2
max_depth = 8
allow_normalization = true  # Auto-normalize on --normalize flag

[key_validation.modifiers.language]
allowed = ["de", "en", "fr", "es", "it", "pt", "nl", "pl", "ru", "ja", "zh", "ar"]
max_count = 1  # Only one language per key

[key_validation.modifiers.environment]
allowed = ["dev", "prod", "staging", "test"]
max_count = 1

[key_validation.modifiers.season]
allowed = ["christmas", "easter", "summer", "winter"]
max_count = 1

[key_validation.modifiers.variant]
allowed = ["mobile", "desktop", "tablet"]
max_count = 1

[key_validation.namespace]
# Optional: restrict allowed namespaces
# allowed = ["page", "blog", "landing", "meta"]
# If not specified, all namespaces allowed

# ===== PART 2: Column Validation =====

[[columns]]
name = "id"
type = "integer"
required = true
unique = true
primary_key = true

[[columns]]
name = "name"
type = "string"
required = true
min_length = 1
max_length = 100

[[columns]]
name = "email"
type = "string"
required = false
pattern = "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"

[[columns]]
name = "age"
type = "integer"
required = false
min = 0
max = 150

[[columns]]
name = "created_at"
type = "timestamp"
required = true
```

### Column Types

- **string**: Text data
- **integer**: Whole numbers
- **float**: Decimal numbers
- **boolean**: true/false
- **timestamp**: Unix timestamp (seconds since epoch)

### Constraints

- **required**: Field must be present and non-empty
- **unique**: Value must be unique across all rows
- **primary_key**: Unique identifier (implies required + unique)
- **min/max**: Numeric range constraints
- **min_length/max_length**: String length constraints
- **pattern**: Regex pattern for string validation

### Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Load schema | < 5ms | TOML parsing |
| Validate row | < 1ms | Single row validation |
| Validate 100 rows | < 100ms | Batch validation |
| Check uniqueness | < 10ms | HashMap lookup for all rows |

## Implementation Files

### Primary Implementation

**`reedbase/src/schema/validation.rs`**

One file = Schema validation only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Schema validation for CSV tables.
//!
//! Validates data against defined schemas to ensure data quality.

use crate::types::{ReedResult, ReedError, Schema, CsvRow, ValidationError};
use std::collections::HashSet;
use regex::Regex;

/// Validate row against schema.
///
/// ## Input
/// - `row`: CSV row to validate
/// - `schema`: Table schema
///
/// ## Output
/// - `ReedResult<()>`: Ok if valid, error with details if invalid
///
/// ## Performance
/// - < 1ms per row typical
///
/// ## Error Conditions
/// - ValidationError: Row violates schema constraints
///
/// ## Example Usage
/// ```rust
/// let row = CsvRow {
///     key: "user_1".to_string(),
///     values: vec!["1".to_string(), "Alice".to_string(), "alice@example.com".to_string()],
/// };
/// validate_row(&row, &schema)?;
/// ```
pub fn validate_row(row: &CsvRow, schema: &Schema) -> ReedResult<()> {
    if row.values.len() != schema.columns.len() {
        return Err(ReedError::ValidationError(ValidationError::FieldCount {
            expected: schema.columns.len(),
            actual: row.values.len(),
        }));
    }
    
    for (i, column) in schema.columns.iter().enumerate() {
        let value = &row.values[i];
        validate_field(value, column)?;
    }
    
    Ok(())
}

/// Validate single field against column definition.
///
/// ## Input
/// - `value`: Field value
/// - `column`: Column definition
///
/// ## Output
/// - `ReedResult<()>`: Ok if valid
///
/// ## Performance
/// - < 100μs per field typical
///
/// ## Error Conditions
/// - ValidationError: Field violates constraints
fn validate_field(value: &str, column: &ColumnDef) -> ReedResult<()> {
    // Check required
    if column.required && value.is_empty() {
        return Err(ReedError::ValidationError(ValidationError::RequiredField {
            column: column.name.clone(),
        }));
    }
    
    // Skip validation for empty optional fields
    if value.is_empty() && !column.required {
        return Ok(());
    }
    
    // Type validation
    match column.col_type.as_str() {
        "string" => validate_string(value, column)?,
        "integer" => validate_integer(value, column)?,
        "float" => validate_float(value, column)?,
        "boolean" => validate_boolean(value)?,
        "timestamp" => validate_timestamp(value)?,
        _ => return Err(ReedError::ValidationError(ValidationError::InvalidType {
            column: column.name.clone(),
            expected: column.col_type.clone(),
        })),
    }
    
    Ok(())
}

/// Validate string field.
///
/// ## Input
/// - `value`: String value
/// - `column`: Column definition
///
/// ## Output
/// - `ReedResult<()>`: Ok if valid
///
/// ## Performance
/// - < 50μs typical (< 500μs if regex pattern)
///
/// ## Error Conditions
/// - ValidationError: Length or pattern violation
fn validate_string(value: &str, column: &ColumnDef) -> ReedResult<()> {
    // Check length constraints
    if let Some(min_length) = column.min_length {
        if value.len() < min_length {
            return Err(ReedError::ValidationError(ValidationError::MinLength {
                column: column.name.clone(),
                min: min_length,
                actual: value.len(),
            }));
        }
    }
    
    if let Some(max_length) = column.max_length {
        if value.len() > max_length {
            return Err(ReedError::ValidationError(ValidationError::MaxLength {
                column: column.name.clone(),
                max: max_length,
                actual: value.len(),
            }));
        }
    }
    
    // Check pattern constraint
    if let Some(ref pattern) = column.pattern {
        let regex = Regex::new(pattern)
            .map_err(|e| ReedError::ValidationError(ValidationError::InvalidPattern {
                pattern: pattern.clone(),
                reason: e.to_string(),
            }))?;
        
        if !regex.is_match(value) {
            return Err(ReedError::ValidationError(ValidationError::PatternMismatch {
                column: column.name.clone(),
                pattern: pattern.clone(),
                value: value.to_string(),
            }));
        }
    }
    
    Ok(())
}

/// Validate integer field.
///
/// ## Input
/// - `value`: Integer string
/// - `column`: Column definition
///
/// ## Output
/// - `ReedResult<()>`: Ok if valid
///
/// ## Performance
/// - < 10μs typical
///
/// ## Error Conditions
/// - ValidationError: Parse error or range violation
fn validate_integer(value: &str, column: &ColumnDef) -> ReedResult<()> {
    let num = value.parse::<i64>()
        .map_err(|_| ReedError::ValidationError(ValidationError::InvalidInteger {
            column: column.name.clone(),
            value: value.to_string(),
        }))?;
    
    if let Some(min) = column.min {
        if num < min {
            return Err(ReedError::ValidationError(ValidationError::MinValue {
                column: column.name.clone(),
                min,
                actual: num,
            }));
        }
    }
    
    if let Some(max) = column.max {
        if num > max {
            return Err(ReedError::ValidationError(ValidationError::MaxValue {
                column: column.name.clone(),
                max,
                actual: num,
            }));
        }
    }
    
    Ok(())
}

/// Validate float field.
///
/// ## Input
/// - `value`: Float string
/// - `column`: Column definition
///
/// ## Output
/// - `ReedResult<()>`: Ok if valid
///
/// ## Performance
/// - < 10μs typical
///
/// ## Error Conditions
/// - ValidationError: Parse error
fn validate_float(value: &str, column: &ColumnDef) -> ReedResult<()> {
    value.parse::<f64>()
        .map_err(|_| ReedError::ValidationError(ValidationError::InvalidFloat {
            column: column.name.clone(),
            value: value.to_string(),
        }))?;
    
    Ok(())
}

/// Validate boolean field.
///
/// ## Input
/// - `value`: Boolean string
///
/// ## Output
/// - `ReedResult<()>`: Ok if valid
///
/// ## Performance
/// - < 5μs typical
///
/// ## Error Conditions
/// - ValidationError: Invalid boolean value
fn validate_boolean(value: &str) -> ReedResult<()> {
    if !matches!(value, "true" | "false" | "1" | "0") {
        return Err(ReedError::ValidationError(ValidationError::InvalidBoolean {
            value: value.to_string(),
        }));
    }
    
    Ok(())
}

/// Validate timestamp field.
///
/// ## Input
/// - `value`: Timestamp string (Unix timestamp)
///
/// ## Output
/// - `ReedResult<()>`: Ok if valid
///
/// ## Performance
/// - < 5μs typical
///
/// ## Error Conditions
/// - ValidationError: Invalid timestamp format
fn validate_timestamp(value: &str) -> ReedResult<()> {
    value.parse::<u64>()
        .map_err(|_| ReedError::ValidationError(ValidationError::InvalidTimestamp {
            value: value.to_string(),
        }))?;
    
    Ok(())
}

/// Validate uniqueness constraints across all rows.
///
/// ## Input
/// - `rows`: All rows in table
/// - `schema`: Table schema
///
/// ## Output
/// - `ReedResult<()>`: Ok if all unique constraints satisfied
///
/// ## Performance
/// - O(n*m) where n = rows, m = unique columns
/// - < 10ms for 100 rows with 2 unique columns
///
/// ## Error Conditions
/// - ValidationError: Duplicate value found
///
/// ## Example Usage
/// ```rust
/// validate_uniqueness(&all_rows, &schema)?;
/// ```
pub fn validate_uniqueness(rows: &[CsvRow], schema: &Schema) -> ReedResult<()> {
    for (col_idx, column) in schema.columns.iter().enumerate() {
        if column.unique || column.primary_key {
            let mut seen = HashSet::new();
            
            for row in rows {
                if col_idx >= row.values.len() {
                    continue;
                }
                
                let value = &row.values[col_idx];
                
                if !value.is_empty() && !seen.insert(value.clone()) {
                    return Err(ReedError::ValidationError(ValidationError::DuplicateValue {
                        column: column.name.clone(),
                        value: value.clone(),
                    }));
                }
            }
        }
    }
    
    Ok(())
}

/// Validate multiple rows in batch.
///
/// ## Input
/// - `rows`: Rows to validate
/// - `schema`: Table schema
///
/// ## Output
/// - `ReedResult<()>`: Ok if all valid
///
/// ## Performance
/// - < 1ms per row
/// - < 100ms for 100 rows
///
/// ## Error Conditions
/// - ValidationError: First invalid row stops validation
///
/// ## Example Usage
/// ```rust
/// validate_rows(&rows, &schema)?;
/// ```
pub fn validate_rows(rows: &[CsvRow], schema: &Schema) -> ReedResult<()> {
    for row in rows {
        validate_row(row, schema)?;
    }
    
    validate_uniqueness(rows, schema)?;
    
    Ok(())
}
```

**`reedbase/src/schema/loader.rs`**

One file = Schema loading only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Schema loading from TOML files.
//!
//! Loads and parses table schemas.

use crate::types::{ReedResult, ReedError, Schema, ColumnDef};
use std::path::Path;
use std::fs;

/// Load schema from file.
///
/// ## Input
/// - `table_name`: Table name
///
/// ## Output
/// - `ReedResult<Schema>`: Loaded schema
///
/// ## Performance
/// - < 5ms typical (TOML parsing)
///
/// ## Error Conditions
/// - SchemaNotFound: schema.toml does not exist
/// - InvalidSchema: TOML parse error or invalid structure
///
/// ## Example Usage
/// ```rust
/// let schema = load_schema("users")?;
/// println!("Loaded schema with {} columns", schema.columns.len());
/// ```
pub fn load_schema(table_name: &str) -> ReedResult<Schema> {
    let schema_path = get_schema_path(table_name);
    
    if !schema_path.exists() {
        return Err(ReedError::SchemaNotFound {
            table: table_name.to_string(),
        });
    }
    
    let content = fs::read_to_string(&schema_path)
        .map_err(|e| ReedError::IoError {
            path: schema_path.to_string_lossy().to_string(),
            source: e,
        })?;
    
    parse_schema(&content)
}

/// Parse schema from TOML string.
///
/// ## Input
/// - `content`: TOML content
///
/// ## Output
/// - `ReedResult<Schema>`: Parsed schema
///
/// ## Performance
/// - < 3ms typical
///
/// ## Error Conditions
/// - InvalidSchema: TOML syntax error or missing required fields
fn parse_schema(content: &str) -> ReedResult<Schema> {
    use toml::Value;
    
    let value: Value = toml::from_str(content)
        .map_err(|e| ReedError::InvalidSchema {
            reason: format!("TOML parse error: {}", e),
        })?;
    
    let version = value["schema"]["version"]
        .as_str()
        .ok_or_else(|| ReedError::InvalidSchema {
            reason: "Missing schema.version".to_string(),
        })?
        .to_string();
    
    let strict = value["schema"]["strict"]
        .as_bool()
        .unwrap_or(true);
    
    let columns = value["columns"]
        .as_array()
        .ok_or_else(|| ReedError::InvalidSchema {
            reason: "Missing columns array".to_string(),
        })?
        .iter()
        .map(|col| parse_column(col))
        .collect::<ReedResult<Vec<ColumnDef>>>()?;
    
    Ok(Schema {
        version,
        strict,
        columns,
    })
}

/// Parse column definition from TOML value.
///
/// ## Input
/// - `value`: TOML value for column
///
/// ## Output
/// - `ReedResult<ColumnDef>`: Parsed column definition
///
/// ## Performance
/// - < 500μs typical
///
/// ## Error Conditions
/// - InvalidSchema: Missing required fields
fn parse_column(value: &toml::Value) -> ReedResult<ColumnDef> {
    let name = value["name"]
        .as_str()
        .ok_or_else(|| ReedError::InvalidSchema {
            reason: "Column missing 'name' field".to_string(),
        })?
        .to_string();
    
    let col_type = value["type"]
        .as_str()
        .ok_or_else(|| ReedError::InvalidSchema {
            reason: format!("Column '{}' missing 'type' field", name),
        })?
        .to_string();
    
    let required = value["required"].as_bool().unwrap_or(false);
    let unique = value["unique"].as_bool().unwrap_or(false);
    let primary_key = value["primary_key"].as_bool().unwrap_or(false);
    
    let min = value["min"].as_integer();
    let max = value["max"].as_integer();
    let min_length = value["min_length"].as_integer().map(|i| i as usize);
    let max_length = value["max_length"].as_integer().map(|i| i as usize);
    let pattern = value["pattern"].as_str().map(|s| s.to_string());
    
    Ok(ColumnDef {
        name,
        col_type,
        required: required || primary_key,
        unique: unique || primary_key,
        primary_key,
        min,
        max,
        min_length,
        max_length,
        pattern,
    })
}

/// Get schema file path.
///
/// ## Input
/// - `table_name`: Table name
///
/// ## Output
/// - `PathBuf`: Schema file path
///
/// ## Performance
/// - O(1) operation
/// - < 1μs
fn get_schema_path(table_name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!(".reed/tables/{}/schema.toml", table_name))
}

/// Create default schema for table.
///
/// ## Input
/// - `table_name`: Table name
/// - `column_names`: Column names from CSV header
///
/// ## Output
/// - `ReedResult<Schema>`: Generated schema (all columns as optional strings)
///
/// ## Performance
/// - < 1ms typical
///
/// ## Example Usage
/// ```rust
/// let schema = create_default_schema("users", &["id", "name", "email"])?;
/// save_schema("users", &schema)?;
/// ```
pub fn create_default_schema(table_name: &str, column_names: &[String]) -> ReedResult<Schema> {
    let columns = column_names.iter()
        .map(|name| ColumnDef {
            name: name.clone(),
            col_type: "string".to_string(),
            required: false,
            unique: false,
            primary_key: false,
            min: None,
            max: None,
            min_length: None,
            max_length: None,
            pattern: None,
        })
        .collect();
    
    Ok(Schema {
        version: "1.0".to_string(),
        strict: false, // Lenient for auto-generated schemas
        columns,
    })
}

/// Save schema to file.
///
/// ## Input
/// - `table_name`: Table name
/// - `schema`: Schema to save
///
/// ## Output
/// - `ReedResult<()>`: Success or error
///
/// ## Performance
/// - < 10ms typical (TOML serialization + write)
///
/// ## Error Conditions
/// - IoError: Cannot write schema file
///
/// ## Example Usage
/// ```rust
/// save_schema("users", &schema)?;
/// ```
pub fn save_schema(table_name: &str, schema: &Schema) -> ReedResult<()> {
    let schema_path = get_schema_path(table_name);
    
    fs::create_dir_all(schema_path.parent().unwrap())
        .map_err(|e| ReedError::IoError {
            path: schema_path.parent().unwrap().to_string_lossy().to_string(),
            source: e,
        })?;
    
    let toml = serialize_schema(schema)?;
    
    fs::write(&schema_path, toml)
        .map_err(|e| ReedError::IoError {
            path: schema_path.to_string_lossy().to_string(),
            source: e,
        })?;
    
    Ok(())
}

/// Serialize schema to TOML string.
///
/// ## Input
/// - `schema`: Schema to serialize
///
/// ## Output
/// - `ReedResult<String>`: TOML string
///
/// ## Performance
/// - < 5ms typical
///
/// ## Error Conditions
/// - SerializationError: TOML serialization failed
fn serialize_schema(schema: &Schema) -> ReedResult<String> {
    let mut toml = String::new();
    
    toml.push_str("[schema]\n");
    toml.push_str(&format!("version = \"{}\"\n", schema.version));
    toml.push_str(&format!("strict = {}\n\n", schema.strict));
    
    for column in &schema.columns {
        toml.push_str("[[columns]]\n");
        toml.push_str(&format!("name = \"{}\"\n", column.name));
        toml.push_str(&format!("type = \"{}\"\n", column.col_type));
        toml.push_str(&format!("required = {}\n", column.required));
        
        if column.unique {
            toml.push_str("unique = true\n");
        }
        
        if column.primary_key {
            toml.push_str("primary_key = true\n");
        }
        
        if let Some(min) = column.min {
            toml.push_str(&format!("min = {}\n", min));
        }
        
        if let Some(max) = column.max {
            toml.push_str(&format!("max = {}\n", max));
        }
        
        if let Some(min_length) = column.min_length {
            toml.push_str(&format!("min_length = {}\n", min_length));
        }
        
        if let Some(max_length) = column.max_length {
            toml.push_str(&format!("max_length = {}\n", max_length));
        }
        
        if let Some(ref pattern) = column.pattern {
            toml.push_str(&format!("pattern = \"{}\"\n", pattern));
        }
        
        toml.push('\n');
    }
    
    Ok(toml)
}
```

**`reedbase/src/types.rs`** (additions)

```rust
/// Table schema.
#[derive(Debug, Clone)]
pub struct Schema {
    pub version: String,
    pub strict: bool,
    pub columns: Vec<ColumnDef>,
}

/// Column definition.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: String,
    pub required: bool,
    pub unique: bool,
    pub primary_key: bool,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

/// Validation errors.
#[derive(Debug, Clone)]
pub enum ValidationError {
    FieldCount { expected: usize, actual: usize },
    RequiredField { column: String },
    InvalidType { column: String, expected: String },
    InvalidInteger { column: String, value: String },
    InvalidFloat { column: String, value: String },
    InvalidBoolean { value: String },
    InvalidTimestamp { value: String },
    MinValue { column: String, min: i64, actual: i64 },
    MaxValue { column: String, max: i64, actual: i64 },
    MinLength { column: String, min: usize, actual: usize },
    MaxLength { column: String, max: usize, actual: usize },
    PatternMismatch { column: String, pattern: String, value: String },
    InvalidPattern { pattern: String, reason: String },
    DuplicateValue { column: String, value: String },
}

/// Additional ReedBase errors.
#[derive(Error, Debug)]
pub enum ReedError {
    // ... (existing errors)
    
    #[error("Schema not found for table '{table}'")]
    SchemaNotFound {
        table: String,
    },
    
    #[error("Invalid schema: {reason}")]
    InvalidSchema {
        reason: String,
    },
    
    #[error("Validation error: {0:?}")]
    ValidationError(ValidationError),
}
```

### Test Files

**`reedbase/src/schema/validation.test.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_schema() -> Schema {
        Schema {
            version: "1.0".to_string(),
            strict: true,
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    col_type: "integer".to_string(),
                    required: true,
                    unique: true,
                    primary_key: true,
                    min: Some(1),
                    max: None,
                    min_length: None,
                    max_length: None,
                    pattern: None,
                },
                ColumnDef {
                    name: "name".to_string(),
                    col_type: "string".to_string(),
                    required: true,
                    unique: false,
                    primary_key: false,
                    min: None,
                    max: None,
                    min_length: Some(1),
                    max_length: Some(100),
                    pattern: None,
                },
                ColumnDef {
                    name: "email".to_string(),
                    col_type: "string".to_string(),
                    required: false,
                    unique: true,
                    primary_key: false,
                    min: None,
                    max: None,
                    min_length: None,
                    max_length: None,
                    pattern: Some("^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$".to_string()),
                },
            ],
        }
    }
    
    #[test]
    fn test_validate_valid_row() {
        let schema = create_test_schema();
        let row = CsvRow {
            key: "1".to_string(),
            values: vec!["1".to_string(), "Alice".to_string(), "alice@example.com".to_string()],
        };
        
        assert!(validate_row(&row, &schema).is_ok());
    }
    
    #[test]
    fn test_validate_missing_required_field() {
        let schema = create_test_schema();
        let row = CsvRow {
            key: "1".to_string(),
            values: vec!["1".to_string(), "".to_string(), "alice@example.com".to_string()],
        };
        
        let result = validate_row(&row, &schema);
        assert!(matches!(result, Err(ReedError::ValidationError(ValidationError::RequiredField { .. }))));
    }
    
    #[test]
    fn test_validate_invalid_integer() {
        let schema = create_test_schema();
        let row = CsvRow {
            key: "1".to_string(),
            values: vec!["invalid".to_string(), "Alice".to_string(), "alice@example.com".to_string()],
        };
        
        let result = validate_row(&row, &schema);
        assert!(matches!(result, Err(ReedError::ValidationError(ValidationError::InvalidInteger { .. }))));
    }
    
    #[test]
    fn test_validate_string_too_short() {
        let schema = create_test_schema();
        let row = CsvRow {
            key: "1".to_string(),
            values: vec!["1".to_string(), "".to_string(), "alice@example.com".to_string()],
        };
        
        let result = validate_row(&row, &schema);
        assert!(matches!(result, Err(ReedError::ValidationError(ValidationError::RequiredField { .. }))));
    }
    
    #[test]
    fn test_validate_string_too_long() {
        let schema = create_test_schema();
        let row = CsvRow {
            key: "1".to_string(),
            values: vec!["1".to_string(), "a".repeat(101), "alice@example.com".to_string()],
        };
        
        let result = validate_row(&row, &schema);
        assert!(matches!(result, Err(ReedError::ValidationError(ValidationError::MaxLength { .. }))));
    }
    
    #[test]
    fn test_validate_pattern_mismatch() {
        let schema = create_test_schema();
        let row = CsvRow {
            key: "1".to_string(),
            values: vec!["1".to_string(), "Alice".to_string(), "invalid-email".to_string()],
        };
        
        let result = validate_row(&row, &schema);
        assert!(matches!(result, Err(ReedError::ValidationError(ValidationError::PatternMismatch { .. }))));
    }
    
    #[test]
    fn test_validate_uniqueness() {
        let schema = create_test_schema();
        let rows = vec![
            CsvRow {
                key: "1".to_string(),
                values: vec!["1".to_string(), "Alice".to_string(), "alice@example.com".to_string()],
            },
            CsvRow {
                key: "2".to_string(),
                values: vec!["1".to_string(), "Bob".to_string(), "bob@example.com".to_string()],
            },
        ];
        
        let result = validate_uniqueness(&rows, &schema);
        assert!(matches!(result, Err(ReedError::ValidationError(ValidationError::DuplicateValue { .. }))));
    }
    
    #[test]
    fn test_validate_uniqueness_success() {
        let schema = create_test_schema();
        let rows = vec![
            CsvRow {
                key: "1".to_string(),
                values: vec!["1".to_string(), "Alice".to_string(), "alice@example.com".to_string()],
            },
            CsvRow {
                key: "2".to_string(),
                values: vec!["2".to_string(), "Bob".to_string(), "bob@example.com".to_string()],
            },
        ];
        
        assert!(validate_uniqueness(&rows, &schema).is_ok());
    }
}
```

**`reedbase/src/schema/loader.test.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_parse_schema() {
        let toml = r#"
            [schema]
            version = "1.0"
            strict = true
            
            [[columns]]
            name = "id"
            type = "integer"
            required = true
            unique = true
            primary_key = true
            
            [[columns]]
            name = "name"
            type = "string"
            required = true
            min_length = 1
            max_length = 100
        "#;
        
        let schema = parse_schema(toml).unwrap();
        
        assert_eq!(schema.version, "1.0");
        assert!(schema.strict);
        assert_eq!(schema.columns.len(), 2);
        assert_eq!(schema.columns[0].name, "id");
        assert_eq!(schema.columns[0].col_type, "integer");
        assert!(schema.columns[0].primary_key);
    }
    
    #[test]
    fn test_create_default_schema() {
        let columns = vec!["id".to_string(), "name".to_string(), "email".to_string()];
        let schema = create_default_schema("users", &columns).unwrap();
        
        assert_eq!(schema.columns.len(), 3);
        assert_eq!(schema.columns[0].name, "id");
        assert_eq!(schema.columns[0].col_type, "string");
        assert!(!schema.strict);
    }
    
    #[test]
    fn test_save_and_load_schema() {
        let _temp_dir = TempDir::new().unwrap();
        
        let columns = vec!["id".to_string(), "name".to_string()];
        let schema = create_default_schema("test_table", &columns).unwrap();
        
        save_schema("test_table", &schema).unwrap();
        let loaded = load_schema("test_table").unwrap();
        
        assert_eq!(loaded.columns.len(), schema.columns.len());
        assert_eq!(loaded.columns[0].name, schema.columns[0].name);
    }
    
    #[test]
    fn test_serialize_schema() {
        let schema = Schema {
            version: "1.0".to_string(),
            strict: true,
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    col_type: "integer".to_string(),
                    required: true,
                    unique: true,
                    primary_key: true,
                    min: Some(1),
                    max: None,
                    min_length: None,
                    max_length: None,
                    pattern: None,
                },
            ],
        };
        
        let toml = serialize_schema(&schema).unwrap();
        
        assert!(toml.contains("version = \"1.0\""));
        assert!(toml.contains("strict = true"));
        assert!(toml.contains("name = \"id\""));
        assert!(toml.contains("type = \"integer\""));
        assert!(toml.contains("primary_key = true"));
    }
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Load schema | < 5ms |
| Validate row | < 1ms |
| Validate 100 rows | < 100ms |
| Check uniqueness (100 rows) | < 10ms |
| Save schema | < 10ms |

## Error Conditions

- **SchemaNotFound**: schema.toml does not exist
- **InvalidSchema**: TOML syntax error or missing required fields
- **ValidationError**: Data violates schema constraints
- **IoError**: Cannot read/write schema file

## CLI Commands

```bash
# Show schema
reed schema:show users

# Validate table against schema
reed schema:validate users

# Create schema from existing CSV
reed schema:create users --from-csv

# Edit schema (opens in editor)
reed schema:edit users
```

## Acceptance Criteria

### Part 1: Key Validation (RBKS v2)
- [ ] Parse key with RBKS v2 regex pattern
- [ ] Validate key structure (lowercase, dots, modifiers)
- [ ] Parse modifiers from `<modifier,modifier>` syntax
- [ ] Classify modifiers into categories (language, environment, season, variant, custom)
- [ ] Reject multiple modifiers of same category (e.g., `<de,en>`)
- [ ] Normalize keys (lowercase, sort modifiers, dedup)
- [ ] Generate fallback chain for modifier resolution
- [ ] Validate depth (2-8 levels)
- [ ] Validate no empty segments or trailing commas
- [ ] Load key_validation config from schema.toml
- [ ] CLI `--normalize` flag for auto-normalization
- [ ] Helpful error messages with suggestions ("Did you mean...?")
- [ ] Key validation < 30μs overhead on SET operations

### Part 2: Column Schema Validation
- [ ] Load schema from TOML file
- [ ] Parse schema with all constraint types
- [ ] Parse key_validation section from schema.toml
- [ ] Validate row against schema
- [ ] Validate string fields (length, pattern)
- [ ] Validate integer fields (range)
- [ ] Validate float fields
- [ ] Validate boolean fields
- [ ] Validate timestamp fields
- [ ] Check uniqueness constraints across all rows
- [ ] Validate multiple rows in batch
- [ ] Create default schema from column names
- [ ] Save schema to TOML file (including key_validation)
- [ ] Serialize schema to TOML format

### Quality
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met (key validation + column validation)
- [ ] All code in BBC English
- [ ] All functions have complete documentation
- [ ] Separate test files: `key_validation.test.rs`, `validation.test.rs`, `loader.test.rs`

## Dependencies

**Requires**: 
- REED-19-02 (Universal Table API - integration point)

**Blocks**: None

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-19-00: Layer Overview

## Notes

### Part 1: RBKS v2 Philosophy

**Why Key Validation?**
- **Enables Indices**: Structured keys = O(1) lookups via NamespaceIndex, LanguageIndex
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

---

### Part 2: Column Schema Philosophy

**Schema Philosophy:**
- **Optional by default**: New tables work without schema (lenient)
- **Enforce when present**: If schema.toml exists, strict validation
- **Auto-generation**: `schema:create` generates schema from existing data

**Validation Levels:**
- **None**: No schema file (all data accepted)
- **Lenient**: Schema exists, `strict = false` (warnings only)
- **Strict**: Schema exists, `strict = true` (reject invalid data)

**Trade-offs:**
- **Pro**: Data quality enforcement at both key and column level
- **Pro**: Type safety at database level
- **Pro**: Self-documenting (schema.toml is documentation)
- **Pro**: Enables high-performance indices (via key structure)
- **Con**: Schema must be maintained (mitigated by auto-generation)
- **Con**: Validation overhead (< 30μs key + < 1ms row = acceptable)

---

### Integration with Indices (REED-19-09)

**Key Validation enables:**
- **NamespaceIndex**: `page.%` → O(1) lookup via namespace="page"
- **LanguageIndex**: `%<de>` → O(1) lookup via language="de"
- **EnvironmentIndex**: `%<prod>` → O(1) lookup via environment="prod"
- **HierarchyTrie**: `page.header.%` → O(d) trie walk where d=depth

**Without key validation:**
- ❌ Index lookups unreliable (keys inconsistent)
- ❌ Queries fall back to O(n) full scans
- ❌ Performance degrades over time

**Future Enhancements:**
- Foreign key constraints (reference other tables)
- Check constraints (custom validation expressions)
- Default values (auto-fill missing fields)
- Schema migration system (version upgrades)
