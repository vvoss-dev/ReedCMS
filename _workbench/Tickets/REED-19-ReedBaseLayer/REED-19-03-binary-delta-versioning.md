# REED-19-03: Binary Delta Versioning

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-19-03
- **Title**: Binary Delta Versioning
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-19-02 (Universal Table API)
- **Estimated Time**: 1 week

## Objective

Implement binary delta compression using bsdiff for table versioning. Store only deltas instead of full snapshots for 95%+ disk savings.

## Requirements

### File Structure

```
.reed/tables/{table_name}/
├── current.csv          # Active version
├── 1736860800.bsdiff    # Delta from previous to this version
├── 1736860900.bsdiff    # Delta from previous to this version
└── version.log          # Encoded version history
```

### Delta Generation

```
Previous Version (1736860800.csv) + Delta (1736860900.bsdiff) = New Version (1736860900.csv)
```

### Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Generate delta (100 rows) | < 50ms | bsdiff + XZ compression |
| Apply delta (100 rows) | < 30ms | bspatch + XZ decompression |
| Delta size | < 5% of full | For typical CSV changes |
| Delta size (1 row change) | < 500 bytes | Single row modification |

## Implementation Files

### Primary Implementation

**`reedbase/src/version/delta.rs`**

One file = Binary delta operations only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Binary delta compression for ReedBase versioning.
//!
//! Uses bsdiff for delta generation and bspatch for applying deltas.

use std::path::Path;
use std::fs;
use crate::types::{ReedResult, ReedError};

/// Generate binary delta from old version to new version.
///
/// ## Input
/// - `old_path`: Path to previous version CSV
/// - `new_path`: Path to new version CSV
/// - `delta_path`: Path to output delta file
///
/// ## Output
/// - `ReedResult<DeltaInfo>`: Delta metadata (size, compression ratio)
///
/// ## Performance
/// - O(n) where n = file size
/// - < 50ms for 100-row CSV (~10KB)
/// - < 500ms for 1000-row CSV (~100KB)
///
/// ## Error Conditions
/// - IoError: Cannot read old/new file or write delta
/// - DeltaGenerationFailed: bsdiff operation failed
///
/// ## Example Usage
/// ```rust
/// let info = generate_delta(
///     "1736860800.csv",
///     "current.csv",
///     "1736860900.bsdiff"
/// )?;
/// println!("Delta size: {} bytes ({}% of original)", info.size, info.ratio);
/// ```
pub fn generate_delta<P: AsRef<Path>>(
    old_path: P,
    new_path: P,
    delta_path: P,
) -> ReedResult<DeltaInfo> {
    let old_data = fs::read(old_path.as_ref())
        .map_err(|e| ReedError::IoError {
            path: old_path.as_ref().to_string_lossy().to_string(),
            source: e,
        })?;
    
    let new_data = fs::read(new_path.as_ref())
        .map_err(|e| ReedError::IoError {
            path: new_path.as_ref().to_string_lossy().to_string(),
            source: e,
        })?;
    
    let delta = create_bsdiff(&old_data, &new_data)?;
    let compressed = compress_delta(&delta)?;
    
    fs::write(delta_path.as_ref(), &compressed)
        .map_err(|e| ReedError::IoError {
            path: delta_path.as_ref().to_string_lossy().to_string(),
            source: e,
        })?;
    
    Ok(DeltaInfo {
        size: compressed.len(),
        original_size: new_data.len(),
        ratio: (compressed.len() as f64 / new_data.len() as f64 * 100.0) as u8,
    })
}

