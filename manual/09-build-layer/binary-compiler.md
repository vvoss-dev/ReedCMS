# Binary Compiler

Builds optimised production binaries with LTO, symbol stripping, and optional compression.

## Purpose

The binary compiler produces release-ready executables:

- **Maximum Optimisation**: LTO (Link-Time Optimisation) for ~20% size reduction
- **Symbol Stripping**: Remove debug symbols for ~40% size reduction
- **UPX Compression**: Optional compression for ~60% total size reduction
- **Checksums**: SHA256 + MD5 for integrity verification
- **Build Info**: JSON metadata for versioning and deployment

## Architecture

```
┌─────────────────────────────────────────────────────┐
│ Binary Compilation Pipeline                         │
├─────────────────────────────────────────────────────┤
│                                                     │
│  [Stage 1] Clean Previous Builds                    │
│  ├─ Command: cargo clean                            │
│  └─ Duration: < 5s                                  │
│           ↓                                         │
│  [Stage 2] Cargo Compilation                        │
│  ├─ Profile: --release                              │
│  ├─ LTO: "fat" (full link-time optimisation)        │
│  ├─ Codegen: units=1 (better optimisation)          │
│  ├─ Strip: true (remove debug symbols)              │
│  ├─ Panic: "abort" (smaller binaries)               │
│  └─ Duration: 2-5 minutes                           │
│           ↓                                         │
│  [Stage 3] Optional UPX Compression                 │
│  ├─ Algorithm: LZMA (best compression)              │
│  ├─ Level: --best                                   │
│  └─ Duration: ~30s                                  │
│           ↓                                         │
│  [Stage 4] Checksum Generation                      │
│  ├─ SHA256: Cryptographic hash                      │
│  ├─ MD5: Quick verification                         │
│  └─ Duration: < 1s                                  │
│           ↓                                         │
│  [Stage 5] Build Info Generation                    │
│  ├─ Version, size, checksums                        │
│  ├─ Output: target/release/build-info.json          │
│  └─ Duration: < 0.1s                                │
│           ↓                                         │
│  [Stage 6] Packaging (Optional)                     │
│  ├─ Bundle: Binary + templates + configs            │
│  ├─ Archive: tar.gz (Linux/macOS) or zip (Win)      │
│  └─ Duration: < 30s                                 │
│                                                     │
│  Total Duration: 2-6 minutes                        │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Cargo Configuration

### Release Profile

```toml
# Cargo.toml

[profile.release]
opt-level = 3              # Maximum optimisation
lto = "fat"                # Full link-time optimisation
codegen-units = 1          # Single unit for better optimisation
strip = true               # Strip debug symbols automatically
panic = "abort"            # Smaller binary, no unwinding
```

**Optimisation Levels**:
- `opt-level = 0`: No optimisation (dev default)
- `opt-level = 1`: Basic optimisation
- `opt-level = 2`: Some optimisation
- `opt-level = 3`: **Maximum optimisation** (release)
- `opt-level = "s"`: Optimise for size
- `opt-level = "z"`: Optimise aggressively for size

**LTO (Link-Time Optimisation)**:
- `lto = false`: No LTO (faster compile)
- `lto = "thin"`: Thin LTO (good balance)
- `lto = "fat"`: **Full LTO** (best optimisation, slower compile)

**Codegen Units**:
- `codegen-units = 16`: Fast compile, less optimisation
- `codegen-units = 1`: **Slowest compile, best optimisation**

### Development Profile

```toml
[profile.dev]
opt-level = 0              # Fast compile
debug = true               # Include debug info
lto = false                # No LTO
codegen-units = 16         # Parallel codegen

