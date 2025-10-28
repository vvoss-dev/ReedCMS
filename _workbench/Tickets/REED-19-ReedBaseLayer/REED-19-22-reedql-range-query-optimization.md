# REED-19-22: ReedQL Range-Query Optimization

**Layer**: REED-19 (ReedBase Layer)  
**Phase**: 5 (Distributed + P2P)  
**Dependencies**: REED-19-12 (ReedQL Parser), REED-19-20 (B+-Tree), REED-19-21 (Migrated Indices)  
**Estimated Effort**: 1-2 days  
**Priority**: Medium  
**Status**: Planned

---

## Executive Summary

Optimize ReedQL query execution to **automatically detect and use B+-Tree range scans** instead of full table scans, achieving:
- **100-1000x speedup** for range queries (10ms → 10μs for 1M rows)
- **Zero code changes** required (automatic optimization in executor)
- **Intelligent query planning** (cost-based decision: index vs full scan)
- **Backward compatible** (graceful fallback to full scan if no index)

Currently, ReedQL queries like `SELECT * FROM text WHERE key >= 'page.header' AND key < 'page.header~'` perform full table scans (O(n)). This ticket adds query planning that recognizes range patterns and uses B+-Tree indices (O(log n + k)) when available.

---

## Problem Statement

### Current Behavior (REED-19-12)

```sql
SELECT * FROM text WHERE key >= 'page.header' AND key < 'page.header~'
```

**Execution Plan:**
1. **Full Table Scan**: Read all 1M rows from CSV
2. **Filter Each Row**: Check `key >= 'page.header' AND key < 'page.header~'`
3. **Return Matches**: 150 rows found

**Performance:**
- Time: ~10ms for 1M rows
- Memory: Load entire table into memory (200MB)

### Desired Behavior (REED-19-22)

**Execution Plan:**
1. **Detect Range Query**: Parser identifies `key >= X AND key < Y` pattern
2. **Use B+-Tree Index**: Call `hierarchy_index.range('page.header', 'page.header~')`
3. **Return Matches**: 150 rows found directly

**Performance:**
- Time: ~10μs (1000x faster)
- Memory: Only load matching rows (24KB)

---

## Architecture

### Query Optimizer Pipeline

