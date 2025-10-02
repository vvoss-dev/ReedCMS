// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Custom MiniJinja functions for component inclusion.
//!
//! Provides path resolution functions for Atomic Design component hierarchy.

/// Creates organism component path resolver function.
///
/// ## Arguments
/// - interaction_mode: Current interaction mode (mouse/touch/reader)
///
/// ## Returns
/// MiniJinja function that resolves organism names to paths
///
/// ## Example
/// ```jinja
/// {% include organism("page-header") %}
/// ```
/// Resolves to: templates/components/organisms/page-header/page-header.mouse.jinja
///
/// ## Performance
/// - O(1) string formatting
/// - < 1μs per function call
pub fn make_organism_function(
    interaction_mode: String,
) -> impl Fn(&str) -> Result<String, minijinja::Error> + Send + Sync + 'static {
    move |name: &str| -> Result<String, minijinja::Error> {
        Ok(format!(
            "templates/components/organisms/{}/{}.{}.jinja",
            name, name, interaction_mode
        ))
    }
}

/// Creates molecule component path resolver function.
///
/// ## Example
/// ```jinja
/// {% include molecule("nav-item") %}
/// ```
/// Resolves to: templates/components/molecules/nav-item/nav-item.mouse.jinja
pub fn make_molecule_function(
    interaction_mode: String,
) -> impl Fn(&str) -> Result<String, minijinja::Error> + Send + Sync + 'static {
    move |name: &str| -> Result<String, minijinja::Error> {
        Ok(format!(
            "templates/components/molecules/{}/{}.{}.jinja",
            name, name, interaction_mode
        ))
    }
}

/// Creates atom component path resolver function.
///
/// ## Example
/// ```jinja
/// {% include atom("icon-logo") %}
/// ```
/// Resolves to: templates/components/atoms/icon-logo/icon-logo.mouse.jinja
pub fn make_atom_function(
    interaction_mode: String,
) -> impl Fn(&str) -> Result<String, minijinja::Error> + Send + Sync + 'static {
    move |name: &str| -> Result<String, minijinja::Error> {
        Ok(format!(
            "templates/components/atoms/{}/{}.{}.jinja",
            name, name, interaction_mode
        ))
    }
}

/// Creates layout path resolver function.
///
/// ## Note
/// Layouts do NOT use interaction_mode variants.
///
/// ## Example
/// ```jinja
/// {% extends layout("page") %}
/// ```
/// Resolves to: templates/layouts/page/page.jinja
pub fn make_layout_function(
) -> impl Fn(&str) -> Result<String, minijinja::Error> + Send + Sync + 'static {
    move |name: &str| -> Result<String, minijinja::Error> {
        Ok(format!("templates/layouts/{}/{}.jinja", name, name))
    }
}
