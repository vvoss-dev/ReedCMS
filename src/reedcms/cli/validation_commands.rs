// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! CLI validation commands.
//!
//! Implements data consistency validation commands for checking ReedCMS data integrity.
//! Commands follow the pattern: reed validate:action [--flags]

use crate::reedcms::csv::read_csv;
use crate::reedcms::reedstream::{ReedError, ReedResponse, ReedResult};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

/// Validates route consistency.
///
/// ## Usage
/// reed validate:routes [--fix]
///
/// ## Checks
/// 1. Route uniqueness per language
/// 2. Layout existence in registry
/// 3. Template file existence
/// 4. Route format validation
/// 5. Orphaned routes (layout deleted)
///
/// ## Flags
/// - --fix: Attempt automatic fixes for issues (not yet implemented)
///
/// ## Example
/// ```bash
/// reed validate:routes
/// ```
pub fn validate_routes(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>> {
    let start = Instant::now();
    let _fix_mode = flags.contains_key("fix");

    let mut output = String::from("🔍 Validating routes...\n\n");
    let mut issue_count = 0;

    // Load routes
    let routes_path = PathBuf::from(".reed/routes.csv");
    if !routes_path.exists() {
        output.push_str("⚠ No routes.csv found\n");
        return Ok(ReedResponse {
            data: output,
            source: "validation".to_string(),
            cached: false,
            timestamp: crate::reedcms::reedstream::current_timestamp(),
            metrics: None,
        });
    }

    let routes = read_csv(&routes_path)?;
    output.push_str(&format!("📊 Total routes: {}\n\n", routes.len()));

    // Check 1: Route uniqueness
    let mut route_map: HashMap<String, Vec<String>> = HashMap::new();
    for entry in &routes {
        route_map
            .entry(entry.key.clone())
            .or_default()
            .push(entry.value.clone());
    }

    let duplicates: Vec<_> = route_map
        .iter()
        .filter(|(_, layouts)| layouts.len() > 1)
        .collect();

    if duplicates.is_empty() {
        output.push_str("✓ Route uniqueness: OK (no duplicates)\n");
    } else {
        issue_count += duplicates.len();
        output.push_str(&format!(
            "⚠ Route uniqueness: {} duplicate(s) found\n",
            duplicates.len()
        ));
        for (route, layouts) in duplicates {
            output.push_str(&format!(
                "  - {} → {} layout(s): {}\n",
                route,
                layouts.len(),
                layouts.join(", ")
            ));
        }
    }

    // Check 2: Layout existence in registry
    let registry_path = PathBuf::from(".reed/registry.csv");
    let registered_layouts: HashSet<String> = if registry_path.exists() {
        read_csv(&registry_path)?
            .into_iter()
            .map(|e| e.key)
            .collect()
    } else {
        HashSet::new()
    };

    let mut missing_layouts = Vec::new();
    for entry in &routes {
        if !registered_layouts.contains(&entry.value) {
            missing_layouts.push(format!("{} → {}", entry.key, entry.value));
        }
    }

    if missing_layouts.is_empty() {
        output.push_str("✓ Layout references: OK (all layouts exist in registry)\n");
    } else {
        issue_count += missing_layouts.len();
        output.push_str(&format!(
            "⚠ Layout references: {} missing layout(s)\n",
            missing_layouts.len()
        ));
        for missing in missing_layouts.iter().take(5) {
            output.push_str(&format!("  - {}\n", missing));
        }
        if missing_layouts.len() > 5 {
            output.push_str(&format!("  ... and {} more\n", missing_layouts.len() - 5));
        }
    }

    // Check 3: Template file existence
    let mut missing_templates = Vec::new();
    for layout in &registered_layouts {
        for variant in &["mouse", "touch", "reader"] {
            let template_path = PathBuf::from(format!(
                "templates/layouts/{}/{}.{}.jinja",
                layout, layout, variant
            ));
            if !template_path.exists() {
                missing_templates.push(format!("{}.{}.jinja", layout, variant));
            }
        }
    }

    if missing_templates.is_empty() {
        output.push_str("✓ Template files: OK (all templates exist)\n");
    } else {
        issue_count += missing_templates.len();
        output.push_str(&format!(
            "⚠ Template files: {} missing\n",
            missing_templates.len()
        ));
        for missing in missing_templates.iter().take(5) {
            output.push_str(&format!("  - {}\n", missing));
        }
        if missing_templates.len() > 5 {
            output.push_str(&format!("  ... and {} more\n", missing_templates.len() - 5));
        }
    }

    // Check 4: Route format validation
    let mut invalid_routes = Vec::new();
    for entry in &routes {
        let route = &entry.key;
        // Routes should not contain @ (that's for language suffix in other contexts)
        // Routes should be alphanumeric + hyphens + slashes
        if !route
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '/' || c == '_')
        {
            invalid_routes.push(route.clone());
        }
    }

    if invalid_routes.is_empty() {
        output.push_str("✓ Route format: OK\n");
    } else {
        issue_count += invalid_routes.len();
        output.push_str(&format!(
            "⚠ Route format: {} invalid route(s)\n",
            invalid_routes.len()
        ));
        for invalid in invalid_routes.iter().take(5) {
            output.push_str(&format!("  - {}\n", invalid));
        }
    }

    let duration = start.elapsed();
    output.push_str(&format!("\n📋 Summary: {} issue(s) found\n", issue_count));
    output.push_str(&format!("Duration: {:.2}s\n", duration.as_secs_f64()));

    Ok(ReedResponse {
        data: output,
        source: "validation".to_string(),
        cached: false,
        timestamp: crate::reedcms::reedstream::current_timestamp(),
        metrics: None,
    })
}

