// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Conflict resolution module for ReedBase.
//!
//! This module provides types and functions for handling conflicts during
//! concurrent CSV merge operations. It supports multiple resolution strategies
//! and manual conflict file management.
//!
//! ## Key Features
//! - Multiple resolution strategies (LastWriteWins, FirstWriteWins, Manual, KeepBoth)
//! - TOML-based conflict files for human readability
//! - Automatic and manual conflict resolution
//! - Conflict file management (list, load, delete)
//!
//! ## Example Usage
//! ```rust
//! use reedbase::conflict::{resolve_conflict, ResolutionStrategy};
//! use reedbase::concurrent::types::CsvRow;
//! use std::path::Path;
//!
//! let base = CsvRow {
//!     key: "test.key".to_string(),
//!     values: vec!["old".to_string()],
//! };
//! let change_a = CsvRow {
//!     key: "test.key".to_string(),
//!     values: vec!["new_a".to_string()],
//! };
//! let change_b = CsvRow {
//!     key: "test.key".to_string(),
//!     values: vec!["new_b".to_string()],
//! };
//!
//! let resolution = resolve_conflict(
//!     Path::new(".reed"),
//!     "text",
//!     "test.key",
//!     Some(base),
//!     change_a,
//!     change_b,
//!     ResolutionStrategy::LastWriteWins,
//! )?;
//! ```

pub mod resolution;
pub mod types;

#[cfg(test)]
mod resolution_test;

// Re-export commonly used types
pub use resolution::{
    count_conflicts, delete_conflict_file, list_conflicts, load_conflict_file, resolve_conflict,
    write_conflict_file,
};
pub use types::{ConflictFile, Resolution, ResolutionStrategy};
