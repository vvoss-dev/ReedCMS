// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Version management for ReedBase.
//!
//! Provides binary delta compression for efficient versioning.

pub mod delta;

#[cfg(test)]
mod delta_test;

// Re-export public API
pub use delta::{apply_delta, calculate_savings, generate_delta, DeltaInfo};