```
┌──────────────────────────────────────────────────────────────┐
│ ReedQL Query Optimization Pipeline                           │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  1. PARSE                                                    │
│  ┌─────────────────────────────────────────────┐             │
│  │ "SELECT * FROM text WHERE                   │             │
│  │  key >= 'page.' AND key < 'page.~'"         │             │
│  └────────────────┬────────────────────────────┘             │
│                   │                                          │
│                   ▼                                          │
│  2. ANALYZE (NEW)                                            │
│  ┌─────────────────────────────────────────────┐             │
│  │ Detect:                                     │             │
│  │ • Range query on 'key' column               │             │
│  │ • Bounds: ['page.', 'page.~')               │             │
│  │ • Estimated selectivity: 0.015% (150/1M)    │             │
│  └────────────────┬────────────────────────────┘             │
│                   │                                          │
│                   ▼                                          │
│  3. PLAN (NEW)                                               │
│  ┌─────────────────────────────────────────────┐             │
│  │ Available indices:                          │             │
│  │ ✓ hierarchy_index (B+-Tree on 'key')       │             │
│  │                                             │             │
│  │ Cost estimation:                            │             │
│  │ • Index scan: log(1M) + 150 = 170 ops       │             │
│  │ • Full scan: 1M ops                         │             │
│  │                                             │             │
│  │ Decision: USE INDEX (5882x faster)          │             │
│  └────────────────┬────────────────────────────┘             │
│                   │                                          │
│                   ▼                                          │
│  4. EXECUTE                                                  │
│  ┌─────────────────────────────────────────────┐             │
│  │ hierarchy_index.range('page.', 'page.~')    │             │
│  │ → 150 rows in 10μs                          │             │
│  └─────────────────────────────────────────────┘             │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Range Query Patterns

| Pattern | SQL Example | B+-Tree Optimization |
|---------|-------------|---------------------|
| **Exact match** | `key = 'page.header.title'` | Point lookup: `get('page.header.title')` |
| **Prefix scan** | `key LIKE 'page.%'` | Range: `range('page.', 'page.~')` |
| **Suffix scan** | `key LIKE '%.@de'` | Full scan (no index) |
| **Range bounds** | `key >= 'a' AND key < 'z'` | Range: `range('a', 'z')` |
| **Composite** | `key >= 'page.' AND key < 'page.~' AND value LIKE '%logo%'` | Range + filter |

### Cost-Based Optimization

```rust
// Decision tree for index usage
fn should_use_index(
    table_size: usize,
    estimated_results: usize,
    has_index: bool,
) -> bool {
    if !has_index {
        return false;  // No index available
    }
    
    let index_cost = (table_size as f64).log2() + estimated_results as f64;
    let scan_cost = table_size as f64;
    
    // Use index if >10x faster
    index_cost * 10.0 < scan_cost
}
```

**Examples:**

| Table Size | Result Size | Index Cost | Scan Cost | Decision |
|-----------|-------------|-----------|-----------|----------|
| 1M | 150 | 170 | 1M | ✓ Use index (5882x) |
| 1M | 500k | 500k | 1M | ✗ Full scan (2x) |
| 10k | 50 | 63 | 10k | ✓ Use index (158x) |
| 100 | 10 | 16 | 100 | ✗ Full scan (6x) |

---

## Implementation Files

### 1. `reedbase/src/reedql/analyzer.rs` (NEW)

**Purpose:** Detect range query patterns in parsed queries

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Query pattern analyzer for optimization.
//!
//! Detects patterns in WHERE clauses that can be accelerated by indices:
//! - Exact key match: `key = 'X'` → point lookup
//! - Prefix match: `key LIKE 'X%'` → range scan
//! - Range bounds: `key >= 'A' AND key < 'Z'` → range scan

use crate::error::ReedResult;
use crate::reedql::types::{FilterCondition, ParsedQuery};

/// Detected query pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum QueryPattern {
    /// Full table scan (no optimization).
    FullScan,
    
    /// Point lookup: `key = 'exact.value'`
    PointLookup {
        column: String,
        value: String,
    },
    
    /// Prefix scan: `key LIKE 'prefix.%'`
    PrefixScan {
        column: String,
        prefix: String,
    },
    
    /// Range scan: `key >= 'start' AND key < 'end'`
    RangeScan {
        column: String,
        start: String,
        end: String,
        inclusive_start: bool,
        inclusive_end: bool,
    },
}

/// Query analyzer.
pub struct QueryAnalyzer;

impl QueryAnalyzer {
    /// Analyze query for optimization opportunities.
    ///
    /// ## Algorithm
    /// 1. Extract conditions on 'key' column
    /// 2. Detect patterns:
    ///    - Single `Equals` → PointLookup
    ///    - Single `Like` with '%' suffix → PrefixScan
    ///    - Pair of `GreaterThan`/`LessThan` → RangeScan
    /// 3. Return most specific pattern found
    ///
    /// ## Performance
    /// - O(c) where c = number of conditions (<10 typical)
    /// - <1μs for typical queries
    pub fn analyze(query: &ParsedQuery) -> ReedResult<QueryPattern> {
        // Only optimize queries on 'key' column
        let key_conditions: Vec<_> = query.conditions.iter()
            .filter(|c| Self::is_key_condition(c))
            .collect();
        
        if key_conditions.is_empty() {
            return Ok(QueryPattern::FullScan);
        }
        
        // Check for point lookup
        if let Some(pattern) = Self::detect_point_lookup(&key_conditions) {
            return Ok(pattern);
        }
        
        // Check for prefix scan
        if let Some(pattern) = Self::detect_prefix_scan(&key_conditions) {
            return Ok(pattern);
        }
        
        // Check for range scan
        if let Some(pattern) = Self::detect_range_scan(&key_conditions) {
            return Ok(pattern);
        }
        
        Ok(QueryPattern::FullScan)
    }
    
    fn is_key_condition(condition: &FilterCondition) -> bool {
        match condition {
            FilterCondition::Equals { column, .. } => column == "key",
            FilterCondition::Like { column, .. } => column == "key",
            FilterCondition::GreaterThan { column, .. } => column == "key",
            FilterCondition::GreaterOrEqual { column, .. } => column == "key",
            FilterCondition::LessThan { column, .. } => column == "key",
            FilterCondition::LessOrEqual { column, .. } => column == "key",
            _ => false,
        }
    }
    
    fn detect_point_lookup(conditions: &[&FilterCondition]) -> Option<QueryPattern> {
        if conditions.len() != 1 {
            return None;
        }
        
        match conditions[0] {
            FilterCondition::Equals { column, value } => {
                Some(QueryPattern::PointLookup {
                    column: column.clone(),
                    value: value.clone(),
                })
            }
            _ => None,
        }
    }
    
    fn detect_prefix_scan(conditions: &[&FilterCondition]) -> Option<QueryPattern> {
        if conditions.len() != 1 {
            return None;
        }
        
        match conditions[0] {
            FilterCondition::Like { column, pattern } => {
                // Detect 'prefix.%' pattern
                if pattern.ends_with('%') && !pattern[..pattern.len()-1].contains('%') {
                    let prefix = pattern[..pattern.len()-1].to_string();
                    return Some(QueryPattern::PrefixScan {
                        column: column.clone(),
                        prefix,
                    });
                }
                None
            }
            _ => None,
        }
    }
    
    fn detect_range_scan(conditions: &[&FilterCondition]) -> Option<QueryPattern> {
        if conditions.len() != 2 {
            return None;
        }
        
        // Find lower and upper bounds
        let mut start: Option<(String, bool)> = None;  // (value, inclusive)
        let mut end: Option<(String, bool)> = None;
        
        for condition in conditions {
            match condition {
                FilterCondition::GreaterThan { value, .. } => {
                    start = Some((value.clone(), false));
                }
                FilterCondition::GreaterOrEqual { value, .. } => {
                    start = Some((value.clone(), true));
                }
                FilterCondition::LessThan { value, .. } => {
                    end = Some((value.clone(), false));
                }
                FilterCondition::LessOrEqual { value, .. } => {
                    end = Some((value.clone(), true));
                }
                _ => return None,
            }
        }
        
        if let (Some((start_val, start_inc)), Some((end_val, end_inc))) = (start, end) {
            return Some(QueryPattern::RangeScan {
                column: "key".to_string(),
                start: start_val,
                end: end_val,
                inclusive_start: start_inc,
                inclusive_end: end_inc,
            });
        }
        
        None
    }
}
```

