# REED-09-02: Asset Pipeline and Build Orchestration

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
- **ID**: REED-09-02
- **Title**: Asset Pipeline and Build Orchestration
- **Layer**: Build Layer (REED-09)
- **Priority**: High
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-08-01, REED-08-02, REED-08-03

## Summary Reference
- **Section**: Asset Pipeline
- **Lines**: 1038-1040 in project_summary.md
- **Key Concepts**: Build orchestration, parallel processing, incremental builds, cache busting

## Objective
Implement comprehensive asset pipeline that orchestrates CSS bundling, JavaScript bundling, asset pre-compression, cache busting via content hashing, and parallel processing for optimal build performance with incremental rebuild capability.

## Requirements

### Build Pipeline Stages

```
1. Clean (optional)
   └─> Remove public/ directory

2. CSS Pipeline (parallel)
   ├─> Bundle CSS per layout/variant
   ├─> Minify CSS
   ├─> Generate source maps
   └─> Output to public/css/

3. JavaScript Pipeline (parallel)
   ├─> Resolve dependencies
   ├─> Bundle JS per layout/variant
   ├─> Tree shake unused code
   ├─> Minify JS
   ├─> Generate source maps
   └─> Output to public/js/

4. Asset Pre-compression (parallel)
   ├─> Compress CSS with gzip/brotli
   ├─> Compress JS with gzip/brotli
   └─> Output .gz and .br files

5. Cache Busting
   ├─> Calculate content hashes
   ├─> Rename files with hashes
   └─> Generate asset manifest

6. Copy Static Assets
   ├─> Copy images/
   ├─> Copy fonts/
   └─> Copy docs/

7. Verification
   ├─> Verify all files exist
   ├─> Check file sizes
   └─> Generate build report
```

### Implementation (`build/pipeline.rs`)

