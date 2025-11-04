# ReedBase Benchmarks

Comprehensive benchmark suite for ReedBase performance measurement using Criterion.

## Overview

ReedBase includes 4 benchmark suites covering different performance aspects:

| Benchmark | File | Status | Focus Area |
|-----------|------|--------|------------|
| **queries** | `benches/queries.rs` | ✅ Working | ReedQL parsing, execution, indices |
| **core_ops** | `benches/core_ops.rs` | ⚠️ Registry issues | Table operations, versioning |
| **concurrent** | `benches/concurrent.rs` | ⚠️ Registry issues | Concurrent operations, locking |
| **versioning** | `benches/versioning.rs` | ⚠️ Registry issues | Delta operations, backups |

## Running Benchmarks

### Run All Benchmarks
```bash
cargo bench
```

### Run Specific Benchmark Suite
```bash
cargo bench --bench queries
cargo bench --bench core_ops
cargo bench --bench concurrent
cargo bench --bench versioning
```

### Run Specific Benchmark Function
```bash
cargo bench --bench queries -- "simple_select"
cargo bench --bench queries -- "table_scan"
```

### Test Benchmarks Without Running
```bash
cargo bench --bench queries -- --test
```

## Benchmark Suites

### 1. queries.rs (ReedQL & Indices)

**Status**: ✅ Working (with parser limitations)

**Benchmarks**:
- **Query Parsing** (target: < 1ms)
  - `simple_select` - Basic SELECT parsing
  - `complex_select` - Complex query with WHERE, ORDER BY, LIMIT
  - `with_subquery` - Subquery parsing
  
- **Table Scan** (target: < 100ms for 10k rows)
  - `table_scan/1000` - 1k row full scan
  - `table_scan/5000` - 5k row full scan
  - `table_scan/10000` - 10k row full scan
  
- **Aggregates** (target: < 50ms for 10k rows)
  - `count` - COUNT(*) aggregate
  - `sum` - SUM(column) aggregate
  - `avg` - AVG(column) aggregate
  - `multiple_aggs` - Multiple aggregates
  
