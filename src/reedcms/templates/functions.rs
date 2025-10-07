// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

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
/// Resolves to: components/organisms/page-header/page-header.mouse.jinja
///
/// ## Performance
/// - O(1) string formatting
/// - < 1μs per function call
pub fn make_organism_function(
    interaction_mode: String,
) -> impl Fn(&str) -> String + Send + Sync + 'static {
    move |name: &str| -> String {
        format!(
            "components/organisms/{}/{}.{}.jinja",
            name, name, interaction_mode
        )
    }
}

/// Creates molecule component path resolver function.
///
/// ## Example
/// ```jinja
/// {% include molecule("nav-item") %}
/// ```
/// Resolves to: components/molecules/nav-item/nav-item.mouse.jinja
pub fn make_molecule_function(
    interaction_mode: String,
) -> impl Fn(&str) -> String + Send + Sync + 'static {
    move |name: &str| -> String {
        format!(
            "components/molecules/{}/{}.{}.jinja",
            name, name, interaction_mode
        )
    }
}

/// Creates atom component path resolver function.
///
/// ## Example
/// ```jinja
/// {% include atom("icon-logo") %}
/// ```
/// Resolves to: components/atoms/icon-logo/icon-logo.mouse.jinja
pub fn make_atom_function(
    interaction_mode: String,
) -> impl Fn(&str) -> String + Send + Sync + 'static {
    move |name: &str| -> String {
        format!(
            "components/atoms/{}/{}.{}.jinja",
            name, name, interaction_mode
        )
    }
}

/// Creates layout path resolver function with variant support.
///
/// ## Arguments
/// - interaction_mode: Current interaction mode (mouse/touch/reader)
///
/// ## Resolution Order
/// 1. Try variant-specific: layouts/{name}/{name}.{variant}.jinja
/// 2. Fallback: layouts/{name}/{name}.jinja (legacy support)
///
/// ## Example
/// ```jinja
/// {% extends layout("page") %}
/// ```
/// Resolves to: layouts/page/page.mouse.jinja (or page.jinja if variant doesn't exist)
pub fn make_layout_function(
    interaction_mode: String,
) -> impl Fn(&str) -> String + Send + Sync + 'static {
    move |name: &str| -> String {
        // Try variant-specific layout first
        let variant_path = format!("layouts/{}/{}.{}.jinja", name, name, interaction_mode);
        if std::path::Path::new(&format!("templates/{}", variant_path)).exists() {
            return variant_path;
        }

        // Fallback to non-variant layout (legacy support)
        format!("layouts/{}/{}.jinja", name, name)
    }
}
