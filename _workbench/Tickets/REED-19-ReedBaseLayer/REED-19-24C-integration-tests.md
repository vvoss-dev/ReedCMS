# REED-19-24C: Integration Tests

**Parent**: REED-19-24 (High-Level Database API & CLI)  
**Status**: Open  
**Priority**: High  
**Complexity**: Medium  
**Depends On**: REED-19-24A (Database API), REED-19-24B (CLI Tool)  
**Layer**: REED-19 (ReedBase)

## Overview

Comprehensive integration tests for the Database API and CLI tool to ensure correctness, performance, and reliability. Tests verify end-to-end functionality with real databases and measure performance against targets.

## Motivation

**Current State**: Database API and CLI implemented but no automated tests  
**Problem**: Can't verify correctness or detect regressions  
**Solution**: Integration test suite with performance benchmarks

## Goals

1. ✅ **API Integration Tests** - Test Database API with real data
2. ✅ **CLI Integration Tests** - Test reedbase commands
3. ✅ **Performance Tests** - Verify speed targets
4. ✅ **Correctness Tests** - Verify query results
5. ✅ **Concurrency Tests** - Multiple operations
6. ✅ **Error Handling Tests** - Validate error cases

## Non-Goals

- ❌ Unit tests for individual functions (those exist inline)
- ❌ Fuzzing / property testing (nice-to-have)
- ❌ Load testing / stress testing (separate effort)

## Test Categories

### 1. Database API Tests

Test the programmatic Rust API.

#### Test File: `reedbase/tests/database_api_test.rs`

```rust
// Basic Operations
#[test]
fn test_database_open_create() { }

#[test]
fn test_database_create_table() { }

#[test]
fn test_database_insert_query() { }

#[test]
fn test_database_update_query() { }

#[test]
fn test_database_delete_query() { }

// Complex Queries
#[test]
fn test_query_with_where_clause() { }

#[test]
fn test_query_with_like_pattern() { }

#[test]
fn test_query_with_order_by() { }

#[test]
fn test_query_with_limit_offset() { }

#[test]
fn test_query_with_aggregation() { }

// Index Operations
#[test]
fn test_create_index_speeds_up_query() { }

#[test]
fn test_auto_index_creation() { }

#[test]
fn test_list_indices() { }

#[test]
fn test_drop_index() { }

// Concurrency
#[test]
fn test_concurrent_reads() { }

#[test]
fn test_concurrent_writes() { }

#[test]
fn test_read_during_write() { }

// Error Cases
#[test]
fn test_query_nonexistent_table() { }

#[test]
fn test_invalid_sql_syntax() { }

#[test]
fn test_insert_duplicate_key() { }

// Versioning
#[test]
fn test_insert_creates_version() { }

#[test]
fn test_update_creates_delta() { }

#[test]
fn test_rollback_to_version() { }

// Statistics
#[test]
fn test_database_stats_accurate() { }

#[test]
fn test_query_metrics_collected() { }
```

### 2. CLI Integration Tests

Test the command-line tool.

#### Test File: `reedbase/tests/cli_test.rs`

```rust
use assert_cmd::Command;
use predicates::prelude::*;

// Query Command
#[test]
fn test_cli_query_basic() {
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&["query", "SELECT * FROM text LIMIT 5", "test_data/.reed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rows"));
}

#[test]
fn test_cli_query_json_format() { }

#[test]
fn test_cli_query_csv_format() { }

#[test]
fn test_cli_query_to_file() { }

// Exec Command
#[test]
fn test_cli_exec_insert() { }

#[test]
fn test_cli_exec_update() { }

#[test]
fn test_cli_exec_delete() { }

// Shell Command
#[test]
fn test_cli_shell_starts() { }

#[test]
fn test_cli_shell_executes_query() { }

#[test]
fn test_cli_shell_dot_commands() { }

// Tables Command
#[test]
fn test_cli_tables_list() { }

#[test]
fn test_cli_tables_create() { }

#[test]
fn test_cli_tables_drop() { }

// Indices Command
#[test]
fn test_cli_indices_list() { }

#[test]
fn test_cli_indices_create() { }

#[test]
fn test_cli_indices_drop() { }

// Stats Command
#[test]
fn test_cli_stats_display() { }

#[test]
fn test_cli_stats_json() { }

// Explain Command
#[test]
fn test_cli_explain_query() { }

// Error Cases
#[test]
fn test_cli_invalid_path() { }

#[test]
fn test_cli_invalid_sql() { }

#[test]
fn test_cli_missing_arguments() { }
```

### 3. Performance Tests

Verify speed targets are met.

#### Test File: `reedbase/tests/performance_test.rs`

