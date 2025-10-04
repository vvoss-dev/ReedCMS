// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Component Discovery for CSS/JS Bundling
//!
//! Parses Jinja templates to discover component dependencies and asset files.
//! Supports automatic discovery of organisms, molecules, and atoms.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Layout assets structure containing CSS and JS file paths.
#[derive(Debug, Clone)]
pub struct LayoutAssets {
    pub css_files: Vec<String>,
    pub js_files: Vec<String>,
}

/// Discovers all assets required for a layout.
///
/// ## Input
/// - `layout`: Layout name (e.g., "landing", "knowledge")
/// - `variant`: Variant name (e.g., "mouse", "touch", "reader")
///
/// ## Output
/// - LayoutAssets with ordered CSS and JS file paths
///
/// ## Process
/// 1. Parse layout template ({layout}.jinja)
/// 2. Extract {% include organism("...") %} statements
/// 3. Recursively discover organism dependencies (molecules, atoms)
/// 4. Collect all CSS/JS files from components
/// 5. Return ordered list of asset paths
///
/// ## Order
/// 1. Layout CSS/JS
/// 2. Organism CSS/JS (in inclusion order)
/// 3. Molecule CSS/JS (recursive dependencies)
/// 4. Atom CSS/JS (recursive dependencies)
///
/// ## Performance
/// - < 50ms per layout
///
/// ## Example
/// ```rust
/// let assets = discover_layout_assets("landing", "mouse")?;
/// // css_files: ["templates/layouts/landing/landing.mouse.css",
/// //             "templates/components/organisms/landing-hero/landing-hero.mouse.css", ...]
/// ```
pub fn discover_layout_assets(layout: &str, variant: &str) -> ReedResult<LayoutAssets> {
    let template_path = format!("templates/layouts/{}/{}.jinja", layout, layout);
    let template_content = fs::read_to_string(&template_path).map_err(|e| ReedError::IoError {
        operation: "read".to_string(),
        path: template_path.clone(),
        reason: e.to_string(),
    })?;

    let mut css_files = Vec::new();
    let mut js_files = Vec::new();
    let mut processed_components = HashSet::new();

    // 1. Add layout CSS/JS first
    let layout_css = format!("templates/layouts/{}/{}.{}.css", layout, layout, variant);
    if Path::new(&layout_css).exists() {
        css_files.push(layout_css);
    }

    let layout_js = format!("templates/layouts/{}/{}.js", layout, layout);
    if Path::new(&layout_js).exists() {
        js_files.push(layout_js);
    }

    // 2. Extract organism includes
    let organisms = extract_organisms(&template_content)?;

    // 3. Process each organism and its dependencies
    for organism in organisms {
        discover_component_assets(
            "organisms",
            &organism,
            variant,
            &mut css_files,
            &mut js_files,
            &mut processed_components,
        )?;
    }

    Ok(LayoutAssets {
        css_files,
        js_files,
    })
}

/// Recursively discovers assets for a component and its dependencies.
///
/// ## Input
/// - `component_type`: Type of component ("organisms", "molecules", "atoms")
/// - `component_name`: Name of component
/// - `variant`: Variant name
/// - `css_files`: Mutable vector to accumulate CSS paths
/// - `js_files`: Mutable vector to accumulate JS paths
/// - `processed_components`: Set of already processed components (prevents circular dependencies)
///
/// ## Process
/// 1. Check if component already processed
/// 2. Add component CSS/JS files
/// 3. Parse component template for includes
/// 4. Recursively process dependencies
fn discover_component_assets(
    component_type: &str,
    component_name: &str,
    variant: &str,
    css_files: &mut Vec<String>,
    js_files: &mut Vec<String>,
    processed_components: &mut HashSet<String>,
) -> ReedResult<()> {
    // Prevent circular dependencies
    let component_id = format!("{}:{}", component_type, component_name);
    if processed_components.contains(&component_id) {
        return Ok(());
    }
    processed_components.insert(component_id);

    // Add component CSS
    let css_path = format!(
        "templates/components/{}/{}/{}.{}.css",
        component_type, component_name, component_name, variant
    );
    if Path::new(&css_path).exists() {
        css_files.push(css_path);
    }

    // Add component JS (variant-independent)
    let js_path = format!(
        "templates/components/{}/{}/{}.js",
        component_type, component_name, component_name
    );
    if Path::new(&js_path).exists() {
        js_files.push(js_path);
    }

    // Parse component template for dependencies
    let template_path = format!(
        "templates/components/{}/{}/{}.{}.jinja",
        component_type, component_name, component_name, variant
    );

    if let Ok(template_content) = fs::read_to_string(&template_path) {
        // Extract molecule includes
        if let Ok(molecules) = extract_molecules(&template_content) {
            for molecule in molecules {
                discover_component_assets(
                    "molecules",
                    &molecule,
                    variant,
                    css_files,
                    js_files,
                    processed_components,
                )?;
            }
        }

        // Extract atom includes
        if let Ok(atoms) = extract_atoms(&template_content) {
            for atom in atoms {
                discover_component_assets(
                    "atoms",
                    &atom,
                    variant,
                    css_files,
                    js_files,
                    processed_components,
                )?;
            }
        }
    }

    Ok(())
}