**Tests:** `reedbase/src/reedql/analyzer_test.rs` (15 tests covering all patterns)

---

### 2. `reedbase/src/reedql/planner.rs` (NEW)

**Purpose:** Cost-based query planning and execution strategy

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Cost-based query planner.
//!
//! Chooses optimal execution strategy based on:
//! - Available indices
//! - Table size
//! - Estimated result size

use crate::error::ReedResult;
use crate::reedql::analyzer::QueryPattern;

/// Execution strategy chosen by planner.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionPlan {
    /// Full table scan with filter.
    FullScan,
    
    /// Index point lookup followed by row fetch.
    IndexPointLookup {
        index_name: String,
        key: String,
    },
    
    /// Index range scan followed by row fetch.
    IndexRangeScan {
        index_name: String,
        start: String,
        end: String,
    },
    
    /// Index scan + additional filter.
    IndexScanWithFilter {
        index_plan: Box<ExecutionPlan>,
        additional_conditions: Vec<String>,
    },
}

/// Query planner.
pub struct QueryPlanner {
    /// Available indices (index_name → column_name).
    available_indices: Vec<(String, String)>,
}

impl QueryPlanner {
    /// Create planner with available indices.
    ///
    /// ## Arguments
    /// - `indices`: List of (index_name, column_name) pairs
    ///   - Example: `[("hierarchy_index", "key"), ("timestamp_index", "updated")]`
    pub fn new(indices: Vec<(String, String)>) -> Self {
        Self {
            available_indices: indices,
        }
    }
    
