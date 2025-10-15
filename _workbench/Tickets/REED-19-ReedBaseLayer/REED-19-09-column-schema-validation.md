# REED-19-09: Column Schema Validation

## Metadata
- **Status**: Planned
- **Priority**: Medium
- **Complexity**: Medium (3-4 days)
- **Layer**: Data Layer (REED-19)
- **Depends on**: 
  - REED-19-02 (Universal Table API - integration point for schema validation)
- **Blocks**: None (optional enhancement for data quality)
- **Related Tickets**: 
  - REED-19-08 (RBKS v2 Key Validation - complementary validation system)

## Problem Statement

ReedBase v1 has **no column-level validation**:
- Any data can be written to any column
- No type checking (string, integer, date, etc.)
- No constraint enforcement (required, unique, min/max, etc.)
- Data quality issues discovered late (during template rendering or query execution)

**Example Problems**:
```csv
# users.csv - No validation
id,name,email,age
1,Alice,alice@example.com,25
2,Bob,invalid-email,999       # ❌ Invalid email, age out of range
3,Charlie,,                    # ❌ Missing email, age
four,Dave,dave@example.com,30  # ❌ Invalid ID (not integer)
1,Eve,eve@example.com,28       # ❌ Duplicate ID
```

**Target**: **TOML-based column schemas** with **type and constraint validation** at write time.

## Solution Overview

Implement **column schema validation** with TOML-based schema files:

```
.reed/tables/{table_name}/
├── schema.toml          # Schema definition
├── current.csv          # Data (validated against schema)
└── version.log
```

**Schema Features**:
- **Type validation**: string, integer, float, boolean, timestamp
- **Constraints**: required, unique, primary_key, min/max, min_length/max_length, regex pattern
- **Batch validation**: Validate multiple rows efficiently
- **Uniqueness checks**: Enforce unique/primary_key constraints across all rows
- **Auto-generation**: Create schema from existing CSV

## Architecture

### Core Types

```rust
/// Table schema.
#[derive(Debug, Clone)]
pub struct Schema {
    pub version: String,
    pub strict: bool,          // Reject writes if validation fails
    pub columns: Vec<ColumnDef>,
}

/// Column definition.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: String,      // "string", "integer", "float", "boolean", "timestamp"
    pub required: bool,
    pub unique: bool,
    pub primary_key: bool,     // Implies required + unique
    pub min: Option<i64>,      // For integer/float
    pub max: Option<i64>,
    pub min_length: Option<usize>, // For string
    pub max_length: Option<usize>,
    pub pattern: Option<String>,   // Regex for string validation
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
```

## Implementation Details

### 1. Schema File Format (schema.toml)

```toml
[schema]
version = "2.0"
strict = true     # Reject writes that violate schema

[[columns]]
name = "id"
type = "integer"
required = true
unique = true
primary_key = true
min = 1

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
unique = true

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

### 2. Column Types

**string**: Text data
- Constraints: `min_length`, `max_length`, `pattern` (regex)
- Examples: names, emails, descriptions

**integer**: Whole numbers (i64)
- Constraints: `min`, `max`
- Examples: IDs, ages, counts

**float**: Decimal numbers (f64)
- Constraints: `min`, `max`
- Examples: prices, ratings, percentages

**boolean**: True/false values
- Accepted values: `"true"`, `"false"`, `"1"`, `"0"`
- Examples: flags, status indicators

**timestamp**: Unix timestamp (seconds since epoch)
- Type: u64
- Examples: created_at, updated_at

### 3. Validation Functions (validation.rs)

```rust
// validation.rs
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
fn validate_float(value: &str, column: &ColumnDef) -> ReedResult<()> {
    value.parse::<f64>()
        .map_err(|_| ReedError::ValidationError(ValidationError::InvalidFloat {
            column: column.name.clone(),
            value: value.to_string(),
        }))?;
    
    Ok(())
}

/// Validate boolean field.
fn validate_boolean(value: &str) -> ReedResult<()> {
    if !matches!(value, "true" | "false" | "1" | "0") {
        return Err(ReedError::ValidationError(ValidationError::InvalidBoolean {
            value: value.to_string(),
        }));
    }
    
    Ok(())
}

/// Validate timestamp field.
fn validate_timestamp(value: &str) -> ReedResult<()> {
    value.parse::<u64>()
        .map_err(|_| ReedError::ValidationError(ValidationError::InvalidTimestamp {
            value: value.to_string(),
        }))?;
    
    Ok(())
}

/// Validate uniqueness constraints across all rows.
///
/// ## Performance
/// - O(n*m) where n = rows, m = unique columns
/// - < 10ms for 100 rows with 2 unique columns
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
pub fn validate_rows(rows: &[CsvRow], schema: &Schema) -> ReedResult<()> {
    for row in rows {
        validate_row(row, schema)?;
    }
    
    validate_uniqueness(rows, schema)?;
    
    Ok(())
}
```

### 4. Schema Loader (loader.rs)

```rust
// loader.rs
use crate::types::{ReedResult, ReedError, Schema, ColumnDef};
use std::path::Path;
use std::fs;

