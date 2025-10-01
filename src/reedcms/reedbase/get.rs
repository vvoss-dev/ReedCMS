// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase Get Service
//!
//! Provides get() function for O(1) key-value lookups with environment fallback.

use crate::reedcms::reedstream::{
    current_timestamp, not_found, ReedRequest, ReedResponse, ReedResult,
};
use std::collections::HashMap;

/// Retrieves a value by key from the cache.
///
/// ## Input
/// - `request`: ReedRequest with key and optional environment
/// - `cache`: Reference to HashMap cache
///
/// ## Output
/// - `ReedResult<ReedResponse<String>>`: Value or error
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100μs typical
///
/// ## Behaviour
/// - Attempts to find exact key match first
/// - If environment is specified and key not found, tries fallback:
///   - `key@env` → `key`
/// - Returns NotFound error if key doesn't exist
/// - Marks response as cached=true
///
/// ## Error Conditions
/// - Returns `ReedError::NotFound` if key doesn't exist (with or without environment)
///
/// ## Example Usage
/// ```
/// let request = ReedRequest {
///     key: "page.title".to_string(),
///     language: Some("en".to_string()),
///     environment: Some("dev".to_string()),
///     context: None,
///     value: None,
///     description: None,
/// };
/// let cache = HashMap::new(); // Pre-populated cache
/// let response = get(request, &cache)?;
/// ```
pub fn get(
    request: ReedRequest,
    cache: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    // Build full key with language if provided
    let base_key = if let Some(lang) = &request.language {
        format!("{}@{}", request.key, lang)
    } else {
        request.key.clone()
    };

    // Try with environment suffix first if provided
    let lookup_key = if let Some(env) = &request.environment {
        format!("{}@{}", base_key, env)
    } else {
        base_key.clone()
    };

    // Attempt lookup with environment
    if let Some(value) = cache.get(&lookup_key) {
        return Ok(ReedResponse {
            data: value.clone(),
            source: lookup_key,
            cached: true,
            timestamp: current_timestamp(),
            metrics: None,
        });
    }

    // Fallback: try without environment suffix
    if request.environment.is_some() {
        if let Some(value) = cache.get(&base_key) {
            return Ok(ReedResponse {
                data: value.clone(),
                source: base_key,
                cached: true,
                timestamp: current_timestamp(),
                metrics: None,
            });
        }
    }

    // Not found
    Err(not_found(&request.key).with_context(format!(
        "language={:?}, environment={:?}",
        request.language, request.environment
    )))
}