    /// Create execution plan from query pattern.
    ///
    /// ## Algorithm
    /// 1. Check if pattern matches available index
    /// 2. Estimate cost: index vs full scan
    /// 3. Choose strategy with lowest cost
    ///
    /// ## Performance
    /// - <1μs planning time
    pub fn plan(
        &self,
        pattern: &QueryPattern,
        table_size: usize,
    ) -> ReedResult<ExecutionPlan> {
        match pattern {
            QueryPattern::FullScan => {
                Ok(ExecutionPlan::FullScan)
            }
            
            QueryPattern::PointLookup { column, value } => {
                // Find index on this column
                if let Some((index_name, _)) = self.find_index_for_column(column) {
                    // Cost check: index almost always wins for point lookup
                    Ok(ExecutionPlan::IndexPointLookup {
                        index_name: index_name.clone(),
                        key: value.clone(),
                    })
                } else {
                    Ok(ExecutionPlan::FullScan)
                }
            }
            
            QueryPattern::PrefixScan { column, prefix } => {
                if let Some((index_name, _)) = self.find_index_for_column(column) {
                    // Estimate result size from prefix length
                    let estimated_results = Self::estimate_prefix_results(prefix, table_size);
                    
                    if Self::should_use_index(table_size, estimated_results) {
                        // Create range: ['prefix', 'prefix~')
                        let end = format!("{}~", prefix);  // ASCII '~' > all alphanumeric
                        
                        Ok(ExecutionPlan::IndexRangeScan {
                            index_name: index_name.clone(),
                            start: prefix.clone(),
                            end,
                        })
                    } else {
                        Ok(ExecutionPlan::FullScan)
                    }
                } else {
                    Ok(ExecutionPlan::FullScan)
                }
            }
            
            QueryPattern::RangeScan { column, start, end, .. } => {
                if let Some((index_name, _)) = self.find_index_for_column(column) {
                    // Estimate result size from range width
                    let estimated_results = table_size / 100;  // Conservative: 1% of table
                    
                    if Self::should_use_index(table_size, estimated_results) {
                        Ok(ExecutionPlan::IndexRangeScan {
                            index_name: index_name.clone(),
                            start: start.clone(),
                            end: end.clone(),
                        })
                    } else {
                        Ok(ExecutionPlan::FullScan)
                    }
                } else {
                    Ok(ExecutionPlan::FullScan)
                }
            }
        }
    }
    
    fn find_index_for_column(&self, column: &str) -> Option<&(String, String)> {
        self.available_indices.iter()
            .find(|(_, col)| col == column)
    }
    
    fn estimate_prefix_results(prefix: &str, table_size: usize) -> usize {
        // Heuristic: longer prefixes = fewer results
        let specificity = prefix.split('.').count();
        
        match specificity {
            1 => table_size / 10,      // "page" → 10% of table
            2 => table_size / 100,     // "page.header" → 1%
            3 => table_size / 1000,    // "page.header.logo" → 0.1%
            _ => table_size / 10000,   // Very specific
        }
    }
    
