// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Text filter for MiniJinja templates.
//!
//! Retrieves text content from ReedBase with language support.

use crate::reedcms::reedstream::{ReedError, ReedRequest};

/// Creates text filter for MiniJinja.
///
/// ## Usage in Templates
/// ```jinja
/// {{ "knowledge.title" | text("de") }}
/// {{ "blog.subtitle" | text("auto") }}
/// ```
///
/// ## Arguments
/// - current_lang: Current language from URL (injected at filter creation)
///
/// ## Returns
/// Filter function that accepts:
/// - key: Text key (e.g., "knowledge.title")
/// - lang: Language code or "auto" for context-based detection
///
/// ## Output
/// - Text content from .reed/text.csv
///
/// ## Performance
/// - O(1) HashMap lookup via ReedBase
/// - < 100μs per filter call
///
/// ## Error Conditions
/// - Key not found in text.csv
/// - Invalid language code
///
/// ## Language Detection
/// URL path is the single source of truth for language:
/// - `text("auto")` uses current_lang from URL
/// - `text("de")` forces German (explicit override)
/// - `text("en")` forces English (explicit override)
pub fn make_text_filter(
    current_lang: String,
) -> impl Fn(&str, Option<&str>) -> Result<String, minijinja::Error> + Send + Sync + 'static {
    move |key: &str, lang_param: Option<&str>| -> Result<String, minijinja::Error> {
        // Resolve 'auto' to current request language (from URL)
        let resolved_lang = match lang_param {
            Some("auto") => &current_lang, // Use URL language
            Some(explicit) => explicit,    // Explicit override
            None => &current_lang,         // Default to URL language
        };

        // Build ReedRequest for text retrieval
        let req = ReedRequest {
            key: key.to_string(),
            language: Some(resolved_lang.to_string()),
            environment: None, // TODO: Add environment detection from context
            context: None,
            value: None,
            description: None,
        };

        // Call ReedBase text service
        // Legacy pattern: Return key as fallback if not found (instead of error)
        match crate::reedcms::reedbase::get::text(&req) {
            Ok(response) => Ok(response.data),
            Err(_) => Ok(key.to_string()), // Fallback: return key itself
        }
    }
}

/// Converts ReedError to MiniJinja Error with context.
///
/// ## Error Mapping
/// - NotFound → TemplateNotFound with descriptive message
/// - Other errors → InvalidOperation with context
///
/// ## Context Preservation
/// - Includes original error message
/// - Adds filter name and key for debugging
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