- **Smart Indices** (target: < 1ms)
  - `exact_lookup` - HashMap exact key lookup
  - ~~`range_scan`~~ - Removed (HashMap doesn't support ranges)
  
- **Index Build** (target: < 500ms for 10k rows)
  - `index_build/1000` - Build index for 1k rows
  - `index_build/5000` - Build index for 5k rows
  - `index_build/10000` - Build index for 10k rows
  
- **GROUP BY** (target: < 100ms)
  - `group_by_single_column` - Simplified (parser limitation)
  - `group_by_with_aggregates` - Simplified (parser limitation)
  
- **ORDER BY** (target: < 200ms for 10k rows)
  - `order_by_single` - Single column sort
  - `order_by_desc` - Descending sort
  
- **LIMIT** (target: < 10ms)
  - `limit/10` - LIMIT 10
  - `limit/100` - LIMIT 100
  - `limit/1000` - LIMIT 1000

**Known Limitations**:
- Parser doesn't support multiple columns with aggregates in SELECT (e.g., `SELECT city, COUNT(*)`)
- Parser doesn't support multiple aggregates in same SELECT (e.g., `SELECT COUNT(*), SUM(score)`)
- HashMap indices don't support range scans (would need BTree implementation)

**Run**:
```bash
cargo bench --bench queries
```

---

### 2. core_ops.rs (Table Operations)

**Status**: ⚠️ Has registry initialization issues

**Benchmarks**:
- **read_current()** (target: < 10ms for 1MB)
  - Measures raw read performance for different file sizes
  
- **write() with delta** (target: < 50ms for 1MB)
  - Measures write + delta generation performance
  
- **list_versions()** (target: < 10ms for 100 versions)
  - Measures version listing performance
  
- **rollback()** (target: < 100ms)
  - Measures rollback performance
  
- **read_current_as_rows()** (target: < 20ms for 1000 rows)
  - Measures CSV parsing performance
  
- **table.exists()** (target: < 1ms)
  - Measures existence check performance
  
- **concurrent_reads** (target: linear scaling)
  - Measures read scaling across threads

**Issues**:
- Registry not initialized in benchmark setup
- Causes "No such file or directory" errors for dictionary files
- Needs refactoring to initialize registry per benchmark

**Potential Fix**:
```rust
let temp_dir = TempDir::new().unwrap();
let db_path = temp_dir.path().join(".reed");
reedbase::registry::init_registry(&db_path).unwrap();
reedbase::registry::set_base_path(db_path.clone());
```

---

### 3. concurrent.rs (Concurrency)

**Status**: ⚠️ Has registry initialization issues

**Benchmarks**:
- **concurrent_reads** (target: < 2x single-thread for 10 threads)
  - Measures read throughput scaling
  
- **lock_acquire_uncontended** (target: < 1ms)
  - Measures uncontended lock acquisition
  
- **lock_contention** (target: measure degradation)
  - Measures lock contention with multiple threads
  
- **auto_merge_non_conflicting** (target: < 5ms)
  - Measures auto-merge for non-conflicting changes
  
- **conflict_detection** (target: < 10ms)
  - Measures conflict detection time
  
- **multiple_conflicts** (target: < 20ms for 10 conflicts)
  - Measures multi-conflict detection
  
- **mixed_workload** (80% read, 20% write)
  - Realistic performance under mixed load
  
- **write_throughput**
  - Sequential vs concurrent write comparison

**Issues**:
- Same registry initialization issues as core_ops
- May also need proper registry cleanup between benchmarks

---

### 4. versioning.rs (Versioning & Deltas)

**Status**: ⚠️ Has registry initialization issues

**Benchmarks**:
- **delta_generation** (target: < 50ms for 1KB)
  - Binary delta generation performance
  
- **delta_application** (target: < 20ms)
  - Delta application performance
  
- **version_index_insert** (target: < 1ms)
  - Version index insert performance
  
- **version_index_lookup** (target: < 1ms)
  - Timestamp/frame lookup performance
  
- **version_index_stats** (target: < 10ms)
  - Version statistics calculation
  
- **backup_creation** (target: < 500ms for 10MB)
  - Backup creation with compression
  
- **backup_restoration** (target: < 1s for 10MB)
  - Backup restoration performance
  
- **delta_compression_ratio**
  - Measures compression effectiveness by change percentage

**Issues**:
- Registry initialization issues
- Backup/restore operations may also need Database initialization

---

## Performance Targets Summary

| Operation | Target | Benchmark |
|-----------|--------|-----------|
| Query parsing | < 1ms | queries |
| Indexed lookup | < 1ms | queries |
| Full scan (10k rows) | < 100ms | queries |
| Aggregate query | < 50ms | queries |
| Index build (10k rows) | < 500ms | queries |
| Table read (1MB) | < 10ms | core_ops |
| Table write (1MB) | < 50ms | core_ops |
| Version list (100) | < 10ms | core_ops |
| Rollback | < 100ms | core_ops |
| Lock acquisition | < 1ms | concurrent |
| Auto-merge | < 5ms | concurrent |
| Conflict detection | < 10ms | concurrent |
| Delta generation (1KB) | < 50ms | versioning |
| Delta application | < 20ms | versioning |
| Backup (10MB) | < 500ms | versioning |

## Output

Criterion generates:
- **Console output**: Summary statistics
- **HTML reports**: `target/criterion/*/report/index.html`
- **Baseline comparison**: Compare against previous runs

### View HTML Reports
```bash
open target/criterion/queries/report/index.html
```

## Baseline Management

### Save Baseline
```bash
cargo bench --bench queries -- --save-baseline main
```

### Compare Against Baseline
```bash
cargo bench --bench queries -- --baseline main
```

### List Baselines
```bash
ls target/criterion/*/base/
```

## Troubleshooting

### Registry Errors
**Problem**: `No such file or directory (os error 2)` for dictionary files

**Cause**: Benchmarks don't initialize registry before creating tables

**Workaround**: Use test fixtures or Database::open() instead of Table::new()

### Parser Limitations
**Problem**: Complex queries fail with parse errors

**Solution**: Queries benchmark has been simplified to work within parser capabilities

### Slow Benchmarks
**Problem**: Benchmarks take too long

**Solution**: Use `--sample-size` to reduce iterations:
```bash
cargo bench --bench queries -- --sample-size 10
```

## Current Status & Recommendations

### Working ✅
- **queries.rs** - Fully functional with documented parser limitations

### Needs Fixes ⚠️
- **core_ops.rs** - Registry initialization required
- **concurrent.rs** - Registry initialization required
- **versioning.rs** - Registry initialization required

### Recommended Actions
1. **Short-term**: Focus on queries benchmarks for ReedQL performance
2. **Medium-term**: Add registry initialization to remaining benchmarks
3. **Long-term**: Consider test fixtures or helper functions for consistent setup

## Integration with CI/CD

### GitHub Actions Example
```yaml
- name: Run benchmarks (no baseline)
  run: cargo bench --bench queries --no-run

- name: Run performance tests
  run: cargo test --test performance_test
```

**Note**: Full benchmark runs are expensive and should be:
- Run on dedicated benchmark hardware
- Compared against saved baselines
- Triggered manually or on release branches

## See Also

- `tests/performance_test.rs` - Performance tests with assertions
- `TEST_FIXTURES.md` - Test fixture documentation
- Criterion documentation: https://bheisler.github.io/criterion.rs/