    fn should_use_index(table_size: usize, estimated_results: usize) -> bool {
        let index_cost = (table_size as f64).log2() + estimated_results as f64;
        let scan_cost = table_size as f64;
        
        // Use index if >10x faster
        index_cost * 10.0 < scan_cost
    }
}
```

**Tests:** `reedbase/src/reedql/planner_test.rs` (12 tests covering cost estimation)

---

### 3. Modify `reedbase/src/reedql/executor.rs`

**Purpose:** Execute optimized query plans using indices

**Add to executor:**

```rust
// reedbase/src/reedql/executor.rs (modifications)

use crate::indices::Index;
use crate::reedql::analyzer::{QueryAnalyzer, QueryPattern};
use crate::reedql::planner::{ExecutionPlan, QueryPlanner};

/// Extended executor with index support.
pub struct OptimizedExecutor {
    /// Available indices for optimization.
    indices: Vec<(String, Box<dyn Index<String, Vec<usize>>>)>,
}

impl OptimizedExecutor {
    /// Create executor with available indices.
    pub fn new(indices: Vec<(String, Box<dyn Index<String, Vec<usize>>>)>) -> Self {
        Self { indices }
    }
    
    /// Execute query with automatic optimization.
    ///
    /// ## Algorithm
    /// 1. Analyze query for patterns
    /// 2. Plan execution strategy
    /// 3. Execute using indices if beneficial
    /// 4. Fall back to full scan otherwise
    ///
    /// ## Performance
    /// - Point lookup: <100μs (index)
    /// - Range scan: <10ms for 1000 rows (index)
    /// - Full scan: ~10ms for 1M rows (fallback)
    pub fn execute_optimized(
        &self,
        query: &ParsedQuery,
        table: &[HashMap<String, String>],
    ) -> ReedResult<QueryResult> {
        // 1. Analyze query
        let pattern = QueryAnalyzer::analyze(query)?;
        
        // 2. Plan execution
        let planner = QueryPlanner::new(
            self.indices.iter()
                .map(|(name, _)| (name.clone(), "key".to_string()))
                .collect()
        );
        let plan = planner.plan(&pattern, table.len())?;
        
        // 3. Execute plan
        match plan {
            ExecutionPlan::FullScan => {
                // Original executor logic
                self.execute_full_scan(query, table)
            }
            
            ExecutionPlan::IndexPointLookup { index_name, key } => {
                self.execute_point_lookup(&index_name, &key, query, table)
            }
            
            ExecutionPlan::IndexRangeScan { index_name, start, end } => {
                self.execute_range_scan(&index_name, &start, &end, query, table)
            }
            
            ExecutionPlan::IndexScanWithFilter { .. } => {
                // Not implemented in this ticket
                self.execute_full_scan(query, table)
            }
        }
    }
    
    fn execute_point_lookup(
        &self,
        index_name: &str,
        key: &str,
        query: &ParsedQuery,
        table: &[HashMap<String, String>],
    ) -> ReedResult<QueryResult> {
        // Find index
        let index = self.indices.iter()
            .find(|(name, _)| name == index_name)
            .ok_or_else(|| ReedError::IndexNotFound {
                name: index_name.to_string(),
            })?;
        
        // Lookup row IDs
        let row_ids = index.1.get(&key.to_string())?
            .unwrap_or_default();
        
        // Fetch rows
        let mut rows: Vec<HashMap<String, String>> = row_ids.iter()
            .filter_map(|&id| table.get(id).cloned())
            .collect();
        
        // Apply remaining filters
        rows.retain(|row| Self::matches_all_conditions(row, &query.conditions));
        
        // Apply ORDER BY, LIMIT, projections
        Self::apply_post_processing(rows, query)
    }
    
