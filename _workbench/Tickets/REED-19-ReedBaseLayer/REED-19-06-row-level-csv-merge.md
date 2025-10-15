# REED-19-06: Row-Level CSV Merge

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
- **Title**: Row-Level CSV Merge
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-19-06 (Concurrent Write System)
- **Estimated Time**: 1 week

## Objective

Implement intelligent row-level merging for concurrent writes. Automatically merge non-conflicting changes (different rows) and detect conflicts (same row modified).

## Requirements

### Merge Scenarios

**Scenario 1: Different Rows (Auto-merge)**
```
Base:      1|Alice|30
           2|Bob|25

Process A: 1|Alice|31  (updates row 1)
Process B: 3|Charlie|35 (inserts row 3)

Merged:    1|Alice|31
           2|Bob|25
           3|Charlie|35
```

**Scenario 2: Same Row (Conflict)**
```
Base:      1|Alice|30

Process A: 1|Alice|31  (updates age to 31)
Process B: 1|Alice|32  (updates age to 32)

Result:    CONFLICT - requires manual resolution
```

**Scenario 3: Insert + Delete (Auto-merge)**
```
Base:      1|Alice|30
           2|Bob|25

Process A: (deletes row 2)
Process B: 3|Charlie|35 (inserts row 3)

Merged:    1|Alice|30
           3|Charlie|35
```

### Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Parse CSV (100 rows) | < 10ms | Into in-memory structure |
| Detect conflicts (100 rows) | < 5ms | Row key comparison |
| Merge non-conflicting (100 rows) | < 20ms | Combine changes |
| Write merged CSV (100 rows) | < 15ms | Back to disk |
| Total merge (100 rows, no conflicts) | < 50ms | End-to-end |

## Implementation Files

### Primary Implementation

**`reedbase/src/merge/csv.rs`**

One file = CSV merging only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Row-level CSV merging for concurrent writes.
//!
//! Automatically merges non-conflicting changes at row level.

use std::collections::HashMap;
use crate::types::{ReedResult, ReedError, CsvRow};

/// Merge two sets of changes into base CSV.
///
/// ## Input
/// - `base`: Base CSV rows
/// - `changes_a`: Changes from process A
/// - `changes_b`: Changes from process B
///
/// ## Output
/// - `ReedResult<MergeResult>`: Merged rows or conflicts
///
/// ## Performance
/// - O(n) where n = total rows
/// - < 50ms for 100 rows (no conflicts)
///
/// ## Error Conditions
/// - ParseError: Invalid CSV format
///
/// ## Example Usage
/// ```rust
/// let merged = merge_changes(&base, &changes_a, &changes_b)?;
/// match merged {
///     MergeResult::Success(rows) => write_csv(rows)?,
///     MergeResult::Conflicts(conflicts) => handle_conflicts(conflicts)?,
/// }
/// ```
pub fn merge_changes(
    base: &[CsvRow],
    changes_a: &[CsvRow],
    changes_b: &[CsvRow],
) -> ReedResult<MergeResult> {
    let mut merged = build_row_map(base);
    let mut conflicts = Vec::new();
    
    // Apply changes from A
    for row in changes_a {
        merged.insert(row.key.clone(), row.clone());
    }
    
    // Apply changes from B, detecting conflicts
    for row in changes_b {
        if let Some(existing) = merged.get(&row.key) {
            // Check if this row was also modified by A
            if changes_a.iter().any(|a| a.key == row.key) {
                // Conflict: both A and B modified same row
                conflicts.push(Conflict {
                    key: row.key.clone(),
                    base: base.iter().find(|b| b.key == row.key).cloned(),
                    change_a: existing.clone(),
                    change_b: row.clone(),
                });
                continue;
            }
        }
        merged.insert(row.key.clone(), row.clone());
    }
    
    if conflicts.is_empty() {
        let mut rows: Vec<_> = merged.into_values().collect();
        rows.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(MergeResult::Success(rows))
    } else {
        Ok(MergeResult::Conflicts(conflicts))
    }
}

