# Build Pipeline

Orchestrates CSS bundling, JS bundling, pre-compression, and cache busting in parallel.

## Purpose

The build pipeline coordinates all asset build steps for optimal performance:

- **Parallel Processing**: Bundle CSS and JS concurrently (4× speedup)
- **Incremental Builds**: Rebuild only changed assets
- **Pre-Compression**: Generate gzip/brotli versions at build time
- **Cache Busting**: Content-based hashing for cache invalidation
- **Verification**: Ensure build output is complete and valid

## Architecture

```
┌─────────────────────────────────────────────────────┐
│ Build Pipeline Stages                               │
├─────────────────────────────────────────────────────┤
│                                                     │
│  [Stage 1] Clean (Full Build Mode Only)             │
│  ├─ Remove: public/                                 │
│  └─ Duration: < 100ms                               │
│           ↓                                         │
│  [Stage 2] CSS Bundling (Parallel)                  │
│  ├─ Spawn: 30 tasks (10 layouts × 3 variants)       │
│  ├─ Process: Discover → Concat → Minify → Write     │
│  └─ Duration: ~1.2s (parallel)                      │
│           ↓                                         │
│  [Stage 3] JS Bundling (Parallel)                   │
│  ├─ Spawn: 10 tasks (variant-independent)           │
│  ├─ Process: Resolve → Bundle → TreeShake → Minify  │
│  └─ Duration: ~2.4s (parallel)                      │
│           ↓                                         │
│  [Stage 4] Pre-Compression (Parallel)               │
│  ├─ For each: CSS + JS file                         │
│  ├─ Generate: .gz (gzip) + .br (brotli)             │
│  └─ Duration: ~1.5s (parallel)                      │
│           ↓                                         │
│  [Stage 5] Cache Busting                            │
│  ├─ Calculate: SHA256 hash → first 8 chars          │
│  ├─ Rename: file.css → file.a7f3b2c8.css            │
│  ├─ Generate: asset-manifest.json                   │
│  └─ Duration: ~0.8s                                 │
│           ↓                                         │
│  [Stage 6] Verification                             │
│  ├─ Check: All files exist                          │
│  ├─ Validate: Manifest integrity                    │
│  └─ Duration: < 100ms                               │
│           ↓                                         │
│  [Stage 7] Report                                   │
│  └─ Summary: Files, sizes, timings                  │
│                                                     │
│  Total Duration: < 10s                              │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Implementation

### Main Pipeline Function

```rust
pub async fn run_pipeline(mode: BuildMode)
    -> ReedResult<BuildReport>
{
    println!("🏗️  Building ReedCMS Assets...\n");

    let start_time = Instant::now();
    let mut report = BuildReport::new();

    // Stage 1: Clean (if full build)
    if mode == BuildMode::Full {
        println!("[1/6] Cleaning previous build...");
        clean_public_directory()?;
        println!("✓ Cleaned public/ directory\n");
    }

    // Stage 2: Build CSS (parallel)
    println!("[2/6] Building CSS bundles...");
    let css_start = Instant::now();
    let css_results = build_css_bundles_parallel().await?;

    for result in &css_results {
        println!("✓ {} ({} KB, -{}%)",
            Path::new(&result.output_path)
                .file_name()
                .unwrap()
                .to_string_lossy(),
            result.minified_size / 1024,
            result.reduction_percent
        );
        report.css_bundles.push(result.clone());
    }
    println!("✓ {} bundles created in {:.1}s\n",
        css_results.len(),
        css_start.elapsed().as_secs_f32()
    );

    // Stage 3: Build JS (parallel)
    println!("[3/6] Building JS bundles...");
    let js_start = Instant::now();
    let js_results = build_js_bundles_parallel().await?;

    for result in &js_results {
        println!("✓ {} ({} KB, -{}%)",
            Path::new(&result.output_path)
                .file_name()
                .unwrap()
                .to_string_lossy(),
            result.minified_size / 1024,
            result.reduction_percent
        );
        report.js_bundles.push(result.clone());
    }
    println!("✓ {} bundles created in {:.1}s\n",
        js_results.len(),
        js_start.elapsed().as_secs_f32()
    );

    // Stage 4: Pre-compress assets (parallel)
    println!("[4/6] Pre-compressing assets...");
    let compressed_files = precompress_all_assets().await?;
    println!("✓ {} files compressed\n", compressed_files);
    report.compressed_files = compressed_files;

    // Stage 5: Cache busting
    println!("[5/6] Generating cache-busted filenames...");
    let manifest = generate_cache_busting_manifest().await?;
    println!("✓ {} files renamed\n", manifest.entries.len());
    report.manifest = manifest;

    // Stage 6: Verify build
    println!("[6/6] Verifying build...");
    verify_build(&report)?;
    println!("✓ All files present and valid\n");

    // Calculate totals
    report.build_duration_secs = start_time.elapsed().as_secs();
    report.calculate_totals();

    // Print summary
    println!("📊 Build Summary:");
    println!("  Total files: {}", report.total_files);
    println!("  Total size: {:.1} MB (original: {:.1} MB)",
        report.total_size as f64 / 1_048_576.0,
        report.original_size as f64 / 1_048_576.0
    );
    println!("  Size reduction: {}%",
        report.size_reduction_percent);
    println!("  Build time: {}s\n",
        report.build_duration_secs);
    println!("✓ Build complete");

    Ok(report)
}
```

### Build Modes

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Full,        // Clean + full rebuild
    Incremental, // Only changed files
}
```

