# REED-09-01: Binary Compiler and Release System

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
- **ID**: REED-09-01
- **Title**: Binary Compiler and Release System
- **Layer**: Build Layer (REED-09)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: All previous layers

## Summary Reference
- **Section**: Build System
- **Lines**: 1035-1037 in project_summary.md
- **Key Concepts**: Cargo build, release optimisation, binary packaging, version management

## Objective
Implement binary compilation system that builds optimised release binaries for production deployment, manages versioning, strips debug symbols, performs link-time optimisation (LTO), and packages binaries with necessary assets.

## Requirements

### Build Configuration (Cargo.toml)

```toml
[package]
name = "reed"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.4"
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
csv = "1.3"
# ... other dependencies

[profile.release]
opt-level = 3              # Maximum optimisation
lto = "fat"                # Link-time optimisation
codegen-units = 1          # Better optimisation, slower compile
strip = true               # Strip debug symbols
panic = "abort"            # Smaller binary size

[profile.dev]
opt-level = 0
debug = true

[profile.dev.package."*"]
opt-level = 2              # Optimise dependencies in dev
```

### Implementation (`build/compiler.rs`)

```rust
/// Compiles ReedCMS binary for release.
///
/// ## Build Process
/// 1. Clean previous builds
/// 2. Set release flags
/// 3. Compile with cargo
/// 4. Strip debug symbols (if not done by cargo)
/// 5. Compress binary with UPX (optional)
/// 6. Calculate checksums
/// 7. Generate build info
///
/// ## Optimisations
/// - LTO (Link-Time Optimisation): Reduces binary size ~20%
/// - Codegen units = 1: Better optimisation
/// - Strip symbols: Reduces size ~40%
/// - UPX compression: Reduces size ~60% (optional)
///
/// ## Performance
/// - Compile time: 2-5 minutes (release build)
/// - Binary size: ~15MB (stripped)
/// - Binary size: ~6MB (UPX compressed)
///
/// ## Output
/// ```
/// 🔨 Building ReedCMS v0.1.0...
///   Compiling with --release
///   LTO: enabled
///   Codegen units: 1
///   Strip: enabled
/// ✓ Compilation complete (3m 24s)
/// 📦 Binary: target/release/reed (14.2 MB)
/// 🗜️  Compressing with UPX...
/// ✓ Compressed: target/release/reed (5.8 MB, -59%)
/// ✓ Build complete
/// ```
pub fn build_release() -> ReedResult<BuildInfo> {
    println!("🔨 Building ReedCMS v{}...", env!("CARGO_PKG_VERSION"));

    let start_time = std::time::Instant::now();

    // 1. Clean previous builds
    clean_previous_builds()?;

    // 2. Run cargo build --release
    let output = run_cargo_build()?;

    // 3. Get binary path
    let binary_path = "target/release/reed";

    if !std::path::Path::new(binary_path).exists() {
        return Err(ReedError::BuildError {
            component: "compiler".to_string(),
            reason: "Binary not found after compilation".to_string(),
        });
    }

    // 4. Get binary size
    let metadata = std::fs::metadata(binary_path)?;
    let binary_size = metadata.len() as usize;

    println!("✓ Compilation complete ({:?})", start_time.elapsed());
    println!("📦 Binary: {} ({:.1} MB)", binary_path, binary_size as f64 / 1_048_576.0);

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

/// Cleans previous build artefacts.
fn clean_previous_builds() -> ReedResult<()> {
    println!("🧹 Cleaning previous builds...");

    let output = std::process::Command::new("cargo")
        .arg("clean")
        .output()
        .map_err(|e| ReedError::BuildError {
            component: "cargo_clean".to_string(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(ReedError::BuildError {
            component: "cargo_clean".to_string(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    Ok(())
}

/// Runs cargo build with release profile.
fn run_cargo_build() -> ReedResult<std::process::Output> {
    println!("  Compiling with --release");
    println!("  LTO: enabled");
    println!("  Codegen units: 1");
    println!("  Strip: enabled");

    let output = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .output()
        .map_err(|e| ReedError::BuildError {
            component: "cargo_build".to_string(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(ReedError::BuildError {
            component: "cargo_build".to_string(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    Ok(output)
}

/// Checks if UPX compression should be used.
fn should_use_upx() -> bool {
    // Check if UPX is available
    std::process::Command::new("upx")
        .arg("--version")
        .output()
        .is_ok()
}

/// Compresses binary with UPX.
fn compress_with_upx(binary_path: &str) -> ReedResult<usize> {
    println!("🗜️  Compressing with UPX...");

    let output = std::process::Command::new("upx")
        .arg("--best")
        .arg("--lzma")
        .arg(binary_path)
        .output()
        .map_err(|e| ReedError::BuildError {
            component: "upx".to_string(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(ReedError::BuildError {
            component: "upx".to_string(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let metadata = std::fs::metadata(binary_path)?;
    Ok(metadata.len() as usize)
}

/// Calculates SHA256 checksum of binary.
fn calculate_sha256(path: &str) -> ReedResult<String> {
    use sha2::{Digest, Sha256};

    let content = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Calculates MD5 checksum of binary.
fn calculate_md5(path: &str) -> ReedResult<String> {
    use md5::{Digest, Md5};

    let content = std::fs::read(path)?;
    let mut hasher = Md5::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Build information structure.
#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    pub version: String,
    pub binary_path: String,
    pub original_size: usize,
    pub compressed_size: Option<usize>,
    pub sha256: String,
    pub md5: String,
    pub build_time: String,
    pub build_duration: std::time::Duration,
}

/// Writes build info to JSON file.
fn write_build_info(info: &BuildInfo) -> ReedResult<()> {
    let json = serde_json::to_string_pretty(info).map_err(|e| ReedError::SerializationError {
        data_type: "BuildInfo".to_string(),
        reason: e.to_string(),
    })?;

    std::fs::write("target/release/build-info.json", json).map_err(|e| ReedError::IoError {
        operation: "write".to_string(),
        path: "target/release/build-info.json".to_string(),
        reason: e.to_string(),
    })
}
```

### Release Packaging (`build/packager.rs`)

```rust
/// Packages release binary with assets.
///
/// ## Package Contents
/// - reed binary
/// - .reed/ directory (config templates)
/// - templates/ directory (layout templates)
/// - README.md
/// - LICENSE
/// - CHANGELOG.md
///
/// ## Package Format
/// - tar.gz for Linux/macOS
/// - zip for Windows
///
/// ## Output
/// ```
/// 📦 Packaging ReedCMS v0.1.0...
///   Adding binary: reed (5.8 MB)
///   Adding configs: .reed/ (12 files)
///   Adding templates: templates/ (24 files)
///   Adding docs: README.md, LICENSE, CHANGELOG.md
/// ✓ Package created: reed-v0.1.0-linux-x86_64.tar.gz (6.2 MB)
/// ```
pub fn package_release(build_info: &BuildInfo) -> ReedResult<PackageInfo> {
    println!("📦 Packaging ReedCMS v{}...", build_info.version);

    let package_name = format!(
        "reed-v{}-{}-{}",
        build_info.version,
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let package_dir = format!("target/release/{}", package_name);

    // 1. Create package directory
    std::fs::create_dir_all(&package_dir)?;

    // 2. Copy binary
    println!("  Adding binary: reed ({:.1} MB)",
        build_info.compressed_size.unwrap_or(build_info.original_size) as f64 / 1_048_576.0
    );
    std::fs::copy(&build_info.binary_path, format!("{}/reed", package_dir))?;

    // 3. Copy config templates
    println!("  Adding configs: .reed/");
    copy_dir_recursive(".reed/", &format!("{}/.reed/", package_dir))?;

    // 4. Copy templates
    println!("  Adding templates: templates/");
    copy_dir_recursive("templates/", &format!("{}/templates/", package_dir))?;

    // 5. Copy documentation
    println!("  Adding docs: README.md, LICENSE, CHANGELOG.md");
    std::fs::copy("README.md", format!("{}/README.md", package_dir))?;
    std::fs::copy("LICENSE", format!("{}/LICENSE", package_dir))?;
    std::fs::copy("CHANGELOG.md", format!("{}/CHANGELOG.md", package_dir))?;

    // 6. Create archive
    let archive_path = if cfg!(target_os = "windows") {
        create_zip_archive(&package_name, &package_dir)?
    } else {
        create_tar_gz_archive(&package_name, &package_dir)?
    };

    let archive_size = std::fs::metadata(&archive_path)?.len() as usize;

    println!("✓ Package created: {} ({:.1} MB)",
        archive_path,
        archive_size as f64 / 1_048_576.0
    );

    Ok(PackageInfo {
        package_name,
        archive_path,
        archive_size,
        sha256: calculate_sha256(&archive_path)?,
    })
}

/// Recursively copies directory.
fn copy_dir_recursive(src: &str, dst: &str) -> ReedResult<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let dst_path = format!("{}/{}", dst.trim_end_matches('/'), file_name.to_string_lossy());

        if path.is_dir() {
            copy_dir_recursive(path.to_str().unwrap(), &dst_path)?;
        } else {
            std::fs::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}

/// Creates tar.gz archive.
fn create_tar_gz_archive(package_name: &str, package_dir: &str) -> ReedResult<String> {
    let archive_path = format!("target/release/{}.tar.gz", package_name);

    let tar_gz = std::fs::File::create(&archive_path)?;
    let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    tar.append_dir_all(package_name, package_dir)
        .map_err(|e| ReedError::IoError {
            operation: "tar".to_string(),
            path: archive_path.clone(),
            reason: e.to_string(),
        })?;

    tar.finish().map_err(|e| ReedError::IoError {
        operation: "tar_finish".to_string(),
        path: archive_path.clone(),
        reason: e.to_string(),
    })?;

    Ok(archive_path)
}

/// Creates zip archive.
fn create_zip_archive(package_name: &str, package_dir: &str) -> ReedResult<String> {
    let archive_path = format!("target/release/{}.zip", package_name);

    let file = std::fs::File::create(&archive_path)?;
    let mut zip = zip::ZipWriter::new(file);

    // Add files to zip
    for entry in walkdir::WalkDir::new(package_dir) {
        let entry = entry.map_err(|e| ReedError::IoError {
            operation: "walkdir".to_string(),
            path: package_dir.to_string(),
            reason: e.to_string(),
        })?;

        let path = entry.path();
        let name = path.strip_prefix(package_dir).unwrap();

        if path.is_file() {
            zip.start_file(name.to_string_lossy().to_string(), Default::default())
                .map_err(|e| ReedError::IoError {
                    operation: "zip_start_file".to_string(),
                    path: path.display().to_string(),
                    reason: e.to_string(),
                })?;

            let content = std::fs::read(path)?;
            zip.write_all(&content).map_err(|e| ReedError::IoError {
                operation: "zip_write".to_string(),
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;
        }
    }

    zip.finish().map_err(|e| ReedError::IoError {
        operation: "zip_finish".to_string(),
        path: archive_path.clone(),
        reason: e.to_string(),
    })?;

    Ok(archive_path)
}

/// Package information structure.
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub package_name: String,
    pub archive_path: String,
    pub archive_size: usize,
    pub sha256: String,
}
```

## Implementation Files

### Primary Implementation
- `build/compiler.rs` - Binary compiler
- `build/packager.rs` - Release packager
- `build/version.rs` - Version management
- `Cargo.toml` - Build configuration

### Test Files
- `build/compiler.test.rs`
- `build/packager.test.rs`
- `build/version.test.rs`

## Testing Requirements

### Unit Tests
- [ ] Test build info generation
- [ ] Test checksum calculation
- [ ] Test package directory creation
- [ ] Test archive creation
- [ ] Test version parsing

### Integration Tests
- [ ] Test complete build workflow
- [ ] Test release packaging
- [ ] Test binary execution post-build
- [ ] Test package extraction

### Performance Tests
- [ ] Release build: < 5 minutes
- [ ] Package creation: < 30 seconds
- [ ] Binary size: < 15MB (stripped)
- [ ] Archive size: < 10MB

## Acceptance Criteria
- [ ] Release binary compilation working
- [ ] LTO and optimisations applied
- [ ] Debug symbols stripped
- [ ] UPX compression optional
- [ ] Checksums calculated (SHA256, MD5)
- [ ] Build info JSON generated
- [ ] Release packaging functional
- [ ] tar.gz creation for Linux/macOS
- [ ] zip creation for Windows
- [ ] All necessary files included in package
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: All previous layers (complete system)

## Blocks
- REED-09-03 (File Watcher uses build system)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1035-1037 in `project_summary.md`

## Notes
Binary compilation is the final step before deployment. LTO and optimisations reduce binary size significantly. Stripped binaries remove debug symbols for production. UPX compression is optional but can reduce size by 60%. Checksums enable integrity verification. Release packages include all necessary files for standalone deployment. tar.gz and zip formats ensure compatibility across platforms.