```rust
/// Runs complete asset build pipeline.
///
/// ## Build Modes
/// - Full: Clean + all stages
/// - Incremental: Only changed files
/// - Watch: Continuous rebuild on changes
///
/// ## Performance
/// - Full build: < 10s for 10 layouts
/// - Incremental: < 2s for single layout change
/// - Parallel processing: 4x faster than sequential
///
/// ## Output
/// ```
/// 🏗️  Building ReedCMS Assets...
///
/// [1/7] Cleaning previous build...
/// ✓ Cleaned public/ directory
///
/// [2/7] Building CSS bundles...
/// ✓ knowledge.mouse.css (3.8 KB, -67%)
/// ✓ knowledge.touch.css (3.2 KB, -65%)
/// ✓ blog.mouse.css (4.1 KB, -68%)
/// ✓ 15 bundles created in 1.2s
///
/// [3/7] Building JS bundles...
/// ✓ knowledge.mouse.js (9.2 KB, -70%)
/// ✓ knowledge.touch.js (8.1 KB, -68%)
/// ✓ blog.mouse.js (10.3 KB, -72%)
/// ✓ 15 bundles created in 2.4s
///
/// [4/7] Pre-compressing assets...
/// ✓ 30 files compressed (gzip: -68%, brotli: -73%)
///
/// [5/7] Generating cache-busted filenames...
/// ✓ knowledge.mouse.a7f3k9s2.css
/// ✓ 30 files renamed with content hashes
///
/// [6/7] Copying static assets...
/// ✓ Copied 24 images, 4 fonts, 2 documents
///
/// [7/7] Verifying build...
/// ✓ All files present and valid
///
/// 📊 Build Summary:
///   Total files: 89
///   Total size: 2.4 MB (original: 7.8 MB)
///   Size reduction: 69%
///   Build time: 8.7s
///
/// ✓ Build complete
/// ```
pub async fn run_pipeline(mode: BuildMode) -> ReedResult<BuildReport> {
    println!("🏗️  Building ReedCMS Assets...\n");

    let start_time = std::time::Instant::now();
    let mut report = BuildReport::new();

    // Stage 1: Clean (if full build)
    if mode == BuildMode::Full {
        println!("[1/7] Cleaning previous build...");
        clean_public_directory()?;
        println!("✓ Cleaned public/ directory\n");
    }

    // Stage 2: Build CSS (parallel)
    println!("[2/7] Building CSS bundles...");
    let css_start = std::time::Instant::now();
    let css_results = build_css_bundles_parallel().await?;

    for result in &css_results {
        println!("✓ {} ({} KB, -{}%)",
            std::path::Path::new(&result.output_path).file_name().unwrap().to_string_lossy(),
            result.minified_size / 1024,
            result.reduction_percent
        );
        report.add_css_bundle(result);
    }
    println!("✓ {} bundles created in {:.1}s\n",
        css_results.len(),
        css_start.elapsed().as_secs_f32()
    );

    // Stage 3: Build JS (parallel)
    println!("[3/7] Building JS bundles...");
    let js_start = std::time::Instant::now();
    let js_results = build_js_bundles_parallel().await?;

    for result in &js_results {
        println!("✓ {} ({} KB, -{}%)",
            std::path::Path::new(&result.output_path).file_name().unwrap().to_string_lossy(),
            result.minified_size / 1024,
            result.reduction_percent
        );
        report.add_js_bundle(result);
    }
    println!("✓ {} bundles created in {:.1}s\n",
        js_results.len(),
        js_start.elapsed().as_secs_f32()
    );

    // Stage 4: Pre-compress assets (parallel)
    println!("[4/7] Pre-compressing assets...");
    let compress_stats = precompress_assets_parallel().await?;
    println!("✓ {} files compressed (gzip: -{}%, brotli: -{}%)\n",
        compress_stats.total_files,
        100 - (compress_stats.total_gzip_size * 100 / compress_stats.total_original_size),
        100 - (compress_stats.total_brotli_size * 100 / compress_stats.total_original_size)
    );
    report.compression_stats = compress_stats;

    // Stage 5: Cache busting
    println!("[5/7] Generating cache-busted filenames...");
    let manifest = generate_cache_busting_manifest().await?;
    println!("✓ {} files renamed with content hashes\n", manifest.entries.len());
    report.manifest = manifest;

    // Stage 6: Copy static assets
    println!("[6/7] Copying static assets...");
    let copy_stats = copy_static_assets()?;
    println!("✓ Copied {} images, {} fonts, {} documents\n",
        copy_stats.images_count,
        copy_stats.fonts_count,
        copy_stats.docs_count
    );
    report.copy_stats = copy_stats;

    // Stage 7: Verify build
    println!("[7/7] Verifying build...");
    verify_build(&report)?;
    println!("✓ All files present and valid\n");

    // Calculate totals
    report.build_duration = start_time.elapsed();
    report.calculate_totals();

    // Print summary
    println!("📊 Build Summary:");
    println!("  Total files: {}", report.total_files);
    println!("  Total size: {:.1} MB (original: {:.1} MB)",
        report.total_size as f64 / 1_048_576.0,
        report.original_size as f64 / 1_048_576.0
    );
    println!("  Size reduction: {}%", report.size_reduction_percent);
    println!("  Build time: {:.1}s", report.build_duration.as_secs_f32());
    println!("\n✓ Build complete");

    Ok(report)
}

/// Build mode enum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Full,        // Clean + full rebuild
    Incremental, // Only changed files
    Watch,       // Continuous rebuild
}

/// Cleans public directory.
fn clean_public_directory() -> ReedResult<()> {
    if std::path::Path::new("public").exists() {
        std::fs::remove_dir_all("public")?;
    }
    std::fs::create_dir_all("public")?;
    Ok(())
}

/// Builds CSS bundles in parallel.
async fn build_css_bundles_parallel() -> ReedResult<Vec<CssBundleResult>> {
    let layouts = discover_layouts()?;
    let variants = vec!["mouse", "touch", "reader"];

    let mut tasks = Vec::new();

    for layout in layouts {
        for variant in &variants {
            let layout = layout.clone();
            let variant = variant.to_string();

            tasks.push(tokio::spawn(async move {
                reedcms::assets::css::bundler::bundle_css(&layout, &variant)
            }));
        }
    }

    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(Ok(result)) => results.push(result),
            Ok(Err(e)) => eprintln!("⚠ CSS bundle error: {:?}", e),
            Err(e) => eprintln!("⚠ CSS task error: {:?}", e),
        }
    }

    Ok(results)
}

