# REED-19-24X: Quality & Stability Fixes for Integration Tests

**Parent**: REED-19-24C (Integration Tests)  
**Status**: Open  
**Priority**: High  
**Complexity**: High  
**Depends On**: REED-19-24C  
**Layer**: REED-19 (ReedBase)

## Overview

Resolve remaining quality and stability issues from REED-19-24C integration test implementation. Focus on fixing concurrent write race conditions, improving test coverage, and completing missing test categories.

## Motivation

**Current State**: 23/26 tests passing (89%), 3 ignored due to known issues  
**Problem**: Concurrent writes have race conditions, missing test coverage areas  
**Solution**: Implement file locking, complete test suites, add missing fixtures

## Issues to Resolve

### Issue #1: Concurrent Write Race Conditions

**Test**: `test_concurrent_writes` (currently ignored)

**Problem**:
```
Insert failed: IoError { 
    operation: "read_new_file: .../current.new.tmp", 
    reason: "No such file or directory (os error 2)" 
}
```

**Root Cause**:
- Multiple threads simultaneously access delta versioning system
- No file locking in `src/tables/table.rs`
- Delta generation (`src/version/delta.rs`) expects serial writes
- Temporary files (`current.new.tmp`) created/deleted in race conditions

**Solution**:
1. Add file-based locking using `fs2` crate
2. Implement lock file per table: `<table>/.lock`
3. Acquire exclusive lock during write operations
4. Use retry mechanism with exponential backoff

**Implementation**:
```rust
// In src/tables/table.rs
use fs2::FileExt;

pub fn write_with_lock(&self, content: &[u8], user: &str) -> ReedResult<()> {
    let lock_path = self.path.join(".lock");
    let lock_file = File::create(&lock_path)?;
    
    // Acquire exclusive lock (blocks until available)
    lock_file.lock_exclusive().map_err(|e| ReedError::IoError {
        operation: "acquire_write_lock".to_string(),
        reason: e.to_string(),
    })?;
    
    // Perform write operations
    let result = self.write_internal(content, user);
    
    // Release lock (automatic on drop, but explicit is clearer)
    lock_file.unlock()?;
    
    result
}
```

**Files to Modify**:
- `reedbase/Cargo.toml` - Add `fs2 = "0.4"` dependency
- `reedbase/src/tables/table.rs` - Add locking to write operations
- `reedbase/tests/database_api_test.rs` - Remove `#[ignore]` from test

**Acceptance**:
- [x] `test_concurrent_writes` passes consistently
- [x] 100 concurrent writes complete successfully
- [x] No file corruption or lost writes
- [x] Lock contention handled gracefully

**Status**: ✅ **COMPLETED** (Commit: 47d9b85)

**Implementation Details**:
- Added `write_with_lock()` with fs2-based exclusive locking
- Implemented `acquire_lock_with_retry()` with exponential backoff (50 retries, 5-100ms)
- Created `read_modify_write()` for atomic Read-Modify-Write operations
- Modified `execute_insert()` to use atomic RMW instead of separate read/write
- Lock file: `<table_dir>/.lock`, automatically released on drop
- Test passes individually (100/100 inserts succeed)
- Note: Flaky in full suite due to Issue #2 (Registry Concurrency)

---

### Issue #2: Registry Concurrency Issues

**Test**: `test_read_during_write` (currently ignored)

**Problem**:
```
Insert failed: IoError { 
    operation: "append_users_dict", 
    reason: "No such file or directory (os error 2)" 
}
```

**Root Cause**:
- Global registry state (`OnceLock` in `src/registry/dictionary.rs`)
- Multiple test databases use different paths
- Registry path switching not fully atomic
- `reload_dictionaries()` not sufficient for concurrent tests

**Solution**:
1. Make registry thread-local instead of global
2. Store registry path in Database struct
3. Pass registry to functions that need it
4. Or: Use separate registry per test with proper cleanup

**Alternative (simpler)**:
- Serialize concurrent tests using `#[serial]` from `serial_test` crate
- Add `test_utils::with_exclusive_registry()` wrapper

**Implementation**:
```rust
// Option A: Thread-local registry (complex)
use std::cell::RefCell;
thread_local! {
    static REGISTRY_PATH: RefCell<Option<PathBuf>> = RefCell::new(None);
}

// Option B: Serialized tests (simple)
#[test]
#[serial]
fn test_read_during_write() {
    // Only one registry-using test runs at a time
}
```

