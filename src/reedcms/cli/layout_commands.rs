// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Layout management CLI commands.
//!
//! Provides flag-based layout creation with automatic file generation,
//! registry updates, and default content creation.

use crate::reedcms::csv::{read_csv, write_csv, CsvRecord};
use crate::reedcms::reedstream::{current_timestamp, ReedError, ReedResponse, ReedResult};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Template variant types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemplateVariant {
    Mouse,
    Touch,
    Reader,
}

impl TemplateVariant {
    /// Converts string to variant.
    pub fn from_str(s: &str) -> ReedResult<Self> {
        match s.to_lowercase().as_str() {
            "mouse" => Ok(Self::Mouse),
            "touch" => Ok(Self::Touch),
            "reader" => Ok(Self::Reader),
            _ => Err(ReedError::ValidationError {
                field: "variant".to_string(),
                value: s.to_string(),
                constraint: "Must be: mouse, touch, or reader".to_string(),
            }),
        }
    }

    /// Converts variant to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mouse => "mouse",
            Self::Touch => "touch",
            Self::Reader => "reader",
        }
    }
}

/// Layout configuration.
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub variants: Vec<TemplateVariant>,
    pub languages: Vec<String>,
    pub routes: HashMap<String, String>,
    pub parent: Option<String>,
}

