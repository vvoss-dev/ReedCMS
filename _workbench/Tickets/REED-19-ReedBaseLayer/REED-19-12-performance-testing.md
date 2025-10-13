# REED-19-12: Performance Testing & Benchmarking

**Status**: Not Started  
**Priority**: High  
**Estimated Effort**: 1 week  
**Layer**: ReedBase (Data Layer)  
**Dependencies**: All REED-19-01 through REED-19-11  

---

## Overview

This ticket implements comprehensive performance testing, benchmarking, and profiling infrastructure for all ReedBase operations to ensure performance targets are met and maintained.

**Purpose**: Provide automated performance testing that validates all ReedBase operations meet specified performance targets and detect performance regressions.

**Scope**:
- Benchmark all core operations (read, write, query, merge)
- Load testing with realistic workloads
- Concurrent operation stress testing
- Memory profiling and leak detection
- Disk I/O profiling
- Performance regression detection
- Continuous benchmarking infrastructure

---

## MANDATORY Development Standards

1. **Language**: All code comments and documentation in BBC English
2. **Principle**: KISS (Keep It Simple, Stupid)
3. **File Naming**: Each file has unique theme and clear responsibility
4. **Files**: One file = One responsibility (no multi-purpose files)
5. **Functions**: One function = One distinctive job (no Swiss Army knives)
6. **Testing**: Separate test files as `{name}.test.rs` (never inline `#[cfg(test)]`)
7. **Avoid**: Generic names like `handler.rs`, `middleware.rs`, `utils.rs`
8. **Templates**: Reference `service-template.md` and `service-template.test.md`

---

## Implementation Files

### 1. `src/reedbase/benchmarks/core_ops.rs`

**Purpose**: Benchmark core CRUD operations (Create, Read, Update, Delete).

**Benchmarks**:

```rust
/// Benchmark single row read operation.
///
/// ## Measures
/// - Time to read one row from current.csv
/// - Target: < 100μs
///
/// ## Dataset
/// - 10,000 rows pre-populated
/// - Random key lookup
///
/// ## Example Output
/// ```
/// test bench_read_single_row ... bench: 45,234 ns/iter (+/- 2,103)
/// ```
#[bench]
fn bench_read_single_row(b: &mut Bencher)

/// Benchmark bulk read operation.
///
/// ## Measures
/// - Time to read 1000 rows sequentially
/// - Target: < 10ms
///
/// ## Dataset
/// - 10,000 rows pre-populated
///
/// ## Example Output
/// ```
/// test bench_read_bulk_1000 ... bench: 8,456,123 ns/iter (+/- 234,567)
/// ```
#[bench]
fn bench_read_bulk_1000(b: &mut Bencher)

/// Benchmark single row write operation.
///
/// ## Measures
/// - Time to write one new row
/// - Target: < 1ms (including fsync)
///
/// ## Dataset
/// - Fresh table per iteration
///
/// ## Example Output
/// ```
/// test bench_write_single_row ... bench: 654,321 ns/iter (+/- 45,123)
/// ```
#[bench]
fn bench_write_single_row(b: &mut Bencher)

/// Benchmark bulk write operation.
///
/// ## Measures
/// - Time to write 1000 rows in batch
/// - Target: < 100ms
///
/// ## Dataset
/// - 1000 unique rows per iteration
///
/// ## Example Output
/// ```
/// test bench_write_bulk_1000 ... bench: 89,123,456 ns/iter (+/- 3,456,789)
/// ```
#[bench]
fn bench_write_bulk_1000(b: &mut Bencher)

/// Benchmark row update operation.
///
/// ## Measures
/// - Time to update existing row value
/// - Target: < 1ms
///
/// ## Dataset
/// - 10,000 rows pre-populated
/// - Random row update
///
/// ## Example Output
/// ```
/// test bench_update_single_row ... bench: 723,456 ns/iter (+/- 34,567)
/// ```
#[bench]
fn bench_update_single_row(b: &mut Bencher)

