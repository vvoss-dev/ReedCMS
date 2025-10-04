// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! JavaScript Bundler Module
//!
//! Provides JavaScript bundling, minification, tree shaking, and dependency resolution.
//! Implements on-demand bundle generation with session hash versioning.

pub mod bundler;
pub mod minifier;
pub mod resolver;
pub mod tree_shake;

// Re-export main types and functions
pub use bundler::{bundle_all_js, bundle_js, ensure_bundles_exist, write_js_file, BundleResult};
pub use minifier::{calculate_reduction, minify_js};
pub use resolver::{parse_imports, resolve_import_path, DependencyResolver, Module};
pub use tree_shake::{parse_exports, parse_import_names, tree_shake};

// Note: We intentionally don't re-export parse_exports to avoid confusion
// Use tree_shake::parse_exports if needed