```rust
use std::time::Instant;

// Query Performance
#[test]
fn test_query_with_index_fast() {
    // Target: < 100μs for exact match with index
    let start = Instant::now();
    db.query("SELECT * FROM text WHERE key = 'page.title'").unwrap();
    let duration = start.elapsed();
    assert!(duration.as_micros() < 100);
}

#[test]
fn test_query_range_with_index() {
    // Target: < 1ms for range scan with index
}

#[test]
fn test_query_full_scan_10k_rows() {
    // Target: < 10ms for full scan of 10k rows
}

#[test]
fn test_insert_speed() {
    // Target: < 5ms typical
}

#[test]
fn test_update_speed() {
    // Target: < 10ms typical
}

#[test]
fn test_delete_speed() {
    // Target: < 5ms typical
}

#[test]
fn test_index_creation_10k_rows() {
    // Target: < 50ms for 10k rows
}

#[test]
fn test_database_open_cold_start() {
    // Target: < 100ms with persistent indices
}

// Auto-Indexing
#[test]
fn test_auto_index_triggers_after_threshold() {
    // Execute same query 10x, verify index created
}

// Large Result Sets
#[test]
fn test_query_1000_rows() {
    // Verify correctness and speed
}
```

### 4. Correctness Tests

Verify query results are accurate.

#### Test File: `reedbase/tests/correctness_test.rs`

```rust
// SQL Semantics
#[test]
fn test_where_equals_correct() {
    let result = db.query("SELECT * FROM text WHERE key = 'page.title'").unwrap();
    // Verify only exact match returned
}

#[test]
fn test_where_like_pattern_correct() {
    let result = db.query("SELECT * FROM text WHERE key LIKE '%.@de'").unwrap();
    // Verify all German keys returned
}

#[test]
fn test_order_by_ascending() {
    let result = db.query("SELECT * FROM text ORDER BY key ASC").unwrap();
    // Verify sorted correctly
}

#[test]
fn test_order_by_descending() { }

#[test]
fn test_limit_offset_correct() {
    let result = db.query("SELECT * FROM text LIMIT 5 OFFSET 10").unwrap();
    // Verify exactly 5 rows, starting from 11th
}

#[test]
fn test_count_aggregation() {
    let result = db.query("SELECT COUNT(*) FROM text").unwrap();
    // Verify count matches actual rows
}

#[test]
fn test_sum_aggregation() { }

#[test]
fn test_avg_aggregation() { }

#[test]
fn test_min_max_aggregation() { }

// CRUD Correctness
#[test]
fn test_insert_persists() {
    db.execute("INSERT INTO text (key, value) VALUES ('test', 'value')", "admin").unwrap();
    let result = db.query("SELECT * FROM text WHERE key = 'test'").unwrap();
    // Verify inserted row is queryable
}

#[test]
fn test_update_modifies() {
    db.execute("UPDATE text SET value = 'new' WHERE key = 'test'", "admin").unwrap();
    let result = db.query("SELECT value FROM text WHERE key = 'test'").unwrap();
    // Verify value changed
}

#[test]
fn test_delete_removes() {
    db.execute("DELETE FROM text WHERE key = 'test'", "admin").unwrap();
    let result = db.query("SELECT * FROM text WHERE key = 'test'").unwrap();
    // Verify row no longer exists
}

// Edge Cases
#[test]
fn test_empty_result_set() { }

#[test]
fn test_query_empty_table() { }

#[test]
fn test_special_characters_in_values() { }

#[test]
fn test_very_long_values() { }

#[test]
fn test_unicode_characters() { }
```

### 5. Concurrency Tests

Test thread safety and concurrent operations.

#### Test File: `reedbase/tests/concurrency_test.rs`

```rust
use std::thread;
use std::sync::Arc;

#[test]
fn test_multiple_readers() {
    let db = Arc::new(Database::open("test.reed").unwrap());
    let mut handles = vec![];

    // Spawn 10 threads, each executing 100 queries
    for i in 0..10 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                db_clone.query("SELECT * FROM text LIMIT 10").unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify database state is consistent
}

#[test]
fn test_multiple_writers() {
    // Multiple threads inserting different rows
}

#[test]
fn test_readers_during_writes() {
    // Some threads query, others insert
    // Verify readers see consistent snapshots
}

#[test]
fn test_concurrent_index_creation() {
    // Multiple threads creating different indices
}
```

### 6. Error Handling Tests

Verify proper error messages and recovery.

#### Test File: `reedbase/tests/error_handling_test.rs`

```rust
#[test]
fn test_table_not_found_error() {
    let result = db.query("SELECT * FROM nonexistent");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ReedError::TableNotFound { .. }));
}

#[test]
fn test_invalid_sql_syntax_error() {
    let result = db.query("SELECT FORM text");
    assert!(result.is_err());
}

#[test]
fn test_invalid_column_error() {
    let result = db.query("SELECT nonexistent_column FROM text");
    // Should return empty result or error?
}

#[test]
fn test_insert_invalid_column() {
    let result = db.execute("INSERT INTO text (invalid_col) VALUES ('test')", "admin");
    assert!(result.is_err());
}

#[test]
fn test_index_already_exists_error() {
    db.create_index("text", "key").unwrap();
    let result = db.create_index("text", "key");
    assert!(result.is_err());
}

#[test]
fn test_index_not_found_error() {
    // Try to drop non-existent index
}

#[test]
fn test_corrupted_database_recovery() {
    // Simulate corrupted file, verify error handling
}
```

