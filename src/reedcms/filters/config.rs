// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Config filter for MiniJinja templates.
//!
//! Retrieves configuration values with automatic project./server. prefix detection.

use crate::reedcms::reedstream::{ReedError, ReedRequest};

/// Creates config filter for MiniJinja.
///
/// ## Usage in Templates
/// ```jinja
/// {{ "languages" | config }}           {# Auto-resolves to project.languages #}
/// {{ "auth.enabled" | config }}        {# Auto-resolves to server.auth.enabled #}
/// ```
///
/// ## Auto-Detection Logic
/// 1. Try project.{key}
/// 2. Try server.{key}
/// 3. Return error if neither found
///
/// ## Arguments
/// - key: Config key (without project./server. prefix)
///
/// ## Output
/// - Configuration value (string)
///
/// ## Performance
/// - 2x O(1) HashMap lookups (worst case)
/// - < 200μs per filter call
pub fn make_config_filter(
) -> impl Fn(&str) -> Result<String, minijinja::Error> + Send + Sync + 'static {
    move |key: &str| -> Result<String, minijinja::Error> {
        // Try project first
        let project_key = format!("project.{}", key);
        let req_project = ReedRequest {
            key: project_key.clone(),
            language: None,
            environment: None,
            context: None,
            value: None,
            description: None,
        };

        if let Ok(response) = crate::reedcms::reedbase::get::project(&req_project) {
            return Ok(response.data);
        }

        // Try server second
        let server_key = format!("server.{}", key);
        let req_server = ReedRequest {
            key: server_key.clone(),
            language: None,
            environment: None,
            context: None,
            value: None,
            description: None,
        };

        match crate::reedcms::reedbase::get::server(&req_server) {
            Ok(response) => Ok(response.data),
            Err(e) => Err(convert_reed_error_to_jinja(
                e,
                "config",
                &format!("{} (tried project.{} and server.{})", key, key, key),
            )),
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