/// Benchmark row delete operation.
///
/// ## Measures
/// - Time to delete single row
/// - Target: < 1ms
///
/// ## Dataset
/// - 10,000 rows pre-populated
/// - Random row deletion
///
/// ## Example Output
/// ```
/// test bench_delete_single_row ... bench: 678,901 ns/iter (+/- 23,456)
/// ```
#[bench]
fn bench_delete_single_row(b: &mut Bencher)
```

---

### 2. `src/reedbase/benchmarks/versioning.rs`

**Purpose**: Benchmark versioning operations (delta generation, compression, rollback).

**Benchmarks**:

```rust
/// Benchmark binary delta generation.
///
/// ## Measures
/// - Time to generate bsdiff delta between versions
/// - Target: < 50ms for 10,000 row CSV (2MB)
///
/// ## Dataset
/// - Two versions with 10% row changes
///
/// ## Example Output
/// ```
/// test bench_generate_delta ... bench: 42,345,678 ns/iter (+/- 1,234,567)
/// ```
#[bench]
fn bench_generate_delta(b: &mut Bencher)

/// Benchmark delta compression with XZ.
///
/// ## Measures
/// - Time to compress delta with XZ
/// - Target: < 20ms for typical delta (~200KB)
///
/// ## Dataset
/// - Pre-generated binary delta
///
/// ## Example Output
/// ```
/// test bench_compress_delta ... bench: 18,234,567 ns/iter (+/- 876,543)
/// ```
#[bench]
fn bench_compress_delta(b: &mut Bencher)

/// Benchmark delta application (patch).
///
/// ## Measures
/// - Time to apply bsdiff delta to base version
/// - Target: < 30ms for 10,000 row CSV
///
/// ## Dataset
/// - Base version + compressed delta
///
/// ## Example Output
/// ```
/// test bench_apply_delta ... bench: 27,456,789 ns/iter (+/- 987,654)
/// ```
#[bench]
fn bench_apply_delta(b: &mut Bencher)

/// Benchmark version list operation.
///
/// ## Measures
/// - Time to list all versions from version.log
/// - Target: < 1ms for 100 versions
///
/// ## Dataset
/// - version.log with 100 entries
///
/// ## Example Output
/// ```
/// test bench_list_versions ... bench: 456,789 ns/iter (+/- 23,456)
/// ```
#[bench]
fn bench_list_versions(b: &mut Bencher)

/// Benchmark rollback operation.
///
/// ## Measures
/// - Time to rollback to previous version
/// - Target: < 100ms for typical table
///
/// ## Dataset
/// - 10 versions with deltas
///
/// ## Example Output
/// ```
/// test bench_rollback_version ... bench: 87,654,321 ns/iter (+/- 2,345,678)
/// ```
#[bench]
fn bench_rollback_version(b: &mut Bencher)
```

---

### 3. `src/reedbase/benchmarks/concurrent.rs`

**Purpose**: Benchmark concurrent operations and lock contention.

**Benchmarks**:

```rust
/// Benchmark concurrent reads (no locks).
///
/// ## Measures
/// - Throughput of parallel read operations
/// - Target: Linear scaling up to CPU cores
///
/// ## Dataset
/// - 10,000 rows
/// - 8 parallel readers
///
/// ## Example Output
/// ```
/// test bench_concurrent_reads_8_threads ... bench: 1,234,567 ns/iter (+/- 67,890)
/// Throughput: ~6,480 ops/sec per thread
/// ```
#[bench]
fn bench_concurrent_reads_8_threads(b: &mut Bencher)

/// Benchmark concurrent writes (with locks).
///
/// ## Measures
/// - Throughput of serialised write operations
/// - Target: > 1,000 writes/sec with 4 concurrent writers
///
/// ## Dataset
/// - Fresh table
/// - 4 parallel writers
///
/// ## Example Output
/// ```
/// test bench_concurrent_writes_4_threads ... bench: 3,456,789 ns/iter (+/- 123,456)
/// Throughput: ~1,156 ops/sec aggregate
/// ```
#[bench]
fn bench_concurrent_writes_4_threads(b: &mut Bencher)

