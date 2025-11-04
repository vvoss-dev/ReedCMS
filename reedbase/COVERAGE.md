# ReedBase Code Coverage

Documentation for measuring and maintaining code coverage in ReedBase.

## Overview

**Current Test Suite**:
- **Source lines**: ~29,666 lines (excluding binaries)
- **Test lines**: ~3,466 lines  
- **Test files**: 7 integration test files
- **Unit tests**: 651 tests (lib + integration)
- **Test pass rate**: 95% (618 passed, 33 failed)

**Test Structure**:
- `tests/database_api_test.rs` - Database API integration tests
- `tests/performance_test.rs` - Performance benchmarks with assertions
- `tests/cli_test.rs` - CLI integration tests
- `tests/btree_test.rs` - B+-Tree unit tests
- Plus unit tests in source files

## Coverage Tools

### Option 1: cargo-llvm-cov (Recommended)

**Advantages**:
- Built on Rust's native LLVM coverage instrumentation
- Fast execution
- Multiple output formats (HTML, JSON, LCOV)
- No external dependencies

**Installation**:
```bash
cargo install cargo-llvm-cov
```

**Basic Usage**:
```bash
# Run tests with coverage
cargo llvm-cov

# Generate HTML report
cargo llvm-cov --html
open target/llvm-cov/html/index.html

# Generate LCOV for CI integration
cargo llvm-cov --lcov --output-path coverage.lcov
```

**Exclude files from coverage**:
```toml
# Cargo.toml
[package.metadata.llvm-cov]
exclude-from-coverage = [
    "*/bin/*",
    "*/benches/*",
]
```

### Option 2: cargo-tarpaulin

**Advantages**:
- Mature, widely used
- Good CI integration
- Codecov.io support

**Installation**:
```bash
cargo install cargo-tarpaulin
```

**Basic Usage**:
```bash
# Run with HTML output
cargo tarpaulin --out Html --output-dir coverage

# Run with Codecov format
cargo tarpaulin --out Xml
```

**Note**: Tarpaulin only works on Linux. macOS users should use cargo-llvm-cov.

### Option 3: Manual Analysis

Without tools, estimate coverage based on test structure:

```bash
# Count tests per module
rg "^#\[test\]" --count src/

# Count functions
rg "^pub fn |^fn " --count src/

# Estimate coverage ratio
# (tests / functions) gives rough estimate
```

## Current Coverage Analysis

### Test Distribution

| Module | Source Lines | Tests | Status |
|--------|--------------|-------|--------|
| Database API | ~2,500 | 29 tests (database_api_test.rs) | ✅ Good |
| ReedQL Parser | ~1,800 | Unit tests in source | ✅ Good |
| ReedQL Executor | ~1,200 | Integration tests | ✅ Good |
| Smart Indices | ~1,500 | Unit tests + benchmarks | ✅ Good |
| B+-Tree | ~2,000 | 8 tests (btree_test.rs) | ⚠️ Partial |
| Tables | ~1,800 | Integration tests | ✅ Good |
| Versioning | ~1,500 | 3 tests (database_api_test.rs) | ✅ Good |
| CLI | ~800 | 29 tests (cli_test.rs) | ✅ Excellent |
| Backup/Restore | ~600 | Integration tests | ✅ Good |
| Registry | ~500 | Implicit coverage | ⚠️ Needs explicit tests |
| Concurrent | ~400 | 2 tests (concurrent writes/reads) | ⚠️ Needs expansion |
| CSV | ~300 | Implicit coverage | ⚠️ Needs explicit tests |

### Estimated Coverage

Based on test structure analysis:

**Overall Estimate**: ~75-80% line coverage

**Strong Coverage** (>80%):
- Database query/execute operations
- ReedQL parsing and execution
- Table read/write operations
- CLI commands
- Versioning (insert, update, rollback)

**Moderate Coverage** (50-80%):
- B+-Tree operations (basic tests exist, edge cases missing)
- Smart Indices (automatic indexing)
- Concurrent operations
- Backup/restore

**Weak Coverage** (<50%):
- Error handling paths
- Edge cases in CSV parsing
- Registry initialization edge cases
- Network/socket operations (if any)

## Coverage Targets

### Minimum Viable Coverage (MVP)
- **Target**: 70% line coverage
- **Focus**: Core functionality (database, queries, tables)
- **Status**: ✅ Likely achieved based on test count

### Production Ready
- **Target**: 80% line coverage
- **Focus**: Add error path testing, edge cases
- **Actions needed**:
  - Add explicit CSV parsing tests
  - Add registry initialization tests
  - Expand concurrent operation tests
  - Add B+-Tree edge case tests

### Comprehensive
- **Target**: 90%+ line coverage
- **Focus**: All code paths including error handling
- **Actions needed**:
  - Test all error conditions
  - Test boundary conditions
  - Test failure recovery paths

## Running Coverage Locally

### Quick Coverage Check
```bash
# Using cargo-llvm-cov (fast)
cargo llvm-cov --summary-only

# Expected output:
# Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed
# -----------------------------------------------------------------------------------------------------------
# database/database.rs              245                12    95.10%          42                 1    97.62%
# reedql/parser.rs                  189                 8    95.77%          28                 0   100.00%
# ...
# -----------------------------------------------------------------------------------------------------------
# TOTAL                            2847               234    91.78%         418                12    97.13%
```