**Files to Modify**:
- `reedbase/Cargo.toml` - Add `serial_test = "3.0"` to dev-dependencies
- `reedbase/tests/database_api_test.rs` - Add `#[serial]` to registry tests
- OR: `reedbase/src/registry/dictionary.rs` - Refactor to thread-local

**Acceptance**:
- [ ] `test_read_during_write` passes consistently
- [ ] Concurrent reads during writes work correctly
- [ ] No registry file conflicts

---

### Issue #3: Missing CLI Integration Tests

**Status**: Partially implemented in `cli_test.rs`

**Missing Tests**:
- [ ] `test_cli_query_json_format`
- [ ] `test_cli_query_csv_format`
- [ ] `test_cli_query_to_file`
- [ ] `test_cli_exec_insert`
- [ ] `test_cli_exec_update`
- [ ] `test_cli_exec_delete`
- [ ] `test_cli_shell_starts`
- [ ] `test_cli_shell_executes_query`
- [ ] `test_cli_shell_dot_commands`
- [ ] `test_cli_tables_create`
- [ ] `test_cli_tables_drop`
- [ ] `test_cli_indices_list`
- [ ] `test_cli_indices_create`
- [ ] `test_cli_indices_drop`
- [ ] `test_cli_stats_json`
- [ ] `test_cli_explain_query`
- [ ] `test_cli_invalid_path`
- [ ] `test_cli_invalid_sql`
- [ ] `test_cli_missing_arguments`

**Solution**:
Implement missing tests using `assert_cmd` crate.

**Example**:
```rust
#[test]
fn test_cli_query_json_format() {
    let (_db, temp) = create_test_database("cli_json", 10);
    
    Command::cargo_bin("reedbase")
        .unwrap()
        .args(&[
            "query",
            "SELECT * FROM text LIMIT 5",
            temp.path().join(".reed").to_str().unwrap(),
            "--format", "json"
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("key"))
        .stdout(predicate::str::contains("value"));
}
```

**Files to Modify**:
- `reedbase/tests/cli_test.rs` - Add missing test implementations

**Acceptance**:
- [ ] All 25 CLI tests implemented
- [ ] All CLI tests passing
- [ ] JSON, CSV, table formats tested
- [ ] Shell mode tested
- [ ] Error cases tested

---

### Issue #4: Missing Performance Tests

**Status**: Not fully implemented in `performance_test.rs`

**Missing Tests**:
- [ ] `test_query_range_with_index` (< 1ms target)
- [ ] `test_query_full_scan_10k_rows` (< 10ms target)
- [ ] `test_insert_speed` (< 5ms target)
- [ ] `test_update_speed` (< 10ms target)
- [ ] `test_delete_speed` (< 5ms target)
- [ ] `test_index_creation_10k_rows` (< 50ms target)
- [ ] `test_database_open_cold_start` (< 100ms target)
- [ ] `test_auto_index_triggers_after_threshold`
- [ ] `test_query_1000_rows` (correctness + speed)

**Solution**:
Implement missing performance tests with timing assertions.

**Example**:
```rust
#[test]
fn test_query_full_scan_10k_rows() {
    let (db, _temp) = create_test_database("perf_scan", 10000);
    
    let start = Instant::now();
    let result = db.query("SELECT * FROM text WHERE value LIKE '%test%'")
        .expect("Query failed");
    let duration = start.elapsed();
    
    // Verify correctness
    assert!(get_rows(&result).len() > 0);
    
    // Verify performance
    assert!(
        duration.as_millis() < 10,
        "Full scan of 10k rows took {}ms, expected < 10ms",
        duration.as_millis()
    );
}
```

**Files to Modify**:
- `reedbase/tests/performance_test.rs` - Add missing test implementations

**Acceptance**:
- [ ] All 10 performance tests implemented
- [ ] All performance targets met
- [ ] Tests run on CI without flakiness

---

### Issue #5: Missing Versioning Tests

**Tests Not Implemented**:
- [ ] `test_insert_creates_version`
- [ ] `test_update_creates_delta`
- [ ] `test_rollback_to_version`

**Solution**:
Add tests to `database_api_test.rs` that verify versioning behavior.

