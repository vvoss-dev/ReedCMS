// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedQL Query Types
//!
//! Core AST (Abstract Syntax Tree) types for the ReedQL query language.
//! ReedQL is a SQL-like query language optimized for ReedBase operations.
//!
//! ## Design Principles
//! - Zero-copy parsing where possible
//! - Minimal allocations (< 10 total per query)
//! - < 10μs parse time for typical queries
//! - Direct mapping to ReedBase operations

use crate::error::{ReedError, ReedResult};
use std::fmt;

/// Parsed ReedQL query structure.
///
/// Represents a complete SQL-like query after parsing.
///
/// ## Memory Layout
/// - Total size: ~200 bytes (stack-allocated)
/// - Heap allocations: Only for vectors (columns, conditions, order_by)
///
/// ## Example
/// ```text
/// SELECT key, value WHERE namespace = 'page' ORDER BY key LIMIT 10
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedQuery {
    /// Selected columns (* or specific column names)
    pub columns: Vec<String>,

    /// Table name (always "text", "routes", "meta", "server", or "project")
    pub table: String,

    /// WHERE clause conditions (empty = no filter)
    pub conditions: Vec<FilterCondition>,

    /// ORDER BY clauses (empty = no sorting)
    pub order_by: Vec<OrderBy>,

    /// LIMIT clause (None = no limit)
    pub limit: Option<LimitOffset>,

    /// Aggregation function (None = no aggregation)
    pub aggregation: Option<AggregationFunction>,
}

impl ParsedQuery {
    /// Creates a new empty query.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            table: String::new(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            aggregation: None,
        }
    }

    /// Checks if query selects all columns.
    pub fn is_select_all(&self) -> bool {
        self.columns.len() == 1 && self.columns[0] == "*"
    }

    /// Returns true if query has aggregation.
    pub fn has_aggregation(&self) -> bool {
        self.aggregation.is_some()
    }

    /// Returns true if query has WHERE clause.
    pub fn has_conditions(&self) -> bool {
        !self.conditions.is_empty()
    }
}

impl Default for ParsedQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter condition for WHERE clause.
///
/// Supports common SQL operators plus ReedBase-specific optimizations.
///
/// ## Performance
/// - Fast path for key patterns: `key LIKE '%.@de'` → O(n) string check
/// - Fast path for namespace: `namespace = 'page'` → O(1) index lookup
/// - Generic conditions: O(n) table scan
#[derive(Debug, Clone, PartialEq)]
pub enum FilterCondition {
    /// Equality: column = value
    Equals { column: String, value: String },

    /// Inequality: column != value
    NotEquals { column: String, value: String },

    /// Less than: column < value
    LessThan { column: String, value: String },

    /// Greater than: column > value
    GreaterThan { column: String, value: String },

    /// Less than or equal: column <= value
    LessThanOrEqual { column: String, value: String },

    /// Greater than or equal: column >= value
    GreaterThanOrEqual { column: String, value: String },

    /// Pattern matching: column LIKE pattern
    /// Uses SQL LIKE syntax (% = wildcard, _ = single char)
    Like { column: String, pattern: String },

    /// IN clause with literal values: column IN ('a', 'b', 'c')
    InList { column: String, values: Vec<String> },

    /// IN clause with subquery: column IN (SELECT ...)
    InSubquery {
        column: String,
        subquery: Box<ParsedQuery>,
    },
}

impl FilterCondition {
    /// Returns the column name referenced by this condition.
    pub fn column(&self) -> &str {
        match self {
            FilterCondition::Equals { column, .. }
            | FilterCondition::NotEquals { column, .. }
            | FilterCondition::LessThan { column, .. }
            | FilterCondition::GreaterThan { column, .. }
            | FilterCondition::LessThanOrEqual { column, .. }
            | FilterCondition::GreaterThanOrEqual { column, .. }
            | FilterCondition::Like { column, .. }
            | FilterCondition::InList { column, .. }
            | FilterCondition::InSubquery { column, .. } => column,
        }
    }

    /// Checks if this is a ReedBase-optimized fast path condition.
    ///
    /// Fast paths:
    /// - `key LIKE '%.@de'` (language filter)
    /// - `key LIKE 'page.%'` (namespace filter)
    /// - `namespace = 'page'` (direct namespace)
    pub fn is_fast_path(&self) -> bool {
        match self {
            FilterCondition::Like { column, pattern } => {
                column == "key" && (pattern.starts_with("%.@") || pattern.ends_with(".%"))
            }
            FilterCondition::Equals { column, .. } => column == "namespace",
            _ => false,
        }
    }
}