## Test Data Fixtures

### Fixture: Small Database (100 rows)

```
test_data/small/.reed/
├── tables/
│   └── text/
│       ├── current.csv  # 100 rows
│       └── version.log
```

Used for: Basic correctness tests

### Fixture: Medium Database (10,000 rows)

```
test_data/medium/.reed/
├── tables/
│   ├── text/       # 10,000 rows
│   ├── routes/     # 500 rows
│   └── meta/       # 1,000 rows
```

Used for: Performance tests, index tests

### Fixture: Large Database (100,000 rows)

```
test_data/large/.reed/
├── tables/
│   └── text/       # 100,000 rows
```

Used for: Stress tests, large result set tests

### Fixture: Multi-Version Database

```
test_data/versioned/.reed/
├── tables/
│   └── text/
│       ├── current.csv
│       ├── 1000.bsdiff
│       ├── 2000.bsdiff
│       └── version.log  # 50 versions
```

Used for: Version and rollback tests

## Test Utilities

### Helper Functions

```rust
// test_utils.rs
pub fn create_test_database(name: &str, rows: usize) -> Database { }

pub fn insert_test_data(db: &Database, count: usize) { }

pub fn assert_query_result_count(result: &QueryResult, expected: usize) { }

pub fn assert_execution_time_under(duration: Duration, max_ms: u64) { }

pub fn cleanup_test_database(path: &Path) { }
```

### Test Macros

```rust
macro_rules! assert_query_returns {
    ($db:expr, $sql:expr, $expected:expr) => {
        let result = $db.query($sql).unwrap();
        assert_eq!(result.row_count(), $expected);
    };
}

macro_rules! assert_exec_affects {
    ($db:expr, $sql:expr, $expected:expr) => {
        let result = $db.execute($sql, "test").unwrap();
        assert_eq!(result.rows_affected, $expected);
    };
}
```

## Test Execution

### Run All Tests

```bash
cd reedbase
cargo test --tests
```

### Run Specific Test Suite

```bash
cargo test --test database_api_test
cargo test --test cli_test
cargo test --test performance_test
cargo test --test correctness_test
```

### Run With Output

```bash
cargo test -- --nocapture
```

### Run Performance Tests Only

```bash
cargo test --test performance_test -- --nocapture
```

## Performance Benchmarks

Use criterion for detailed benchmarks.

### Add Dependency

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "query_benchmarks"
harness = false
```

### Benchmark File: `reedbase/benches/query_benchmarks.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use reedbase::Database;

fn bench_query_with_index(c: &mut Criterion) {
    let db = Database::open("bench_data/.reed").unwrap();
    
    c.bench_function("query_exact_match_with_index", |b| {
        b.iter(|| {
            db.query("SELECT * FROM text WHERE key = 'page.title'").unwrap()
        });
    });
}

fn bench_query_full_scan(c: &mut Criterion) {
    let db = Database::open("bench_data/.reed").unwrap();
    
    let mut group = c.benchmark_group("full_scan");
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                db.query(&format!("SELECT * FROM text LIMIT {}", size)).unwrap()
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_query_with_index, bench_query_full_scan);
criterion_main!(benches);
```

### Run Benchmarks

```bash
cargo bench
```

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
      - name: Run benchmarks
        run: cargo bench --no-run
```

## Coverage Target

- **Target**: 80% code coverage
- **Tool**: cargo-tarpaulin or cargo-llvm-cov
- **Command**: `cargo tarpaulin --out Html`

## Acceptance Criteria

- [ ] All API integration tests pass
- [ ] All CLI integration tests pass
- [ ] All performance tests meet targets
- [ ] All correctness tests pass
- [ ] Concurrency tests pass (no data races)
- [ ] Error handling tests pass
- [ ] Test coverage ≥ 80%
- [ ] Benchmarks show expected performance
- [ ] CI/CD pipeline runs tests automatically
- [ ] Test documentation complete

## Future Enhancements

- Property-based testing with proptest
- Fuzzing with cargo-fuzz
- Load testing with realistic workloads
- Memory leak detection with valgrind
- Performance regression tracking

## Related Tickets

- **REED-19-24A**: Database API (completed)
- **REED-19-24B**: CLI Tool (in progress)
- **REED-19-24D**: B+-Tree Integration (next)

## Notes

- Use `tempdir` for test database isolation
- Clean up test databases after tests
- Mock time for deterministic timestamp tests
- Use `assert_cmd` for CLI testing
- Run tests in parallel where possible