/// Benchmark row-level merge operations.
///
/// ## Measures
/// - Time to merge two non-conflicting write batches
/// - Target: < 50ms for 1000 rows each
///
/// ## Dataset
/// - Two write batches with different rows
///
/// ## Example Output
/// ```
/// test bench_row_level_merge ... bench: 45,678,901 ns/iter (+/- 1,234,567)
/// ```
#[bench]
fn bench_row_level_merge(b: &mut Bencher)

/// Benchmark conflict detection.
///
/// ## Measures
/// - Time to detect conflicts between concurrent writes
/// - Target: < 10ms for 1000 rows
///
/// ## Dataset
/// - Two write batches with 10% overlapping keys
///
/// ## Example Output
/// ```
/// test bench_conflict_detection ... bench: 8,234,567 ns/iter (+/- 456,789)
/// ```
#[bench]
fn bench_conflict_detection(b: &mut Bencher)

/// Benchmark lock acquisition and release.
///
/// ## Measures
/// - Overhead of advisory file locks
/// - Target: < 100μs
///
/// ## Dataset
/// - Empty lock file
///
/// ## Example Output
/// ```
/// test bench_lock_overhead ... bench: 67,890 ns/iter (+/- 3,456)
/// ```
#[bench]
fn bench_lock_overhead(b: &mut Bencher)
```

---

### 4. `src/reedbase/benchmarks/queries.rs`

**Purpose**: Benchmark query operations (filter, aggregate, sort).

**Benchmarks**:

```rust
/// Benchmark simple key lookup.
///
/// ## Measures
/// - Time to find row by primary key
/// - Target: < 100μs
///
/// ## Dataset
/// - 10,000 rows
///
/// ## Example Output
/// ```
/// test bench_query_by_key ... bench: 45,678 ns/iter (+/- 2,345)
/// ```
#[bench]
fn bench_query_by_key(b: &mut Bencher)

/// Benchmark filter operation (WHERE clause).
///
/// ## Measures
/// - Time to filter rows by column value
/// - Target: < 10ms for 10,000 rows
///
/// ## Dataset
/// - 10,000 rows with indexed column
///
/// ## Example Output
/// ```
/// test bench_query_filter ... bench: 8,345,678 ns/iter (+/- 234,567)
/// ```
#[bench]
fn bench_query_filter(b: &mut Bencher)

/// Benchmark aggregation operation (COUNT, SUM).
///
/// ## Measures
/// - Time to compute aggregation over rows
/// - Target: < 20ms for 10,000 rows
///
/// ## Dataset
/// - 10,000 rows with numeric column
///
/// ## Example Output
/// ```
/// test bench_query_aggregate ... bench: 18,456,789 ns/iter (+/- 678,901)
/// ```
#[bench]
fn bench_query_aggregate(b: &mut Bencher)

/// Benchmark sort operation (ORDER BY).
///
/// ## Measures
/// - Time to sort rows by column
/// - Target: < 50ms for 10,000 rows
///
/// ## Dataset
/// - 10,000 rows unsorted
///
/// ## Example Output
/// ```
/// test bench_query_sort ... bench: 45,678,901 ns/iter (+/- 1,234,567)
/// ```
#[bench]
fn bench_query_sort(b: &mut Bencher)

/// Benchmark function cache hit.
///
/// ## Measures
/// - Time to retrieve cached function result
/// - Target: < 100ns
///
/// ## Dataset
/// - Pre-populated cache with 1000 entries
///
/// ## Example Output
/// ```
/// test bench_function_cache_hit ... bench: 67 ns/iter (+/- 5)
/// ```
#[bench]
fn bench_function_cache_hit(b: &mut Bencher)