/// Merge single change set into base.
///
/// ## Input
/// - `base`: Base CSV rows
/// - `changes`: Changes to apply
///
/// ## Output
/// - `ReedResult<Vec<CsvRow>>`: Merged rows
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 20ms for 100 rows
///
/// ## Error Conditions
/// - ParseError: Invalid CSV format
///
/// ## Example Usage
/// ```rust
/// let merged = merge_single(&base, &changes)?;
/// write_csv(&merged)?;
/// ```
pub fn merge_single(
    base: &[CsvRow],
    changes: &[CsvRow],
) -> ReedResult<Vec<CsvRow>> {
    let mut merged = build_row_map(base);
    
    for row in changes {
        merged.insert(row.key.clone(), row.clone());
    }
    
    let mut rows: Vec<_> = merged.into_values().collect();
    rows.sort_by(|a, b| a.key.cmp(&b.key));
    
    Ok(rows)
}

/// Build HashMap from CSV rows for fast lookup.
///
/// ## Input
/// - `rows`: CSV rows
///
/// ## Output
/// - `HashMap<String, CsvRow>`: Row map (key → row)
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 5ms for 100 rows
///
/// ## Example Usage
/// ```rust
/// let map = build_row_map(&rows);
/// if let Some(row) = map.get("user_123") {
///     println!("Found: {:?}", row);
/// }
/// ```
fn build_row_map(rows: &[CsvRow]) -> HashMap<String, CsvRow> {
    rows.iter()
        .map(|row| (row.key.clone(), row.clone()))
        .collect()
}

/// Detect conflicts between two change sets.
///
/// ## Input
/// - `changes_a`: Changes from process A
/// - `changes_b`: Changes from process B
///
/// ## Output
/// - `Vec<String>`: List of conflicting row keys
///
/// ## Performance
/// - O(n*m) where n,m = number of changes
/// - < 5ms for 100 changes each
///
/// ## Example Usage
/// ```rust
/// let conflicts = detect_conflicts(&changes_a, &changes_b);
/// if !conflicts.is_empty() {
///     println!("Conflicts detected: {:?}", conflicts);
/// }
/// ```
pub fn detect_conflicts(
    changes_a: &[CsvRow],
    changes_b: &[CsvRow],
) -> Vec<String> {
    let keys_a: Vec<&String> = changes_a.iter().map(|r| &r.key).collect();
    
    changes_b.iter()
        .filter(|row| keys_a.contains(&&row.key))
        .map(|row| row.key.clone())
        .collect()
}

/// Check if rows have same values.
///
/// ## Input
/// - `row_a`: First row
/// - `row_b`: Second row
///
/// ## Output
/// - `bool`: True if values match
///
/// ## Performance
/// - O(n) where n = number of columns
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// if rows_equal(&row_a, &row_b) {
///     println!("No actual change");
/// }
/// ```
pub fn rows_equal(row_a: &CsvRow, row_b: &CsvRow) -> bool {
    row_a.key == row_b.key && row_a.values == row_b.values
}

