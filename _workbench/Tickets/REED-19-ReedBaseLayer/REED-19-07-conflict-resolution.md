# REED-19-06: Conflict Resolution

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
- **ID**: REED-19-06
- **Title**: Conflict Resolution
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-19-05 (Row-Level CSV Merge)
- **Estimated Time**: 5 days

## Objective

Provide conflict resolution strategies and CLI interface for manual conflict resolution. Support automatic resolution policies (last-write-wins, first-write-wins, manual) and conflict file generation.

## Requirements

### Conflict File Format

```
.reed/tables/{table_name}/
├── current.csv
└── conflicts/
    └── {timestamp}.conflict     # Conflict marker file
```

**Conflict file content:**
```toml
[conflict]
timestamp = 1736860900
table = "users"
key = "user_123"

[base]
values = ["Alice", "30", "alice@example.com"]

[change_a]
timestamp = 1736860850
user = "admin"
values = ["Alice", "31", "alice@example.com"]

[change_b]
timestamp = 1736860860
user = "editor"
values = ["Alice", "30", "alice.new@example.com"]
```

### Resolution Strategies

**1. Last-Write-Wins (LWW)**
```rust
// Automatically accept change with newest timestamp
resolve_conflict(conflict, ResolutionStrategy::LastWriteWins)
// Result: change_b (timestamp 1736860860)
```

**2. First-Write-Wins (FWW)**
```rust
// Automatically accept change with oldest timestamp
resolve_conflict(conflict, ResolutionStrategy::FirstWriteWins)
// Result: change_a (timestamp 1736860850)
```

**3. Manual**
```rust
// Write conflict file, require manual resolution
resolve_conflict(conflict, ResolutionStrategy::Manual)
// Result: Conflict file written, operation queued
```

### Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Write conflict file | < 10ms | TOML serialization |
| Load conflict file | < 5ms | TOML parsing |
| Apply resolution | < 20ms | Update CSV + delta |
| List conflicts | < 50ms | Scan conflicts directory |

## Implementation Files

### Primary Implementation

**`reedbase/src/conflict/resolution.rs`**

One file = Conflict resolution logic only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Conflict resolution for concurrent writes.
//!
//! Provides automatic and manual resolution strategies.

use std::path::PathBuf;
use std::fs;
use crate::types::{ReedResult, ReedError, Conflict, CsvRow};

/// Resolve conflict using specified strategy.
///
/// ## Input
/// - `conflict`: Conflict to resolve
/// - `strategy`: Resolution strategy
///
/// ## Output
/// - `ReedResult<CsvRow>`: Resolved row
///
/// ## Performance
/// - < 20ms for automatic strategies
/// - < 10ms for writing manual conflict file
///
/// ## Error Conditions
/// - IoError: Cannot write conflict file (manual strategy)
/// - InvalidStrategy: Strategy not applicable
///
/// ## Example Usage
/// ```rust
/// let resolved = resolve_conflict(&conflict, ResolutionStrategy::LastWriteWins)?;
/// apply_resolution("users", &resolved)?;
/// ```
pub fn resolve_conflict(
    conflict: &Conflict,
    strategy: ResolutionStrategy,
) -> ReedResult<Resolution> {
    match strategy {
        ResolutionStrategy::LastWriteWins => {
            Ok(Resolution::Automatic(conflict.change_b.clone()))
        }
        ResolutionStrategy::FirstWriteWins => {
            Ok(Resolution::Automatic(conflict.change_a.clone()))
        }
        ResolutionStrategy::Manual => {
            let conflict_id = write_conflict_file(conflict)?;
            Ok(Resolution::Manual(conflict_id))
        }
        ResolutionStrategy::KeepBoth => {
            // Keep both changes as separate rows (append suffix to key)
            let mut row_a = conflict.change_a.clone();
            let mut row_b = conflict.change_b.clone();
            row_a.key = format!("{}_a", conflict.key);
            row_b.key = format!("{}_b", conflict.key);
            Ok(Resolution::KeepBoth(row_a, row_b))
        }
    }
}