/// Benchmark function cache miss (compute + store).
///
/// ## Measures
/// - Time to compute and cache new result
/// - Target: < 10μs
///
/// ## Dataset
/// - Empty cache
///
/// ## Example Output
/// ```
/// test bench_function_cache_miss ... bench: 8,456 ns/iter (+/- 345)
/// ```
#[bench]
fn bench_function_cache_miss(b: &mut Bencher)
```

---

### 5. `src/reedbase/benchmarks/memory.rs`

**Purpose**: Memory profiling and allocation tracking.

**Tests**:

```rust
/// Measure memory usage for table with 10k rows.
///
/// ## Measures
/// - Heap allocations during read_current()
/// - Target: < 5MB for 10,000 rows
///
/// ## Dataset
/// - 10,000 rows (~2MB CSV)
///
/// ## Example Output
/// ```
/// Memory allocated: 4,234,567 bytes
/// Peak RSS: 8,456,789 bytes
/// ```
#[test]
fn test_memory_usage_10k_rows()

/// Measure memory allocations per operation.
///
/// ## Measures
/// - Number of heap allocations per read/write
/// - Target: < 10 allocations per operation
///
/// ## Dataset
/// - Single row operations
///
/// ## Example Output
/// ```
/// Read operation: 3 allocations, 456 bytes
/// Write operation: 7 allocations, 1,234 bytes
/// ```
#[test]
fn test_allocations_per_operation()

/// Test for memory leaks in long-running operations.
///
/// ## Measures
/// - Memory growth over 10,000 iterations
/// - Target: No growth (constant memory usage)
///
/// ## Dataset
/// - Repeated read/write cycles
///
/// ## Example Output
/// ```
/// Initial RSS: 5,234,567 bytes
/// After 10k ops: 5,234,890 bytes (+323 bytes, acceptable)
/// No memory leak detected.
/// ```
#[test]
fn test_memory_leak_detection()

/// Measure cache memory overhead.
///
/// ## Measures
/// - Memory used by function memoization cache
/// - Target: < 1MB for 1000 cached entries
///
/// ## Dataset
/// - Cache with 1000 entries
///
/// ## Example Output
/// ```
/// Cache entries: 1,000
/// Memory used: 456,789 bytes
/// Average per entry: 456 bytes
/// ```
#[test]
fn test_cache_memory_overhead()
```

---

### 6. `src/reedbase/benchmarks/io.rs`

**Purpose**: Disk I/O profiling and optimisation validation.

**Benchmarks**:

```rust
/// Benchmark sequential CSV read.
///
/// ## Measures
/// - Throughput of reading entire CSV sequentially
/// - Target: > 50 MB/s
///
/// ## Dataset
/// - 10MB CSV file
///
/// ## Example Output
/// ```
/// test bench_sequential_read ... bench: 178,456,789 ns/iter (+/- 5,678,901)
/// Throughput: 56.1 MB/s
/// ```
#[bench]
fn bench_sequential_read(b: &mut Bencher)

/// Benchmark random CSV access.
///
/// ## Measures
/// - Time for random row lookups
/// - Target: < 500μs per lookup
///
/// ## Dataset
/// - 10,000 rows
///
/// ## Example Output
/// ```
/// test bench_random_access ... bench: 345,678 ns/iter (+/- 23,456)
/// ```
#[bench]
fn bench_random_access(b: &mut Bencher)

/// Benchmark fsync overhead.
///
/// ## Measures
/// - Time to fsync after write
/// - Target: < 10ms
///
/// ## Dataset
/// - 1KB write
///
/// ## Example Output
/// ```
/// test bench_fsync_overhead ... bench: 8,456,789 ns/iter (+/- 567,890)
/// ```
#[bench]
fn bench_fsync_overhead(b: &mut Bencher)

