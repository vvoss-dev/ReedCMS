// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Meta filter for MiniJinja templates.
//!
//! Retrieves metadata from ReedBase.

use crate::reedcms::reedstream::{ReedError, ReedRequest};

/// Creates meta filter for MiniJinja.
///
/// ## Usage in Templates
/// ```jinja
/// {{ "knowledge.cache.ttl" | meta }}
/// {{ "blog.access.level" | meta }}
/// ```
///
/// ## Arguments
/// - key: Meta key (e.g., "layout.cache.ttl")
///
/// ## Output
/// - Meta value (string)
///
/// ## Performance
/// - O(1) HashMap lookup via ReedBase
/// - < 100μs per filter call
///
/// ## Note
/// Meta data is language-independent (no @lang suffix)
pub fn make_meta_filter(
) -> impl Fn(&str) -> Result<String, minijinja::Error> + Send + Sync + 'static {
    move |key: &str| -> Result<String, minijinja::Error> {
        // Build ReedRequest for meta retrieval
        let req = ReedRequest {
            key: key.to_string(),
            language: None, // Meta data is language-independent
            environment: None,
            context: None,
            value: None,
            description: None,
        };

        // Call ReedBase meta service
        match crate::reedcms::reedbase::get::meta(&req) {
            Ok(response) => Ok(response.data),
            Err(err) => Err(convert_reed_error_to_jinja(err, "meta", key)),
        }
    }
}

/// Converts ReedError to MiniJinja Error with context.
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