/// Calculate merge statistics.
///
/// ## Input
/// - `base`: Base rows count
/// - `merged`: Merged rows count
/// - `conflicts`: Number of conflicts
///
/// ## Output
/// - `MergeStats`: Merge statistics
///
/// ## Performance
/// - O(1) operation
/// - < 1μs
///
/// ## Example Usage
/// ```rust
/// let stats = calculate_merge_stats(100, 105, 2);
/// println!("Added: {}, Modified: {}, Conflicts: {}",
///     stats.added, stats.modified, stats.conflicts);
/// ```
pub fn calculate_merge_stats(
    base_count: usize,
    merged_count: usize,
    conflicts: usize,
) -> MergeStats {
    MergeStats {
        added: if merged_count > base_count {
            merged_count - base_count
        } else {
            0
        },
        deleted: if base_count > merged_count {
            base_count - merged_count
        } else {
            0
        },
        modified: 0, // TODO: Track modifications separately
        conflicts,
    }
}
```

**`reedbase/src/merge/diff.rs`**

One file = Diff calculation only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CSV diff calculation for change detection.
//!
//! Calculates row-level differences between CSV versions.

use crate::types::{ReedResult, CsvRow, RowChange};
use std::collections::HashSet;

/// Calculate diff between two CSV versions.
///
/// ## Input
/// - `old`: Old version rows
/// - `new`: New version rows
///
/// ## Output
/// - `ReedResult<Vec<RowChange>>`: List of changes
///
/// ## Performance
/// - O(n+m) where n,m = number of rows
/// - < 15ms for 100 rows
///
/// ## Error Conditions
/// - None (pure computation)
///
/// ## Example Usage
/// ```rust
/// let changes = calculate_diff(&old_rows, &new_rows)?;
/// for change in changes {
///     match change {
///         RowChange::Insert(row) => println!("+ {}", row.key),
///         RowChange::Update(row) => println!("~ {}", row.key),
///         RowChange::Delete(key) => println!("- {}", key),
///     }
/// }
/// ```
pub fn calculate_diff(
    old: &[CsvRow],
    new: &[CsvRow],
) -> ReedResult<Vec<RowChange>> {
    let old_keys: HashSet<&String> = old.iter().map(|r| &r.key).collect();
    let new_keys: HashSet<&String> = new.iter().map(|r| &r.key).collect();
    
    let mut changes = Vec::new();
    
    // Find deletions
    for key in old_keys.difference(&new_keys) {
        changes.push(RowChange::Delete((*key).clone()));
    }
    
    // Find insertions
    for row in new {
        if !old_keys.contains(&row.key) {
            changes.push(RowChange::Insert(row.clone()));
        }
    }
    
    // Find updates
    for new_row in new {
        if let Some(old_row) = old.iter().find(|r| r.key == new_row.key) {
            if old_row.values != new_row.values {
                changes.push(RowChange::Update(new_row.clone()));
            }
        }
    }
    
    Ok(changes)
}

/// Apply changes to base rows.
///
/// ## Input
/// - `base`: Base rows
/// - `changes`: Changes to apply
///
/// ## Output
/// - `ReedResult<Vec<CsvRow>>`: Updated rows
///
/// ## Performance
/// - O(n) where n = number of rows
/// - < 10ms for 100 rows
///
/// ## Error Conditions
/// - None (pure computation)
///
/// ## Example Usage
/// ```rust
/// let updated = apply_changes(&base, &changes)?;
/// ```
pub fn apply_changes(
    base: &[CsvRow],
    changes: &[RowChange],
) -> ReedResult<Vec<CsvRow>> {
    use std::collections::HashMap;
    
    let mut rows: HashMap<String, CsvRow> = base.iter()
        .map(|r| (r.key.clone(), r.clone()))
        .collect();
    
    for change in changes {
        match change {
            RowChange::Insert(row) | RowChange::Update(row) => {
                rows.insert(row.key.clone(), row.clone());
            }
            RowChange::Delete(key) => {
                rows.remove(key);
            }
        }
    }
    
    let mut result: Vec<_> = rows.into_values().collect();
    result.sort_by(|a, b| a.key.cmp(&b.key));
    
    Ok(result)
}

/// Count changes by type.
///
/// ## Input
/// - `changes`: List of changes
///
/// ## Output
/// - `(usize, usize, usize)`: (inserts, updates, deletes)
///
/// ## Performance
/// - O(n) where n = number of changes
/// - < 1ms for 100 changes
///
/// ## Example Usage
/// ```rust
/// let (ins, upd, del) = count_changes(&changes);
/// println!("+ {} ~ {} - {}", ins, upd, del);
/// ```
pub fn count_changes(changes: &[RowChange]) -> (usize, usize, usize) {
    let inserts = changes.iter().filter(|c| matches!(c, RowChange::Insert(_))).count();
    let updates = changes.iter().filter(|c| matches!(c, RowChange::Update(_))).count();
    let deletes = changes.iter().filter(|c| matches!(c, RowChange::Delete(_))).count();
    
    (inserts, updates, deletes)
}
```

**`reedbase/src/types.rs`** (additions)

```rust
/// Row change types.
#[derive(Debug, Clone)]
pub enum RowChange {
    Insert(CsvRow),
    Update(CsvRow),
    Delete(String), // Key only
}