/// Benchmark atomic write operation (temp file + rename).
///
/// ## Measures
/// - Time for complete atomic write
/// - Target: < 20ms for 1000 rows
///
/// ## Dataset
/// - 1000 row CSV
///
/// ## Example Output
/// ```
/// test bench_atomic_write ... bench: 18,234,567 ns/iter (+/- 876,543)
/// ```
#[bench]
fn bench_atomic_write(b: &mut Bencher)
```

---

### 7. `src/reedbase/benchmarks/regression.rs`

**Purpose**: Detect performance regressions between versions.

**Functions**:

```rust
/// Compare current performance against baseline.
///
/// ## Arguments
/// - baseline_path: Path to baseline benchmark results (JSON)
///
/// ## Returns
/// - RegressionReport with performance comparison
///
/// ## Performance
/// - Runs all benchmarks and compares
/// - ~1 minute for complete suite
///
/// ## Error Conditions
/// - ReedError::FileNotFound: Baseline file missing
/// - ReedError::InvalidFormat: Cannot parse baseline
///
/// ## Example Usage
/// ```rust
/// let report = compare_to_baseline("benchmarks/baseline.json")?;
/// if report.has_regressions {
///     eprintln!("Performance regressions detected!");
///     for regression in &report.regressions {
///         eprintln!("  {}: +{:.1}% slower", regression.test, regression.percent_change);
///     }
/// }
/// ```
pub fn compare_to_baseline(baseline_path: &Path) -> ReedResult<RegressionReport>

/// Save current benchmark results as new baseline.
///
/// ## Arguments
/// - output_path: Path to save baseline results
/// - results: BenchmarkResults to save
///
/// ## Returns
/// - () on success
///
/// ## Performance
/// - O(1) - JSON serialisation
/// - < 10ms
///
/// ## Error Conditions
/// - ReedError::WriteError: Cannot write file
///
/// ## Example Usage
/// ```rust
/// let results = run_all_benchmarks()?;
/// save_baseline("benchmarks/baseline-v2.0.0.json", &results)?;
/// ```
pub fn save_baseline(output_path: &Path, results: &BenchmarkResults) -> ReedResult<()>

/// Generate performance comparison report.
///
/// ## Arguments
/// - current: Current benchmark results
/// - baseline: Baseline benchmark results
///
/// ## Returns
/// - RegressionReport with detailed comparison
///
/// ## Performance
/// - O(n) where n = number of benchmarks
/// - < 10ms
///
/// ## Error Conditions
/// - None (pure computation)
///
/// ## Example Usage
/// ```rust
/// let report = generate_comparison_report(&current_results, &baseline_results)?;
/// println!("{}", report.summary());
/// ```
pub fn generate_comparison_report(
    current: &BenchmarkResults,
    baseline: &BenchmarkResults,
) -> RegressionReport
```

**Key Types**:

```rust
pub struct BenchmarkResults {
    pub timestamp: SystemTime,
    pub git_commit: String,
    pub benchmarks: HashMap<String, BenchmarkResult>,
}

pub struct BenchmarkResult {
    pub name: String,
    pub mean_ns: u64,
    pub stddev_ns: u64,
    pub iterations: u64,
}

pub struct RegressionReport {
    pub has_regressions: bool,
    pub regressions: Vec<Regression>,
    pub improvements: Vec<Improvement>,
    pub unchanged: Vec<String>,
}

pub struct Regression {
    pub test: String,
    pub baseline_ns: u64,
    pub current_ns: u64,
    pub percent_change: f64,
}

pub struct Improvement {
    pub test: String,
    pub baseline_ns: u64,
    pub current_ns: u64,
    pub percent_change: f64,
}
```

---

### 8. `src/reedbase/benchmarks/load_test.rs`

**Purpose**: Realistic load testing with mixed workloads.

**Tests**:

```rust
/// Load test: 80% reads, 20% writes.
///
/// ## Measures
/// - Sustained throughput over 60 seconds
/// - Target: > 5,000 ops/sec
///
/// ## Workload
/// - 8 concurrent threads
/// - Random operations
///
/// ## Example Output
/// ```
/// Duration: 60.1s
/// Total operations: 312,456
/// Throughput: 5,201 ops/sec
/// Read latency p50: 45μs, p99: 234μs
/// Write latency p50: 567μs, p99: 2.1ms
/// ```
#[test]
fn load_test_read_heavy_workload()