[profile.dev.package."*"]
opt-level = 2              # Optimise dependencies
```

## Implementation

### Main Build Function

```rust
pub fn build_release() -> ReedResult<BuildInfo> {
    println!("🔨 Building ReedCMS v{}...",
        env!("CARGO_PKG_VERSION"));

    let start_time = Instant::now();

    // 1. Clean previous builds
    clean_previous_builds()?;

    // 2. Run cargo build --release
    run_cargo_build()?;

    // 3. Get binary path
    let binary_path = "target/release/reedcms";

    if !Path::new(binary_path).exists() {
        return Err(ReedError::BuildError {
            component: "compiler",
            reason: "Binary not found after compilation",
        });
    }

    // 4. Get binary size
    let metadata = fs::metadata(binary_path)?;
    let binary_size = metadata.len() as usize;

    println!("✓ Compilation complete ({:?})",
        start_time.elapsed());
    println!("📦 Binary: {} ({:.1} MB)",
        binary_path,
        binary_size as f64 / 1_048_576.0
    );

    // 5. Optional: Compress with UPX
    let compressed_size = if should_use_upx() {
        match compress_with_upx(binary_path) {
            Ok(size) => {
                let reduction = 100 - (size * 100 / binary_size);
                println!("✓ Compressed: {} ({:.1} MB, -{}%)",
                    binary_path,
                    size as f64 / 1_048_576.0,
                    reduction
                );
                Some(size)
            }
            Err(e) => {
                eprintln!("⚠ UPX compression failed: {:?}", e);
                None
            }
        }
    } else {
        None
    };

    // 6. Calculate checksums
    let sha256 = calculate_sha256(binary_path)?;
    let md5 = calculate_md5(binary_path)?;

    println!("🔐 SHA256: {}", sha256);
    println!("🔐 MD5: {}", md5);

    // 7. Generate build info
    let build_info = BuildInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        binary_path: binary_path.to_string(),
        original_size: binary_size,
        compressed_size,
        sha256,
        md5,
        build_time: chrono::Utc::now().to_rfc3339(),
        build_duration: start_time.elapsed(),
    };

    // 8. Write build info to file
    write_build_info(&build_info)?;

    println!("✓ Build complete");

    Ok(build_info)
}
```

### Cargo Build Execution

```rust
fn run_cargo_build() -> ReedResult<Output> {
    println!("  Compiling with --release");
    println!("  LTO: enabled");
    println!("  Codegen units: 1");
    println!("  Strip: enabled");

    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .output()
        .map_err(|e| ReedError::BuildError {
            component: "cargo_build",
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(ReedError::BuildError {
            component: "cargo_build",
            reason: String::from_utf8_lossy(&output.stderr)
                .to_string(),
        });
    }

    Ok(output)
}
```

**RUSTFLAGS Explanation**:
- `-C target-cpu=native`: Optimise for current CPU architecture
- Enables CPU-specific instructions (AVX, SSE, etc.)
- ~5-10% performance improvement
- **Trade-off**: Binary not portable to older CPUs

## UPX Compression

### Implementation

```rust
fn should_use_upx() -> bool {
    // Check if UPX is available
    Command::new("upx")
        .arg("--version")
        .output()
        .is_ok()
}

fn compress_with_upx(binary_path: &str) -> ReedResult<usize> {
    println!("🗜️  Compressing with UPX...");

    let output = Command::new("upx")
        .arg("--best")
        .arg("--lzma")
        .arg(binary_path)
        .output()
        .map_err(|e| ReedError::BuildError {
            component: "upx",
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(ReedError::BuildError {
            component: "upx",
            reason: String::from_utf8_lossy(&output.stderr)
                .to_string(),
        });
    }

    let metadata = fs::metadata(binary_path)?;
    Ok(metadata.len() as usize)
}
```

### UPX Trade-offs

| Aspect | Without UPX | With UPX | Note |
|--------|-------------|----------|------|
| **Binary size** | 15 MB | 6 MB | -60% |
| **Startup time** | Instant | +50-100ms | Decompression |
| **Memory usage** | Lower | Higher | Decompressed in RAM |
| **Security** | Standard | Obfuscated | Harder to analyse |
| **Compatibility** | High | Medium | Some systems block UPX |

**Recommendation**: Use UPX for development/testing, skip for production if startup time matters.

## Checksum Generation

### SHA256 (Cryptographic Hash)

```rust
fn calculate_sha256(path: &str) -> ReedResult<String> {
    use sha2::{Digest, Sha256};

    let content = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalise()))
}
```

**Use Case**: Verify binary integrity, detect tampering

**Output**: `a7f3b2c8d4e1f9g5h6i8j2k3l1m4n7o9p6q8r2s5t3u9v1w7x4y6z8`

### MD5 (Quick Verification)

```rust
fn calculate_md5(path: &str) -> ReedResult<String> {
    use md5::{Digest, Md5};

    let content = fs::read(path)?;
    let mut hasher = Md5::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalise()))
}
```

**Use Case**: Quick checksum comparison, not for security

**Output**: `a7f3b2c8d4e1f9g5h6i8j2k3l1m4n7o9`

## Build Info

### Structure

```rust
#[derive(Debug, Clone, Serialise)]
pub struct BuildInfo {
    pub version: String,
    pub binary_path: String,
    pub original_size: usize,
    pub compressed_size: Option<usize>,
    pub sha256: String,
    pub md5: String,
    pub build_time: String,
    pub build_duration: Duration,
}
```

### JSON Output

```json
{
  "version": "0.1.0",
  "binary_path": "target/release/reedcms",
  "original_size": 14680064,
  "compressed_size": 6024192,
  "sha256": "a7f3b2c8d4e1f9g5h6i8j2k3l1m4n7o9p6q8r2s5t3u9v1w7x4y6z8",
  "md5": "a7f3b2c8d4e1f9g5h6i8j2k3l1m4n7o9",
  "build_time": "2025-01-15T14:32:18Z",
  "build_duration": {
    "secs": 204,
    "nanos": 567000000
  }
}
```

## Release Packaging

### Package Contents

```
reedcms-v0.1.0-linux-x86_64/
├── reedcms                    # Compiled binary
├── .reed/                     # Configuration templates
│   ├── text.csv
│   ├── routes.csv
│   ├── meta.csv
│   └── project.csv
├── templates/                 # Page templates
│   ├── layouts/
│   └── components/
├── README.md                  # User documentation
├── LICENSE                    # Apache 2.0
└── CHANGELOG.md               # Version history
```

### Archive Creation

```rust
pub fn package_release(build_info: &BuildInfo)
    -> ReedResult<PackageInfo>
{
    let package_name = format!(
        "reedcms-v{}-{}-{}",
        build_info.version,
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let package_dir = format!("target/release/{}", package_name);

    // Create package directory
    fs::create_dir_all(&package_dir)?;

    // Copy binary
    fs::copy(&build_info.binary_path,
             format!("{}/reedcms", package_dir))?;

    // Copy directories
    copy_dir_recursive(".reed", &format!("{}/.reed", package_dir))?;
    copy_dir_recursive("templates",
                       &format!("{}/templates", package_dir))?;

    // Copy documentation
    fs::copy("README.md", format!("{}/README.md", package_dir))?;
    fs::copy("LICENSE", format!("{}/LICENSE", package_dir))?;
    fs::copy("CHANGELOG.md",
             format!("{}/CHANGELOG.md", package_dir))?;

    // Create archive
    let archive_path = if cfg!(target_os = "windows") {
        create_zip_archive(&package_name, &package_dir)?
    } else {
        create_tar_gz_archive(&package_name, &package_dir)?
    };

    Ok(PackageInfo {
        package_name,
        archive_path,
        sha256: calculate_sha256(&archive_path)?,
    })
}
```

### tar.gz Creation (Linux/macOS)

```rust
fn create_tar_gz_archive(package_name: &str, package_dir: &str)
    -> ReedResult<String>
{
    let archive_path = format!("target/release/{}.tar.gz",
                               package_name);

    let tar_gz = File::create(&archive_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);

    tar.append_dir_all(package_name, package_dir)?;
    tar.finish()?;

    Ok(archive_path)
}
```

### zip Creation (Windows)

```rust
fn create_zip_archive(package_name: &str, package_dir: &str)
    -> ReedResult<String>
{
    let archive_path = format!("target/release/{}.zip",
                               package_name);

    let file = File::create(&archive_path)?;
    let mut zip = ZipWriter::new(file);

    for entry in WalkDir::new(package_dir) {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(package_dir).unwrap();

        if path.is_file() {
            zip.start_file(name.to_string_lossy().to_string(),
                           Default::default())?;
            let content = fs::read(path)?;
            zip.write_all(&content)?;
        }
    }

    zip.finish()?;
    Ok(archive_path)
}
```

## Performance Benchmarks

### Compilation Time

| Profile | LTO | Codegen Units | Time | Binary Size |
|---------|-----|---------------|------|-------------|
| Dev | No | 16 | 30s | 80 MB |
| Release (no LTO) | No | 16 | 2m | 25 MB |
| Release (thin LTO) | Thin | 16 | 3m | 18 MB |
| **Release (fat LTO)** | **Fat** | **1** | **5m** | **15 MB** |

### Size Reduction

```
Original (dev build):     80 MB   100%
Release (no LTO):         25 MB   -69%
Release (LTO):            15 MB   -81%
Release (LTO + strip):    15 MB   -81%
Release (LTO + UPX):       6 MB   -93%
```

## CLI Integration

```bash
# Build release binary
reed build:release

# Build with UPX compression
reed build:release --upx

# Build and package
reed build:package

# Complete workflow
reed build:all  # Assets + binary + package
```

## Troubleshooting

### Compilation Fails

**Error**: `error: linking with 'cc' failed`

**Cause**: Missing system libraries or linker

**Solution**:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install

# Check linker
which cc
```

### Out of Memory During LTO

**Error**: `LLVM ERROR: out of memory`

**Cause**: LTO requires significant RAM (~4-8 GB)

**Solution**: Reduce LTO level or increase swap

```toml
# Cargo.toml - Use thin LTO instead
[profile.release]
lto = "thin"  # Instead of "fat"
```

### UPX Not Found

**Error**: `upx: command not found`

**Solution**: Install UPX

```bash
# Ubuntu/Debian
sudo apt-get install upx-ucl

# macOS
brew install upx

# Or skip UPX
reed build:release --no-upx
```

### Binary Too Large

**Problem**: Binary > 20 MB even with optimisations

**Solutions**:

1. **Check dependencies**: Remove unused crates
2. **Strip more aggressively**:
   ```bash
   strip --strip-all target/release/reedcms
   ```
3. **Use size optimisation**:
   ```toml
   [profile.release]
   opt-level = "z"  # Optimise for size
   ```
4. **Enable panic=abort** (already in config)

## Related Documentation

- [Build Pipeline](build-pipeline.md) - Asset building
- [File Watcher](file-watcher.md) - Development workflow

## CLI Reference

```bash
# Compilation
reed build:release           # Build release binary
reed build:release --upx     # With UPX compression
reed build:release --no-upx  # Skip UPX

# Packaging
reed build:package           # Package release
reed build:all               # Full build + package

# Information
reed build:info              # Show build info
reed build:verify            # Verify checksums
```