/// Merge result.
#[derive(Debug)]
pub enum MergeResult {
    Success(Vec<CsvRow>),
    Conflicts(Vec<Conflict>),
}

/// Merge conflict.
#[derive(Debug, Clone)]
pub struct Conflict {
    pub key: String,
    pub base: Option<CsvRow>,
    pub change_a: CsvRow,
    pub change_b: CsvRow,
}

/// Merge statistics.
#[derive(Debug, Clone)]
pub struct MergeStats {
    pub added: usize,
    pub deleted: usize,
    pub modified: usize,
    pub conflicts: usize,
}
```

### Test Files

**`reedbase/src/merge/csv.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_row(key: &str, values: Vec<&str>) -> CsvRow {
        CsvRow {
            key: key.to_string(),
            values: values.into_iter().map(|s| s.to_string()).collect(),
        }
    }
    
    #[test]
    fn test_merge_different_rows() {
        let base = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];
        
        let changes_a = vec![
            create_row("1", vec!["Alice", "31"]), // Update row 1
        ];
        
        let changes_b = vec![
            create_row("3", vec!["Charlie", "35"]), // Insert row 3
        ];
        
        let result = merge_changes(&base, &changes_a, &changes_b).unwrap();
        
        match result {
            MergeResult::Success(rows) => {
                assert_eq!(rows.len(), 3);
                assert_eq!(rows[0].key, "1");
                assert_eq!(rows[0].values[1], "31");
                assert_eq!(rows[2].key, "3");
            }
            MergeResult::Conflicts(_) => panic!("Expected success, got conflicts"),
        }
    }
    
    #[test]
    fn test_merge_same_row_conflict() {
        let base = vec![
            create_row("1", vec!["Alice", "30"]),
        ];
        
        let changes_a = vec![
            create_row("1", vec!["Alice", "31"]),
        ];
        
        let changes_b = vec![
            create_row("1", vec!["Alice", "32"]),
        ];
        
        let result = merge_changes(&base, &changes_a, &changes_b).unwrap();
        
        match result {
            MergeResult::Success(_) => panic!("Expected conflicts, got success"),
            MergeResult::Conflicts(conflicts) => {
                assert_eq!(conflicts.len(), 1);
                assert_eq!(conflicts[0].key, "1");
                assert_eq!(conflicts[0].change_a.values[1], "31");
                assert_eq!(conflicts[0].change_b.values[1], "32");
            }
        }
    }
    
    #[test]
    fn test_merge_single() {
        let base = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];
        
        let changes = vec![
            create_row("1", vec!["Alice", "31"]),
            create_row("3", vec!["Charlie", "35"]),
        ];
        
        let merged = merge_single(&base, &changes).unwrap();
        
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].values[1], "31");
        assert_eq!(merged[2].key, "3");
    }
    
    #[test]
    fn test_detect_conflicts() {
        let changes_a = vec![
            create_row("1", vec!["Alice", "31"]),
            create_row("2", vec!["Bob", "26"]),
        ];
        
        let changes_b = vec![
            create_row("1", vec!["Alice", "32"]),
            create_row("3", vec!["Charlie", "35"]),
        ];
        
        let conflicts = detect_conflicts(&changes_a, &changes_b);
        
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0], "1");
    }
    
    #[test]
    fn test_rows_equal() {
        let row_a = create_row("1", vec!["Alice", "30"]);
        let row_b = create_row("1", vec!["Alice", "30"]);
        let row_c = create_row("1", vec!["Alice", "31"]);
        
        assert!(rows_equal(&row_a, &row_b));
        assert!(!rows_equal(&row_a, &row_c));
    }
    
    #[test]
    fn test_calculate_merge_stats() {
        let stats = calculate_merge_stats(100, 105, 2);
        
        assert_eq!(stats.added, 5);
        assert_eq!(stats.deleted, 0);
        assert_eq!(stats.conflicts, 2);
    }
}
```

**`reedbase/src/merge/diff.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_row(key: &str, values: Vec<&str>) -> CsvRow {
        CsvRow {
            key: key.to_string(),
            values: values.into_iter().map(|s| s.to_string()).collect(),
        }
    }
    
    #[test]
    fn test_calculate_diff_insert() {
        let old = vec![
            create_row("1", vec!["Alice", "30"]),
        ];
        
        let new = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];
        
        let changes = calculate_diff(&old, &new).unwrap();
        
        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], RowChange::Insert(row) if row.key == "2"));
    }
    
    #[test]
    fn test_calculate_diff_update() {
        let old = vec![
            create_row("1", vec!["Alice", "30"]),
        ];
        
        let new = vec![
            create_row("1", vec!["Alice", "31"]),
        ];
        
        let changes = calculate_diff(&old, &new).unwrap();
        
        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], RowChange::Update(row) if row.values[1] == "31"));
    }
    
    #[test]
    fn test_calculate_diff_delete() {
        let old = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];
        
        let new = vec![
            create_row("1", vec!["Alice", "30"]),
        ];
        
        let changes = calculate_diff(&old, &new).unwrap();
        
        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], RowChange::Delete(key) if key == "2"));
    }
    
    #[test]
    fn test_apply_changes() {
        let base = vec![
            create_row("1", vec!["Alice", "30"]),
            create_row("2", vec!["Bob", "25"]),
        ];
        
        let changes = vec![
            RowChange::Update(create_row("1", vec!["Alice", "31"])),
            RowChange::Delete("2".to_string()),
            RowChange::Insert(create_row("3", vec!["Charlie", "35"])),
        ];
        
        let result = apply_changes(&base, &changes).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].values[1], "31");
        assert_eq!(result[1].key, "3");
    }
    
    #[test]
    fn test_count_changes() {
        let changes = vec![
            RowChange::Insert(create_row("1", vec![])),
            RowChange::Insert(create_row("2", vec![])),
            RowChange::Update(create_row("3", vec![])),
            RowChange::Delete("4".to_string()),
        ];
        
        let (ins, upd, del) = count_changes(&changes);
        
        assert_eq!(ins, 2);
        assert_eq!(upd, 1);
        assert_eq!(del, 1);
    }
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Parse CSV (100 rows) | < 10ms |
| Detect conflicts (100 rows) | < 5ms |
| Merge non-conflicting (100 rows) | < 20ms |
| Calculate diff (100 rows) | < 15ms |
| Apply changes (100 rows) | < 10ms |
| Total merge (no conflicts) | < 50ms |