/// Creates new layout(s) with flag-based configuration.
///
/// ## Arguments
/// - args: Layout name(s) (one or more)
/// - flags["languages"]: Comma-separated language codes (default: "de,en")
/// - flags["variants"]: Comma-separated variants (default: "mouse,touch,reader")
/// - flags["routes"]: Language:route pairs (default: layout name for all languages)
/// - flags["parent"]: Optional parent layout name
///
/// ## Example
/// ```bash
/// reed init:layout knowledge --languages de,en,fr --variants mouse,touch
/// ```
///
/// ## Performance
/// - Single layout: < 500ms
/// - Multiple layouts: < 1000ms for 5 layouts
pub fn init_layout(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: "init:layout".to_string(),
            reason: "Requires at least 1 layout name".to_string(),
        });
    }

    // Parse configuration from flags
    let config = parse_layout_config(flags)?;

    // Validate layout names
    for name in args {
        validate_layout_name(name)?;
        if layout_exists(name)? {
            return Err(ReedError::ValidationError {
                field: "layout_name".to_string(),
                value: name.clone(),
                constraint: "Layout already exists".to_string(),
            });
        }
    }

    // Create layouts
    let mut created = Vec::new();
    for name in args {
        create_layout(name, &config)?;
        created.push(name.clone());
    }

    // Format output
    let output = format!(
        "✓ Created {} layout{}: {}\n  Languages: {}\n  Variants: {}\n  Routes: {}",
        created.len(),
        if created.len() == 1 { "" } else { "s" },
        created.join(", "),
        config.languages.join(", "),
        config
            .variants
            .iter()
            .map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        config.routes.len()
    );

    Ok(ReedResponse {
        data: output,
        source: "cli::layout_commands::init_layout".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Parses layout configuration from flags.
fn parse_layout_config(flags: &HashMap<String, String>) -> ReedResult<LayoutConfig> {
    // Parse languages (default: de,en)
    let languages = if let Some(langs) = flags.get("languages") {
        langs.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        vec!["de".to_string(), "en".to_string()]
    };

    // Parse variants (default: mouse,touch,reader)
    let variants = if let Some(vars) = flags.get("variants") {
        vars.split(',')
            .map(|s| TemplateVariant::from_str(s.trim()))
            .collect::<ReedResult<Vec<_>>>()?
    } else {
        vec![
            TemplateVariant::Mouse,
            TemplateVariant::Touch,
            TemplateVariant::Reader,
        ]
    };

    // Parse routes (default: empty, will be filled per layout)
    let routes = if let Some(route_str) = flags.get("routes") {
        parse_routes(route_str)?
    } else {
        HashMap::new()
    };

    // Parse parent
    let parent = flags.get("parent").cloned();

    Ok(LayoutConfig {
        variants,
        languages,
        routes,
        parent,
    })
}

/// Parses route string (format: "de:wissen,en:knowledge").
fn parse_routes(route_str: &str) -> ReedResult<HashMap<String, String>> {
    let mut routes = HashMap::new();

    for pair in route_str.split(',') {
        let parts: Vec<&str> = pair.trim().split(':').collect();
        if parts.len() != 2 {
            return Err(ReedError::ValidationError {
                field: "routes".to_string(),
                value: pair.to_string(),
                constraint: "Format: lang:route (e.g., de:wissen,en:knowledge)".to_string(),
            });
        }
        routes.insert(parts[0].to_string(), parts[1].to_string());
    }

    Ok(routes)
}

/// Validates layout name.
///
/// ## Rules
/// - Alphanumeric + hyphen
/// - 3-32 characters
/// - Must start with letter
/// - No reserved names
pub fn validate_layout_name(name: &str) -> ReedResult<()> {
    if name.len() < 3 || name.len() > 32 {
        return Err(ReedError::ValidationError {
            field: "layout_name".to_string(),
            value: name.to_string(),
            constraint: "Must be 3-32 characters".to_string(),
        });
    }

    if !name.chars().next().unwrap().is_ascii_alphabetic() {
        return Err(ReedError::ValidationError {
            field: "layout_name".to_string(),
            value: name.to_string(),
            constraint: "Must start with letter".to_string(),
        });
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ReedError::ValidationError {
            field: "layout_name".to_string(),
            value: name.to_string(),
            constraint: "Only alphanumeric, hyphen, underscore allowed".to_string(),
        });
    }

    // Reserved names
    let reserved = ["admin", "system", "api", "debug", "test"];
    if reserved.contains(&name) {
        return Err(ReedError::ValidationError {
            field: "layout_name".to_string(),
            value: name.to_string(),
            constraint: "Reserved name".to_string(),
        });
    }

    Ok(())
}

/// Checks if layout exists.
pub fn layout_exists(name: &str) -> ReedResult<bool> {
    let layout_dir = format!("templates/layouts/{}", name);
    Ok(Path::new(&layout_dir).exists())
}

/// Creates single layout with all files and data.
fn create_layout(name: &str, config: &LayoutConfig) -> ReedResult<()> {
    // Create directory
    let layout_dir = format!("templates/layouts/{}", name);
    fs::create_dir_all(&layout_dir).map_err(|e| ReedError::IoError {
        operation: "create_dir_all".to_string(),
        path: layout_dir.clone(),
        reason: e.to_string(),
    })?;

    // Generate template files
    for variant in &config.variants {
        generate_template_file(name, *variant)?;
        generate_css_file(name, *variant)?;
    }

    // Update registry
    update_registry(name, config)?;

    // Add routes (use layout name if not specified)
    let routes = if config.routes.is_empty() {
        config
            .languages
            .iter()
            .map(|lang| (lang.clone(), name.to_string()))
            .collect()
    } else {
        config.routes.clone()
    };
    add_routes(name, &routes)?;

    // Add default text content
    add_default_text(name, &config.languages)?;

    // Add default meta data
    add_default_meta(name)?;

    Ok(())
}

/// Generates Jinja template file.
fn generate_template_file(layout: &str, variant: TemplateVariant) -> ReedResult<()> {
    let file_path = format!(
        "templates/layouts/{}/{}.{}.jinja",
        layout,
        layout,
        variant.as_str()
    );

    let content = format!(
        r#"{{# Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0. #}}
{{# SPDX-License-Identifier: Apache-2.0 #}}

<!DOCTYPE html>
<html lang="{{{{ lang }}}}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{{{ "{}.title" | text(lang) }}}}</title>
    <meta name="description" content="{{{{ "{}.description" | meta }}}}">
    <link rel="stylesheet" href="/css/{}.{}.css">
</head>
<body class="{}">
    <main>
        <h1>{{{{ "{}.heading" | text(lang) }}}}</h1>
        <p>{{{{ "{}.content" | text(lang) }}}}</p>
    </main>
</body>
</html>
"#,
        layout,
        layout,
        layout,
        variant.as_str(),
        variant.as_str(),
        layout,
        layout
    );

    fs::write(&file_path, content).map_err(|e| ReedError::IoError {
        operation: "write".to_string(),
        path: file_path,
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Generates CSS file.
fn generate_css_file(layout: &str, variant: TemplateVariant) -> ReedResult<()> {
    let file_path = format!(
        "templates/layouts/{}/{}.{}.css",
        layout,
        layout,
        variant.as_str()
    );

    let content = format!(
        r#"/* Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0. */
/* SPDX-License-Identifier: Apache-2.0 */

/* Layout: {} */
/* Variant: {} */

* {{
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}}

body.{} {{
    font-family: system-ui, -apple-system, sans-serif;
    line-height: 1.6;
    padding: 2rem;
}}

main {{
    max-width: 1200px;
    margin: 0 auto;
}}

h1 {{
    margin-bottom: 1rem;
}}
"#,
        layout,
        variant.as_str(),
        variant.as_str()
    );

    fs::write(&file_path, content).map_err(|e| ReedError::IoError {
        operation: "write".to_string(),
        path: file_path,
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Updates registry with new layout.
fn update_registry(layout: &str, config: &LayoutConfig) -> ReedResult<()> {
    let registry_path = ".reed/registry.csv";

    // Ensure directory exists
    fs::create_dir_all(".reed").ok();

    // Read existing registry
    let mut records = if Path::new(registry_path).exists() {
        read_csv(registry_path)?
    } else {
        vec![]
    };

    // Create registry entry
    let variants_str = config
        .variants
        .iter()
        .map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(",");

    let languages_str = config.languages.join(",");

    let parent_str = config.parent.clone().unwrap_or_default();

    let entry = CsvRecord {
        key: layout.to_string(),
        value: format!(
            "{}|{}|{}|{}|true",
            variants_str,
            languages_str,
            parent_str,
            current_timestamp()
        ),
        description: Some(format!("Layout: {}", layout)),
    };

    records.push(entry);

    // Write registry
    write_csv(registry_path, &records)?;

    Ok(())
}

/// Adds routes for layout.
fn add_routes(layout: &str, routes: &HashMap<String, String>) -> ReedResult<()> {
    let routes_path = ".reed/routes.csv";

    // Read existing routes
    let mut records = if Path::new(routes_path).exists() {
        read_csv(routes_path)?
    } else {
        vec![]
    };

    // Add new routes
    for (lang, route) in routes {
        let key = format!("{}@{}", layout, lang);
        let record = CsvRecord {
            key,
            value: route.clone(),
            description: Some(format!("{} route for {} layout", lang, layout)),
        };
        records.push(record);
    }

    // Write routes
    write_csv(routes_path, &records)?;

    Ok(())
}

/// Adds default text content for layout.
fn add_default_text(layout: &str, languages: &[String]) -> ReedResult<()> {
    let text_path = ".reed/text.csv";

    // Read existing text
    let mut records = if Path::new(text_path).exists() {
        read_csv(text_path)?
    } else {
        vec![]
    };

    // Default text entries per language
    let defaults = [
        ("title", "Title"),
        ("heading", "Heading"),
        ("description", "Description"),
        ("content", "Content"),
    ];

    for lang in languages {
        for (key_suffix, default_value) in &defaults {
            let key = format!("{}.{}@{}", layout, key_suffix, lang);
            let record = CsvRecord {
                key,
                value: format!("{} {}", layout, default_value),
                description: Some(format!("{} {} text", layout, key_suffix)),
            };
            records.push(record);
        }
    }

    // Write text
    write_csv(text_path, &records)?;

    Ok(())
}

/// Adds default meta data for layout.
fn add_default_meta(layout: &str) -> ReedResult<()> {
    let meta_path = ".reed/meta.csv";

    // Read existing meta
    let mut records = if Path::new(meta_path).exists() {
        read_csv(meta_path)?
    } else {
        vec![]
    };

    // Default meta entries
    let defaults = [
        (format!("{}.cache.ttl", layout), "3600"),
        (format!("{}.cache.enabled", layout), "true"),
    ];

    for (key, value) in &defaults {
        let record = CsvRecord {
            key: key.clone(),
            value: value.to_string(),
            description: Some(format!("{} meta", layout)),
        };
        records.push(record);
    }

    // Write meta
    write_csv(meta_path, &records)?;

    Ok(())
}