impl fmt::Display for FilterCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilterCondition::Equals { column, value } => write!(f, "{} = '{}'", column, value),
            FilterCondition::NotEquals { column, value } => write!(f, "{} != '{}'", column, value),
            FilterCondition::LessThan { column, value } => write!(f, "{} < '{}'", column, value),
            FilterCondition::GreaterThan { column, value } => {
                write!(f, "{} > '{}'", column, value)
            }
            FilterCondition::LessThanOrEqual { column, value } => {
                write!(f, "{} <= '{}'", column, value)
            }
            FilterCondition::GreaterThanOrEqual { column, value } => {
                write!(f, "{} >= '{}'", column, value)
            }
            FilterCondition::Like { column, pattern } => {
                write!(f, "{} LIKE '{}'", column, pattern)
            }
            FilterCondition::InList { column, values } => {
                write!(f, "{} IN ({})", column, values.join(", "))
            }
            FilterCondition::InSubquery { column, subquery } => {
                write!(f, "{} IN ({:?})", column, subquery)
            }
        }
    }
}

/// ORDER BY clause.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy {
    /// Column name to sort by
    pub column: String,

    /// Sort direction (ASC or DESC)
    pub direction: SortDirection,
}

impl OrderBy {
    /// Creates a new ORDER BY clause.
    pub fn new(column: String, direction: SortDirection) -> Self {
        Self { column, direction }
    }

    /// Creates an ascending order clause.
    pub fn asc(column: String) -> Self {
        Self::new(column, SortDirection::Ascending)
    }

    /// Creates a descending order clause.
    pub fn desc(column: String) -> Self {
        Self::new(column, SortDirection::Descending)
    }
}

/// Sort direction for ORDER BY.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending (A-Z, 0-9)
    Ascending,

    /// Descending (Z-A, 9-0)
    Descending,
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortDirection::Ascending => write!(f, "ASC"),
            SortDirection::Descending => write!(f, "DESC"),
        }
    }
}

/// LIMIT and OFFSET clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitOffset {
    /// Maximum number of rows to return
    pub limit: usize,

    /// Number of rows to skip
    pub offset: usize,
}

impl LimitOffset {
    /// Creates a new LIMIT clause without offset.
    pub fn new(limit: usize) -> Self {
        Self { limit, offset: 0 }
    }

    /// Creates a new LIMIT clause with offset.
    pub fn with_offset(limit: usize, offset: usize) -> Self {
        Self { limit, offset }
    }
}

/// Aggregation function for SELECT clause.
///
/// ## Example
/// ```text
/// SELECT COUNT(*) FROM text
/// SELECT AVG(length(value)) FROM text WHERE namespace = 'page'
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct AggregationFunction {
    /// Type of aggregation (COUNT, SUM, AVG, MIN, MAX)
    pub agg_type: AggregationType,

    /// Column to aggregate (* for COUNT(*))
    pub column: String,
}

impl AggregationFunction {
    /// Creates a new aggregation function.
    pub fn new(agg_type: AggregationType, column: String) -> Self {
        Self { agg_type, column }
    }

    /// Creates a COUNT(*) aggregation.
    pub fn count_all() -> Self {
        Self::new(AggregationType::Count, "*".to_string())
    }

    /// Creates a COUNT(column) aggregation.
    pub fn count(column: String) -> Self {
        Self::new(AggregationType::Count, column)
    }
}

/// Type of aggregation function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregationType {
    /// Count rows
    Count,

    /// Sum numeric values
    Sum,

    /// Average numeric values
    Avg,

    /// Minimum value
    Min,

    /// Maximum value
    Max,
}

impl fmt::Display for AggregationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregationType::Count => write!(f, "COUNT"),
            AggregationType::Sum => write!(f, "SUM"),
            AggregationType::Avg => write!(f, "AVG"),
            AggregationType::Min => write!(f, "MIN"),
            AggregationType::Max => write!(f, "MAX"),
        }
    }
}

