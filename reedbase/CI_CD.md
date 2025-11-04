# CI/CD Integration for ReedBase

Complete guide for Continuous Integration and Continuous Deployment setup.

## Overview

ReedBase uses GitHub Actions for automated testing, coverage reporting, and benchmarking.

**Workflows**:
- ✅ **test.yml** - Test suite (Ubuntu + macOS, clippy, fmt)
- ✅ **coverage.yml** - Code coverage with Codecov integration
- ✅ **benchmark.yml** - Performance benchmarks (main branch only)

**Status**: All workflows configured and ready to use

## Quick Start

### 1. Enable Workflows

Workflows are in `../.github/workflows/` and activate automatically when:
- You push to `main` or `develop` branches
- You create a pull request to `main`

### 2. View Status

Check workflow status:
- Go to GitHub repository → Actions tab
- See all workflow runs and their status
- Click on a run for detailed logs

### 3. Add Status Badges to README

```markdown
[![Tests](https://github.com/YOUR_USERNAME/ReedCMS/actions/workflows/test.yml/badge.svg)](https://github.com/YOUR_USERNAME/ReedCMS/actions/workflows/test.yml)
[![Coverage](https://codecov.io/gh/YOUR_USERNAME/ReedCMS/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/ReedCMS)
```

## Workflow Details

### Test Workflow (test.yml)

**What it does**:
- Runs on Ubuntu and macOS
- Builds the project
- Runs 651 unit tests
- Runs 29+ integration tests
- Checks clippy warnings (enforced as errors)
- Verifies code formatting

**Duration**: ~5-7 minutes per platform

**Triggers**:
- Push to `main` or `develop`
- Pull requests to `main`

**Caching**: Yes (cargo registry, index, build artifacts)

### Coverage Workflow (coverage.yml)

**What it does**:
- Measures code coverage with cargo-llvm-cov
- Uploads results to Codecov
- Enforces 70% minimum coverage threshold
- Fails if coverage drops below threshold

**Duration**: ~3-4 minutes

**Triggers**:
- Push to `main` or `develop`
- Pull requests to `main`

**Requirements**:
- `CODECOV_TOKEN` secret (see setup below)

### Benchmark Workflow (benchmark.yml)

**What it does**:
- Compiles all benchmarks (validation)
- Runs queries benchmark in test mode
- Full benchmark run only on `main` branch
- Comments on PRs with status

**Duration**: ~2-3 minutes

**Triggers**:
- Push to `main` (automatic)
- Manual workflow dispatch

**Note**: Full benchmarks are expensive and only run on main branch. PRs validate compilation only.

## Setup Instructions

### Step 1: Codecov Integration (Optional)

For coverage badges and reports:

1. **Sign up**: Go to https://codecov.io and sign in with GitHub

2. **Add repository**: 
   - Click "Add new repository"
   - Select `ReedCMS` from the list
   - Click "Activate"

3. **Get upload token**:
   - Go to repository settings on Codecov
   - Copy the "Repository Upload Token"

4. **Add token to GitHub**:
   - Go to GitHub: Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `CODECOV_TOKEN`
   - Value: Paste the token
   - Click "Add secret"

5. **Verify**: Push a commit and check Actions tab

### Step 2: Verify Workflows Work

1. **Create a test branch**:
   ```bash
   git checkout -b test-ci
   ```

2. **Make a trivial change**:
   ```bash
   echo "# CI Test" >> reedbase/CI_CD.md
   git add reedbase/CI_CD.md
   git commit -m "test: verify CI workflows"
   git push origin test-ci
   ```

3. **Create a PR**: Go to GitHub and create PR from `test-ci` to `main`

4. **Check workflows**: Go to Actions tab and verify:
   - ✅ Tests pass on Ubuntu and macOS
   - ✅ Clippy passes
   - ✅ Formatting passes
   - ✅ Coverage runs (may fail without CODECOV_TOKEN)
   - ✅ Benchmarks compile

### Step 3: Add Badges to README

