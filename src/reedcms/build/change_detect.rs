// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Change detection and rebuild scope determination.
//!
//! Analyzes file paths to determine what assets need rebuilding.

/// Rebuild scope enum.
#[derive(Debug, Clone, PartialEq)]
pub enum RebuildScope {
    AllCss,
    SpecificCss { layout: String, variant: String },
    AllJs,
    SpecificJs { layout: String, variant: String },
    Template { path: String },
    Config { path: String },
    None,
}

/// Detects what needs rebuilding based on changed file.
///
/// ## Detection Rules
/// - Core CSS change → Rebuild all CSS
/// - Layout CSS change → Rebuild specific layout CSS
/// - Core JS change → Rebuild all JS
/// - Layout JS change → Rebuild specific layout JS
/// - Template change → Reload template
/// - Config change → Reload config
///
/// ## Input
/// - `path`: Changed file path
///
/// ## Output
/// - `RebuildScope`: What needs rebuilding
///
/// ## Example Usage
/// ```rust
/// let scope = detect_rebuild_scope("assets/css/layouts/knowledge/knowledge.mouse.css");
/// match scope {
///     RebuildScope::SpecificCss { layout, variant } => {
///         println!("Rebuild {}.{}.css", layout, variant);
///     }
///     _ => {}
/// }
/// ```
///
/// ## Performance
/// - < 1ms for path analysis
pub fn detect_rebuild_scope(path: &str) -> RebuildScope {
    if path.starts_with("assets/css/core/") || path.starts_with("assets/css/components/") {
        RebuildScope::AllCss
    } else if path.starts_with("assets/css/layouts/") {
        if let Some((layout, variant)) = extract_layout_variant(path, "css") {
            RebuildScope::SpecificCss { layout, variant }
        } else {
            RebuildScope::AllCss
        }
    } else if path.starts_with("assets/js/core/") || path.starts_with("assets/js/components/") {
        RebuildScope::AllJs
    } else if path.starts_with("assets/js/layouts/") {
        if let Some((layout, variant)) = extract_layout_variant(path, "js") {
            RebuildScope::SpecificJs { layout, variant }
        } else {
            RebuildScope::AllJs
        }
    } else if path.starts_with("templates/") {
        RebuildScope::Template {
            path: path.to_string(),
        }
    } else if path.starts_with(".reed/") {
        RebuildScope::Config {
            path: path.to_string(),
        }
    } else {
        RebuildScope::None
    }
}

/// Extracts layout and variant from file path.
///
/// ## Input
/// - `path`: File path
/// - `asset_type`: "css" or "js"
///
/// ## Output
/// - `Option<(String, String)>`: (layout, variant) or None
///
/// ## Examples
/// - `assets/css/layouts/knowledge/knowledge.mouse.css` → `("knowledge", "mouse")`
/// - `assets/js/layouts/blog/blog.touch.js` → `("blog", "touch")`
///
/// ## Performance
/// - < 1ms for path parsing
pub fn extract_layout_variant(path: &str, asset_type: &str) -> Option<(String, String)> {
    let pattern = format!("assets/{}/layouts/", asset_type);

    if let Some(start) = path.find(&pattern) {
        let after = &path[start + pattern.len()..];
        let parts: Vec<&str> = after.split('/').collect();

        if parts.len() >= 2 {
            let layout = parts[0].to_string();
            let filename = parts[1];

            // Extract variant from filename (e.g., knowledge.mouse.css → mouse)
            let name_parts: Vec<&str> = filename.split('.').collect();
            if name_parts.len() >= 3 {
                let variant = name_parts[1].to_string();
                return Some((layout, variant));
            }
        }
    }

    None
}
