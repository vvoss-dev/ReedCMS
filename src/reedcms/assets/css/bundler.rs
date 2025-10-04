// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CSS Bundler
//!
//! Bundles and minifies CSS files for layouts with session hash versioning.
//! Combines component CSS in correct order, minifies output, and generates source maps.

use crate::reedcms::reedstream::ReedResult;
use std::fs;

use super::discovery::{discover_layout_assets, discover_layouts};
use super::minifier::{calculate_reduction, minify_css};
use super::session_hash::get_session_hash;
use super::source_map::SourceMap;
use super::writer::{clean_old_bundles, ensure_output_dir, write_css_file, write_source_map};

/// Bundle result structure.
#[derive(Debug, Clone)]
pub struct BundleResult {
    pub output_path: String,
    pub original_size: usize,
    pub minified_size: usize,
    pub reduction_percent: u32,
}

/// Bundles and minifies CSS files for a specific layout and variant.
///
/// ## Input
/// - `layout`: Layout name (e.g., "landing", "knowledge")
/// - `variant`: Variant name (e.g., "mouse", "touch", "reader")
///
/// ## Output
/// - BundleResult with statistics
///
/// ## Process
/// 1. Discover layout CSS files via discover_layout_assets()
/// 2. Combine files in correct order
/// 3. Minify CSS via minify_css()
/// 4. Generate source maps
/// 5. Write to public/session/styles/
///
/// ## Order of Inclusion
/// 1. Layout CSS
/// 2. Organism CSS (in inclusion order)
/// 3. Molecule CSS (recursive dependencies)
/// 4. Atom CSS (recursive dependencies)
///
/// ## Performance
/// - Bundle time: < 100ms per layout
/// - Minification: ~60-70% size reduction
/// - Source map generation: < 10ms
///
/// ## Output
/// ```
/// Bundling CSS for knowledge.mouse...
/// - Included: templates/layouts/knowledge/knowledge.mouse.css (5.2 KB)
/// - Included: templates/components/organisms/page-header/page-header.mouse.css (3.4 KB)
/// → Output: public/session/styles/knowledge.a3f5b2c8.mouse.css (3.8 KB, -67%)
/// ✓ Bundle complete
/// ```
///
/// ## Example
/// ```rust
/// let result = bundle_css("landing", "mouse")?;
/// println!("Minified size: {} KB", result.minified_size / 1024);
/// ```
pub fn bundle_css(layout: &str, variant: &str) -> ReedResult<BundleResult> {
    println!("📦 Bundling CSS for {}.{}...", layout, variant);

    // Get session hash
    let session_hash = get_session_hash()?;

    // 1. Discover CSS files for this layout
    let assets = discover_layout_assets(layout, variant)?;

    // 2. Combine CSS content
    let mut combined_css = String::new();
    let mut source_map = SourceMap::new();

    for css_path in &assets.css_files {
        match fs::read_to_string(css_path) {
            Ok(css_content) => {
                let file_size = css_content.len();
                println!(
                    "  - Included: {} ({:.1} KB)",
                    css_path,
                    file_size as f64 / 1024.0
                );

                source_map.add_source(css_path, &css_content);
                combined_css.push_str(&css_content);
                combined_css.push('\n');
            }
            Err(_) => {
                eprintln!("  ⚠ CSS file not found: {}", css_path);
            }
        }
    }

    let original_size = combined_css.len();

    // 3. Minify CSS
    let minified = minify_css(&combined_css)?;
    let minified_size = minified.len();

    // 4. Generate source map
    let source_map_content = source_map.generate()?;
    let source_map_filename = format!("{}.{}.{}.css.map", layout, session_hash, variant);

    // 5. Append source map comment to CSS
    let minified_with_map = SourceMap::append_comment(&minified, &source_map_filename);

    // 6. Write output files
    ensure_output_dir("public/session/styles")?;

    let output_path = format!(
        "public/session/styles/{}.{}.{}.css",
        layout, session_hash, variant
    );
    let source_map_path = format!("public/session/styles/{}", source_map_filename);

    write_css_file(&output_path, &minified_with_map)?;
    write_source_map(&source_map_path, &source_map_content)?;

    let reduction = calculate_reduction(original_size, minified_size);

    println!(
        "  → Output: {} ({:.1} KB, -{}%)",
        output_path,
        minified_size as f64 / 1024.0,
        reduction
    );
    println!("✓ Bundle complete");

    Ok(BundleResult {
        output_path,
        original_size,
        minified_size,
        reduction_percent: reduction,
    })
}

