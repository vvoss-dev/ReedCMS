# ReedBase Test Fixtures

This document describes the test fixture generation system for ReedBase integration and performance testing.

## Overview

Test fixtures are standardised databases with known data characteristics, used for:
- Performance benchmarking
- Integration testing
- Stress testing
- Version control testing

## Fixture Types

| Fixture | Rows | Purpose | Location |
|---------|------|---------|----------|
| **small** | 100 | Unit tests, quick validation | `test_data/small/.reed` |
| **medium** | 10,000 | Integration tests, moderate load | `test_data/medium/.reed` |
| **large** | 50,000 | Performance tests, stress testing | `test_data/large/.reed` |
| **versioned** | 10 rows, 50 versions | Version control testing | `test_data/versioned/.reed` |

## Generation

### Generate Default Fixtures

Generates `small` and `versioned` fixtures (recommended for most development):

```bash
cargo run --bin generate_fixtures
```

### Generate Specific Fixtures

```bash
# Single fixture
cargo run --bin generate_fixtures small
cargo run --bin generate_fixtures medium
cargo run --bin generate_fixtures large
cargo run --bin generate_fixtures versioned

# Multiple fixtures
cargo run --bin generate_fixtures small medium

# All fixtures
cargo run --bin generate_fixtures all
```

## Fixture Details

### Small Fixture (100 rows)
- **Rows**: 100
- **Table**: `text`
- **Schema**: `key|value|description`
- **Keys**: `test.key.000000` through `test.key.000099`
- **Values**: `test value 0` through `test value 99`
- **Generation time**: ~1 second
- **Use case**: Fast unit tests, CI/CD pipeline tests

### Medium Fixture (10,000 rows)
- **Rows**: 10,000
- **Table**: `text`
- **Schema**: Same as small
- **Keys**: `test.key.000000` through `test.key.009999`
- **Generation time**: ~10-15 seconds
- **Use case**: Integration tests, moderate query performance testing

### Large Fixture (50,000 rows)
- **Rows**: 50,000
- **Table**: `text`
- **Schema**: Same as small
- **Keys**: `test.key.000000` through `test.key.049999`
- **Generation time**: ~60-90 seconds
- **Use case**: Stress testing, full-scan performance, index creation benchmarks

### Versioned Fixture (50 versions)
- **Initial rows**: 10
- **Versions**: 50 (created by UPDATE operations)
- **Table**: `text`
- **Schema**: Same as small
- **Keys**: `test.key.000000` through `test.key.000009`
- **Version pattern**: Cycles through 10 rows, updating each 5 times
- **Users**: `user0` through `user4` (rotating)
- **Generation time**: ~5 seconds
- **Use case**: Version control testing, rollback testing, delta compression validation

## Implementation Details

### Directory Structure
```
test_data/
├── small/
│   └── .reed/
│       ├── registry/
│       └── tables/
│           └── text/
│               ├── current.csv
│               ├── version.log
│               └── *.bsdiff
├── medium/
│   └── .reed/
├── large/
│   └── .reed/
└── versioned/
    └── .reed/
```

### Cleanup Behaviour
The generator automatically removes existing fixtures before regeneration to ensure clean state.

### Batch Insertion
- Uses 1,000-row batches for efficient insertion
- Progress reporting every 10,000 rows
- Single transaction per INSERT for version control

## Git Ignore

Test fixtures are excluded from version control (`.gitignore`):
```
test_data/
```

Each developer must generate fixtures locally.

## Usage in Tests

### Example: Performance Test
```rust
use reedbase::Database;

#[test]
fn test_large_query_performance() {
    let db = Database::open("test_data/large/.reed").expect("Open failed");
    
    let start = Instant::now();
    let result = db.query("SELECT * FROM text WHERE key LIKE 'test.key.001%'")
        .expect("Query failed");
    let duration = start.elapsed();
    
    assert!(duration < Duration::from_millis(50), "Query too slow: {:?}", duration);
}
```

### Example: Version Control Test
```rust
use reedbase::Database;
use reedbase::tables::Table;

#[test]
fn test_rollback() {
    let db = Database::open("test_data/versioned/.reed").expect("Open failed");
    let table = Table::new("test_data/versioned/.reed", "text");
    
    let versions = table.list_versions().expect("List failed");
    assert!(versions.len() >= 50, "Not enough versions");
    
    // Test rollback to specific version
    let target = versions[10].timestamp;
    table.rollback(target, "admin").expect("Rollback failed");
}
```

## Regeneration

Regenerate fixtures when:
- Schema changes in ReedBase
- Test data requirements change
- Fixtures become corrupted
- After major refactoring

```bash
# Clean and regenerate all
rm -rf test_data/
cargo run --bin generate_fixtures all
```

## Performance Notes

| Operation | Small | Medium | Large |
|-----------|-------|--------|-------|
| Generation | ~1s | ~15s | ~90s |
| Full scan | <1ms | ~5ms | ~20ms |
| Indexed query | <1ms | <1ms | <1ms |
| Disk size | ~50KB | ~5MB | ~25MB |

## Troubleshooting

### "Table already exists" Error
**Solution**: The generator now automatically cleans up existing fixtures. If you still see this error, manually remove:
```bash
rm -rf test_data/
```

### Slow Generation
**Solution**: Large fixture generation is CPU-intensive due to versioning. This is expected behaviour.

### Missing Fixtures
**Solution**: Run the generator before tests:
```bash
cargo run --bin generate_fixtures
cargo test
```

## See Also

- `src/bin/generate_fixtures.rs` - Implementation
- `tests/performance_test.rs` - Performance benchmarks using fixtures
- `tests/database_api_test.rs` - Integration tests using fixtures