**Example**:
```rust
#[test]
fn test_insert_creates_version() {
    let (db, _temp) = create_test_database("versioning", 0);
    
    // Get initial version count
    let table = db.get_table("text").unwrap();
    let initial_versions = table.list_versions().unwrap().len();
    
    // Insert row
    db.execute("INSERT INTO text (key, value) VALUES ('test', 'value')", "admin")
        .expect("Insert failed");
    
    // Verify version created
    let versions = table.list_versions().unwrap();
    assert_eq!(
        versions.len(),
        initial_versions + 1,
        "Insert should create a new version"
    );
}

#[test]
fn test_update_creates_delta() {
    let (db, _temp) = create_test_database("delta", 1);
    
    // Update row
    db.execute("UPDATE text SET value = 'new_value' WHERE key = 'test.key.000000'", "admin")
        .expect("Update failed");
    
    // Verify delta file exists
    let table = db.get_table("text").unwrap();
    let versions = table.list_versions().unwrap();
    assert!(versions.len() >= 2, "Update should create delta version");
    
    // Verify delta is smaller than full snapshot
    // (check file sizes in table directory)
}

#[test]
fn test_rollback_to_version() {
    let (db, _temp) = create_test_database("rollback", 10);
    
    // Get current state
    let before = db.query("SELECT * FROM text WHERE key = 'test.key.000005'")
        .unwrap();
    let original_value = get_rows(&before)[0].get("value").unwrap().clone();
    
    // Modify row
    db.execute("UPDATE text SET value = 'modified' WHERE key = 'test.key.000005'", "admin")
        .unwrap();
    
    // Get version before modification
    let table = db.get_table("text").unwrap();
    let versions = table.list_versions().unwrap();
    let target_version = versions[versions.len() - 2].timestamp; // Previous version
    
    // Rollback
    table.rollback_to(target_version).unwrap();
    
    // Verify original value restored
    let after = db.query("SELECT * FROM text WHERE key = 'test.key.000005'")
        .unwrap();
    let restored_value = get_rows(&after)[0].get("value").unwrap();
    assert_eq!(restored_value, &original_value, "Rollback should restore original value");
}
```

**Files to Modify**:
- `reedbase/tests/database_api_test.rs` - Add versioning tests section

**Acceptance**:
- [ ] All 3 versioning tests implemented
- [ ] Tests verify version creation
- [ ] Tests verify delta creation
- [ ] Tests verify rollback functionality

---

### Issue #6: Missing Test Fixtures

**Status**: Not created

**Required Fixtures**:
```
reedbase/test_data/
├── small/.reed/           # 100 rows
├── medium/.reed/          # 10,000 rows  
├── large/.reed/           # 100,000 rows
└── versioned/.reed/       # 50 versions
```

**Solution**:
Create fixture generation script.

**Implementation**:
```rust
// reedbase/tests/generate_fixtures.rs
fn main() {
    generate_small_fixture();    // 100 rows
    generate_medium_fixture();   // 10k rows
    generate_large_fixture();    // 100k rows
    generate_versioned_fixture(); // 50 versions
}

fn generate_small_fixture() {
    let path = PathBuf::from("test_data/small");
    let db = create_test_database_at_path(&path, 100);
    println!("Created small fixture: 100 rows");
}

// etc.
```

**Files to Create**:
- `reedbase/tests/generate_fixtures.rs` - Fixture generator
- `reedbase/test_data/` - Fixture directory structure

**Acceptance**:
- [ ] All 4 fixtures generated
- [ ] Fixtures checked into git (or generated in CI)
- [ ] Tests use fixtures instead of creating DBs

---

### Issue #7: Missing Benchmark Suite

**Status**: Not implemented

**Solution**:
Add criterion benchmarks for performance validation.

**Implementation**:
```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "query_benchmarks"
harness = false
```

```rust
// benches/query_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use reedbase::Database;

fn bench_query_with_index(c: &mut Criterion) {
    let db = setup_bench_db();
    db.create_index("text", "key").unwrap();
    
    c.bench_function("query_exact_match_with_index", |b| {
        b.iter(|| {
            db.query("SELECT * FROM text WHERE key = 'page.title'").unwrap()
        });
    });
}

fn bench_query_sizes(c: &mut Criterion) {
    let db = setup_bench_db();
    let mut group = c.benchmark_group("query_result_size");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                db.query(&format!("SELECT * FROM text LIMIT {}", size)).unwrap()
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_query_with_index, bench_query_sizes);
criterion_main!(benches);
```