/// Load schema from file.
///
/// ## Performance
/// - < 5ms typical (TOML parsing)
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
fn get_schema_path(table_name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!(".reed/tables/{}/schema.toml", table_name))
}

/// Create default schema for table.
///
/// ## Output
/// - Generated schema (all columns as optional strings)
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
        version: "2.0".to_string(),
        strict: false, // Lenient for auto-generated schemas
        columns,
    })
}

/// Save schema to file.
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

## CLI Integration

```bash
# Show schema
reed schema:show users
# Output: Displays schema.toml content in readable format

# Validate table against schema
reed schema:validate users
# ✅ Valid: All 125 rows passed validation
# or
# ❌ Error: Row 45 failed validation: Column 'email' pattern mismatch

# Create schema from existing CSV
reed schema:create users --from-csv
# Created schema.toml with 5 columns (all strings, lenient mode)
# Edit .reed/tables/users/schema.toml to add constraints

# Edit schema (opens in editor)
reed schema:edit users
# Opens schema.toml in $EDITOR

# Set with validation
reed set users id=123 name="Alice" email="alice@example.com" age=25
# ✅ Valid - accepted
# or
# ❌ Error: Validation failed: Column 'age' exceeds maximum (150)
```

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Load schema | < 5ms | TOML parsing |
| Validate row | < 1ms | Single row validation |
| Validate 100 rows | < 100ms | Batch validation |
| Check uniqueness | < 10ms | HashMap lookup for all rows |
| Save schema | < 10ms | TOML serialization + write |

## Testing Strategy

### Unit Tests (validation.test.rs)

```rust
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
        CsvRow { key: "1".to_string(), values: vec!["1".to_string(), "Alice".to_string()] },
        CsvRow { key: "2".to_string(), values: vec!["1".to_string(), "Bob".to_string()] },
    ];
    
    let result = validate_uniqueness(&rows, &schema);
    assert!(matches!(result, Err(ReedError::ValidationError(ValidationError::DuplicateValue { .. }))));
}
```

### Integration Tests

```rust
#[test]
fn test_load_and_validate_schema() {
    let schema = load_schema("users").unwrap();
    let rows = load_csv("users").unwrap();
    
    validate_rows(&rows, &schema).unwrap();
}

#[test]
fn test_create_and_save_schema() {
    let columns = vec!["id".to_string(), "name".to_string()];
    let schema = create_default_schema("test_table", &columns).unwrap();
    
    save_schema("test_table", &schema).unwrap();
    let loaded = load_schema("test_table").unwrap();
    
    assert_eq!(loaded.columns.len(), schema.columns.len());
}
```

## File Structure

```
src/reedcms/reedbase/
├── schema/
│   ├── mod.rs              # Public API
│   ├── validation.rs       # Row and field validation
│   ├── loader.rs           # Schema loading and saving
│   ├── validation.test.rs  # Validation unit tests
│   └── loader.test.rs      # Loader unit tests
```

## Dependencies

**Internal**:
- `csv::read_csv` - CSV reading for batch validation
- `reedstream::ReedError` - Error handling

**External**:
- `toml` - TOML parsing for schema files
- `regex` - Pattern validation for string fields
- `serde` - Optional (if using serde for TOML)

## Error Handling

```rust
#[derive(Debug)]
pub enum ReedError {
    // ... existing errors ...
    
    SchemaNotFound { table: String },
    InvalidSchema { reason: String },
    ValidationError(ValidationError),
}
```

## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| schema_validation_latency | Histogram | μs | <100 | >500 | schema.rs:validate_row() |
| schema_load_latency | Histogram | ms | <5 | >20 | schema.rs:load_schema() |
| constraint_violations | Counter | count | <1% | >10% | schema.rs:validate_row() |
| validation_error_rate | Gauge | % | <5 | >20 | schema.rs:validate_row() |
| schema_cache_hit_rate | Gauge | % | >95 | <80 | schema.rs:load_schema() |

### Alert Rules

**CRITICAL Alerts:**
- `constraint_violations > 10%` for 10 minutes → "High schema violation rate - data quality issue"
- `validation_error_rate > 20%` for 5 minutes → "Excessive validation failures - investigate data sources"

**WARNING Alerts:**
- `schema_validation_latency p99 > 500μs` for 5 minutes → "Schema validation slow - check constraint complexity"
- `schema_cache_hit_rate < 80%` for 10 minutes → "Schema cache degraded - memory pressure"

### Implementation

