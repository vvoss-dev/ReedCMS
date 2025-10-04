// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Build system module for ReedCMS.
//!
//! This module provides binary compilation, release packaging, and version management.

pub mod cache_bust;
pub mod compiler;
pub mod packager;
pub mod pipeline;
pub mod version;

#[cfg(test)]
mod cache_bust_test;
#[cfg(test)]
mod compiler_test;
#[cfg(test)]
mod packager_test;
#[cfg(test)]
mod pipeline_test;
#[cfg(test)]
mod version_test;

// Re-export main public API
pub use cache_bust::{
    generate_cache_busting_manifest, get_hashed_filename, load_manifest, AssetManifest,
};
pub use compiler::{build_release, BuildInfo};
pub use packager::{package_release, PackageInfo};
pub use pipeline::{run_full_build, run_incremental_build, run_pipeline, BuildMode, BuildReport};
pub use version::{
    get_build_metadata, get_version, get_version_with_suffix, is_compatible, parse_version,
    BuildMetadata,
};