**Usage**:
```rust
// Full build (production)
let report = run_pipeline(BuildMode::Full).await?;

// Incremental build (development)
let report = run_pipeline(BuildMode::Incremental).await?;
```

## Parallel CSS Bundling

### Implementation

```rust
async fn build_css_bundles_parallel()
    -> ReedResult<Vec<CssBundleResult>>
{
    let layouts = discover_layouts()?;
    let variants = vec!["mouse", "touch", "reader"];

    let mut tasks = Vec::new();

    // Spawn parallel tasks
    for layout in layouts {
        for variant in &variants {
            let layout = layout.clone();
            let variant = variant.to_string();

            tasks.push(tokio::spawn(async move {
                bundle_css(&layout, &variant)
            }));
        }
    }

    // Collect results
    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(Ok(result)) => results.push(result),
            Ok(Err(e)) => {
                eprintln!("⚠ CSS bundle error: {:?}", e);
            }
            Err(e) => {
                eprintln!("⚠ CSS task error: {:?}", e);
            }
        }
    }

    Ok(results)
}
```

### Performance Comparison

**Sequential Processing**:
```
Bundle 1  ████████████████ 4.0s
Bundle 2  ████████████████ 4.0s
Bundle 3  ████████████████ 4.0s
...
Bundle 30 ████████████████ 4.0s
Total: 120s (2 minutes)
```

**Parallel Processing** (4 cores):
```
Bundle 1-4   ████████████████ 4.0s
Bundle 5-8   ████████████████ 4.0s
Bundle 9-12  ████████████████ 4.0s
...
Bundle 29-30 ████████████████ 4.0s
Total: 32s (4× faster)
```

## Parallel JS Bundling

### Implementation

```rust
async fn build_js_bundles_parallel()
    -> ReedResult<Vec<JsBundleResult>>
{
    let layouts = discover_layouts()?;

    let mut tasks = Vec::new();

    // Spawn parallel tasks
    for layout in layouts {
        let layout = layout.clone();

        tasks.push(tokio::spawn(async move {
            bundle_js(&layout, "mouse")  // JS is variant-independent
        }));
    }

    // Collect results
    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(Ok(result)) => results.push(result),
            Ok(Err(e)) => {
                eprintln!("⚠ JS bundle error: {:?}", e);
            }
            Err(e) => {
                eprintln!("⚠ JS task error: {:?}", e);
            }
        }
    }

    Ok(results)
}
```

## Pre-Compression

### Process