/// Apply binary delta to reconstruct version.
///
/// ## Input
/// - `old_path`: Path to base version CSV
/// - `delta_path`: Path to delta file
/// - `output_path`: Path to output reconstructed CSV
///
/// ## Output
/// - `ReedResult<()>`: Success or error
///
/// ## Performance
/// - O(n) where n = file size
/// - < 30ms for 100-row CSV
/// - < 300ms for 1000-row CSV
///
/// ## Error Conditions
/// - IoError: Cannot read base/delta or write output
/// - DeltaApplicationFailed: bspatch operation failed
/// - CorruptedDelta: Delta file corrupted
///
/// ## Example Usage
/// ```rust
/// apply_delta(
///     "1736860800.csv",
///     "1736860900.bsdiff",
///     "reconstructed.csv"
/// )?;
/// ```
pub fn apply_delta<P: AsRef<Path>>(
    old_path: P,
    delta_path: P,
    output_path: P,
) -> ReedResult<()> {
    let old_data = fs::read(old_path.as_ref())
        .map_err(|e| ReedError::IoError {
            path: old_path.as_ref().to_string_lossy().to_string(),
            source: e,
        })?;
    
    let compressed = fs::read(delta_path.as_ref())
        .map_err(|e| ReedError::IoError {
            path: delta_path.as_ref().to_string_lossy().to_string(),
            source: e,
        })?;
    
    let delta = decompress_delta(&compressed)?;
    let new_data = apply_bspatch(&old_data, &delta)?;
    
    fs::write(output_path.as_ref(), &new_data)
        .map_err(|e| ReedError::IoError {
            path: output_path.as_ref().to_string_lossy().to_string(),
            source: e,
        })?;
    
    Ok(())
}

/// Create bsdiff binary delta.
///
/// ## Input
/// - `old_data`: Previous version data
/// - `new_data`: New version data
///
/// ## Output
/// - `ReedResult<Vec<u8>>`: Binary delta (uncompressed)
///
/// ## Performance
/// - O(n) where n = max(old_data.len(), new_data.len())
/// - < 40ms for 10KB files
///
/// ## Error Conditions
/// - DeltaGenerationFailed: bsdiff library error
fn create_bsdiff(old_data: &[u8], new_data: &[u8]) -> ReedResult<Vec<u8>> {
    let mut delta = Vec::new();
    
    bsdiff::diff(old_data, new_data, &mut delta)
        .map_err(|e| ReedError::DeltaGenerationFailed {
            reason: format!("bsdiff error: {}", e),
        })?;
    
    Ok(delta)
}

/// Apply bspatch to reconstruct data.
///
/// ## Input
/// - `old_data`: Base version data
/// - `delta`: Binary delta (uncompressed)
///
/// ## Output
/// - `ReedResult<Vec<u8>>`: Reconstructed data
///
/// ## Performance
/// - O(n) where n = output size
/// - < 20ms for 10KB output
///
/// ## Error Conditions
/// - DeltaApplicationFailed: bspatch library error
/// - CorruptedDelta: Delta data invalid
fn apply_bspatch(old_data: &[u8], delta: &[u8]) -> ReedResult<Vec<u8>> {
    let mut new_data = Vec::new();
    
    bsdiff::patch(old_data, delta, &mut new_data)
        .map_err(|e| ReedError::DeltaApplicationFailed {
            reason: format!("bspatch error: {}", e),
        })?;
    
    Ok(new_data)
}

/// Compress delta using XZ.
///
/// ## Input
/// - `delta`: Uncompressed delta data
///
/// ## Output
/// - `ReedResult<Vec<u8>>`: Compressed delta
///
/// ## Performance
/// - O(n) where n = delta size
/// - < 10ms for typical deltas (< 1KB)
/// - Compression ratio: ~30-50% of uncompressed
///
/// ## Error Conditions
/// - CompressionFailed: XZ compression error
fn compress_delta(delta: &[u8]) -> ReedResult<Vec<u8>> {
    use xz2::write::XzEncoder;
    use std::io::Write;
    
    let mut encoder = XzEncoder::new(Vec::new(), 6);
    encoder.write_all(delta)
        .map_err(|e| ReedError::CompressionFailed {
            reason: format!("XZ write error: {}", e),
        })?;
    
    encoder.finish()
        .map_err(|e| ReedError::CompressionFailed {
            reason: format!("XZ finish error: {}", e),
        })
}