/// Write conflict to file for manual resolution.
///
/// ## Input
/// - `conflict`: Conflict to write
///
/// ## Output
/// - `ReedResult<String>`: Conflict ID (timestamp)
///
/// ## Performance
/// - < 10ms typical (TOML serialization + write)
///
/// ## Error Conditions
/// - IoError: Cannot write conflict file
/// - SerializationError: Cannot serialize to TOML
///
/// ## Example Usage
/// ```rust
/// let conflict_id = write_conflict_file(&conflict)?;
/// println!("Conflict written: {}", conflict_id);
/// ```
fn write_conflict_file(conflict: &Conflict) -> ReedResult<String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let conflict_id = timestamp.to_string();
    let conflict_path = get_conflicts_dir(&conflict.key)
        .join(format!("{}.conflict", conflict_id));
    
    fs::create_dir_all(conflict_path.parent().unwrap())
        .map_err(|e| ReedError::IoError {
            path: conflict_path.parent().unwrap().to_string_lossy().to_string(),
            source: e,
        })?;
    
    let toml = serialize_conflict(conflict, timestamp)?;
    
    fs::write(&conflict_path, toml)
        .map_err(|e| ReedError::IoError {
            path: conflict_path.to_string_lossy().to_string(),
            source: e,
        })?;
    
    Ok(conflict_id)
}

/// Serialize conflict to TOML format.
///
/// ## Input
/// - `conflict`: Conflict to serialize
/// - `timestamp`: Conflict timestamp
///
/// ## Output
/// - `ReedResult<String>`: TOML string
///
/// ## Performance
/// - < 5ms typical
///
/// ## Error Conditions
/// - SerializationError: TOML serialization failed
fn serialize_conflict(conflict: &Conflict, timestamp: u64) -> ReedResult<String> {
    let mut toml = String::new();
    
    toml.push_str("[conflict]\n");
    toml.push_str(&format!("timestamp = {}\n", timestamp));
    toml.push_str(&format!("key = \"{}\"\n\n", conflict.key));
    
    if let Some(ref base) = conflict.base {
        toml.push_str("[base]\n");
        toml.push_str(&format!("values = {:?}\n\n", base.values));
    }
    
    toml.push_str("[change_a]\n");
    toml.push_str(&format!("values = {:?}\n\n", conflict.change_a.values));
    
    toml.push_str("[change_b]\n");
    toml.push_str(&format!("values = {:?}\n", conflict.change_b.values));
    
    Ok(toml)
}

/// Load conflict from file.
///
/// ## Input
/// - `table_name`: Table name
/// - `conflict_id`: Conflict ID
///
/// ## Output
/// - `ReedResult<Conflict>`: Loaded conflict
///
/// ## Performance
/// - < 5ms typical (TOML parsing)
///
/// ## Error Conditions
/// - IoError: Cannot read conflict file
/// - DeserializationError: Invalid TOML format
///
/// ## Example Usage
/// ```rust
/// let conflict = load_conflict_file("users", "1736860900")?;
/// ```
pub fn load_conflict_file(table_name: &str, conflict_id: &str) -> ReedResult<Conflict> {
    let conflict_path = get_conflicts_dir(table_name)
        .join(format!("{}.conflict", conflict_id));
    
    let toml = fs::read_to_string(&conflict_path)
        .map_err(|e| ReedError::IoError {
            path: conflict_path.to_string_lossy().to_string(),
            source: e,
        })?;
    
    parse_conflict(&toml)
}