## Error Conditions

- **ParseError**: Invalid CSV format or structure
- **MergeConflict**: Cannot auto-merge (same row modified)

## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| merge_latency | Histogram | ms | <50 | >200 | merge.rs:merge_rows() |
| conflict_detection_time | Histogram | ms | <5 | >20 | merge.rs:detect_conflicts() |
| auto_merge_success_rate | Gauge | % | >90 | <70 | merge.rs:merge_rows() |
| rows_merged_per_operation | Histogram | count | <100 | >1000 | merge.rs:merge_rows() |
| conflict_count | Counter | count | <10% | >30% | merge.rs:detect_conflicts() |

### Alert Rules

**CRITICAL Alerts:**
- `auto_merge_success_rate < 70%` for 10 minutes → "High merge conflict rate - review write patterns"
- `conflict_count > 30%` for 5 minutes → "Excessive conflicts - possible concurrent write issue"

**WARNING Alerts:**
- `merge_latency p99 > 200ms` for 5 minutes → "Merge operations slow - check row count"
- `rows_merged_per_operation p99 > 1000` for 10 minutes → "Large merge operations - consider batching"

### Implementation

```rust
use crate::reedbase::metrics::global as metrics;
use std::time::Instant;

pub fn merge_rows(base: &[Row], local: &[Row], remote: &[Row]) -> ReedResult<MergeResult> {
    let start = Instant::now();
    let result = merge_rows_inner(base, local, remote)?;
    
    let success = result.conflicts.is_empty();
    let total_rows = local.len().max(remote.len());
    
    metrics().record(Metric {
        name: "merge_latency".to_string(),
        value: start.elapsed().as_millis() as f64,
        unit: MetricUnit::Milliseconds,
        tags: hashmap!{ "rows" => total_rows.to_string() },
    });
    
    metrics().record(Metric {
        name: "auto_merge_success_rate".to_string(),
        value: if success { 100.0 } else { 0.0 },
        unit: MetricUnit::Percent,
        tags: hashmap!{},
    });
    
    metrics().record(Metric {
        name: "conflict_count".to_string(),
        value: result.conflicts.len() as f64,
        unit: MetricUnit::Count,
        tags: hashmap!{ "total_rows" => total_rows.to_string() },
    });
    
    Ok(result)
}
```

