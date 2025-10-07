// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! MiniJinja template engine configuration.
//!
//! Sets up template environment with filters, functions, and loaders.

use crate::reedcms::filters;
use crate::reedcms::reedstream::ReedResult;
use crate::reedcms::templates::functions;
use minijinja::{AutoEscape, Environment, Error, UndefinedBehavior};
use std::path::Path;

/// Initialises MiniJinja template engine.
///
/// ## Arguments
/// - current_lang: Current language from URL (e.g., "de", "en")
/// - interaction_mode: Current interaction mode (mouse/touch/reader)
///
/// ## Configuration
/// - Template directory: templates/
/// - Auto-escape: Enabled for HTML (.jinja, .html files)
/// - Strict mode: Enabled (undefined variables error)
/// - Filters: text, route, meta, config
/// - Functions: organism, molecule, atom, layout
///
/// ## Environment Detection
/// - DEV: Hot-reload enabled via minijinja loader
/// - PROD: Static template loading
///
/// ## Performance
/// - Initialisation: < 50ms
/// - Template loading: < 10ms per template
///
/// ## Output
/// - Configured MiniJinja Environment
pub fn init_template_engine(
    current_lang: String,
    interaction_mode: String,
) -> ReedResult<Environment<'static>> {
    let mut env = Environment::new();

    // Set template loader
    env.set_loader(template_loader);

    // Register custom filters
    env.add_filter(
        "text",
        filters::text::make_text_filter(current_lang.clone()),
    );
    env.add_filter(
        "route",
        filters::route::make_route_filter(current_lang.clone()),
    );
    env.add_filter("meta", filters::meta::make_meta_filter());
    env.add_filter("config", filters::config::make_config_filter());

    // Register custom functions for component inclusion
    env.add_function(
        "organism",
        functions::make_organism_function(interaction_mode.clone()),
    );
    env.add_function(
        "molecule",
        functions::make_molecule_function(interaction_mode.clone()),
    );
    env.add_function(
        "atom",
        functions::make_atom_function(interaction_mode.clone()),
    );
    env.add_function("layout", functions::make_layout_function());

    // Configure auto-escape for HTML
    env.set_auto_escape_callback(|name| {
        if name.ends_with(".jinja") || name.ends_with(".html") {
            AutoEscape::Html
        } else {
            AutoEscape::None
        }
    });

    // Enable strict mode (undefined variables error)
    env.set_undefined_behavior(UndefinedBehavior::Strict);

    Ok(env)
}

/// Template loader function.
///
/// ## Process
/// 1. Resolve template path from name
/// 2. Check template existence
/// 3. Read template content
/// 4. Return content for MiniJinja parsing
///
/// ## Template Path Resolution
/// - Input: "layouts/knowledge/knowledge.mouse.jinja"
/// - Path: templates/layouts/knowledge/knowledge.mouse.jinja
///
/// ## Note
/// The name parameter is the full relative path from templates/ directory.
pub fn template_loader(name: &str) -> Result<Option<String>, Error> {
    let path = format!("templates/{}", name);

    if !Path::new(&path).exists() {
        return Ok(None);
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(Some(content)),
        Err(e) => Err(Error::new(
            minijinja::ErrorKind::CannotDeserialize,
            format!("Failed to read template {}: {}", name, e),
        )),
    }
}
