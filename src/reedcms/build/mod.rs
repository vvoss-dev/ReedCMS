// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Build system module for ReedCMS.
//!
//! This module provides binary compilation, release packaging, and version management.

pub mod compiler;
pub mod packager;
pub mod version;

#[cfg(test)]
mod compiler_test;
#[cfg(test)]
mod packager_test;
#[cfg(test)]
mod version_test;

// Re-export main public API
pub use compiler::{build_release, BuildInfo};
pub use packager::{package_release, PackageInfo};
pub use version::{
    get_build_metadata, get_version, get_version_with_suffix, is_compatible, parse_version,
    BuildMetadata,
};