/// Parse conflict from TOML string.
///
/// ## Input
/// - `toml`: TOML string
///
/// ## Output
/// - `ReedResult<Conflict>`: Parsed conflict
///
/// ## Performance
/// - < 3ms typical
///
/// ## Error Conditions
/// - DeserializationError: Invalid TOML format
fn parse_conflict(toml: &str) -> ReedResult<Conflict> {
    use toml::Value;
    
    let value: Value = toml::from_str(toml)
        .map_err(|e| ReedError::DeserializationError {
            reason: format!("TOML parse error: {}", e),
        })?;
    
    let key = value["conflict"]["key"]
        .as_str()
        .ok_or_else(|| ReedError::DeserializationError {
            reason: "Missing conflict.key".to_string(),
        })?
        .to_string();
    
    let base = if value.get("base").is_some() {
        Some(parse_csv_row_from_toml(&value["base"])?)
    } else {
        None
    };
    
    let change_a = parse_csv_row_from_toml(&value["change_a"])?;
    let change_b = parse_csv_row_from_toml(&value["change_b"])?;
    
    Ok(Conflict {
        key: key.clone(),
        base,
        change_a: CsvRow { key: key.clone(), values: change_a },
        change_b: CsvRow { key, values: change_b },
    })
}

/// Parse CSV row values from TOML value.
///
/// ## Input
/// - `value`: TOML value
///
/// ## Output
/// - `ReedResult<Vec<String>>`: Row values
///
/// ## Performance
/// - < 1ms typical
///
/// ## Error Conditions
/// - DeserializationError: Invalid format
fn parse_csv_row_from_toml(value: &toml::Value) -> ReedResult<Vec<String>> {
    value["values"]
        .as_array()
        .ok_or_else(|| ReedError::DeserializationError {
            reason: "Missing or invalid values array".to_string(),
        })?
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| ReedError::DeserializationError {
                    reason: "Value is not a string".to_string(),
                })
                .map(|s| s.to_string())
        })
        .collect()
}

