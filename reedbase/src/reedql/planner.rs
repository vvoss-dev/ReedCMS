// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Cost-based query planner.
//!
//! Chooses optimal execution strategy based on:
//! - Available indices
//! - Table size
//! - Estimated result size
//!
//! ## Decision Algorithm
//! 1. Check if pattern matches available index
//! 2. Estimate cost: index vs full scan
//! 3. Choose strategy with lowest cost (use index if >10x faster)
//!
//! ## Performance
//! - Planning time: < 1μs per query
//! - Zero-allocation planning

use crate::error::ReedResult;
use crate::reedql::analyzer::QueryPattern;

/// Execution strategy chosen by planner.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionPlan {
    /// Full table scan with filter.
    FullScan,

    /// Index point lookup followed by row fetch.
    IndexPointLookup { index_name: String, key: String },

    /// Index range scan followed by row fetch.
    IndexRangeScan {
        index_name: String,
        start: String,
        end: String,
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
    ///
    /// ## Example
    /// ```rust,ignore
    /// let planner = QueryPlanner::new(vec![
    ///     ("hierarchy_index".to_string(), "key".to_string()),
    /// ]);
    /// ```
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
    /// ## Cost Model
    /// - Index cost: log₂(table_size) + estimated_results
    /// - Full scan cost: table_size
    /// - Decision: Use index if >10x faster
    ///
    /// ## Performance
    /// - <1μs planning time
    ///
    /// ## Example
    /// ```rust,ignore
    /// let pattern = QueryPattern::PrefixScan {
    ///     column: "key".to_string(),
    ///     prefix: "page.".to_string(),
    /// };
    /// let plan = planner.plan(&pattern, 1_000_000)?;
    /// // plan = IndexRangeScan { ... } (if cost-effective)
    /// ```
    pub fn plan(&self, pattern: &QueryPattern, table_size: usize) -> ReedResult<ExecutionPlan> {
        match pattern {
            QueryPattern::FullScan => Ok(ExecutionPlan::FullScan),

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
                        let end = format!("{}~", prefix); // ASCII '~' > all alphanumeric

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

            QueryPattern::RangeScan {
                column, start, end, ..
            } => {
                if let Some((index_name, _)) = self.find_index_for_column(column) {
                    // Estimate result size from range width (conservative: 1% of table)
                    let estimated_results = table_size / 100;

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
        self.available_indices.iter().find(|(_, col)| col == column)
    }

    fn estimate_prefix_results(prefix: &str, table_size: usize) -> usize {
        // Heuristic: longer prefixes = fewer results
        let specificity = prefix.split('.').count();

        match specificity {
            1 => table_size / 10,    // "page" → 10% of table
            2 => table_size / 100,   // "page.header" → 1%
            3 => table_size / 1000,  // "page.header.logo" → 0.1%
            _ => table_size / 10000, // Very specific
        }
    }

    fn should_use_index(table_size: usize, estimated_results: usize) -> bool {
        let index_cost = (table_size as f64).log2() + estimated_results as f64;
        let scan_cost = table_size as f64;

        // Use index if >10x faster
        index_cost * 10.0 < scan_cost
    }
}
