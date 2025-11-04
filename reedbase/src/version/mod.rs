// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Version management for ReedBase.
//!
//! Provides binary delta compression for efficient versioning and
//! timestamp-indexed B+-Tree for fast version history queries.

pub mod delta;
pub mod index;
pub mod rebuild;

#[cfg(test)]
mod delta_test;
#[cfg(test)]
mod index_test;

// Re-export public API
pub use delta::{apply_delta, calculate_savings, generate_delta, DeltaInfo};
pub use index::{FrameId, IndexStats, Timestamp, VersionId, VersionIndices};
pub use rebuild::{rebuild_indices, VersionEntry};