Edit main project README:

```markdown
# ReedCMS

[![Tests](https://github.com/YOUR_USERNAME/ReedCMS/actions/workflows/test.yml/badge.svg)](https://github.com/YOUR_USERNAME/ReedCMS/actions/workflows/test.yml)
[![Coverage](https://codecov.io/gh/YOUR_USERNAME/ReedCMS/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/ReedCMS)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
```

Replace `YOUR_USERNAME` with actual GitHub username.

## Local CI Simulation

Run the same checks locally before pushing:

### Full CI Simulation Script
```bash
#!/bin/bash
# ci-check.sh - Simulate CI checks locally

cd reedbase

echo "🔨 Building..."
cargo build --verbose || exit 1

echo "🧪 Running tests..."
cargo test --verbose || exit 1

echo "📎 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings || exit 1

echo "✨ Checking formatting..."
cargo fmt --all -- --check || exit 1

echo "📊 Generating coverage..."
if command -v cargo-llvm-cov &> /dev/null; then
    cargo llvm-cov --summary-only
else
    echo "⚠️  cargo-llvm-cov not installed, skipping coverage"
fi

echo "🎯 Building benchmarks..."
cargo bench --no-run || exit 1

echo "✅ All CI checks passed!"
```

Make executable and run:
```bash
chmod +x ci-check.sh
./ci-check.sh
```

### Individual Checks

**Build**:
```bash
cd reedbase && cargo build --verbose
```

**Tests**:
```bash
cd reedbase && cargo test --verbose
```

**Clippy**:
```bash
cd reedbase && cargo clippy --all-targets --all-features -- -D warnings
```

**Formatting**:
```bash
cd reedbase && cargo fmt --all -- --check
```

**Coverage** (requires cargo-llvm-cov):
```bash
cd reedbase
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --summary-only
```

**Benchmarks**:
```bash
cd reedbase && cargo bench --no-run
```

## Troubleshooting

### Tests Fail in CI but Pass Locally

**Possible causes**:
1. **Platform differences**: CI runs on Linux/macOS, you may be on different OS
2. **Environment variables**: Check if tests depend on local environment
3. **File paths**: Use cross-platform path handling (`std::path::PathBuf`)
4. **Registry issues**: Tests may have race conditions (see test serialization)

**Debug**:
- Check CI logs for specific failure
- Run tests with same Rust version: `rustup default stable`
- Check for platform-specific code: `#[cfg(target_os = "...")]`

### Clippy Fails with New Warnings

**Solution**:
```bash
# Fix warnings locally
cd reedbase
cargo clippy --fix --all-targets --all-features

# Or suppress specific warnings (use sparingly)
#[allow(clippy::specific_warning)]
```

### Formatting Check Fails

**Solution**:
```bash
# Format code
cd reedbase
cargo fmt --all

# Commit formatted code
git add .
git commit -m "style: run cargo fmt"
git push
```

### Coverage Below Threshold

**Check coverage locally**:
```bash
cd reedbase
cargo llvm-cov --html --open
```

**Identify gaps**:
- Red lines: Never executed
- Yellow lines: Partially executed branches
- Focus on critical paths first

**Add tests**:
- Prioritize error paths
- Test edge cases
- Add integration tests for user-facing features

### Benchmark Compilation Fails

**Check locally**:
```bash
cd reedbase
cargo bench --no-run
```

**Common issues**:
- Registry not initialized (3/4 benchmark suites)
- Parser limitations in queries benchmark
- See `BENCHMARKS.md` for known issues

## Performance Considerations

### CI Runtime

Typical runtimes per workflow:

| Workflow | Ubuntu | macOS | Total |
|----------|--------|-------|-------|
| test.yml | ~5min | ~7min | ~12min |
| coverage.yml | ~3min | N/A | ~3min |
| benchmark.yml | ~2min | N/A | ~2min |

**Total per main branch push**: ~17 minutes

**Total per PR**: ~17 minutes (may skip benchmarks)

### Cache Effectiveness

