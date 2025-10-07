# Build Layer (09)

Orchestrates asset compilation, binary builds, and development workflow automation.

## Purpose

The Build Layer provides comprehensive build tooling for ReedCMS:

- **Asset Pipeline**: Orchestrates CSS/JS bundling, minification, compression
- **Binary Compiler**: Builds optimised release binaries with LTO
- **File Watcher**: Auto-rebuilds assets during development
- **Cache Busting**: Content-based hashing for browser cache invalidation
- **Release Packaging**: Creates deployable packages with checksums

## Architecture

```
┌─────────────────────────────────────────────────────┐
│ Build Layer Architecture                             │
├─────────────────────────────────────────────────────┤
│                                                       │
│  Development Mode                                    │
│  ┌──────────────────────────────────────────┐       │
│  │ File Watcher (reed build:watch)          │       │
│  │  ├─ Monitor: templates/, assets/         │       │
│  │  ├─ Detect: Changes (debounced 300ms)    │       │
│  │  ├─ Trigger: Incremental rebuilds        │       │
│  │  └─ Hot-Reload: Templates, configs       │       │
│  └──────────────────────────────────────────┘       │
│           ↓                                           │
│  ┌──────────────────────────────────────────┐       │
│  │ Asset Pipeline (incremental)              │       │
│  │  ├─ Changed CSS → Rebuild affected       │       │
│  │  ├─ Changed JS → Rebuild affected        │       │
│  │  └─ < 2s rebuild time                     │       │
│  └──────────────────────────────────────────┘       │
│                                                       │
│  Production Mode                                     │
│  ┌──────────────────────────────────────────┐       │
│  │ Asset Pipeline (reed build:assets)       │       │
│  │  ├─ Stage 1: Clean public/               │       │
│  │  ├─ Stage 2: Bundle CSS (parallel)       │       │
│  │  ├─ Stage 3: Bundle JS (parallel)        │       │
│  │  ├─ Stage 4: Pre-compress (gzip/brotli)  │       │
│  │  ├─ Stage 5: Cache bust (content hash)   │       │
│  │  ├─ Stage 6: Verify build                │       │
│  │  └─ < 10s total build time                │       │
│  └──────────────────────────────────────────┘       │
│           ↓                                           │
│  ┌──────────────────────────────────────────┐       │
│  │ Binary Compiler (reed build:release)     │       │
│  │  ├─ Cargo build --release                │       │
│  │  ├─ LTO: "fat" (link-time optimisation)  │       │
│  │  ├─ Strip: Debug symbols                 │       │
│  │  ├─ UPX: Optional compression            │       │
│  │  ├─ Checksums: SHA256 + MD5              │       │
│  │  └─ 2-5 minutes compile time              │       │
│  └──────────────────────────────────────────┘       │
│           ↓                                           │
│  ┌──────────────────────────────────────────┐       │
│  │ Release Packager (reed build:package)    │       │
│  │  ├─ Bundle: Binary + templates + configs │       │
│  │  ├─ Archive: tar.gz (Linux) / zip (Win)  │       │
│  │  ├─ Checksums: SHA256 for integrity      │       │
│  │  └─ Ready for deployment                  │       │
│  └──────────────────────────────────────────┘       │
│                                                       │
└─────────────────────────────────────────────────────┘
```

## Key Components

### 1. Asset Pipeline

**Purpose**: Orchestrates complete asset build process

**Process**:
```
1. Clean public/ directory (optional)
2. Bundle CSS → 30 bundles (10 layouts × 3 variants)
3. Bundle JS → 10 bundles (variant-independent)
4. Pre-compress → .gz + .br files for each asset
5. Cache bust → Rename with content hashes
6. Verify → Ensure all files present
```

**Performance**:
- Full build: < 10s
- Parallel processing: 4× faster than sequential
- Size reduction: ~70% (minification + compression)

**See**: [Build Pipeline](build-pipeline.md)

### 2. Binary Compiler

**Purpose**: Builds optimised production binaries

**Optimisations**:
```toml
[profile.release]
opt-level = 3              # Maximum optimisation
lto = "fat"                # Link-time optimisation
codegen-units = 1          # Better optimisation
strip = true               # Strip debug symbols
panic = "abort"            # Smaller binary
```

**Output**:
- Stripped binary: ~15 MB
- UPX compressed: ~6 MB (-60%)
- Checksums: SHA256 + MD5 for integrity

**See**: [Binary Compiler](binary-compiler.md)

### 3. File Watcher

**Purpose**: Auto-rebuilds during development

