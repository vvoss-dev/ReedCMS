# REED-02-05: Matrix CSV Handler System

**Status**: Open  
**Priority**: High  
**Complexity**: High  
**Layer**: Data Layer  
**Depends on**: REED-02-02 (CSV Handler System)

---

## Objective

Implement Matrix CSV handler system with support for all 4 value types used in complex data structures (users, roles, taxonomy).

---

## Background

Matrix CSV files extend the simple key-value CSV format with structured data types:

1. **Type 1**: Single values (simple text)
2. **Type 2**: Lists (comma-separated values)
3. **Type 3**: Single values with modifiers (value[condition])
4. **Type 4**: Lists with modifiers (item1[cond1],item2[cond2])

Used in:
- `users.matrix.csv` (Type 1 & 2)
- `roles.matrix.csv` (Type 4 - Unix permissions)
- `taxonomy.matrix.csv` (Type 2 & 4)

---

## Requirements

### Core Functionality

1. **MatrixRecord Structure**
   - Support variable number of columns
   - Type-aware value parsing
   - Preserve field order
   - Optional description field

2. **Value Type Parsers**
   - Type 1: Simple string values
   - Type 2: Comma-separated lists
   - Type 3: Values with modifiers `value[modifier]`
   - Type 4: Lists with modifiers `item1[mod1],item2[mod2]`

3. **Modifier Parsing**
   - Extract modifier from brackets: `minify[prod]` → value="minify", modifier="prod"
   - Support multiple modifiers: `file[dev,test]`
   - Unix permission format: `text[rwx]`

4. **Matrix CSV Operations**
   - `read_matrix_csv()`: Load matrix CSV into structured records
   - `write_matrix_csv()`: Write records with atomic operation
   - `parse_matrix_row()`: Parse single row with type detection
   - `create_matrix_row()`: Create row from structured data

---

## Technical Specifications

### Data Structures

```rust
/// Matrix CSV record with variable fields
pub struct MatrixRecord {
    /// Field name → value mapping
    pub fields: HashMap<String, MatrixValue>,
    
    /// Field order (for CSV output)
    pub field_order: Vec<String>,
    
    /// Optional description
    pub description: Option<String>,
}

/// Matrix value types
pub enum MatrixValue {
    /// Type 1: Simple string
    Single(String),
    
    /// Type 2: List of strings
    List(Vec<String>),
    
    /// Type 3: Value with modifier
    Modified(String, Vec<String>), // (value, modifiers)
    
    /// Type 4: List with modifiers
    ModifiedList(Vec<(String, Vec<String>)>), // [(value, modifiers)]
}
```

### File Locations

```
src/reedcms/matrix/
├── mod.rs              # Module exports
├── record.rs           # MatrixRecord + MatrixValue
├── read.rs             # read_matrix_csv()
├── write.rs            # write_matrix_csv()
├── parse.rs            # Value type parsers
├── record_test.rs
├── read_test.rs
├── write_test.rs
└── parse_test.rs
```

---

## Implementation Details

### Type Detection Algorithm

```rust
// Type 4: List with modifiers - item1[mod],item2[mod]
if value.contains(',') && value.contains('[') {
    return MatrixValue::ModifiedList(parse_modified_list(value));
}

// Type 3: Single with modifier - value[mod]
if value.contains('[') && value.contains(']') {
    return MatrixValue::Modified(parse_modified_value(value));
}

// Type 2: List - item1,item2,item3
if value.contains(',') {
    return MatrixValue::List(parse_list(value));
}

// Type 1: Simple value
MatrixValue::Single(value.to_string())
```

### Modifier Parsing

```rust
// Parse "text[rwx]" → ("text", ["rwx"])
// Parse "file[dev,prod]" → ("file", ["dev", "prod"])
fn parse_modifiers(value: &str) -> (String, Vec<String>) {
    if let Some(bracket_pos) = value.find('[') {
        let val = &value[..bracket_pos];
        let mods_str = &value[bracket_pos+1..value.len()-1];
        let mods = mods_str.split(',').map(|s| s.trim().to_string()).collect();
        (val.to_string(), mods)
    } else {
        (value.to_string(), vec![])
    }
}
```

---

## Performance Requirements

- **read_matrix_csv()**: < 20ms for < 1000 rows
- **write_matrix_csv()**: < 20ms for < 1000 rows (with atomic write)
- **parse_matrix_row()**: < 50μs per row
- **Type detection**: O(1) per value

---

## Testing Requirements

### Unit Tests
- Type 1: Simple value parsing
- Type 2: List parsing with various separators
- Type 3: Modifier extraction
- Type 4: Complex list with modifiers
- Edge cases: Empty modifiers, nested brackets
- Round-trip: Parse → Create → Parse equality

### Integration Tests
- Read/write matrix CSV files
- Preserve field order
- Atomic write verification
- Backup integration

### Performance Tests
- 1000 row parsing benchmark
- Type detection performance
- Memory usage validation

---

## Examples

### Type 1: Simple Values
```csv
username|status|desc
admin|active|System Administrator
editor|inactive|Content Editor
```

### Type 2: Lists
```csv
username|roles|desc
jane|editor,author|Multi-role user
admin|admin,editor,author|Full access
```

### Type 3: Values with Modifiers
```csv
asset|optimization|desc
main.css|minify[prod]|Main stylesheet
app.js|bundle[prod,test]|Application code
```

### Type 4: Lists with Modifiers (Unix Permissions)
```csv
rolename|permissions|desc
editor|text[rwx],route[rw-],content[rw-]|Standard Editor
admin|*[rwx]|Full Administrator
viewer|text[r--],route[r--]|Read-only access
```

---

## Acceptance Criteria

- [x] MatrixRecord structure implemented with HashMap fields
- [x] MatrixValue enum with all 4 types
- [x] Type detection algorithm working
- [x] Modifier parser extracting brackets correctly
- [x] read_matrix_csv() loads files into MatrixRecord vector
- [x] write_matrix_csv() with atomic operation
- [x] All 4 value types parse correctly
- [x] Round-trip preservation (parse → write → parse)
- [x] Performance benchmarks meet targets
- [x] 100% test coverage
- [x] Integration with backup system

---

## Dependencies

- csv crate (already included)
- Backup system (REED-02-04)
- ReedStream error types (REED-01-02)

---

## Related Tickets

- REED-02-02: CSV Handler System (dependency)
- REED-03-01: User Management (will use Type 1 & 2)
- REED-03-02: Role Permissions (will use Type 4)
- REED-03-03: Taxonomy System (will use Type 2 & 4)