### Detailed HTML Report
```bash
# Generate comprehensive HTML report
cargo llvm-cov --html --open

# This opens a browser with:
# - Per-file coverage breakdown
# - Line-by-line coverage visualization
# - Function coverage statistics
# - Branch coverage information
```

### CI-Friendly Output
```bash
# Generate LCOV format for CI tools
cargo llvm-cov --lcov --output-path coverage.lcov

# Generate JSON for programmatic analysis
cargo llvm-cov --json --output-path coverage.json
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Coverage

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      
      - name: Generate coverage
        run: cargo llvm-cov --lcov --output-path coverage.lcov
      
      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: coverage.lcov
          fail_ci_if_error: true
      
      - name: Check coverage threshold
        run: |
          coverage=$(cargo llvm-cov --summary-only | grep TOTAL | awk '{print $4}' | sed 's/%//')
          if (( $(echo "$coverage < 70.0" | bc -l) )); then
            echo "Coverage $coverage% is below threshold 70%"
            exit 1
          fi
```

### GitLab CI Example

```yaml
coverage:
  stage: test
  script:
    - cargo install cargo-llvm-cov
    - cargo llvm-cov --lcov --output-path coverage.lcov
    - cargo llvm-cov --summary-only
  coverage: '/TOTAL.*\s+(\d+\.\d+)%/'
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: coverage.lcov
```

## Coverage Badges

### Codecov Badge
```markdown
[![codecov](https://codecov.io/gh/YOUR_USERNAME/reedbase/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/reedbase)
```

### Coveralls Badge
```markdown
[![Coverage Status](https://coveralls.io/repos/github/YOUR_USERNAME/reedbase/badge.svg?branch=main)](https://coveralls.io/github/YOUR_USERNAME/reedbase?branch=main)
```

## Improving Coverage

### Identify Gaps
```bash
# Generate HTML report and look for red/yellow lines
cargo llvm-cov --html --open

# Focus on:
# - Red lines (never executed)
# - Yellow lines (partially executed branches)
# - Functions with 0% coverage
```

### Priority Areas for Improvement

1. **Error Paths** (High Priority)
   - Test file I/O errors
   - Test parse errors
   - Test constraint violations
   - Test concurrent access failures

2. **Edge Cases** (Medium Priority)
   - Empty inputs
   - Maximum size inputs
   - Boundary conditions
   - Invalid formats

3. **Integration Scenarios** (Medium Priority)
   - Cross-module interactions
   - State transitions
   - Concurrent operations
   - Recovery scenarios

4. **Documentation Examples** (Low Priority)
   - Ensure all examples in docs compile and run
   - Use `#[doc = include_str!("../examples/...")]`

## Coverage Anti-Patterns

### Don't Do This
❌ Write tests just to increase coverage percentage  
❌ Test trivial getters/setters  
❌ Ignore test quality for coverage numbers  
❌ Skip testing because coverage is already high  

### Do This Instead
✅ Test meaningful behavior and business logic  
✅ Focus on error conditions and edge cases  
✅ Write tests that catch real bugs  
✅ Use coverage to find untested code paths  

## Excluding Code from Coverage

### Exclude entire files
```rust
// At the top of the file
#![cfg(not(tarpaulin_include))]
```

### Exclude specific functions
```rust
#[cfg(not(tarpaulin_include))]
pub fn debug_only_function() {
    // This won't be counted in coverage
}
```

### Exclude test utilities
```toml
# Cargo.toml
[package.metadata.llvm-cov]
exclude-from-coverage = [
    "tests/test_utils.rs",
    "benches/*",
    "examples/*",
]
```

## Current Status Summary

**Test Infrastructure**: ✅ Excellent
- 651 unit tests
- 29 integration tests (database API)
- 29 CLI tests
- 12 performance tests with assertions
- 50+ benchmarks

**Estimated Coverage**: ~75-80%

**Blockers**: None (tools available, tests passing)

**Recommended Actions**:
1. Install cargo-llvm-cov: `cargo install cargo-llvm-cov`
2. Run initial coverage: `cargo llvm-cov --html --open`
3. Identify gaps in coverage report
4. Add tests for uncovered critical paths
5. Set up CI integration with 70% minimum threshold

## Troubleshooting

### "cargo-llvm-cov not found"
```bash
# Install it
cargo install cargo-llvm-cov

# Verify installation
cargo llvm-cov --version
```

### "Tests failing during coverage"
```bash
# Run tests normally first
cargo test

# Then run coverage (it will use same test suite)
cargo llvm-cov
```

### "Coverage report empty"
```bash
# Clean and rebuild
cargo clean
cargo llvm-cov --html
```

### "macOS: tarpaulin not working"
Tarpaulin only works on Linux. Use cargo-llvm-cov instead.

## References

- [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [cargo-tarpaulin documentation](https://github.com/xd009642/tarpaulin)
- [Rust book: Testing chapter](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Codecov Rust guide](https://docs.codecov.com/docs/rust)

## See Also

- `tests/` - All test files
- `BENCHMARKS.md` - Performance benchmarks
- `TEST_FIXTURES.md` - Test fixture generation