/// Decompress delta using XZ.
///
/// ## Input
/// - `compressed`: Compressed delta data
///
/// ## Output
/// - `ReedResult<Vec<u8>>`: Uncompressed delta
///
/// ## Performance
/// - O(n) where n = compressed size
/// - < 5ms for typical deltas
///
/// ## Error Conditions
/// - DecompressionFailed: XZ decompression error
/// - CorruptedDelta: Invalid XZ data
fn decompress_delta(compressed: &[u8]) -> ReedResult<Vec<u8>> {
    use xz2::read::XzDecoder;
    use std::io::Read;
    
    let mut decoder = XzDecoder::new(compressed);
    let mut delta = Vec::new();
    
    decoder.read_to_end(&mut delta)
        .map_err(|e| ReedError::DecompressionFailed {
            reason: format!("XZ read error: {}", e),
        })?;
    
    Ok(delta)
}

/// Calculate delta size savings.
///
/// ## Input
/// - `delta_size`: Size of delta file in bytes
/// - `full_size`: Size of full version in bytes
///
/// ## Output
/// - `f64`: Percentage saved (0.0 to 100.0)
///
/// ## Performance
/// - O(1) operation
/// - < 1μs
///
/// ## Example Usage
/// ```rust
/// let saved = calculate_savings(500, 10000);
/// assert_eq!(saved, 95.0); // 95% savings
/// ```
pub fn calculate_savings(delta_size: usize, full_size: usize) -> f64 {
    if full_size == 0 {
        return 0.0;
    }
    ((full_size - delta_size) as f64 / full_size as f64) * 100.0
}
```

**`reedbase/src/types.rs`** (additions)

```rust
/// Delta metadata.
#[derive(Debug, Clone)]
pub struct DeltaInfo {
    pub size: usize,
    pub original_size: usize,
    pub ratio: u8, // Percentage (0-100)
}

/// Additional ReedBase errors.
#[derive(Error, Debug)]
pub enum ReedError {
    // ... (existing errors)
    
    #[error("Delta generation failed: {reason}")]
    DeltaGenerationFailed {
        reason: String,
    },
    
    #[error("Delta application failed: {reason}")]
    DeltaApplicationFailed {
        reason: String,
    },
    
    #[error("Corrupted delta: {reason}")]
    CorruptedDelta {
        reason: String,
    },
    
    #[error("Compression failed: {reason}")]
    CompressionFailed {
        reason: String,
    },
    
    #[error("Decompression failed: {reason}")]
    DecompressionFailed {
        reason: String,
    },
}
```

### Test Files

**`reedbase/src/version/delta.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_generate_and_apply_delta() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");
        let output_path = temp_dir.path().join("output.csv");
        
        // Create test files
        fs::write(&old_path, "id|name\n1|Alice\n2|Bob\n").unwrap();
        fs::write(&new_path, "id|name\n1|Alice\n2|Bob\n3|Charlie\n").unwrap();
        
        // Generate delta
        let info = generate_delta(&old_path, &new_path, &delta_path).unwrap();
        assert!(info.size < info.original_size);
        assert!(info.ratio < 100);
        
        // Apply delta
        apply_delta(&old_path, &delta_path, &output_path).unwrap();
        
        // Verify output matches new version
        let output = fs::read_to_string(&output_path).unwrap();
        let expected = fs::read_to_string(&new_path).unwrap();
        assert_eq!(output, expected);
    }
    
    #[test]
    fn test_generate_delta_one_line_change() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");
        
        fs::write(&old_path, "id|name|age\n1|Alice|30\n2|Bob|25\n").unwrap();
        fs::write(&new_path, "id|name|age\n1|Alice|31\n2|Bob|25\n").unwrap();
        
        let info = generate_delta(&old_path, &new_path, &delta_path).unwrap();
        
        // Delta should be very small (< 10% of original)
        assert!(info.ratio < 10);
        assert!(info.size < 500); // < 500 bytes for 1-char change
    }
    
    #[test]
    fn test_create_bsdiff() {
        let old = b"hello world";
        let new = b"hello rust world";
        
        let delta = create_bsdiff(old, new).unwrap();
        assert!(!delta.is_empty());
        assert!(delta.len() < new.len());
    }
    
    #[test]
    fn test_apply_bspatch() {
        let old = b"hello world";
        let new = b"hello rust world";
        
        let delta = create_bsdiff(old, new).unwrap();
        let reconstructed = apply_bspatch(old, &delta).unwrap();
        
        assert_eq!(reconstructed, new);
    }
    
    #[test]
    fn test_compress_decompress_delta() {
        let delta = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        
        let compressed = compress_delta(&delta).unwrap();
        assert!(compressed.len() < delta.len());
        
        let decompressed = decompress_delta(&compressed).unwrap();
        assert_eq!(decompressed, delta);
    }
    
    #[test]
    fn test_calculate_savings() {
        assert_eq!(calculate_savings(500, 10000), 95.0);
        assert_eq!(calculate_savings(5000, 10000), 50.0);
        assert_eq!(calculate_savings(10000, 10000), 0.0);
        assert_eq!(calculate_savings(0, 0), 0.0);
    }
    
    #[test]
    fn test_generate_delta_file_not_found() {
        let result = generate_delta(
            "/nonexistent/old.csv",
            "/nonexistent/new.csv",
            "/tmp/delta.bsdiff"
        );
        assert!(matches!(result, Err(ReedError::IoError { .. })));
    }
    
    #[test]
    fn test_apply_delta_corrupted() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");
        let output_path = temp_dir.path().join("output.csv");
        
        fs::write(&old_path, "test").unwrap();
        fs::write(&delta_path, "corrupted data").unwrap();
        
        let result = apply_delta(&old_path, &delta_path, &output_path);
        assert!(matches!(result, Err(ReedError::DecompressionFailed { .. })));
    }
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Generate delta (100 rows, 10KB) | < 50ms |
| Generate delta (1000 rows, 100KB) | < 500ms |
| Apply delta (100 rows) | < 30ms |
| Apply delta (1000 rows) | < 300ms |
| Delta size (1 row change) | < 500 bytes |
| Delta size (10% change) | < 5% of original |
| Compression ratio | 30-50% of uncompressed delta |

