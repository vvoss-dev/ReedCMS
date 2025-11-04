// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! High-Level Database API for ReedBase
//!
//! This module provides the primary user-facing API for ReedBase, integrating:
//! - Tables (versioned CSV storage)
//! - ReedQL (SQL-like query language)
//! - Smart Indices (HashMap + B+-Tree)
//! - Auto-indexing (pattern-based optimization)
//!
//! ## Design Philosophy
//!
//! **Users interact with ReedBase ONLY through Database API and ReedQL.**
//!
//! The Database struct is the single entry point for all operations:
//! - Programmatic: `Database::open()` → `db.query()` / `db.execute()`
//! - CLI: `reedbase` command → ReedQCommand interface
//!
//! ## Quick Start
//!
//! ```no_run
//! use reedbase::database::Database;
//!
//! // Open database
//! let db = Database::open(".reed")?;
//!
//! // Query data (SELECT)
//! let result = db.query("SELECT * FROM text WHERE key LIKE '%.@de' LIMIT 10")?;
//!
//! // Modify data (INSERT/UPDATE/DELETE)
//! db.execute("INSERT INTO text (key, value) VALUES ('page.title@de', 'Willkommen')", "admin")?;
//! db.execute("UPDATE text SET value = 'Hallo' WHERE key = 'page.title@de'", "admin")?;
//! db.execute("DELETE FROM text WHERE key = 'page.title@de'", "admin")?;
//!
//! // Create table
//! db.create_table("users", None)?;
//!
//! // Create index (manual - auto-indexing happens automatically)
//! db.create_index("text", "key")?;
//!
//! # Ok::<(), reedbase::ReedError>(())
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Database API (Public)                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │  query()     execute()   create_table()   create_index()    │
//! │     │            │              │               │            │
//! │     ▼            ▼              ▼               ▼            │
//! ├─────────────────────────────────────────────────────────────┤
//! │                  ReedQL Parser & Executor                   │
//! │                    (REED-19-10, 19-12)                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │   Smart Indices          │        Tables                    │
//! │  (REED-19-11, 19-23)     │   (REED-19-01 - 19-07)          │
//! │   - HashMap (O(1))       │   - Versioned CSV                │
//! │   - B+-Tree (O(log n))   │   - Binary Delta                 │
//! │   - Auto-detection       │   - Conflict Resolution          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Performance
//!
//! - **Query (with index)**: < 100μs for exact match, < 1ms for range
//! - **Query (no index)**: ~10ms for 10k rows
//! - **Insert/Update/Delete**: < 5ms typical (includes versioning)
//! - **Auto-indexing detection**: After 10x repeated patterns
//! - **Cold start**: < 100ms (persistent B+-Tree indices)
//!
//! ## Auto-Indexing Strategy
//!
//! The Database automatically creates indices based on query patterns:
//!
//! 1. **Primary key index**: Created on table initialization
//! 2. **Pattern detection**: After 10x queries on same column → auto-create index
//! 3. **Foreign key patterns**: Automatic for `*_id` columns after 3x queries
//! 4. **ReedCMS patterns**: Language (`%.@de`) and namespace (`page.%`) optimized
//!
//! ## Module Structure
//!
//! - `types`: Core types (Database, QueryResult, ExecuteResult, etc.)
//! - `query`: Query execution (SELECT via ReedQL)
//! - `execute`: Command execution (INSERT/UPDATE/DELETE)
//! - `index`: Index management (create, auto-detect, optimize)
//! - `stats`: Statistics and query pattern tracking

pub mod database;
pub mod execute;
pub mod index;
pub mod query;
pub mod stats;
pub mod types;

// Unit tests moved to integration tests in tests/ directory
// #[cfg(test)]
// mod database_test;
// #[cfg(test)]
// mod execute_test;
// #[cfg(test)]
// mod query_test;

// Re-export public API
pub use database::Database;
pub use execute::{ExecuteResult, ExecuteStatement};
pub use index::create_index_internal; // For auto-indexing
pub use query::QueryResultFormatter;
pub use types::{AutoIndexConfig, DatabaseStats, IndexInfo, QueryMetrics};