/// Load test: 50% reads, 50% writes.
///
/// ## Measures
/// - Balanced workload performance
/// - Target: > 2,000 ops/sec
///
/// ## Workload
/// - 8 concurrent threads
///
/// ## Example Output
/// ```
/// Duration: 60.1s
/// Total operations: 134,567
/// Throughput: 2,240 ops/sec
/// ```
#[test]
fn load_test_balanced_workload()

/// Load test: Version creation under load.
///
/// ## Measures
/// - Version creation rate during active writes
/// - Target: Version every 10s without blocking writes
///
/// ## Workload
/// - Continuous writes
/// - Version created every 10s
///
/// ## Example Output
/// ```
/// Versions created: 6
/// Write operations: 23,456
/// Average version time: 87ms
/// Writes blocked: 0
/// ```
#[test]
fn load_test_versioning_under_load()

/// Stress test: Maximum concurrent writers.
///
/// ## Measures
/// - System behaviour under extreme concurrency
/// - Target: Graceful degradation, no crashes
///
/// ## Workload
/// - 32 concurrent writers
/// - 1000 writes per thread
///
/// ## Example Output
/// ```
/// Threads: 32
/// Total writes: 32,000
/// Successful: 31,987 (99.96%)
/// Failed: 13 (timeout)
/// Average latency: 12.3ms
/// Max latency: 234ms
/// ```
#[test]
fn stress_test_extreme_concurrency()
```

---

### 9. `src/reedbase/benchmarks/mod.rs`

**Purpose**: Benchmark runner and reporting.

**Functions**:

```rust
/// Run all benchmark suites.
///
/// ## Arguments
/// - None
///
/// ## Returns
/// - BenchmarkResults with all results
///
/// ## Performance
/// - ~5-10 minutes for complete suite
///
/// ## Error Conditions
/// - ReedError::BenchmarkFailed: One or more benchmarks failed
///
/// ## Example Usage
/// ```rust
/// let results = run_all_benchmarks()?;
/// println!("Completed {} benchmarks", results.benchmarks.len());
/// ```
pub fn run_all_benchmarks() -> ReedResult<BenchmarkResults>

/// Run specific benchmark suite.
///
/// ## Arguments
/// - suite: Name of suite ("core_ops", "versioning", "concurrent", etc.)
///
/// ## Returns
/// - BenchmarkResults for that suite
///
/// ## Performance
/// - ~1 minute per suite
///
/// ## Error Conditions
/// - ReedError::NotFound: Suite name not recognised
///
/// ## Example Usage
/// ```rust
/// let results = run_benchmark_suite("core_ops")?;
/// ```
pub fn run_benchmark_suite(suite: &str) -> ReedResult<BenchmarkResults>
```

---

## CLI Commands

### `reed bench:run`
**Purpose**: Run performance benchmarks.

```bash
# Run all benchmarks
reed bench:run

# Run specific suite
reed bench:run --suite core_ops

# Run with regression detection
reed bench:run --compare benchmarks/baseline.json

# Save results as new baseline
reed bench:run --save-baseline benchmarks/baseline-v2.1.0.json

# Output:
# Running ReedBase Benchmarks
# ============================
# 
# Core Operations:
#   read_single_row................ 45.2μs ± 2.1μs  ✓
#   write_single_row............... 654ns ± 45ns    ✓
#   ...
#
# Versioning:
#   generate_delta................. 42.3ms ± 1.2ms  ✓
#   ...
#
# Summary: 47/47 benchmarks passed
# Total duration: 8m 23s
```

### `reed bench:compare`
**Purpose**: Compare benchmark results against baseline.

```bash
# Compare current performance to baseline
reed bench:compare benchmarks/baseline-v2.0.0.json