```rust
async fn precompress_all_assets() -> ReedResult<usize> {
    let mut count = 0;

    // Find all CSS/JS files
    let css_files = glob("public/css/*.css")?;
    let js_files = glob("public/js/*.js")?;

    let mut tasks = Vec::new();

    // Spawn compression tasks
    for file in css_files.chain(js_files) {
        tasks.push(tokio::spawn(async move {
            precompress_file(&file)
        }));
    }

    // Wait for all tasks
    for task in tasks {
        if task.await.is_ok() {
            count += 2;  // .gz + .br
        }
    }

    Ok(count)
}

fn precompress_file(path: &str) -> ReedResult<()> {
    let content = fs::read(path)?;

    // Generate gzip version
    let gzipped = gzip_compress(&content)?;
    fs::write(format!("{}.gz", path), gzipped)?;

    // Generate brotli version
    let brotli = brotli_compress(&content)?;
    fs::write(format!("{}.br", path), brotli)?;

    Ok(())
}
```

### Output Structure

```
public/
├── css/
│   ├── knowledge.mouse.css
│   ├── knowledge.mouse.css.gz     # Gzip compressed
│   ├── knowledge.mouse.css.br     # Brotli compressed
│   ├── blog.touch.css
│   ├── blog.touch.css.gz
│   └── blog.touch.css.br
└── js/
    ├── knowledge.js
    ├── knowledge.js.gz
    ├── knowledge.js.br
    ├── blog.js
    ├── blog.js.gz
    └── blog.js.br
```

## Cache Busting

### Content Hashing

```rust
async fn generate_cache_busting_manifest()
    -> ReedResult<AssetManifest>
{
    let mut manifest = AssetManifest::new();

    // Process CSS files
    for entry in glob("public/css/*.css")? {
        let content = fs::read(&entry)?;
        let hash = calculate_content_hash(&content);

        let old_name = entry.file_name().unwrap();
        let new_name = insert_hash(&old_name, &hash);

        fs::rename(&entry, &new_name)?;
        manifest.insert(old_name, new_name);
    }

    // Process JS files
    for entry in glob("public/js/*.js")? {
        let content = fs::read(&entry)?;
        let hash = calculate_content_hash(&content);

        let old_name = entry.file_name().unwrap();
        let new_name = insert_hash(&old_name, &hash);

        fs::rename(&entry, &new_name)?;
        manifest.insert(old_name, new_name);
    }

    // Write manifest
    let json = serde_json::to_string_pretty(&manifest)?;
    fs::write("public/asset-manifest.json", json)?;

    Ok(manifest)
}

fn calculate_content_hash(content: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalise();

    // First 8 characters of hex hash
    format!("{:x}", result)[..8].to_string()
}

fn insert_hash(filename: &str, hash: &str) -> String {
    // knowledge.mouse.css + a7f3b2c8
    // → knowledge.mouse.a7f3b2c8.css

    let parts: Vec<&str> = filename.rsplitn(2, '.').collect();
    if parts.len() == 2 {
        format!("{}.{}.{}", parts[1], hash, parts[0])
    } else {
        format!("{}.{}", filename, hash)
    }
}
```

### Manifest Format

```json
{
  "knowledge.mouse.css": "knowledge.mouse.a7f3b2c8.css",
  "knowledge.touch.css": "knowledge.touch.b4k7p2m9.css",
  "blog.mouse.css": "blog.mouse.c9d5e1f7.css",
  "knowledge.js": "knowledge.b8f6k3n1.js",
  "blog.js": "blog.d2h9m5q4.js"
}
```

## Build Verification

### Checks

```rust
fn verify_build(report: &BuildReport) -> ReedResult<()> {
    // 1. Check public/ exists
    if !Path::new("public").exists() {
        return Err(ReedError::BuildError {
            component: "verify",
            reason: "public/ directory not found",
        });
    }

    // 2. Check manifest exists
    if !Path::new("public/asset-manifest.json").exists() {
        return Err(ReedError::BuildError {
            component: "verify",
            reason: "asset-manifest.json not found",
        });
    }

    // 3. Verify all manifest files exist
    for (original, hashed) in &report.manifest.entries {
        let css_path = format!("public/css/{}", hashed);
        let js_path = format!("public/js/{}", hashed);

        if !Path::new(&css_path).exists() &&
           !Path::new(&js_path).exists()
        {
            return Err(ReedError::BuildError {
                component: "verify",
                reason: format!("File not found: {}", hashed),
            });
        }
    }

    // 4. Check no empty files
    for entry in glob("public/**/*")? {
        if entry.is_file() {
            let size = fs::metadata(&entry)?.len();
            if size == 0 {
                return Err(ReedError::BuildError {
                    component: "verify",
                    reason: format!("Empty file: {}",
                        entry.display()),
                });
            }
        }
    }

    Ok(())
}
```