With caching enabled:
- First run: Full compile (~10-15 min)
- Subsequent runs: Incremental compile (~2-5 min)
- Cache invalidation: When `Cargo.lock` changes

### Optimization Tips

1. **Reduce test matrix**: Remove macOS if Linux coverage sufficient
2. **Conditional workflows**: Skip coverage on draft PRs
3. **Benchmark on demand**: Keep benchmark workflow manual-only
4. **Test subsets**: Use `cargo test --lib` for faster feedback

Example conditional workflow:
```yaml
on:
  pull_request:
    branches: [ main ]
    types: [ opened, synchronize, ready_for_review ]
  
jobs:
  test:
    if: github.event.pull_request.draft == false
```

## Security

### Secrets Management

**Available secrets**:
- `CODECOV_TOKEN`: Coverage upload token (optional)

**Adding secrets**:
1. GitHub → Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name and value, then "Add secret"

**Using in workflows**:
```yaml
env:
  SECRET_VAR: ${{ secrets.SECRET_NAME }}
```

### Fork PR Security

Pull requests from forks:
- ✅ Can run tests
- ✅ Can run clippy/fmt
- ❌ Cannot access secrets (security measure)
- ❌ Codecov upload may fail (expected)

This is by design for security. Maintainers can run workflows with secrets after PR approval.

## Best Practices

### ✅ Do This

- **Commit `Cargo.lock`**: Ensures reproducible builds
- **Test locally first**: Run `./ci-check.sh` before pushing
- **Fix warnings promptly**: Don't accumulate clippy warnings
- **Keep coverage high**: Add tests for new code
- **Use caching**: Workflows already configured
- **Monitor CI costs**: Check Actions minutes usage

### ❌ Avoid This

- **Don't skip CI**: Always let workflows complete
- **Don't commit failing code**: Fix locally first
- **Don't ignore warnings**: Fix or explicitly allow
- **Don't disable workflows**: Fix issues instead
- **Don't commit large files**: Keep repo size reasonable

## Workflow Customization

### Change Coverage Threshold

Edit `../.github/workflows/coverage.yml`:
```yaml
if (( $(echo "$COVERAGE < 70.0" | bc -l) )); then
  # Change 70.0 to desired threshold (e.g., 80.0)
```

### Add More Test Platforms

Edit `../.github/workflows/test.yml`:
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]  # Add windows
    rust: [stable, beta]  # Add beta channel
```

### Run Benchmarks on PRs

Edit `../.github/workflows/benchmark.yml`:
```yaml
on:
  push:
    branches: [ main ]
  pull_request:  # Uncomment this section
    branches: [ main ]
```

**Warning**: This significantly increases CI time (~5-10 min per PR).

## Maintenance

### Monthly Tasks

- [ ] Review failed workflows
- [ ] Check for action version updates
- [ ] Monitor coverage trends
- [ ] Review benchmark performance

### Quarterly Tasks

- [ ] Update GitHub Actions to latest versions
- [ ] Review and update OS versions (if deprecated)
- [ ] Check Rust toolchain strategy (stable vs MSRV)
- [ ] Review caching strategy

### Action Version Updates

Current versions (as of 2025):
- `actions/checkout@v4`
- `actions/cache@v4`
- `dtolnay/rust-toolchain@stable`
- `taiki-e/install-action@cargo-llvm-cov`
- `codecov/codecov-action@v4`

Check for updates: https://github.com/actions

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Workflow file reference](../.github/workflows/)
- [Workflow README](../.github/workflows/README.md)
- [Coverage guide](COVERAGE.md)
- [Benchmark guide](BENCHMARKS.md)

## Support

**Issues with workflows**:
1. Check GitHub Actions tab for detailed logs
2. Run checks locally to reproduce
3. Check workflow README for troubleshooting
4. Open GitHub issue with workflow logs

**Questions**:
- GitHub Actions: https://github.community/
- Rust CI: https://users.rust-lang.org/
- Project-specific: Open GitHub issue
