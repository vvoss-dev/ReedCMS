// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Asset preparation and startup tasks.
//!
//! Handles all asset-related initialisation when the server starts:
//! - Session hash generation
//! - CSS bundle generation for all layouts
//! - JS bundle generation (future)

use crate::reedcms::assets::css::bundler::bundle_all_css;
use crate::reedcms::assets::css::session_hash::generate_and_store_session_hash;
use crate::reedcms::reedstream::ReedResult;

/// Prepares all assets for server operation.
///
/// ## Process
/// 1. Generate session hash from all CSS/JS source files
/// 2. Store session hash in .reed/project.csv
/// 3. Generate CSS bundles for all layouts (mouse, touch, reader)
/// 4. Output to public/session/styles/
///
/// ## Performance
/// - Session hash generation: < 50ms
/// - CSS bundling: < 500ms for all layouts
///
/// ## Output
/// - public/session/styles/{layout}.{session_hash}.{variant}.css
///
/// ## Error Conditions
/// - ReedError::FileNotFound: Template files missing
/// - ReedError::IoError: Cannot write bundles
/// - ReedError::ConfigError: Cannot store session hash
///
/// ## Example Usage
/// ```rust
/// prepare_assets()?;
/// // Generates: landing.a3f5b2c8.mouse.css, etc.
/// ```
pub fn prepare_assets() -> ReedResult<()> {
    println!("\n📦 Preparing assets...");

    // 1. Generate and store session hash
    println!("  🔑 Generating session hash...");
    let session_hash = generate_and_store_session_hash()?;
    println!("  ✓ Session hash: {}", session_hash);

    // 2. Generate CSS bundles
    println!("\n  🎨 Generating CSS bundles...");
    let results = bundle_all_css()?;
    println!("  ✓ Generated {} CSS bundle(s)", results.len());

    // 3. Future: Generate JS bundles
    // println!("\n  🔧 Generating JS bundles...");
    // let js_results = bundle_all_js()?;
    // println!("  ✓ Generated {} JS bundle(s)", js_results.len());

    println!("\n✅ Assets prepared successfully\n");

    Ok(())
}