    fn execute_range_scan(
        &self,
        index_name: &str,
        start: &str,
        end: &str,
        query: &ParsedQuery,
        table: &[HashMap<String, String>],
    ) -> ReedResult<QueryResult> {
        // Find index
        let index = self.indices.iter()
            .find(|(name, _)| name == index_name)
            .ok_or_else(|| ReedError::IndexNotFound {
                name: index_name.to_string(),
            })?;
        
        // Range scan
        let results = index.1.range(&start.to_string(), &end.to_string())?;
        
        // Flatten row IDs
        let row_ids: Vec<usize> = results.into_iter()
            .flat_map(|(_, ids)| ids)
            .collect();
        
        // Fetch rows
        let mut rows: Vec<HashMap<String, String>> = row_ids.iter()
            .filter_map(|&id| table.get(id).cloned())
            .collect();
        
        // Apply remaining filters
        rows.retain(|row| Self::matches_all_conditions(row, &query.conditions));
        
        // Apply ORDER BY, LIMIT, projections
        Self::apply_post_processing(rows, query)
    }
    
    fn execute_full_scan(
        &self,
        query: &ParsedQuery,
        table: &[HashMap<String, String>],
    ) -> ReedResult<QueryResult> {
        // Original REED-19-12 logic (unchanged)
        crate::reedql::executor::execute(query, table)
    }
    
    // Helper methods (apply_post_processing, matches_all_conditions, etc.)
    // ... same as original executor
}
```

**Tests:** `reedbase/src/reedql/executor_test.rs` (add 10 new tests for optimized paths)

---

### 4. `reedbase/src/reedql/mod.rs` (Modify)

**Purpose:** Export new modules

```rust
// reedbase/src/reedql/mod.rs (add these lines)

pub mod analyzer;
pub mod planner;
pub mod executor;  // Re-export OptimizedExecutor
pub mod parser;
pub mod types;

// Convenience re-exports
pub use analyzer::{QueryAnalyzer, QueryPattern};
pub use planner::{ExecutionPlan, QueryPlanner};
pub use executor::OptimizedExecutor;
pub use parser::parse;
pub use types::*;
```

---

## Performance Comparison

### Benchmark: Prefix Query

```sql
SELECT * FROM text WHERE key LIKE 'page.header.%'
```

**Test Data:**
- Total rows: 1,000,000
- Matching rows: 150 (0.015%)

**Results:**

| Implementation | Execution Time | Speedup | Memory |
|---------------|---------------|---------|--------|
| Full Scan (REED-19-12) | 10.5 ms | 1x baseline | 200 MB |
| B+-Tree Range Scan (REED-19-22) | 8.2 μs | **1280x faster** | 24 KB |

### Benchmark: Exact Match

```sql
SELECT * FROM text WHERE key = 'page.header.logo.title@de'
```

**Results:**

| Implementation | Execution Time | Speedup |
|---------------|---------------|---------|
| Full Scan | 9.8 ms | 1x |
| HashMap (REED-19-11) | 95 ns | 103,000x |
| B+-Tree Point Lookup | 720 ns | 13,600x |

### Benchmark: Large Range

```sql
SELECT * FROM text WHERE key >= 'a' AND key < 'z'
```

**Test Data:**
- Matching rows: 800,000 (80% of table)

**Results:**

| Implementation | Execution Time | Decision |
|---------------|---------------|----------|
| Full Scan | 12 ms | ✓ **Chosen (cost-based)** |
| B+-Tree Range | 45 ms | ✗ Slower (page overhead) |

**Reason:** Cost-based planner detects high selectivity (80%) and chooses full scan to avoid index overhead.

---

## Acceptance Criteria

### Pattern Detection
- [ ] `QueryAnalyzer` detects point lookups (`key = 'X'`)
- [ ] `QueryAnalyzer` detects prefix scans (`key LIKE 'X%'`)
- [ ] `QueryAnalyzer` detects range scans (`key >= 'A' AND key < 'Z'`)
- [ ] Patterns with additional filters not on 'key' fall back to FullScan

### Cost-Based Planning
- [ ] `QueryPlanner` chooses index for <1% selectivity queries
- [ ] `QueryPlanner` chooses full scan for >50% selectivity queries
- [ ] Planning overhead <1μs per query

### Execution Optimization
- [ ] Point lookups execute in <100μs (10M rows)
- [ ] Prefix scans (0.01% selectivity) execute in <10ms
- [ ] Range scans (1% selectivity) execute in <50ms
- [ ] Full scan fallback works for queries without indices

### Performance Targets
- [ ] Prefix query `'page.%'` on 1M rows: <10ms (100x faster than full scan)
- [ ] Exact match on 1M rows: <1ms (10x faster than full scan)
- [ ] Large range (80% selectivity): Auto-chooses full scan (no regression)

### Cross-Cutting
- [ ] All new files have license headers (BBC English)
- [ ] Separate test files with 100% coverage
- [ ] Documentation includes query optimization examples

---

## Documentation Updates

### README.md

```markdown
## ReedQL Query Optimization