## Error Conditions

- **IoError**: Cannot read/write files
- **DeltaGenerationFailed**: bsdiff operation failed
- **DeltaApplicationFailed**: bspatch operation failed
- **CorruptedDelta**: Delta file corrupted or invalid
- **CompressionFailed**: XZ compression error
- **DecompressionFailed**: XZ decompression error

## CLI Commands

```bash
# Generate delta (internal use, not exposed to CLI)
# Called by Table::write() in REED-19-02

# Rollback to version (uses delta chain)
reed rollback users 1736860800
# Internally: Apply deltas in reverse to reconstruct version
```

## Acceptance Criteria

- [ ] Generate binary delta using bsdiff
- [ ] Apply binary delta using bspatch
- [ ] Compress deltas using XZ
- [ ] Decompress deltas using XZ
- [ ] Delta size < 5% of full for typical changes (10-20% row modifications)
- [ ] Delta size < 500 bytes for single row change
- [ ] Generate delta in < 50ms for 100-row CSV
- [ ] Apply delta in < 30ms for 100-row CSV
- [ ] Calculate savings percentage
- [ ] Handle corrupted delta files gracefully
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation (Input/Output/Performance/Error Conditions/Example Usage)
- [ ] No Swiss Army knife functions (each function = one job)
- [ ] Separate test file as `delta.test.rs`

## Dependencies

**Requires**: 
- REED-19-02 (Universal Table API - integration point)

**Blocks**: 
- REED-19-06 (Concurrent Write System - needs delta generation)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-19-00: Layer Overview

## Notes

**Why bsdiff + XZ?**
- **bsdiff**: Best-in-class binary delta algorithm (used by FreeBSD, Chrome updates)
- **XZ**: Excellent compression for text (better than gzip for CSV data)
- **Combined**: 95%+ disk savings vs full snapshots

**Comparison vs Git:**
- Git uses similar approach (delta compression)
- ReedBase optimized for CSV (line-based changes)
- Performance comparable to Git for similar workloads

**Trade-offs:**
- **Pro**: 95%+ disk savings, Git-like performance
- **Pro**: Can store thousands of versions in < 100MB
- **Con**: Rollback requires applying delta chain (slower than full snapshots)
- **Con**: Corrupted delta breaks reconstruction (mitigated by periodic full snapshots in REED-19-07)

**Future Enhancements:**
- Periodic full snapshots (every 100 deltas) for faster rollback
- Delta chain optimization (merge old deltas)
- Parallel delta application for long chains