/// Builds JS bundles in parallel.
async fn build_js_bundles_parallel() -> ReedResult<Vec<JsBundleResult>> {
    let layouts = discover_layouts()?;
    let variants = vec!["mouse", "touch", "reader"];

    let mut tasks = Vec::new();

    for layout in layouts {
        for variant in &variants {
            let layout = layout.clone();
            let variant = variant.to_string();

            tasks.push(tokio::spawn(async move {
                reedcms::assets::js::bundler::bundle_js(&layout, &variant)
            }));
        }
    }

    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(Ok(result)) => results.push(result),
            Ok(Err(e)) => eprintln!("⚠ JS bundle error: {:?}", e),
            Err(e) => eprintln!("⚠ JS task error: {:?}", e),
        }
    }

    Ok(results)
}

/// Pre-compresses assets in parallel.
async fn precompress_assets_parallel() -> ReedResult<CompressionStats> {
    reedcms::assets::server::precompress::precompress_assets()
}

/// Discovers layouts from templates directory.
fn discover_layouts() -> ReedResult<Vec<String>> {
    let mut layouts = Vec::new();
    let entries = std::fs::read_dir("templates/layouts")?;

    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                layouts.push(name.to_string());
            }
        }
    }

    Ok(layouts)
}
```

### Cache Busting (`build/cache_bust.rs`)

```rust
/// Generates cache-busted filenames with content hashes.
///
/// ## Process
/// 1. Find all assets in public/
/// 2. Calculate content hash (first 8 chars of SHA256)
/// 3. Rename files with hash in filename
/// 4. Generate asset manifest JSON
///
/// ## Filename Format
/// - Original: knowledge.mouse.css
/// - Cache-busted: knowledge.mouse.a7f3k9s2.css
///
/// ## Manifest Format
/// ```json
/// {
///   "knowledge.mouse.css": "knowledge.mouse.a7f3k9s2.css",
///   "knowledge.mouse.js": "knowledge.mouse.b4k7p2m9.js"
/// }
/// ```
///
/// ## Benefits
/// - Browser cache invalidation on file change
/// - Long cache TTLs without stale content
/// - Automatic versioning
pub async fn generate_cache_busting_manifest() -> ReedResult<AssetManifest> {
    let mut manifest = AssetManifest {
        entries: HashMap::new(),
    };

    // Process CSS files
    process_directory("public/css", &mut manifest).await?;

    // Process JS files
    process_directory("public/js", &mut manifest).await?;

    // Write manifest
    write_manifest(&manifest)?;

    Ok(manifest)
}

/// Processes directory for cache busting.
async fn process_directory(dir: &str, manifest: &mut AssetManifest) -> ReedResult<()> {
    let entries = std::fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();

            // Skip already processed files and source maps
            if file_name.contains('.') && !file_name.ends_with(".map")
                && !file_name.ends_with(".gz") && !file_name.ends_with(".br") {

                // Calculate content hash
                let content = std::fs::read(&path)?;
                let hash = calculate_content_hash(&content);

                // Generate new filename
                let new_name = insert_hash_into_filename(&file_name, &hash);
                let new_path = path.parent().unwrap().join(&new_name);

                // Rename file
                std::fs::rename(&path, &new_path)?;

                // Add to manifest
                manifest.entries.insert(file_name, new_name);

                println!("✓ {}", new_name);
            }
        }
    }

    Ok(())
}

/// Calculates content hash (first 8 chars of SHA256).
fn calculate_content_hash(content: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)[..8].to_string()
}

