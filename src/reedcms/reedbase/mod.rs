// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase Core Services
//!
//! Provides O(1) key-value operations with CSV persistence:
//! - get: Retrieve values by key with environment fallback
//! - set: Update values with automatic backup and CSV write
//! - init: Initialize in-memory HashMap cache from CSV files
//! - Thread-safe operations using RwLock

mod get;
mod init;
mod set;

pub use get::get;
pub use init::init;
pub use set::set;

#[cfg(test)]
mod get_test;
#[cfg(test)]
mod init_test;
#[cfg(test)]
mod set_test;
