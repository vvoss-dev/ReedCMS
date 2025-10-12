// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Route filter for MiniJinja templates.
//!
//! Retrieves route URLs from ReedBase with language support.

use crate::reedcms::reedstream::{ReedError, ReedRequest};

/// Creates route filter for MiniJinja.
///
/// ## Usage in Templates
/// ```jinja
/// {{ "knowledge" | route("de") }}
/// {{ "blog" | route("auto") }}
/// ```
///
/// ## Arguments
/// - current_lang: Current language from URL (injected at filter creation)
///
/// ## Returns
/// Filter function that accepts:
/// - key: Layout key (e.g., "knowledge", "blog", "landing")
/// - lang: Language code or "auto" for context-based detection
///
/// ## Output
/// - Route path segment (e.g., "wissen", "portfolio")
/// - Empty string for landing page (root)
///
/// ## Empty Route Handling
/// Landing page routes are stored as empty values in routes.csv:
/// - landing@de||German homepage route (empty = root)
/// - landing@en||English homepage route (empty = root)
///
/// Filter returns empty string for landing, allowing templates to use:
/// <a href="/{{ client.lang }}/{{ pagekey | route('auto') }}/">
///
/// Results:
/// - landing → /de/ (empty route)
/// - knowledge → /de/wissen/
/// - portfolio → /de/portfolio/
///
/// ## Performance
/// - O(1) HashMap lookup via ReedBase
/// - < 100μs per filter call
pub fn make_route_filter(
    current_lang: String,
) -> impl Fn(&str, Option<&str>) -> Result<String, minijinja::Error> + Send + Sync + 'static {
    move |key: &str, lang_param: Option<&str>| -> Result<String, minijinja::Error> {
        // Resolve 'auto' to current request language (from URL)
        let resolved_lang = match lang_param {
            Some("auto") => &current_lang, // Use URL language
            Some(explicit) => explicit,    // Explicit override
            None => &current_lang,         // Default to URL language
        };

        // Build ReedRequest for route retrieval
        let req = ReedRequest {
            key: key.to_string(),
            language: Some(resolved_lang.to_string()),
            environment: None,
            context: None,
            value: None,
            description: None,
        };

        // Call ReedBase route service
        // Legacy pattern: Return key as fallback if not found
        match crate::reedcms::reedbase::get::route(&req) {
            Ok(response) => {
                // Handle empty route (landing page) - return empty string
                // Template will construct: /de/ + "" + / → /de/
                if response.data.is_empty() {
                    Ok(String::new())
                } else {
                    // Return route segment only (no leading/trailing slashes)
                    // Template will construct: /de/ + "wissen" + / → /de/wissen/
                    Ok(response.data)
                }
            }
            Err(_) => Ok(key.to_string()), // Fallback: return key itself
        }
    }
}

/// Converts ReedError to MiniJinja Error with context.
#[allow(dead_code)]
fn convert_reed_error_to_jinja(err: ReedError, filter: &str, key: &str) -> minijinja::Error {
    use minijinja::ErrorKind;

    match err {
        ReedError::NotFound { resource, context } => {
            let msg = format!(
                "Filter '{}': {} not found for key '{}' (context: {:?})",
                filter, resource, key, context
            );
            minijinja::Error::new(ErrorKind::TemplateNotFound, msg)
        }
        ReedError::ValidationError {
            field,
            value,
            constraint,
        } => {
            let msg = format!(
                "Filter '{}': Validation error for key '{}': field={}, value={}, constraint={}",
                filter, key, field, value, constraint
            );
            minijinja::Error::new(ErrorKind::InvalidOperation, msg)
        }
        ReedError::IoError {
            operation,
            path,
            reason,
        } => {
            let msg = format!(
                "Filter '{}': IO error during {} on '{}': {}",
                filter, operation, path, reason
            );
            minijinja::Error::new(ErrorKind::InvalidOperation, msg)
        }
        _ => {
            let msg = format!("Filter '{}': Error for key '{}': {:?}", filter, key, err);
            minijinja::Error::new(ErrorKind::InvalidOperation, msg)
        }
    }
}