**Watches**:
- `templates/` → Hot-reload templates
- `assets/css/` → Rebuild CSS bundles
- `assets/js/` → Rebuild JS bundles
- `.reed/` → Reload configuration

**Intelligence**:
- Core CSS change → Rebuild all CSS
- Layout CSS change → Rebuild specific layout only
- Debouncing: 300ms to batch rapid changes

**See**: [File Watcher](file-watcher.md)

## Build Modes

### Development Mode

```bash
# Start file watcher with hot-reload
reed build:watch

# Output:
👀 Watching for file changes...
  CSS: assets/css/
  JS: assets/js/
  Templates: templates/
  Config: .reed/

[12:34:56] Changed: assets/css/layouts/blog/blog.mouse.css
🔨 Rebuilding blog.mouse.css...
✓ Rebuilt in 1.2s
```

**Features**:
- Incremental rebuilds (< 2s)
- Template hot-reload (no server restart)
- Config reload on CSV changes
- Real-time feedback

### Production Mode

```bash
# Full asset build
reed build:assets

# Output:
🏗️  Building ReedCMS Assets...

[1/6] Cleaning previous build...
✓ Cleaned public/ directory

[2/6] Building CSS bundles...
✓ 30 bundles created in 1.2s

[3/6] Building JS bundles...
✓ 10 bundles created in 2.4s

[4/6] Pre-compressing assets...
✓ 40 files compressed (gzip: -68%, brotli: -73%)

[5/6] Generating cache-busted filenames...
✓ 40 files renamed with content hashes

[6/6] Verifying build...
✓ All files present and valid

📊 Build Summary:
  Total files: 40
  Total size: 2.4 MB (original: 7.8 MB)
  Size reduction: 69%
  Build time: 8.7s
```

### Release Build

```bash
# Build release binary
reed build:release

# Output:
🔨 Building ReedCMS v0.1.0...
  Compiling with --release
  LTO: enabled
  Codegen units: 1
  Strip: enabled
✓ Compilation complete (3m 24s)
📦 Binary: target/release/reedcms (14.2 MB)
🗜️  Compressing with UPX...
✓ Compressed: target/release/reedcms (5.8 MB, -59%)
🔐 SHA256: a7f3b2c8...
✓ Build complete
```

### Release Package

```bash
# Package for deployment
reed build:package

# Output:
📦 Packaging ReedCMS v0.1.0...
  Adding binary: reedcms (5.8 MB)
  Adding configs: .reed/ (12 files)
  Adding templates: templates/ (24 files)
  Adding docs: README.md, LICENSE, CHANGELOG.md
✓ Package created: reedcms-v0.1.0-linux-x86_64.tar.gz (6.2 MB)
🔐 SHA256: b4k7p2m9...
```

## Cache Busting Strategy

### Content-Based Hashing

```
Original filename:
├─ knowledge.mouse.css
└─ knowledge.mouse.js

After cache busting:
├─ knowledge.mouse.a7f3b2c8.css  (SHA256 hash)
└─ knowledge.mouse.b4k7p2m9.js   (SHA256 hash)
```

### Asset Manifest

```json
{
  "knowledge.mouse.css": "knowledge.mouse.a7f3b2c8.css",
  "knowledge.mouse.js": "knowledge.mouse.b4k7p2m9.js",
  "blog.touch.css": "blog.touch.c9d5e1f7.css"
}
```

### Template Integration

```html
<!-- Dynamic lookup via manifest -->
<link rel="stylesheet" 
      href="/css/{{ asset('knowledge.mouse.css') }}">
<!-- Renders: /css/knowledge.mouse.a7f3b2c8.css -->
```

### Cache Headers

```http
Cache-Control: public, max-age=31536000, immutable
```

**Benefit**: 1-year browser cache without stale content risk

## Performance Characteristics

| Operation | Timing | Note |
|-----------|--------|------|
| **Full asset build** | < 10s | 10 layouts × 3 variants |
| **Incremental CSS rebuild** | < 2s | Single layout change |
| **Incremental JS rebuild** | < 2s | Single layout change |
| **Template hot-reload** | < 100ms | No server restart |
| **Config reload** | < 100ms | ReedBase cache refresh |
| **Binary compile** | 2-5 min | Release optimisations |
| **Release packaging** | < 30s | Archive creation |

## Parallel Processing

### Sequential Build (Old)

```
CSS Bundle 1  ████████████████ 4.0s
CSS Bundle 2  ████████████████ 4.0s
CSS Bundle 3  ████████████████ 4.0s
Total: 12s
```

### Parallel Build (Current)

```
CSS Bundle 1  ████████████████ 4.0s
CSS Bundle 2  ████████████████ 4.0s
CSS Bundle 3  ████████████████ 4.0s
Total: 4.0s (3× faster)
```

