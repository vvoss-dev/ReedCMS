# REED-04-08: CLI Build Commands

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function (CONFIG and SYSTEM use identical logic)
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs` - every topic (auth, login,...) has its own scoped rs service file
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-04-08
- **Title**: CLI Build System Commands
- **Layer**: CLI Layer (REED-04)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-04-01, REED-08-01, REED-08-02

## Summary Reference
- **Section**: CLI Build Commands
- **Lines**: 1063-1067 in project_summary.md
- **Key Concepts**: Binary compilation, asset bundling, watch mode, build pipeline

## Objective
Implement build system CLI commands for compiling ReedCMS binary, bundling assets, and watch mode for development with hot-reload capabilities.

## Requirements

### Commands to Implement

```bash
# Kernel compilation
reed build:kernel
reed build:kernel --release
reed build:kernel --target x86_64-unknown-linux-musl

# Public asset building
reed build:public
reed build:public --minify

# Complete build
reed build:complete
reed build:complete --debug "build-log.txt"

# Watch mode for development
reed build:watch
reed build:watch --templates-only
```

### Implementation (`src/reedcms/cli/build_commands.rs`)

```rust
/// Compiles ReedCMS binary kernel.
///
/// ## Flags
/// - --release: Build with optimizations
/// - --target: Specify target triple
/// - --features: Enable specific cargo features
///
/// ## Process
/// 1. Validate Cargo.toml
/// 2. Clean target directory (optional)
/// 3. Run cargo build
/// 4. Copy binary to output
/// 5. Report build statistics
///
/// ## Output
/// 🔨 Compiling ReedCMS kernel...
/// ✓ Cargo.toml validated
/// ✓ Building with profile: release
/// ✓ Target: x86_64-unknown-linux-gnu
///
///   Compiling reedcms v1.0.0
///   Finished release [optimized] target(s) in 2m 15s
///
/// ✓ Binary created: target/release/reed (8.2 MB)
/// ✓ Copied to: ./reed
///
/// Build summary:
/// - Profile: release
/// - Features: default
/// - Duration: 2m 15s
/// - Binary size: 8.2 MB
pub fn build_kernel(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Builds public assets (CSS, JS).
///
/// ## Flags
/// - --minify: Minify output files
/// - --source-maps: Generate source maps
///
/// ## Process
/// 1. Discover all template CSS/JS files
/// 2. Bundle per layout and variant
/// 3. Minify (if --minify)
/// 4. Generate file hashes for cache busting
/// 5. Update asset manifest
///
/// ## Output
/// 🎨 Building public assets...
/// ✓ Discovered 17 layouts
/// ✓ Processing CSS files...
///   - knowledge.mouse.css (15 KB → 8 KB minified)
///   - knowledge.touch.css (12 KB → 6 KB minified)
///   - blog.mouse.css (18 KB → 10 KB minified)
/// ✓ Processing JS files...
///   - app.js (45 KB → 22 KB minified)
/// ✓ Generated asset manifest
///
/// Build summary:
/// - Layouts processed: 17
/// - CSS files: 51 (280 KB → 145 KB)
/// - JS files: 8 (125 KB → 65 KB)
/// - Total reduction: 48.5%
/// - Duration: 3.2s
pub fn build_public(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Runs complete build pipeline.
///
/// ## Flags
/// - --debug: Output debug log to file
/// - --skip-tests: Skip test execution
/// - --parallel: Run builds in parallel
///
/// ## Process
/// 1. Validate project structure
/// 2. Run tests (unless --skip-tests)
/// 3. Build kernel (release mode)
/// 4. Build public assets
/// 5. Generate documentation
/// 6. Create build report
///
/// ## Output
/// 🚀 Running complete ReedCMS build...
///
/// [1/6] Validating project structure...
/// ✓ Project structure valid
///
/// [2/6] Running tests...
/// ✓ 156 tests passed in 8.5s
///
/// [3/6] Building kernel...
/// ✓ Binary built in 2m 15s
///
/// [4/6] Building public assets...
/// ✓ Assets built in 3.2s
///
/// [5/6] Generating documentation...
/// ✓ Documentation generated
///
/// [6/6] Creating build report...
/// ✓ Report saved to: build-report.txt
///
/// 🎉 Complete build finished successfully!
/// Total duration: 2m 35s
pub fn build_complete(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Starts watch mode for development.
///
/// ## Flags
/// - --templates-only: Watch only template files
/// - --debounce: Debounce delay in ms (default: 500)
///
/// ## Process
/// 1. Set up file watchers
/// 2. Monitor for changes
/// 3. Trigger appropriate rebuilds
/// 4. Notify via console
///
/// ## Watched Paths
/// - src/**/*.rs → Full rebuild
/// - templates/**/*.jinja → Template reload
/// - templates/**/*.css → Asset rebuild
/// - .reed/*.csv → Data reload
///
/// ## Output
/// 👀 Watch mode started...
/// Watching:
///   - src/ (Rust files)
///   - templates/ (Jinja + CSS files)
///   - .reed/ (Data files)
///
/// [10:15:32] Change detected: templates/layouts/blog/blog.mouse.jinja
/// [10:15:32] 🔄 Reloading template...
/// [10:15:32] ✓ Template reloaded (15ms)
///
/// [10:16:45] Change detected: src/reedcms/reedbase/get.rs
/// [10:16:45] 🔨 Rebuilding kernel...
/// [10:16:58] ✓ Rebuild complete (13.2s)
///
/// Press Ctrl+C to stop watching...
pub fn build_watch(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>
```

### Build Helpers (`src/reedcms/cli/build_helpers.rs`)

```rust
/// Validates Cargo.toml structure.
pub fn validate_cargo_toml() -> ReedResult<()>

/// Executes cargo build command.
pub fn execute_cargo_build(profile: &str, target: Option<&str>) -> ReedResult<BuildOutput>

/// Discovers asset files for bundling.
pub fn discover_assets(path: &str) -> ReedResult<Vec<AssetFile>>

/// Minifies CSS file.
pub fn minify_css(content: &str) -> ReedResult<String>

/// Minifies JavaScript file.
pub fn minify_js(content: &str) -> ReedResult<String>

/// Generates asset manifest with file hashes.
pub fn generate_asset_manifest(assets: &[ProcessedAsset]) -> ReedResult<String>

/// Creates build report.
pub fn create_build_report(stats: &BuildStats) -> String

#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub success: bool,
    pub duration_ms: u64,
    pub binary_path: String,
    pub binary_size: u64,
}

#[derive(Debug, Clone)]
pub struct BuildStats {
    pub kernel_duration_ms: u64,
    pub assets_duration_ms: u64,
    pub total_duration_ms: u64,
    pub binary_size: u64,
    pub assets_processed: usize,
    pub size_reduction_percent: f32,
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/cli/build_commands.rs` - Build commands
- `src/reedcms/cli/build_helpers.rs` - Build utilities

### Test Files
- `src/reedcms/cli/build_commands.test.rs`
- `src/reedcms/cli/build_helpers.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test cargo build execution
- [ ] Test asset discovery
- [ ] Test CSS minification
- [ ] Test JS minification
- [ ] Test asset manifest generation

### Integration Tests
- [ ] Test complete build pipeline
- [ ] Test watch mode trigger
- [ ] Test parallel builds
- [ ] Test error recovery

### Performance Tests
- [ ] Kernel build: < 3 minutes (release)
- [ ] Asset build: < 5s for 50 files
- [ ] Watch mode reaction: < 100ms
- [ ] Minification: < 1s for 100KB file

## Acceptance Criteria
- [ ] Kernel compilation working (debug + release)
- [ ] Public asset building functional
- [ ] Complete build pipeline operational
- [ ] Watch mode with hot-reload working
- [ ] CSS/JS minification implemented
- [ ] Asset manifest generation
- [ ] Build reports generated
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-04-01 (CLI Foundation)

## Blocks
- REED-09-01 (Binary Compiler needs these commands)
- REED-09-02 (Asset Pipeline needs build system)
- REED-09-03 (File Watcher needs watch mode)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1063-1067 in `project_summary.md`

## Notes
Build system is critical for development workflow. Watch mode enables rapid iteration. Asset minification reduces production bundle sizes. The complete build pipeline ensures all components are properly compiled and tested before deployment.