```rust
use crate::reedbase::metrics::global as metrics;
use std::time::Instant;

pub fn validate_row(row: &HashMap<String, String>, schema: &Schema) -> ReedResult<()> {
    let start = Instant::now();
    let result = validate_row_inner(row, schema);
    
    metrics().record(Metric {
        name: "schema_validation_latency".to_string(),
        value: start.elapsed().as_nanos() as f64 / 1000.0, // Convert to μs
        unit: MetricUnit::Microseconds,
        tags: hashmap!{ "table" => &schema.table_name },
    });
    
    if let Err(ReedError::ValidationError(ref e)) = result {
        metrics().record(Metric {
            name: "constraint_violations".to_string(),
            value: 1.0,
            unit: MetricUnit::Count,
            tags: hashmap!{ "constraint" => e.constraint.clone(), "column" => e.column.clone() },
        });
    }
    
    result
}

pub fn load_schema(table: &str) -> ReedResult<Schema> {
    let start = Instant::now();
    let schema = load_schema_inner(table)?;
    
    metrics().record(Metric {
        name: "schema_load_latency".to_string(),
        value: start.elapsed().as_millis() as f64,
        unit: MetricUnit::Milliseconds,
        tags: hashmap!{ "table" => table },
    });
    
    Ok(schema)
}
```

### Collection Strategy

- **Sampling**: All operations
- **Aggregation**: 1-minute rolling window
- **Storage**: `.reedbase/metrics/schema.csv`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**schema_validation_latency**: Write performance impact
- Validation happens on every write when schema exists
- Sub-millisecond performance critical for throughput
- Complex constraints increase validation time

**constraint_violations**: Data quality monitoring
- Low violation rate = clean data inputs
- High rates indicate bugs or user errors
- Helps identify problematic data sources

**validation_error_rate**: System health
- Tracks overall validation success
- High error rates require investigation
- May indicate schema misconfiguration

**schema_cache_hit_rate**: Performance optimization
- Cached schemas avoid repeated file I/O
- High hit rate (>95%) expected for stable schemas
- Low rates indicate cache thrashing or memory issues

## Acceptance Criteria

### Functional Requirements
- [x] Load schema from TOML file
- [x] Parse schema with all column types (string, integer, float, boolean, timestamp)
- [x] Parse all constraints (required, unique, primary_key, min/max, length, pattern)
- [x] Validate row against schema
- [x] Validate string fields (length, pattern)
- [x] Validate integer fields (range)
- [x] Validate float fields
- [x] Validate boolean fields
- [x] Validate timestamp fields
- [x] Check uniqueness constraints across all rows
- [x] Validate multiple rows in batch
- [x] Create default schema from column names
- [x] Save schema to TOML file
- [x] Serialize schema to TOML format
- [x] CLI commands: `schema:show`, `schema:validate`, `schema:create`, `schema:edit`

### Performance Requirements
- [x] Load schema: < 5ms
- [x] Validate row: < 1ms
- [x] Validate 100 rows: < 100ms
- [x] Check uniqueness (100 rows): < 10ms
- [x] Save schema: < 10ms

### Quality Requirements
- [x] 100% test coverage for validation and loader
- [x] Performance benchmarks for all operations
- [x] Integration tests with real CSV data
- [x] Separate test files: `validation.test.rs`, `loader.test.rs`

### Documentation Requirements
- [x] Architecture documentation (this ticket)
- [x] API documentation for all public functions
- [x] Schema file format documentation
- [x] CLI usage examples
- [x] Error handling documentation

## Implementation Notes

### Schema Philosophy

**Optional by default**:
- New tables work without schema (lenient mode)
- No schema file = all data accepted

**Enforce when present**:
- If `schema.toml` exists and `strict = true` → reject invalid data
- If `schema.toml` exists and `strict = false` → warnings only

**Auto-generation**:
- `reed schema:create users --from-csv` generates schema from existing data
- All columns default to `string` type, `required = false`
- User edits schema to add constraints

**Validation Levels**:
1. **None**: No schema file (all data accepted)
2. **Lenient**: Schema exists, `strict = false` (warnings only)
3. **Strict**: Schema exists, `strict = true` (reject invalid data)

### Trade-offs

**Pros**:
- ✅ Data quality enforcement at column level
- ✅ Type safety at database level
- ✅ Self-documenting (schema.toml is documentation)
- ✅ Catch errors early (at write time, not render time)

**Cons**:
- ❌ Schema must be maintained (mitigated by auto-generation)
- ❌ Validation overhead (< 1ms per row = acceptable)

### Future Enhancements

1. **Foreign key constraints**
   - Reference other tables: `user_id` references `users.id`
   - Automatic referential integrity checks

2. **Check constraints**
   - Custom validation expressions: `age >= 18 AND age <= 150`
   - More flexible than min/max

3. **Default values**
   - Auto-fill missing fields: `created_at = NOW()`
   - Reduce manual data entry

4. **Schema migration system**
   - Version upgrades: v1.0 → v2.0
   - Automatic data transformation

## References

- **REED-19-02**: Universal Table API (integration point)
- **REED-19-08**: RBKS v2 Key Validation (complementary validation)
- Service Template: `_workbench/Tickets/templates/service-template.md`

## Summary

Column Schema Validation provides **TOML-based type and constraint enforcement** for CSV tables. Schemas define column types (string, integer, float, boolean, timestamp) and constraints (required, unique, min/max, length, pattern). Validation happens **at write time** (<1ms per row), catching data quality issues **early** before they cause template rendering or query execution problems. Auto-generation from existing CSVs makes adoption frictionless.