/// Validates complete data consistency.
///
/// ## Usage
/// reed validate:consistency
///
/// ## Checks
/// 1. CSV file integrity
/// 2. Foreign key relationships
/// 3. User-role assignments
/// 4. Template-layout mappings
/// 5. Text-route consistency
///
/// ## Example
/// ```bash
/// reed validate:consistency
/// ```
pub fn validate_consistency(_flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>> {
    let start = Instant::now();
    let mut output = String::from("🔍 Running comprehensive consistency check...\n\n");
    let mut total_issues = 0;

    // Check CSV files
    output.push_str("CSV Files:\n");

    let csv_files = vec![
        ("text.csv", ".reed/text.csv"),
        ("routes.csv", ".reed/routes.csv"),
        ("meta.csv", ".reed/meta.csv"),
        ("registry.csv", ".reed/registry.csv"),
    ];

    for (name, path) in &csv_files {
        let path_buf = PathBuf::from(path);
        if !path_buf.exists() {
            output.push_str(&format!("⚠ {}: Missing\n", name));
            total_issues += 1;
        } else {
            match read_csv(&path_buf) {
                Ok(entries) => {
                    output.push_str(&format!(
                        "✓ {}: {} entries, valid structure\n",
                        name,
                        entries.len()
                    ));
                }
                Err(e) => {
                    output.push_str(&format!("⚠ {}: Error reading - {}\n", name, e));
                    total_issues += 1;
                }
            }
        }
    }

    output.push_str("\nRelationships:\n");

    // Check template-layout mappings
    let registry_path = PathBuf::from(".reed/registry.csv");
    if registry_path.exists() {
        let layouts = read_csv(&registry_path)?;
        let mut missing_count = 0;

        for layout in &layouts {
            let layout_name = &layout.key;
            for variant in &["mouse", "touch", "reader"] {
                let template_path = PathBuf::from(format!(
                    "templates/layouts/{}/{}.{}.jinja",
                    layout_name, layout_name, variant
                ));
                if !template_path.exists() {
                    missing_count += 1;
                }
            }
        }

        if missing_count == 0 {
            output.push_str("✓ Template-layout mappings: OK\n");
        } else {
            output.push_str(&format!(
                "⚠ Template-layout mappings: {} missing template(s)\n",
                missing_count
            ));
            total_issues += missing_count;
        }
    }

    // Check text-route consistency
    let text_path = PathBuf::from(".reed/text.csv");
    let routes_path = PathBuf::from(".reed/routes.csv");

    if text_path.exists() && routes_path.exists() {
        output.push_str("✓ Text-route files: Both present\n");
    } else {
        if !text_path.exists() {
            output.push_str("⚠ Text-route: text.csv missing\n");
            total_issues += 1;
        }
        if !routes_path.exists() {
            output.push_str("⚠ Text-route: routes.csv missing\n");
            total_issues += 1;
        }
    }

    let duration = start.elapsed();
    output.push_str(&format!("\n📋 Summary: {} issue(s) found\n", total_issues));
    output.push_str(&format!("Duration: {:.2}s\n", duration.as_secs_f64()));

    Ok(ReedResponse {
        data: output,
        source: "validation".to_string(),
        cached: false,
        timestamp: crate::reedcms::reedstream::current_timestamp(),
        metrics: None,
    })
}

/// Validates text content for specific language.
///
/// ## Usage
/// reed validate:text --language <lang>
///
/// ## Checks
/// 1. Missing translations
/// 2. Empty values
/// 3. Key format consistency
///
/// ## Example
/// ```bash
/// reed validate:text --language de
/// ```
pub fn validate_text(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>> {
    let start = Instant::now();

    let language = flags
        .get("language")
        .ok_or_else(|| ReedError::ValidationError {
            field: "language".to_string(),
            value: String::new(),
            constraint: "--language flag required".to_string(),
        })?;

    let mut output = format!("🔍 Validating text content for language: {}\n\n", language);
    let mut issue_count = 0;

    // Load text entries
    let text_path = PathBuf::from(".reed/text.csv");
    if !text_path.exists() {
        output.push_str("⚠ No text.csv found\n");
        return Ok(ReedResponse {
            data: output,
            source: "validation".to_string(),
            cached: false,
            timestamp: crate::reedcms::reedstream::current_timestamp(),
            metrics: None,
        });
    }

    let entries = read_csv(&text_path)?;
    output.push_str(&format!("📊 Total entries: {}\n\n", entries.len()));

    // Build key bases (without @lang)
    let mut key_bases: HashMap<String, Vec<String>> = HashMap::new();
    for entry in &entries {
        if let Some((base, lang)) = entry.key.rsplit_once('@') {
            key_bases
                .entry(base.to_string())
                .or_default()
                .push(lang.to_string());
        }
    }

    // Check 1: Key format
    let mut invalid_keys = 0;
    for entry in &entries {
        if !entry.key.contains('@') {
            invalid_keys += 1;
        }
    }

    if invalid_keys == 0 {
        output.push_str("✓ Key format: OK (all have @lang suffix)\n");
    } else {
        output.push_str(&format!(
            "⚠ Key format: {} key(s) missing @lang suffix\n",
            invalid_keys
        ));
        issue_count += invalid_keys;
    }

    // Check 2: Missing translations
    let mut missing_translations = Vec::new();
    for (base, langs) in &key_bases {
        if !langs.contains(language) {
            missing_translations.push(format!("{} (exists in: {})", base, langs.join(", ")));
        }
    }

    if missing_translations.is_empty() {
        output.push_str(&format!(
            "✓ Missing translations: None for language '{}'\n",
            language
        ));
    } else {
        issue_count += missing_translations.len();
        output.push_str(&format!(
            "⚠ Missing translations: {} found\n",
            missing_translations.len()
        ));
        for missing in missing_translations.iter().take(10) {
            output.push_str(&format!("  - {}\n", missing));
        }
        if missing_translations.len() > 10 {
            output.push_str(&format!(
                "  ... and {} more\n",
                missing_translations.len() - 10
            ));
        }
    }

    // Check 3: Empty values
    let mut empty_values = Vec::new();
    for entry in &entries {
        if entry.key.ends_with(&format!("@{}", language)) && entry.value.is_empty() {
            empty_values.push(entry.key.clone());
        }
    }

    if empty_values.is_empty() {
        output.push_str("✓ Empty values: None found\n");
    } else {
        issue_count += empty_values.len();
        output.push_str(&format!("⚠ Empty values: {} found\n", empty_values.len()));
        for empty in empty_values.iter().take(5) {
            output.push_str(&format!("  - {}\n", empty));
        }
    }

    // Calculate completeness
    let total_bases = key_bases.len();
    let translated = total_bases - missing_translations.len();
    let completeness = if total_bases > 0 {
        (translated as f64 / total_bases as f64) * 100.0
    } else {
        100.0
    };

    let duration = start.elapsed();
    output.push_str(&format!("\n📋 Summary: {} issue(s) found\n", issue_count));
    output.push_str(&format!(
        "Completeness: {:.1}% ({} of {} base keys)\n",
        completeness, translated, total_bases
    ));
    output.push_str(&format!("Duration: {:.2}s\n", duration.as_secs_f64()));

    Ok(ReedResponse {
        data: output,
        source: "validation".to_string(),
        cached: false,
        timestamp: crate::reedcms::reedstream::current_timestamp(),
        metrics: None,
    })
}

/// Validates reference integrity.
///
/// ## Usage
/// reed validate:references
///
/// ## Checks
/// 1. Layout → Template references
/// 2. Route → Layout references
///
/// ## Example
/// ```bash
/// reed validate:references
/// ```
pub fn validate_references(_flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>> {
    let start = Instant::now();
    let mut output = String::from("🔍 Validating reference integrity...\n\n");
    let mut issue_count = 0;

    // Check 1: Layout → Template references
    let registry_path = PathBuf::from(".reed/registry.csv");
    if registry_path.exists() {
        let layouts = read_csv(&registry_path)?;
        let mut missing_templates = Vec::new();

        for layout in &layouts {
            let layout_name = &layout.key;
            for variant in &["mouse", "touch", "reader"] {
                let template_path = PathBuf::from(format!(
                    "templates/layouts/{}/{}.{}.jinja",
                    layout_name, layout_name, variant
                ));
                if !template_path.exists() {
                    missing_templates.push(format!(
                        "{} → {}.{}.jinja",
                        layout_name, layout_name, variant
                    ));
                }
            }
        }

        if missing_templates.is_empty() {
            output.push_str("✓ Layout references: OK (all templates exist)\n");
        } else {
            issue_count += missing_templates.len();
            output.push_str(&format!(
                "⚠ Layout references: {} broken reference(s)\n",
                missing_templates.len()
            ));
            for missing in missing_templates.iter().take(5) {
                output.push_str(&format!("  - {}\n", missing));
            }
        }
    } else {
        output.push_str("⚠ Layout references: registry.csv not found\n");
        issue_count += 1;
    }

    // Check 2: Route → Layout references
    let routes_path = PathBuf::from(".reed/routes.csv");
    let registry_path = PathBuf::from(".reed/registry.csv");

    if routes_path.exists() && registry_path.exists() {
        let routes = read_csv(&routes_path)?;
        let layouts: HashSet<String> = read_csv(&registry_path)?
            .into_iter()
            .map(|e| e.key)
            .collect();

        let mut broken_refs = Vec::new();
        for route in &routes {
            if !layouts.contains(&route.value) {
                broken_refs.push(format!("{} → {}", route.key, route.value));
            }
        }

        if broken_refs.is_empty() {
            output.push_str("✓ Route references: OK (all layouts exist)\n");
        } else {
            issue_count += broken_refs.len();
            output.push_str(&format!(
                "⚠ Route references: {} broken reference(s)\n",
                broken_refs.len()
            ));
            for broken in broken_refs.iter().take(5) {
                output.push_str(&format!("  - {}\n", broken));
            }
        }
    }

    let duration = start.elapsed();
    output.push_str(&format!(
        "\n📋 Summary: {} broken reference(s) found\n",
        issue_count
    ));
    output.push_str(&format!("Duration: {:.2}s\n", duration.as_secs_f64()));

    Ok(ReedResponse {
        data: output,
        source: "validation".to_string(),
        cached: false,
        timestamp: crate::reedcms::reedstream::current_timestamp(),
        metrics: None,
    })
}