/// Bundles CSS for all layouts and variants.
///
/// ## Output
/// - Vec of BundleResult for all successful bundles
///
/// ## Process
/// 1. Discover all layouts from templates/layouts/
/// 2. For each layout, bundle all variants (mouse/touch/reader)
/// 3. Report total size savings
/// 4. Clean old bundles with different session hash
///
/// ## Performance
/// - < 500ms for 10 layouts × 3 variants
///
/// ## Example
/// ```rust
/// let results = bundle_all_css()?;
/// println!("Bundled {} layouts", results.len() / 3);
/// ```
pub fn bundle_all_css() -> ReedResult<Vec<BundleResult>> {
    let layouts = discover_layouts()?;
    let variants = vec!["mouse", "touch", "reader"];

    let mut results = Vec::new();
    let mut total_original = 0;
    let mut total_minified = 0;

    println!("\n🔨 Bundling CSS for all layouts...\n");

    for layout in &layouts {
        for variant in &variants {
            match bundle_css(layout, variant) {
                Ok(result) => {
                    total_original += result.original_size;
                    total_minified += result.minified_size;
                    results.push(result);
                }
                Err(e) => {
                    eprintln!("⚠ Failed to bundle {}.{}: {:?}", layout, variant, e);
                }
            }
        }
        println!(); // Blank line between layouts
    }

    // Clean old bundles
    if let Ok(session_hash) = get_session_hash() {
        match clean_old_bundles("public/session/styles", &session_hash) {
            Ok(count) if count > 0 => {
                println!("🧹 Cleaned {} old bundle(s)", count);
            }
            _ => {}
        }
    }

    let total_reduction = calculate_reduction(total_original, total_minified);

    println!("📊 Total CSS Bundle Statistics:");
    println!("  Original size: {:.1} KB", total_original as f64 / 1024.0);
    println!("  Minified size: {:.1} KB", total_minified as f64 / 1024.0);
    println!("  Size reduction: {}%", total_reduction);
    println!("  Bundles created: {}", results.len());

    Ok(results)
}

/// Checks if bundles exist for layout, generates if missing.
///
/// ## Input
/// - `layout`: Layout name
/// - `session_hash`: Current session hash
///
/// ## Output
/// - Success or error
///
/// ## Process
/// 1. Check if any variant bundle is missing
/// 2. If missing, generate all variant bundles
/// 3. Uses on-demand generation strategy
///
/// ## Performance
/// - Check: < 1ms (filesystem stat)
/// - Generation: < 100ms (first request only)
/// - Subsequent requests: Cached, no generation
///
/// ## Example
/// ```rust
/// ensure_bundles_exist("landing", "a3f5b2c8")?;
/// // Generates: landing.a3f5b2c8.mouse.css
/// //            landing.a3f5b2c8.touch.css
/// //            landing.a3f5b2c8.reader.css
/// ```
pub fn ensure_bundles_exist(layout: &str, session_hash: &str) -> ReedResult<()> {
    let variants = ["mouse", "touch", "reader"];

    // Check if any bundle is missing
    let mut needs_generation = false;
    for variant in &variants {
        let css_path = format!(
            "public/session/styles/{}.{}.{}.css",
            layout, session_hash, variant
        );

        if !std::path::Path::new(&css_path).exists() {
            needs_generation = true;
            break;
        }
    }

    // Generate all variants if any is missing
    if needs_generation {
        for variant in &variants {
            bundle_css(layout, variant)?;
        }
    }

    Ok(())
}