/// Inserts hash into filename before extension.
///
/// ## Examples
/// - knowledge.mouse.css + a7f3k9s2 → knowledge.mouse.a7f3k9s2.css
/// - blog.touch.js + b4k7p2m9 → blog.touch.b4k7p2m9.js
fn insert_hash_into_filename(filename: &str, hash: &str) -> String {
    let parts: Vec<&str> = filename.rsplitn(2, '.').collect();
    if parts.len() == 2 {
        format!("{}.{}.{}", parts[1], hash, parts[0])
    } else {
        format!("{}.{}", filename, hash)
    }
}

/// Writes asset manifest to JSON file.
fn write_manifest(manifest: &AssetManifest) -> ReedResult<()> {
    let json = serde_json::to_string_pretty(manifest).map_err(|e| {
        ReedError::SerializationError {
            data_type: "AssetManifest".to_string(),
            reason: e.to_string(),
        }
    })?;

    std::fs::write("public/asset-manifest.json", json).map_err(|e| ReedError::IoError {
        operation: "write".to_string(),
        path: "public/asset-manifest.json".to_string(),
        reason: e.to_string(),
    })
}

/// Asset manifest structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    pub entries: HashMap<String, String>,
}
```

### Static Asset Copying (`build/copy_assets.rs`)

```rust
/// Copies static assets to public directory.
///
/// ## Assets Copied
/// - assets/images/ → public/images/
/// - assets/fonts/ → public/fonts/
/// - assets/docs/ → public/docs/
///
/// ## Performance
/// - Parallel copying with rayon
/// - < 5s for 1000 files
pub fn copy_static_assets() -> ReedResult<CopyStats> {
    let mut stats = CopyStats {
        images_count: 0,
        fonts_count: 0,
        docs_count: 0,
    };

    // Copy images
    if std::path::Path::new("assets/images").exists() {
        stats.images_count = copy_directory("assets/images", "public/images")?;
    }

    // Copy fonts
    if std::path::Path::new("assets/fonts").exists() {
        stats.fonts_count = copy_directory("assets/fonts", "public/fonts")?;
    }

    // Copy docs
    if std::path::Path::new("assets/docs").exists() {
        stats.docs_count = copy_directory("assets/docs", "public/docs")?;
    }

    Ok(stats)
}

/// Copies directory recursively.
fn copy_directory(src: &str, dst: &str) -> ReedResult<usize> {
    std::fs::create_dir_all(dst)?;

    let mut count = 0;
    let entries = std::fs::read_dir(src)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let dst_path = std::path::Path::new(dst).join(file_name);

        if path.is_dir() {
            count += copy_directory(
                path.to_str().unwrap(),
                dst_path.to_str().unwrap()
            )?;
        } else {
            std::fs::copy(&path, &dst_path)?;
            count += 1;
        }
    }

    Ok(count)
}

/// Copy statistics.
#[derive(Debug, Clone)]
pub struct CopyStats {
    pub images_count: usize,
    pub fonts_count: usize,
    pub docs_count: usize,
}
```

### Build Verification (`build/verify.rs`)

```rust
/// Verifies build output.
///
/// ## Checks
/// - All expected files exist
/// - No empty files
/// - File sizes reasonable
/// - Manifest valid
pub fn verify_build(report: &BuildReport) -> ReedResult<()> {
    // Check public directory exists
    if !std::path::Path::new("public").exists() {
        return Err(ReedError::BuildError {
            component: "verify".to_string(),
            reason: "public/ directory not found".to_string(),
        });
    }

    // Check manifest exists
    if !std::path::Path::new("public/asset-manifest.json").exists() {
        return Err(ReedError::BuildError {
            component: "verify".to_string(),
            reason: "asset-manifest.json not found".to_string(),
        });
    }

    // Verify all manifest files exist
    for (_, hashed_name) in &report.manifest.entries {
        let css_path = format!("public/css/{}", hashed_name);
        let js_path = format!("public/js/{}", hashed_name);

        if !std::path::Path::new(&css_path).exists()
            && !std::path::Path::new(&js_path).exists() {
            return Err(ReedError::BuildError {
                component: "verify".to_string(),
                reason: format!("Manifest file not found: {}", hashed_name),
            });
        }
    }

    Ok(())
}
```

### Build Report (`build/report.rs`)

```rust
/// Build report structure.
#[derive(Debug, Clone)]
pub struct BuildReport {
    pub css_bundles: Vec<CssBundleResult>,
    pub js_bundles: Vec<JsBundleResult>,
    pub compression_stats: CompressionStats,
    pub manifest: AssetManifest,
    pub copy_stats: CopyStats,
    pub build_duration: std::time::Duration,
    pub total_files: usize,
    pub original_size: usize,
    pub total_size: usize,
    pub size_reduction_percent: u32,
}