### Collection Strategy

- **Sampling**: All operations
- **Aggregation**: 1-minute rolling window
- **Storage**: `.reedbase/metrics/merge.csv`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**merge_latency**: Concurrent write performance
- Merge happens on EVERY concurrent write scenario
- Slow merges block write completion
- Indicates complexity of changes or data size

**auto_merge_success_rate**: System efficiency
- High rate (>90%) = most concurrent writes succeed automatically
- Low rate = frequent manual conflict resolution needed
- Indicates write pattern compatibility

**conflict_detection_time**: Merge overhead
- Should be fast (<5ms) for typical row counts
- Slow detection indicates algorithmic issues
- Affects total merge latency

**conflict_count**: Concurrency indicator
- Low conflicts = good write isolation
- High conflicts = overlapping modifications
- Helps identify problematic concurrent access patterns

## CLI Commands

```bash
# Merge is automatic during concurrent writes
# No direct CLI command - used internally by write system

# View merge statistics (debug)
reed debug:merge-stats users
# Output: +5 ~10 -2 (conflicts: 0)
```

## Acceptance Criteria

- [ ] Merge non-conflicting changes automatically (different rows)
- [ ] Detect conflicts (same row modified by multiple processes)
- [ ] Calculate diff between CSV versions
- [ ] Apply changes to base CSV
- [ ] Count changes by type (insert/update/delete)
- [ ] Build row map for fast lookup (O(1))
- [ ] Check row equality
- [ ] Calculate merge statistics
- [ ] Sort merged rows by key
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation (Input/Output/Performance/Error Conditions/Example Usage)
- [ ] No Swiss Army knife functions (each function = one job)
- [ ] Separate test files as `csv.test.rs` and `diff.test.rs`

## Dependencies

**Requires**: 
- REED-19-06 (Concurrent Write System - provides locking for merge)

**Blocks**: 
- REED-19-07 (Conflict Resolution - handles conflicts detected by merge)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-19-00: Layer Overview

## Notes

**Auto-merge Success Rate:**
- **90%+** of concurrent writes auto-merge successfully
- **Common pattern**: Different users editing different rows
- **Conflict pattern**: Multiple users editing same user profile

**Merge Algorithm:**
1. Build HashMap from base rows (key → row)
2. Apply changes from process A
3. Apply changes from process B
4. If B modifies row already modified by A → CONFLICT
5. Otherwise → SUCCESS

**Trade-offs:**
- **Pro**: Zero data loss (all changes preserved)
- **Pro**: High success rate (90%+ auto-merge)
- **Pro**: Simple conflict detection (key-based)
- **Con**: Same-row conflicts require manual resolution
- **Con**: No column-level merging (row is atomic unit)

**Future Enhancements:**
- Column-level merging (finer-grained)
- Three-way merge (base + A + B)
- Automatic conflict resolution strategies (last-write-wins, etc.)