/// List all conflicts for a table.
///
/// ## Input
/// - `table_name`: Table name
///
/// ## Output
/// - `ReedResult<Vec<String>>`: List of conflict IDs
///
/// ## Performance
/// - < 50ms for typical conflict directories (< 100 files)
///
/// ## Error Conditions
/// - IoError: Cannot read conflicts directory
///
/// ## Example Usage
/// ```rust
/// let conflicts = list_conflicts("users")?;
/// println!("Found {} conflicts", conflicts.len());
/// ```
pub fn list_conflicts(table_name: &str) -> ReedResult<Vec<String>> {
    let conflicts_dir = get_conflicts_dir(table_name);
    
    if !conflicts_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut conflicts = Vec::new();
    
    for entry in fs::read_dir(&conflicts_dir)
        .map_err(|e| ReedError::IoError {
            path: conflicts_dir.to_string_lossy().to_string(),
            source: e,
        })?
    {
        let entry = entry.map_err(|e| ReedError::IoError {
            path: conflicts_dir.to_string_lossy().to_string(),
            source: e.into(),
        })?;
        
        if let Some(ext) = entry.path().extension() {
            if ext == "conflict" {
                if let Some(stem) = entry.path().file_stem() {
                    conflicts.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    
    conflicts.sort();
    Ok(conflicts)
}

/// Delete conflict file after resolution.
///
/// ## Input
/// - `table_name`: Table name
/// - `conflict_id`: Conflict ID
///
/// ## Output
/// - `ReedResult<()>`: Success or error
///
/// ## Performance
/// - < 5ms typical
///
/// ## Error Conditions
/// - IoError: Cannot delete conflict file
///
/// ## Example Usage
/// ```rust
/// delete_conflict_file("users", "1736860900")?;
/// ```
pub fn delete_conflict_file(table_name: &str, conflict_id: &str) -> ReedResult<()> {
    let conflict_path = get_conflicts_dir(table_name)
        .join(format!("{}.conflict", conflict_id));
    
    fs::remove_file(&conflict_path)
        .map_err(|e| ReedError::IoError {
            path: conflict_path.to_string_lossy().to_string(),
            source: e,
        })?;
    
    Ok(())
}

/// Get conflicts directory path.
///
/// ## Input
/// - `table_name`: Table name
///
/// ## Output
/// - `PathBuf`: Conflicts directory path
///
/// ## Performance
/// - O(1) operation
/// - < 1μs
fn get_conflicts_dir(table_name: &str) -> PathBuf {
    PathBuf::from(format!(".reed/tables/{}/conflicts", table_name))
}

/// Resolution strategy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolutionStrategy {
    LastWriteWins,   // Accept change with newest timestamp
    FirstWriteWins,  // Accept change with oldest timestamp
    Manual,          // Require manual resolution
    KeepBoth,        // Keep both changes as separate rows
}

/// Resolution result.
#[derive(Debug, Clone)]
pub enum Resolution {
    Automatic(CsvRow),
    Manual(String), // Conflict ID
    KeepBoth(CsvRow, CsvRow),
}
```

**`reedbase/src/types.rs`** (additions - already defined in REED-19-05, just documenting here)

```rust
// Already defined in REED-19-05:
// pub struct Conflict { ... }
```

### Test Files

**`reedbase/src/conflict/resolution.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_conflict() -> Conflict {
        Conflict {
            key: "user_123".to_string(),
            base: Some(CsvRow {
                key: "user_123".to_string(),
                values: vec!["Alice".to_string(), "30".to_string()],
            }),
            change_a: CsvRow {
                key: "user_123".to_string(),
                values: vec!["Alice".to_string(), "31".to_string()],
            },
            change_b: CsvRow {
                key: "user_123".to_string(),
                values: vec!["Alice".to_string(), "32".to_string()],
            },
        }
    }
    
    #[test]
    fn test_resolve_last_write_wins() {
        let conflict = create_test_conflict();
        let resolution = resolve_conflict(&conflict, ResolutionStrategy::LastWriteWins).unwrap();
        
        match resolution {
            Resolution::Automatic(row) => {
                assert_eq!(row.values[1], "32");
            }
            _ => panic!("Expected Automatic resolution"),
        }
    }
    
    #[test]
    fn test_resolve_first_write_wins() {
        let conflict = create_test_conflict();
        let resolution = resolve_conflict(&conflict, ResolutionStrategy::FirstWriteWins).unwrap();
        
        match resolution {
            Resolution::Automatic(row) => {
                assert_eq!(row.values[1], "31");
            }
            _ => panic!("Expected Automatic resolution"),
        }
    }
    
    #[test]
    fn test_resolve_keep_both() {
        let conflict = create_test_conflict();
        let resolution = resolve_conflict(&conflict, ResolutionStrategy::KeepBoth).unwrap();
        
        match resolution {
            Resolution::KeepBoth(row_a, row_b) => {
                assert_eq!(row_a.key, "user_123_a");
                assert_eq!(row_b.key, "user_123_b");
                assert_eq!(row_a.values[1], "31");
                assert_eq!(row_b.values[1], "32");
            }
            _ => panic!("Expected KeepBoth resolution"),
        }
    }
    
    #[test]
    fn test_write_and_load_conflict_file() {
        let _temp_dir = TempDir::new().unwrap();
        
        let conflict = create_test_conflict();
        let conflict_id = write_conflict_file(&conflict).unwrap();
        
        let loaded = load_conflict_file("users", &conflict_id).unwrap();
        
        assert_eq!(loaded.key, conflict.key);
        assert_eq!(loaded.change_a.values, conflict.change_a.values);
        assert_eq!(loaded.change_b.values, conflict.change_b.values);
    }
    
    #[test]
    fn test_serialize_conflict() {
        let conflict = create_test_conflict();
        let toml = serialize_conflict(&conflict, 1736860900).unwrap();
        
        assert!(toml.contains("timestamp = 1736860900"));
        assert!(toml.contains("key = \"user_123\""));
        assert!(toml.contains("[change_a]"));
        assert!(toml.contains("[change_b]"));
    }
    
    #[test]
    fn test_list_conflicts() {
        let _temp_dir = TempDir::new().unwrap();
        
        let conflict1 = create_test_conflict();
        let conflict2 = create_test_conflict();
        
        write_conflict_file(&conflict1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        write_conflict_file(&conflict2).unwrap();
        
        let conflicts = list_conflicts("users").unwrap();
        assert_eq!(conflicts.len(), 2);
    }
    
    #[test]
    fn test_delete_conflict_file() {
        let _temp_dir = TempDir::new().unwrap();
        
        let conflict = create_test_conflict();
        let conflict_id = write_conflict_file(&conflict).unwrap();
        
        assert_eq!(list_conflicts("users").unwrap().len(), 1);
        
        delete_conflict_file("users", &conflict_id).unwrap();
        
        assert_eq!(list_conflicts("users").unwrap().len(), 0);
    }
    
    #[test]
    fn test_list_conflicts_empty() {
        let conflicts = list_conflicts("nonexistent_table").unwrap();
        assert_eq!(conflicts.len(), 0);
    }
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Write conflict file | < 10ms |
| Load conflict file | < 5ms |
| Serialize to TOML | < 5ms |
| Parse from TOML | < 3ms |
| List conflicts | < 50ms |
| Delete conflict file | < 5ms |
| Apply automatic resolution | < 20ms |

## Error Conditions

- **IoError**: Cannot read/write conflict files
- **SerializationError**: Cannot serialize conflict to TOML
- **DeserializationError**: Cannot parse TOML or invalid format
- **InvalidStrategy**: Strategy not applicable to conflict type

## CLI Commands

```bash
# List conflicts
reed conflicts:list users
# Output:
# 1736860900 - user_123 (2 changes)
# 1736861000 - user_456 (2 changes)

# Show conflict details
reed conflicts:show users 1736860900
# Output: TOML conflict file content

# Resolve conflict (automatic)
reed conflicts:resolve users 1736860900 --strategy last-write-wins
# Output: ✓ Resolved (accepted change_b)

# Resolve conflict (manual - select specific change)
reed conflicts:resolve users 1736860900 --accept a
# Output: ✓ Resolved (accepted change_a)

# Resolve conflict (manual - provide custom values)
reed conflicts:resolve users 1736860900 --values "Alice,33,alice@example.com"
# Output: ✓ Resolved (applied custom values)
```

## Acceptance Criteria

- [ ] Resolve conflicts with LastWriteWins strategy
- [ ] Resolve conflicts with FirstWriteWins strategy
- [ ] Resolve conflicts with KeepBoth strategy (append suffixes)
- [ ] Write conflict to TOML file (Manual strategy)
- [ ] Load conflict from TOML file
- [ ] Serialize conflict to TOML format
- [ ] Parse conflict from TOML format
- [ ] List all conflicts for a table
- [ ] Delete conflict file after resolution
- [ ] CLI commands for conflict management
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation (Input/Output/Performance/Error Conditions/Example Usage)
- [ ] No Swiss Army knife functions (each function = one job)
- [ ] Separate test file as `resolution.test.rs`

## Dependencies

**Requires**: 
- REED-19-05 (Row-Level CSV Merge - provides Conflict type)

**Blocks**: None

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-19-00: Layer Overview

## Notes

**Resolution Strategies Comparison:**

| Strategy | Auto? | Data Loss? | Use Case |
|----------|-------|------------|----------|
| LastWriteWins | ✅ | ⚠️ Loses older change | Real-time collaborative editing |
| FirstWriteWins | ✅ | ⚠️ Loses newer change | Audit/compliance (first entry wins) |
| Manual | ❌ | ❌ No loss | Critical data, human review required |
| KeepBoth | ✅ | ❌ No loss | Non-critical data, keep all changes |

**Default Strategy:**
- Development: **Manual** (safer, catch issues early)
- Production: **LastWriteWins** (fewer interruptions, acceptable for most data)

**Conflict File Format Choice:**
- **TOML** chosen over JSON for human readability
- Easy to edit manually if needed
- Clear section separation (base/change_a/change_b)

**Trade-offs:**
- **Pro**: Flexible resolution strategies
- **Pro**: Zero data loss (all changes preserved in conflict file)
- **Pro**: Human-readable conflict format
- **Con**: Manual resolution requires user intervention
- **Con**: Conflict files accumulate if not resolved (mitigated by monitoring)

**Future Enhancements:**
- Web UI for conflict resolution
- Automatic resolution policies per table
- Conflict notification system (email/webhook)
- Bulk conflict resolution
