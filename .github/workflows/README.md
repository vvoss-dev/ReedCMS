# GitHub Actions CI/CD Workflows

This directory contains GitHub Actions workflows for automated testing, coverage, and benchmarking.

## Workflows

### 1. test.yml - Test Suite

**Trigger**: Push to `main`/`develop`, Pull Requests to `main`

**Jobs**:
- **test**: Runs on Ubuntu and macOS with stable Rust
  - Builds the project
  - Runs unit tests
  - Runs integration tests
  - Uses caching for faster builds
  
- **clippy**: Linting with clippy
  - Runs on Ubuntu with stable Rust
  - Enforces clippy warnings as errors (`-D warnings`)
  
- **fmt**: Code formatting check
  - Runs on Ubuntu with stable Rust
  - Checks that code follows `rustfmt` style

**Status Badge**:
```markdown
[![Tests](https://github.com/YOUR_USERNAME/ReedCMS/actions/workflows/test.yml/badge.svg)](https://github.com/YOUR_USERNAME/ReedCMS/actions/workflows/test.yml)
```

### 2. coverage.yml - Code Coverage

**Trigger**: Push to `main`/`develop`, Pull Requests to `main`

**Jobs**:
- **coverage**: Measures code coverage
  - Uses `cargo-llvm-cov` for coverage generation
  - Uploads results to Codecov
  - Enforces 70% minimum coverage threshold
  - Fails if coverage drops below threshold

**Requirements**:
- `CODECOV_TOKEN` secret must be set in repository settings
- Get token from https://codecov.io after connecting repository

**Status Badge**:
```markdown
[![Coverage](https://codecov.io/gh/YOUR_USERNAME/ReedCMS/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/ReedCMS)
```

### 3. benchmark.yml - Performance Benchmarks

**Trigger**: 
- Push to `main` (automatic)
- Manual workflow dispatch

**Jobs**:
- **benchmark**: Runs performance benchmarks
  - Builds benchmarks without running (fast check)
  - Runs queries benchmark in test mode
  - Comments on PRs with benchmark status
  - Full benchmark runs only on `main` branch

**Note**: Full benchmark runs are expensive and only run on the main branch. PRs only validate that benchmarks compile.

## Setup Instructions

### 1. Enable GitHub Actions

GitHub Actions are automatically enabled for all repositories. No setup needed.

### 2. Set Up Codecov (Optional)

For coverage reporting:

1. Go to https://codecov.io
2. Sign in with GitHub
3. Add your repository
4. Copy the repository upload token
5. Go to GitHub repo → Settings → Secrets and variables → Actions
6. Add new secret: `CODECOV_TOKEN` with the token value

### 3. Verify Workflows

After pushing the workflow files:

1. Go to GitHub repo → Actions tab
2. You should see the workflows listed
3. Push a commit or create a PR to trigger workflows
4. Check that all jobs pass

## Workflow Status

Check workflow status:
- GitHub repo → Actions tab
- Click on a workflow to see runs
- Click on a run to see job details and logs

## Local Testing

Test workflows locally before pushing:

### Simulate test workflow:
```bash
cd reedbase
cargo build --verbose
cargo test --verbose
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

### Simulate coverage workflow:
```bash
cd reedbase
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
cargo llvm-cov --all-features --workspace --summary-only
```

### Simulate benchmark workflow:
```bash
cd reedbase
cargo bench --no-run
cargo bench --bench queries -- --test
```

## Caching

All workflows use GitHub Actions caching for:
- Cargo registry (`~/.cargo/registry`)
- Cargo git index (`~/.cargo/git`)
- Build artifacts (`target/`)

This significantly speeds up CI runs. Cache is invalidated when `Cargo.lock` changes.

## Matrix Strategy

The test workflow runs on multiple OS:
- **ubuntu-latest**: Linux testing
- **macos-latest**: macOS testing

Add more OS or Rust versions by extending the matrix:
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta, nightly]
```

## Failure Handling

### Tests Fail
- Check job logs in Actions tab
- Reproduce locally: `cargo test`
- Fix issues, commit, and push

### Clippy Warnings
- Check warnings in logs
- Fix locally: `cargo clippy --fix`
- Commit fixes

### Formatting Issues
- Fix locally: `cargo fmt`
- Commit formatted code

### Coverage Below Threshold
- Run coverage locally: `cargo llvm-cov --html --open`
- Identify untested code (red/yellow lines)
- Add tests for critical paths
- Re-run coverage

## Customization

### Change Coverage Threshold

Edit `coverage.yml`:
```yaml
if (( $(echo "$COVERAGE < 70.0" | bc -l) )); then  # Change 70.0 to desired threshold
```

### Add More Test Platforms

Edit `test.yml` matrix:
```yaml
matrix:
  os: [ubuntu-latest, macos-latest, windows-latest]
```

### Run Benchmarks on PRs

Edit `benchmark.yml` trigger:
```yaml
on:
  push:
    branches: [ main ]
  pull_request:  # Add this
    branches: [ main ]
```

Warning: This will significantly increase CI time.

## Cost Considerations

GitHub Actions is free for public repositories with these limits:
- Linux: Unlimited minutes
- macOS: 10x multiplier (6 minutes = 60 minutes quota)
- Windows: 2x multiplier

For private repositories:
- Free tier: 2,000 minutes/month
- Paid plans available

Current workflows estimate:
- Test: ~5 minutes (Linux) + ~7 minutes (macOS) = ~75 equivalent minutes
- Coverage: ~3 minutes (Linux) = 3 minutes
- Benchmark: ~2 minutes (Linux) = 2 minutes

Total per push to main: ~80 minutes equivalent

## Security

### Secrets
- Never commit tokens or secrets to workflows
- Use GitHub Secrets for sensitive data
- Available secrets: `${{ secrets.SECRET_NAME }}`

### Pull Request Security
- Workflows on PRs from forks run with limited permissions
- No access to secrets (for security)
- Codecov upload may fail on fork PRs (expected)

## Troubleshooting

### "cargo: command not found"
Rust toolchain not installed. Check `dtolnay/rust-toolchain` step.

### "cargo-llvm-cov: command not found"
Installation step failed. Check `taiki-e/install-action` step.

### Cache not working
- Verify `Cargo.lock` exists and is committed
- Check cache key in workflow file
- GitHub may evict old caches (normal)

### Slow CI runs
- Check if caching is working
- Consider reducing test matrix (fewer OS/Rust versions)
- Use `cargo test --release` for faster test execution (slower build)

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI with GitHub Actions](https://github.com/actions-rs)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [Codecov GitHub Action](https://github.com/codecov/codecov-action)

## Maintenance

Review and update workflows:
- Monthly: Check for action version updates
- Quarterly: Review OS versions (update if deprecated)
- Annually: Review Rust toolchain strategy (stable vs MSRV)

Subscribe to GitHub Actions updates:
https://github.blog/changelog/label/actions/
