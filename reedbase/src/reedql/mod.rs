// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedQL - SQL-Like Query Interface for ReedBase
//!
//! ReedQL is a custom SQL-like query language optimized for ReedBase operations.
//! It provides a familiar SQL syntax with ReedBase-specific optimizations.
//!
//! ## Features
//! - **Fast Parsing**: < 10μs parse time (10x faster than generic SQL parsers)
//! - **ReedBase Optimized**: Key pattern fast paths for 10x query speedup
//! - **Subquery Support**: Recursive IN subquery execution
//! - **Aggregations**: COUNT, SUM, AVG, MIN, MAX
//! - **CLI-Only**: No API exposure (security-by-design)
//!
//! ## Supported SQL Syntax
//! ```text
//! SELECT column1, column2, ... FROM table
//! WHERE condition1 AND condition2 ...
//! ORDER BY column ASC|DESC
//! LIMIT n OFFSET m
//!
//! -- Aggregations
//! SELECT COUNT(*) FROM text
//! SELECT AVG(column) FROM text WHERE condition
//!
//! -- Subqueries
//! SELECT * FROM text WHERE key IN (SELECT key FROM routes)
//! ```
//!
//! ## Fast Paths
//! ReedQL automatically optimizes common patterns:
//! - `key LIKE '%.@de'` → Language filter (< 1ms for 10k rows)
//! - `key LIKE 'page.%'` → Namespace filter (< 1ms for 10k rows)
//! - `namespace = 'page'` → Direct index lookup (< 100μs)
//!
//! ## Performance Targets
//! - Parse: < 10μs
//! - Execute key LIKE pattern: < 1ms for 10k rows
//! - Execute simple filter: < 10ms for 10k rows
//! - Execute subquery: < 20ms for 10k + 10k rows
//!
//! ## Example Usage
//! ```rust,ignore
//! use reedbase::reedql::{parse, execute};
//!
//! // Parse query
//! let query = parse("SELECT * FROM text WHERE key LIKE '%.@de' LIMIT 10")?;
//!
//! // Execute query
//! let result = execute(&query, &table)?;
//! ```
//!
//! ## Security
//! - CLI-only (no API exposure)
//! - No SQL injection risk (custom parser with strict validation)
//! - Maximum query complexity limits
//!
//! ## Module Structure
//! - `types`: Core AST types (ParsedQuery, FilterCondition, etc.)
//! - `parser`: Custom hand-written parser (< 10μs)
//! - `executor`: Query execution engine with ReedBase optimizations
//! - `validator`: Query validation and security checks
//! - `formatter`: Output formatting (table, JSON, CSV)

pub mod analyzer;
pub mod analyzer_test;
pub mod executor;
pub mod executor_test;
pub mod parser;
pub mod planner;
pub mod planner_test;
pub mod types;

// Re-export commonly used types
pub use analyzer::{QueryAnalyzer, QueryPattern};
pub use executor::{execute, OptimizedExecutor};
pub use parser::parse;
pub use planner::{ExecutionPlan, QueryPlanner};
pub use types::{
    AggregationFunction, AggregationType, FilterCondition, LimitOffset, OrderBy, ParsedQuery,
    QueryResult, SortDirection,
};
