// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CSS Bundler Module
//!
//! Provides CSS bundling, minification, and source map generation for ReedCMS.
//! Implements session hash strategy for cache-busting and on-demand bundle generation.

pub mod bundler;
pub mod discovery;
pub mod minifier;
pub mod session_hash;
pub mod source_map;
pub mod writer;

// Re-export main types and functions
pub use bundler::{bundle_all_css, bundle_css, ensure_bundles_exist, BundleResult};
pub use discovery::{discover_layout_assets, discover_layouts, LayoutAssets};
pub use minifier::{calculate_reduction, minify_css};
pub use session_hash::{
    discover_css_files, discover_js_files, generate_and_store_session_hash, generate_session_hash,
    get_session_hash, store_session_hash,
};
pub use source_map::SourceMap;
pub use writer::{clean_old_bundles, ensure_output_dir, write_css_file, write_source_map};