/// Extracts organism names from template content.
///
/// ## Input
/// - `template_content`: Jinja template content
///
/// ## Output
/// - Vec of organism names
///
/// ## Pattern Matching
/// - Matches: `{% include organism("landing-hero") %}`
/// - Extracts: `"landing-hero"`
///
/// ## Example
/// ```rust
/// let template = r#"{% include organism("landing-hero") %}"#;
/// let organisms = extract_organisms(template)?;
/// assert_eq!(organisms, vec!["landing-hero"]);
/// ```
pub fn extract_organisms(template_content: &str) -> ReedResult<Vec<String>> {
    extract_components_by_type(template_content, "organism")
}

/// Extracts molecule names from template content.
///
/// ## Input
/// - `template_content`: Jinja template content
///
/// ## Output
/// - Vec of molecule names
///
/// ## Pattern Matching
/// - Matches: `{% include molecule("card") %}`
/// - Extracts: `"card"`
pub fn extract_molecules(template_content: &str) -> ReedResult<Vec<String>> {
    extract_components_by_type(template_content, "molecule")
}

/// Extracts atom names from template content.
///
/// ## Input
/// - `template_content`: Jinja template content
///
/// ## Output
/// - Vec of atom names
///
/// ## Pattern Matching
/// - Matches: `{% include atom("button") %}`
/// - Extracts: `"button"`
pub fn extract_atoms(template_content: &str) -> ReedResult<Vec<String>> {
    extract_components_by_type(template_content, "atom")
}

/// Extracts component names by type from template content.
///
/// ## Input
/// - `template_content`: Jinja template content
/// - `component_type`: Type of component ("organism", "molecule", "atom")
///
/// ## Output
/// - Vec of component names
///
/// ## Pattern
/// - `{% include {component_type}("component-name") %}`
///
/// ## Performance
/// - < 1ms per template
fn extract_components_by_type(
    template_content: &str,
    component_type: &str,
) -> ReedResult<Vec<String>> {
    let pattern = format!(
        r#"\{{\%\s*include\s+{}\("([^"]+)"\)\s*\%\}}"#,
        component_type
    );

    let re = Regex::new(&pattern).map_err(|e| ReedError::ParseError {
        input: pattern.clone(),
        reason: e.to_string(),
    })?;

    let mut components = Vec::new();
    for cap in re.captures_iter(template_content) {
        components.push(cap[1].to_string());
    }

    Ok(components)
}

/// Discovers all layouts from templates/layouts/ directory.
///
/// ## Output
/// - Vec of layout names
///
/// ## Performance
/// - < 10ms
///
/// ## Example
/// ```rust
/// let layouts = discover_layouts()?;
/// // Returns: ["landing", "knowledge", "blog", ...]
/// ```
pub fn discover_layouts() -> ReedResult<Vec<String>> {
    let templates_dir = "templates/layouts";
    let mut layouts = Vec::new();

    let entries = fs::read_dir(templates_dir).map_err(|e| ReedError::IoError {
        operation: "read_dir".to_string(),
        path: templates_dir.to_string(),
        reason: e.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ReedError::IoError {
            operation: "read_entry".to_string(),
            path: templates_dir.to_string(),
            reason: e.to_string(),
        })?;

        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            if let Some(name) = entry.file_name().to_str() {
                // Skip hidden directories and special directories
                if !name.starts_with('.') && !name.starts_with('_') {
                    layouts.push(name.to_string());
                }
            }
        }
    }

    layouts.sort();
    Ok(layouts)
}