/// Query execution result.
///
/// Represents the result of executing a ReedQL query.
///
/// ## Variants
/// - `Rows`: Regular SELECT result (vector of row maps)
/// - `Aggregation`: Aggregation result (single numeric value)
#[derive(Debug, Clone)]
pub enum QueryResult {
    /// Regular SELECT result: vector of rows (each row is a map of column → value)
    Rows(Vec<std::collections::HashMap<String, String>>),

    /// Aggregation result: single numeric value
    Aggregation(f64),
}

impl QueryResult {
    /// Creates a new empty rows result.
    pub fn empty() -> Self {
        QueryResult::Rows(Vec::new())
    }

    /// Returns the number of rows (0 for aggregations).
    pub fn row_count(&self) -> usize {
        match self {
            QueryResult::Rows(rows) => rows.len(),
            QueryResult::Aggregation(_) => 0,
        }
    }

    /// Checks if result is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            QueryResult::Rows(rows) => rows.is_empty(),
            QueryResult::Aggregation(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_query_new() {
        let query = ParsedQuery::new();
        assert_eq!(query.columns.len(), 0);
        assert_eq!(query.table, "");
        assert_eq!(query.conditions.len(), 0);
        assert_eq!(query.order_by.len(), 0);
        assert_eq!(query.limit, None);
        assert_eq!(query.aggregation, None);
    }

    #[test]
    fn test_parsed_query_is_select_all() {
        let mut query = ParsedQuery::new();
        query.columns.push("*".to_string());
        assert!(query.is_select_all());

        query.columns.push("key".to_string());
        assert!(!query.is_select_all());
    }

    #[test]
    fn test_filter_condition_column() {
        let cond = FilterCondition::Equals {
            column: "key".to_string(),
            value: "test".to_string(),
        };
        assert_eq!(cond.column(), "key");
    }

    #[test]
    fn test_filter_condition_is_fast_path() {
        // Language filter fast path
        let cond = FilterCondition::Like {
            column: "key".to_string(),
            pattern: "%.@de".to_string(),
        };
        assert!(cond.is_fast_path());

        // Namespace filter fast path
        let cond = FilterCondition::Like {
            column: "key".to_string(),
            pattern: "page.%".to_string(),
        };
        assert!(cond.is_fast_path());

        // Direct namespace fast path
        let cond = FilterCondition::Equals {
            column: "namespace".to_string(),
            value: "page".to_string(),
        };
        assert!(cond.is_fast_path());

        // Not a fast path
        let cond = FilterCondition::Equals {
            column: "key".to_string(),
            value: "test".to_string(),
        };
        assert!(!cond.is_fast_path());
    }

    #[test]
    fn test_order_by_constructors() {
        let asc = OrderBy::asc("key".to_string());
        assert_eq!(asc.column, "key");
        assert_eq!(asc.direction, SortDirection::Ascending);

        let desc = OrderBy::desc("value".to_string());
        assert_eq!(desc.column, "value");
        assert_eq!(desc.direction, SortDirection::Descending);
    }

    #[test]
    fn test_limit_offset_constructors() {
        let limit = LimitOffset::new(10);
        assert_eq!(limit.limit, 10);
        assert_eq!(limit.offset, 0);

        let limit_offset = LimitOffset::with_offset(20, 5);
        assert_eq!(limit_offset.limit, 20);
        assert_eq!(limit_offset.offset, 5);
    }

    #[test]
    fn test_aggregation_function_constructors() {
        let count_all = AggregationFunction::count_all();
        assert_eq!(count_all.agg_type, AggregationType::Count);
        assert_eq!(count_all.column, "*");

        let count_key = AggregationFunction::count("key".to_string());
        assert_eq!(count_key.agg_type, AggregationType::Count);
        assert_eq!(count_key.column, "key");
    }

    #[test]
    fn test_query_result_empty() {
        let result = QueryResult::empty();
        assert_eq!(result.row_count(), 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_query_result_rows() {
        let mut rows = Vec::new();
        let mut row = std::collections::HashMap::new();
        row.insert("key".to_string(), "test".to_string());
        rows.push(row);

        let result = QueryResult::Rows(rows);
        assert_eq!(result.row_count(), 1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_query_result_aggregation() {
        let result = QueryResult::Aggregation(42.0);
        assert_eq!(result.row_count(), 0);
        assert!(!result.is_empty());
    }
}