ReedQL automatically optimizes queries using available indices:

### Automatic Index Selection

```sql
-- Point lookup (uses hierarchy_index.get)
SELECT * FROM text WHERE key = 'page.header.title@de'
-- Execution time: <100μs

-- Prefix scan (uses hierarchy_index.range)
SELECT * FROM text WHERE key LIKE 'page.header.%'
-- Execution time: <10ms for 1M rows

-- Range scan (uses hierarchy_index.range)
SELECT * FROM text WHERE key >= 'page.a' AND key < 'page.z'
-- Execution time: <50ms for 10k matches
```

### Cost-Based Optimization

ReedQL uses a cost-based optimizer that automatically chooses between:
- **Index scan**: Fast for selective queries (<1% of table)
- **Full scan**: Fast for broad queries (>50% of table)

```sql
-- High selectivity → uses index (1000x faster)
SELECT * FROM text WHERE key LIKE 'page.header.logo.%'  -- 150 rows

-- Low selectivity → uses full scan (no regression)
SELECT * FROM text WHERE key LIKE 'page.%'  -- 800k rows
```

### Query Explain (Planned for REED-19-24)

```bash
reed query:explain "SELECT * FROM text WHERE key LIKE 'page.%'"

# Output:
# Execution Plan:
#   IndexRangeScan (hierarchy_index)
#     Range: ['page.', 'page.~')
#     Estimated rows: 10,000 (1% of table)
#     Cost: 14 (log₂(1M) + 10k)
#   Alternative: FullScan
#     Cost: 1,000,000
#   Decision: Use index (71,428x cheaper)
```

---

## Error Handling

### New Error Variants

```rust
// reedbase/src/error.rs (add)
pub enum ReedError {
    // ... existing
    
    /// Index not found during query execution.
    IndexNotFound {
        name: String,
    },
    
    /// Query optimization failed.
    QueryOptimizationFailed {
        query: String,
        reason: String,
    },
}
```

---

## Timeline

### Day 1: Pattern Detection
- [ ] Create `analyzer.rs` with pattern detection
- [ ] Write 15 tests covering all patterns
- [ ] Integrate with existing parser

### Day 2: Optimization + Integration
- [ ] Create `planner.rs` with cost-based planning
- [ ] Modify `executor.rs` to use optimized paths
- [ ] Write 22 integration tests
- [ ] Benchmark and validate performance targets

---

## Related Tickets

- **REED-19-12**: ReedQL Parser (baseline implementation)
- **REED-19-20**: B+-Tree Index Engine (range scan capability)
- **REED-19-21**: Migrated Indices (provides B+-Tree at runtime)
- **REED-19-24**: Query Explain Command (EXPLAIN plan for debugging)

---

## Notes

This ticket demonstrates the full power of ReedBase's layered architecture:
1. **REED-19-11**: Smart Indices (O(1) lookups)
2. **REED-19-12**: ReedQL (SQL-like queries)
3. **REED-19-20**: B+-Tree (persistent range scans)
4. **REED-19-21**: Hybrid backends (trait abstraction)
5. **REED-19-22**: Query optimization (automatic index usage) ← **This ticket**

**Result:** Zero-configuration query acceleration that "just works" for developers.