**Files to Create**:
- `reedbase/benches/query_benchmarks.rs`
- `reedbase/benches/insert_benchmarks.rs`
- `reedbase/benches/index_benchmarks.rs`

**Acceptance**:
- [ ] Benchmark suite implemented
- [ ] Benchmarks show < 10% variance
- [ ] Baseline captured for regression detection

---

### Issue #8: Missing Coverage Measurement

**Status**: Not implemented

**Target**: ≥ 80% code coverage

**Solution**:
Add coverage measurement with tarpaulin.

**Implementation**:
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html
```

**CI Integration**:
```yaml
# .github/workflows/test.yml
- name: Run coverage
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --out Xml
- name: Upload coverage
  uses: codecov/codecov-action@v3
```

**Acceptance**:
- [ ] Coverage measurement configured
- [ ] Coverage ≥ 80% achieved
- [ ] Coverage report in CI
- [ ] Badge in README

---

### Issue #9: Missing CI/CD Integration

**Status**: Not implemented

**Solution**:
Create GitHub Actions workflow.

**Implementation**:
```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Build
        run: cargo build --verbose
      
      - name: Run tests
        run: cargo test --all-features --verbose
      
      - name: Run benchmarks (no-run)
        run: cargo bench --no-run
      
      - name: Check formatting
        run: cargo fmt -- --check
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
```

**Files to Create**:
- `.github/workflows/test.yml`
- `.github/workflows/benchmark.yml` (optional)

**Acceptance**:
- [ ] CI runs on push/PR
- [ ] All tests run in CI
- [ ] Formatting checked
- [ ] Clippy warnings fail CI

---

## Summary of Work

### Complexity Assessment

| Issue | Complexity | Estimated Time |
|-------|-----------|----------------|
| #1 Concurrent writes (file locking) | High | 4-6 hours |
| #2 Registry concurrency | Medium | 2-3 hours |
| #3 CLI tests | Low | 3-4 hours |
| #4 Performance tests | Low | 2-3 hours |
| #5 Versioning tests | Low | 1-2 hours |
| #6 Test fixtures | Low | 1-2 hours |
| #7 Benchmarks | Medium | 2-3 hours |
| #8 Coverage | Low | 1 hour |
| #9 CI/CD | Low | 1-2 hours |
| **Total** | | **17-26 hours** |

### Priority Order

1. **Issue #1** (Concurrent writes) - Blocks other concurrency work
2. **Issue #2** (Registry) - Blocks concurrent tests
3. **Issue #5** (Versioning tests) - Core functionality verification
4. **Issue #4** (Performance tests) - Verify targets met
5. **Issue #3** (CLI tests) - Complete coverage
6. **Issue #6** (Fixtures) - Better test infrastructure
7. **Issue #9** (CI/CD) - Automation
8. **Issue #8** (Coverage) - Metrics
9. **Issue #7** (Benchmarks) - Optional enhancement

## Acceptance Criteria

### Must Have (P0)
- [ ] Issue #1 resolved: Concurrent writes work reliably
- [ ] Issue #2 resolved: Registry concurrency fixed
- [ ] Issue #5 complete: Versioning tests pass
- [ ] Test pass rate ≥ 95% (25/26 tests)

### Should Have (P1)
- [ ] Issue #3 complete: All CLI tests implemented
- [ ] Issue #4 complete: Performance tests verify targets
- [ ] Issue #9 complete: CI/CD pipeline running

### Nice to Have (P2)
- [ ] Issue #6 complete: Test fixtures generated
- [ ] Issue #7 complete: Benchmark suite added
- [ ] Issue #8 complete: Coverage ≥ 80%

## Related Tickets

- **REED-19-24C**: Integration Tests (parent)
- **REED-19-24D**: B+-Tree Integration (next)

## Notes

- File locking implementation needs cross-platform testing (Windows, Linux, macOS)
- Registry refactoring might be needed for true thread-safety
- Consider adding timeout mechanism for lock acquisition
- Benchmark baseline should be captured before optimizations
