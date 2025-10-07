// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! ReedBase Core Services
//!
//! Provides O(1) key-value operations with CSV persistence:
//! - cache: OnceLock-based HashMap cache for O(1) lookups
//! - get: Retrieve values by key with environment fallback
//! - set: Update values with automatic backup and CSV write
//! - init: Initialise in-memory HashMap cache from CSV files
//! - Thread-safe operations using RwLock

pub mod cache;
pub mod get;
pub mod init;
pub mod set;

#[cfg(test)]
mod get_test;
#[cfg(test)]
mod init_test;
#[cfg(test)]
mod set_test;