**Implementation**: Tokio async tasks with `tokio::spawn()`

## Build Verification

### Checks Performed

```rust
✓ public/ directory exists
✓ All CSS bundles present
✓ All JS bundles present
✓ All compressed files present (.gz, .br)
✓ Asset manifest valid JSON
✓ All manifest files exist
✓ No empty files
✓ File sizes reasonable
```

### Error Handling

```
Error: Manifest file not found: knowledge.mouse.a7f3b2c8.css
→ Indicates failed cache busting or file deletion
→ Solution: Re-run reed build:assets
```

## CLI Commands

### Development

```bash
# Start file watcher
reed build:watch

# Build assets once
reed build:assets

# Build specific layout
reed build:css --layout=knowledge
reed build:js --layout=blog
```

### Production

```bash
# Full build workflow
reed build:release    # Compile binary
reed build:package    # Package for deployment

# Or combined
reed build:all        # Assets + binary + package
```

### Cleanup

```bash
# Clean build artefacts
reed build:clean

# Removes:
# - target/
# - public/
# - *.tar.gz
# - *.zip
```

## Integration with Other Layers

### Layer 08 (Asset Layer)

Build Layer **orchestrates** Asset Layer components:
- Calls `css::bundler::bundle_all_css()`
- Calls `js::bundler::bundle_all_js()`
- Calls `server::precompress::precompress_all_assets()`

### Layer 05 (Template Layer)

File watcher triggers template reload:
- Template change detected → Clear template cache
- MiniJinja reloads template automatically
- No server restart required

### Layer 02 (Data Layer)

File watcher triggers config reload:
- CSV change detected → Reload ReedBase cache
- O(1) HashMap refresh
- Changes visible immediately

## Troubleshooting

### Build Fails with "Binary not found"

**Cause**: Cargo compilation failed

**Solution**:
```bash
# Check cargo output
cargo build --release

# Common issues:
# - Missing dependencies
# - Compilation errors
# - Out of memory
```

### File Watcher Not Detecting Changes

**Cause**: File system events not propagating

**Solution**:
```bash
# Check file system limits (Linux)
cat /proc/sys/fs/inotify/max_user_watches

# Increase if needed
echo 524288 | sudo tee /proc/sys/fs/inotify/max_user_watches
```

### Cache Busting Not Working

**Cause**: Manifest not generated or outdated

**Solution**:
```bash
# Regenerate manifest
reed build:assets

# Check manifest exists
cat public/asset-manifest.json
```

### Build Too Slow

**Cause**: Sequential processing or too many files

**Solution**:
```bash
# Check parallel processing is enabled
# Should see concurrent bundle creation

# Reduce layouts if testing
# Or use incremental builds
reed build:watch  # Only rebuilds changes
```

## Related Documentation

- [Build Pipeline](build-pipeline.md) - Asset orchestration details
- [Binary Compiler](binary-compiler.md) - Release compilation
- [File Watcher](file-watcher.md) - Development workflow
- [CSS Bundler](../08-asset-layer/css-bundler.md) - CSS bundling implementation
- [JS Bundler](../08-asset-layer/js-bundler.md) - JavaScript bundling implementation

## Implementation Files

```
src/reedcms/build/
├── pipeline.rs          # Asset pipeline orchestration
├── compiler.rs          # Binary compiler
├── packager.rs          # Release packager
├── watcher.rs           # File watcher
├── change_detect.rs     # Change detection
├── cache_bust.rs        # Cache busting
└── mod.rs               # Module exports
```

## Performance Optimisation Tips

### 1. Use Incremental Builds

```bash
# During development
reed build:watch  # Only rebuilds changes

# Not
reed build:assets  # Full rebuild every time
```

### 2. Optimise Asset Count

- Fewer layouts = Faster builds
- Shared components reduce duplication
- Consider lazy-loading for large apps

### 3. Parallel Processing

- Already enabled by default
- Scales with CPU cores
- 4-core CPU: 4× speedup

### 4. Pre-compression

- Pre-compress in build (not runtime)
- Faster serving, slower builds
- Trade-off: Build time vs. Runtime latency

## CLI Reference

```bash
# Development
reed build:watch          # Start file watcher
reed build:assets         # Build assets once
reed build:css            # Build CSS only
reed build:js             # Build JS only

# Production
reed build:release        # Compile binary
reed build:package        # Package for deployment
reed build:all            # Full build + package

# Utilities
reed build:clean          # Clean artefacts
reed build:verify         # Verify build output
reed build:stats          # Show build statistics
```