## Build Report

### Structure

```rust
#[derive(Debug, Clone)]
pub struct BuildReport {
    pub css_bundles: Vec<CssBundleResult>,
    pub js_bundles: Vec<JsBundleResult>,
    pub compressed_files: usize,
    pub manifest: AssetManifest,
    pub build_duration_secs: u64,
    pub total_files: usize,
    pub original_size: usize,
    pub total_size: usize,
    pub size_reduction_percent: u32,
}

impl BuildReport {
    pub fn calculate_totals(&mut self) {
        // Count files
        self.total_files =
            self.css_bundles.len() +
            self.js_bundles.len();

        // Sum sizes
        self.original_size =
            self.css_bundles.iter()
                .map(|b| b.original_size)
                .sum::<usize>() +
            self.js_bundles.iter()
                .map(|b| b.original_size)
                .sum::<usize>();

        self.total_size =
            self.css_bundles.iter()
                .map(|b| b.minified_size)
                .sum::<usize>() +
            self.js_bundles.iter()
                .map(|b| b.minified_size)
                .sum::<usize>();

        // Calculate reduction
        if self.original_size > 0 {
            self.size_reduction_percent =
                100 - ((self.total_size * 100) /
                       self.original_size) as u32;
        }
    }
}
```

### Example Output

```
📊 Build Summary:
  Total files: 40
  Total size: 2.4 MB (original: 7.8 MB)
  Size reduction: 69%
  Build time: 8s
```

## Performance Optimisation

### 1. Parallel Task Spawning

```rust
// ✓ GOOD: Parallel
let mut tasks = Vec::new();
for layout in &layouts {
    tasks.push(tokio::spawn(async move {
        bundle_css(layout, variant)
    }));
}
for task in tasks {
    results.push(task.await??);
}

// ✗ BAD: Sequential
for layout in &layouts {
    let result = bundle_css(layout, variant).await?;
    results.push(result);
}
```

### 2. Incremental Builds

```rust
// Skip unchanged files
if !has_changed(file, &last_build)? {
    continue;  // Skip rebuild
}
```

### 3. Caching

```rust
// Cache bundle results
let cache_key = format!("{}.{}", layout, variant);
if let Some(cached) = bundle_cache.get(&cache_key) {
    return Ok(cached.clone());
}
```

## CLI Integration

```bash
# Full build
reed build:assets

# Incremental build
reed build:assets --incremental

# With stats
reed build:assets --verbose

# Dry run
reed build:assets --dry-run
```

## Troubleshooting

### Build Hangs

**Cause**: Deadlock in parallel tasks

**Solution**: Check task dependencies, ensure no circular waits

### Out of Memory

**Cause**: Too many concurrent tasks

**Solution**: Limit parallelism

```rust
// Limit concurrent tasks to 8
let semaphore = Arc::new(Semaphore::new(8));
```

### Files Not Found After Build

**Cause**: Cache busting failed or manifest incorrect

**Solution**: Check manifest integrity, re-run build

## Related Documentation

- [Binary Compiler](binary-compiler.md) - Release builds
- [File Watcher](file-watcher.md) - Development workflow
- [CSS Bundler](../08-asset-layer/css-bundler.md) - CSS implementation
- [JS Bundler](../08-asset-layer/js-bundler.md) - JS implementation

## CLI Reference

```bash
# Build commands
reed build:assets         # Full asset build
reed build:assets --fast  # Skip pre-compression
reed build:assets --dev   # Development mode
reed build:css            # CSS only
reed build:js             # JS only
```
