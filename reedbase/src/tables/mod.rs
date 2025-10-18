// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Universal table API for ReedBase.
//!
//! Every table (text, routes, meta, users, etc.) uses identical structure:
//!
//! ```text
//! .reed/tables/{table_name}/
//! ├── current.csv          # Active version
//! ├── {timestamp}.bsdiff   # Binary deltas (XZ compressed)
//! └── version.log          # Encoded metadata
//! ```
//!
//! ## Key Features
//!
//! - **Universal**: Same API for all tables
//! - **Versioned**: Git-like history with binary deltas
//! - **Efficient**: XZ-compressed deltas (95%+ space savings)
//! - **Simple**: Read current, write new, rollback to any version
//!
//! ## Example Usage
//!
//! ```no_run
//! use reedbase::tables::Table;
//! use std::path::Path;
//!
//! // Create table reference
//! let table = Table::new(Path::new(".reed"), "text");
//!
//! // Initialize new table
//! table.init(b"key|value\nfoo|bar\n", "admin")?;
//!
//! // Read current version
//! let content = table.read_current()?;
//!
//! // Write new version
//! table.write(b"key|value\nfoo|baz\n", "admin")?;
//!
//! // List versions
//! let versions = table.list_versions()?;
//! # Ok::<(), reedbase::ReedError>(())
//! ```

pub mod csv_parser;
pub mod helpers;
pub mod table;
pub mod types;

#[cfg(test)]
mod csv_parser_test;
#[cfg(test)]
mod helpers_test;
#[cfg(test)]
mod table_test;

// Re-export public API
pub use csv_parser::{parse_csv, parse_csv_row};
pub use helpers::{list_tables, table_exists, table_stats};
pub use table::Table;
pub use types::{CsvRow, TableStats, VersionInfo, WriteResult};