# Output:
# Performance Comparison vs v2.0.0 Baseline
# ==========================================
#
# Regressions (slower):
#   write_bulk_1000: 89.1ms → 103.4ms (+16.1%) ⚠️
#   conflict_detection: 8.2ms → 9.1ms (+11.0%) ⚠️
#
# Improvements (faster):
#   read_single_row: 67.8μs → 45.2μs (-33.3%) ✓
#   generate_delta: 54.3ms → 42.3ms (-22.1%) ✓
#
# Unchanged (±5%):
#   compress_delta: 18.2ms ✓
#   ...
#
# Summary: 2 regressions, 8 improvements, 37 unchanged
```

### `reed bench:profile`
**Purpose**: Run performance profiling with detailed analysis.

```bash
# Profile specific operation
reed bench:profile read_bulk_1000

# Profile with memory tracking
reed bench:profile --memory write_bulk_1000

# Profile with flamegraph generation
reed bench:profile --flamegraph concurrent_writes_4_threads

# Output:
# Profiling: read_bulk_1000
# ==========================
# Duration: 8.456ms
# CPU time: 8.234ms
# Allocations: 23 (total 45,678 bytes)
# Peak memory: 2.3 MB
#
# Hotspots:
#   csv::parse_line: 45.2% (3.82ms)
#   table::deserialize: 32.1% (2.71ms)
#   io::buffered_read: 15.4% (1.30ms)
#
# Flamegraph saved: profiling/read_bulk_1000.svg
```

### `reed bench:load`
**Purpose**: Run load tests with realistic workloads.

```bash
# Run read-heavy load test (60 seconds)
reed bench:load --workload read-heavy --duration 60

# Run custom workload
reed bench:load --reads 70 --writes 30 --duration 120

# Output:
# Load Test: Read-Heavy Workload
# ================================
# Duration: 60.1s
# Threads: 8
# Total operations: 312,456
# Throughput: 5,201 ops/sec
#
# Operation breakdown:
#   Reads: 249,965 (80.0%) - p50: 45μs, p99: 234μs
#   Writes: 62,491 (20.0%) - p50: 567μs, p99: 2.1ms
#
# Resource usage:
#   CPU: 78.4% average
#   Memory: 45.6 MB peak
#   Disk I/O: 12.3 MB/s average
```

---

## Test Files

### `src/reedbase/benchmarks/core_ops.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_benchmark_read_produces_valid_results()
// Verify: Benchmark runs and produces reasonable timing

#[test]
fn test_benchmark_write_produces_valid_results()
// Verify: Write benchmark runs successfully

#[test]
fn test_benchmark_consistency()
// Verify: Multiple runs produce consistent results (±10%)
```

### `src/reedbase/benchmarks/regression.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_compare_identical_results()
// Verify: Comparison detects no changes when results identical

#[test]
fn test_detect_regression()
// Verify: Correctly identifies performance regressions

#[test]
fn test_detect_improvement()
// Verify: Correctly identifies performance improvements