impl BuildReport {
    pub fn new() -> Self {
        Self {
            css_bundles: Vec::new(),
            js_bundles: Vec::new(),
            compression_stats: CompressionStats::default(),
            manifest: AssetManifest { entries: HashMap::new() },
            copy_stats: CopyStats { images_count: 0, fonts_count: 0, docs_count: 0 },
            build_duration: std::time::Duration::from_secs(0),
            total_files: 0,
            original_size: 0,
            total_size: 0,
            size_reduction_percent: 0,
        }
    }

    pub fn add_css_bundle(&mut self, result: &CssBundleResult) {
        self.css_bundles.push(result.clone());
    }

    pub fn add_js_bundle(&mut self, result: &JsBundleResult) {
        self.js_bundles.push(result.clone());
    }

    pub fn calculate_totals(&mut self) {
        // Calculate total files
        self.total_files = self.css_bundles.len()
            + self.js_bundles.len()
            + self.copy_stats.images_count
            + self.copy_stats.fonts_count
            + self.copy_stats.docs_count;

        // Calculate sizes
        self.original_size = self.css_bundles.iter().map(|b| b.original_size).sum::<usize>()
            + self.js_bundles.iter().map(|b| b.original_size).sum::<usize>();

        self.total_size = self.css_bundles.iter().map(|b| b.minified_size).sum::<usize>()
            + self.js_bundles.iter().map(|b| b.minified_size).sum::<usize>();

        // Calculate reduction
        if self.original_size > 0 {
            self.size_reduction_percent =
                100 - ((self.total_size * 100) / self.original_size) as u32;
        }
    }
}
```

## Implementation Files

### Primary Implementation
- `build/pipeline.rs` - Build orchestration
- `build/cache_bust.rs` - Cache busting
- `build/copy_assets.rs` - Static asset copying
- `build/verify.rs` - Build verification
- `build/report.rs` - Build reporting

### Test Files
- `build/pipeline.test.rs`
- `build/cache_bust.test.rs`
- `build/copy_assets.test.rs`
- `build/verify.test.rs`
- `build/report.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test cache hash generation
- [ ] Test filename hash insertion
- [ ] Test manifest generation
- [ ] Test directory copying
- [ ] Test build verification
- [ ] Test report calculation

### Integration Tests
- [ ] Test complete pipeline execution
- [ ] Test parallel bundle processing
- [ ] Test incremental builds
- [ ] Test cache busting with renames
- [ ] Test verification catches errors

### Performance Tests
- [ ] Full build: < 10s for 10 layouts
- [ ] Incremental build: < 2s
- [ ] Parallel speedup: 3-4x vs sequential
- [ ] Static asset copy: < 5s for 1000 files

## Acceptance Criteria
- [ ] Complete pipeline orchestration working
- [ ] CSS and JS bundling integrated
- [ ] Parallel processing functional
- [ ] Cache busting with content hashes
- [ ] Asset manifest generation
- [ ] Static asset copying
- [ ] Build verification implemented
- [ ] Comprehensive build report
- [ ] Incremental build support
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-08-01 (CSS Bundler), REED-08-02 (JS Bundler), REED-08-03 (Asset Server)

## Blocks
- REED-09-03 (File Watcher triggers pipeline)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1038-1040 in `project_summary.md`

## Notes
Asset pipeline orchestrates all build steps for optimal performance. Parallel processing reduces build time by 3-4x. Cache busting via content hashing enables aggressive browser caching without stale content. Incremental builds dramatically speed up development workflow. Build verification catches issues before deployment. Comprehensive reporting provides visibility into build process and output.
