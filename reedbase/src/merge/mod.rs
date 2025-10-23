// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Row-level CSV merge module.
//!
//! Provides intelligent merging for concurrent writes with automatic
//! conflict detection.

pub mod csv;
pub mod diff;
pub mod types;

// Re-export public APIs
pub use csv::{
    build_row_map, calculate_merge_stats, detect_conflicts, merge_changes, merge_single, rows_equal,
};
pub use diff::{apply_changes, calculate_diff, count_changes};
pub use types::{Conflict, MergeResult, MergeStats, RowChange};

#[cfg(test)]
mod csv_test;
#[cfg(test)]
mod diff_test;