#[test]
fn test_save_and_load_baseline()
// Verify: Baseline can be saved and loaded correctly
```

---

## Performance Requirements

| Benchmark Suite | Duration | Target |
|----------------|----------|--------|
| Core operations | ~30s | All benchmarks complete |
| Versioning | ~45s | All benchmarks complete |
| Concurrent | ~60s | All benchmarks complete |
| Queries | ~45s | All benchmarks complete |
| Load tests | ~5min | All tests complete |
| Full suite | ~10min | 100% success rate |

| Operation Target | Value | Tolerance |
|-----------------|-------|-----------|
| Single row read | < 100μs | ±20% |
| Single row write | < 1ms | ±30% |
| Bulk read (1000) | < 10ms | ±20% |
| Bulk write (1000) | < 100ms | ±30% |
| Generate delta | < 50ms | ±40% |
| Apply delta | < 30ms | ±40% |
| Concurrent reads | Linear scaling | Up to CPU cores |
| Concurrent writes | > 1000 ops/s | With 4 writers |

---

## Error Conditions

### `ReedError::BenchmarkFailed`
**When**: Benchmark execution fails or crashes.  
**Context**: Benchmark name, iteration count, error details.  
**Recovery**: Review benchmark code, check test data, retry.

### `ReedError::PerformanceRegression`
**When**: Performance degrades beyond threshold (>20%).  
**Context**: Benchmark name, baseline vs current timing, percent change.  
**Recovery**: Profile operation, identify bottleneck, optimise code.

### `ReedError::OutOfMemory`
**When**: Benchmark exhausts available memory.  
**Context**: Memory usage, allocation count, operation details.  
**Recovery**: Reduce dataset size, check for memory leaks, optimise allocations.

---

## Acceptance Criteria

- [ ] `core_ops.rs` benchmarks all CRUD operations
- [ ] `versioning.rs` benchmarks delta operations
- [ ] `concurrent.rs` benchmarks parallel operations
- [ ] `queries.rs` benchmarks query and cache operations
- [ ] `memory.rs` profiles memory usage and detects leaks
- [ ] `io.rs` profiles disk I/O performance
- [ ] `regression.rs` implements regression detection
- [ ] `load_test.rs` implements realistic workload testing
- [ ] All benchmarks run successfully with `cargo bench`
- [ ] Performance targets met for all operations
- [ ] Regression detection compares against baseline
- [ ] CLI commands provide clear output and reporting
- [ ] Flamegraph generation works for profiling
- [ ] Load tests run for 60+ seconds without crashes
- [ ] Memory profiling detects no leaks
- [ ] Documentation complete with example outputs
- [ ] CI integration for continuous benchmarking
- [ ] Baseline results committed for regression tracking

---

## Dependencies

- **All REED-19 tickets**: Benchmarks test all implemented features

---

## Notes

### Benchmark Infrastructure

- **Tool**: Use `cargo bench` with criterion.rs for stable benchmarking
- **Statistics**: Criterion provides mean, stddev, outlier detection
- **Comparison**: Automatic comparison against previous runs
- **Output**: Human-readable reports + machine-readable JSON

### Regression Detection

- **Threshold**: Flag regressions > 20% slower than baseline
- **CI Integration**: Run benchmarks on every commit
- **Baseline Management**: Tagged baselines for each release version
- **Notification**: Alert on performance regressions in CI

### Load Testing Strategy

- **Workloads**: Read-heavy (80/20), balanced (50/50), write-heavy (20/80)
- **Duration**: 60-300 seconds for realistic behaviour
- **Metrics**: Throughput (ops/sec), latency (p50, p95, p99), resource usage
- **Validation**: No crashes, graceful degradation under stress

### Memory Profiling

- **Tools**: valgrind (memcheck), heaptrack, DHAT
- **Targets**: Zero memory leaks, minimal allocations in hot paths
- **Tracking**: RSS (resident set size), heap allocations count and size
- **Validation**: Constant memory usage over long runs

### Profiling Tools

- **CPU**: perf, flamegraph, cargo-flamegraph
- **Memory**: valgrind, heaptrack
- **I/O**: iostat, strace
- **Visualisation**: Flamegraphs for hotspot identification

### Continuous Benchmarking

- **CI Pipeline**: Run benchmarks on every PR
- **Baseline**: Compare against main branch baseline
- **Reports**: Automatic comment on PR with comparison
- **Blocking**: Fail CI if regression > 30% without justification

### Performance Budget

Each operation has a "performance budget":
- **Critical path** (read): < 100μs (tight budget)
- **Common operations** (write): < 1ms (moderate budget)
- **Background tasks** (versioning): < 100ms (generous budget)

Budget violations require justification and optimisation plan.

---

## References

- Service Template: `_workbench/Tickets/templates/service-template.md`
- Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Criterion.rs Documentation: https://bheisler.github.io/criterion.rs/
- Rust Performance Book: https://nnethercote.github.io/perf-book/
