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

/// Retrieves text content from ReedBase.
///
/// ## Input
/// - `request`: ReedRequest with key and language
///
/// ## Output
/// - `ReedResult<ReedResponse<String>>`: Text value
///
/// ## Performance
/// - O(1) HashMap lookup (when cache is implemented)
/// - < 100μs typical
///
/// ## Note
/// Placeholder implementation - reads directly from .reed/text.csv
/// Full ReedBase cache implementation pending REED-02-01
pub fn text(request: &ReedRequest) -> ReedResult<ReedResponse<String>> {
    read_from_csv(".reed/text.csv", request)
}

/// Retrieves route from ReedBase.
///
/// ## Input
/// - `request`: ReedRequest with key and language
///
/// ## Output
/// - `ReedResult<ReedResponse<String>>`: Route path segment
///
/// ## Performance
/// - O(1) HashMap lookup (when cache is implemented)
/// - < 100μs typical
///
/// ## Note
/// Placeholder implementation - reads directly from .reed/routes.csv
/// Full ReedBase cache implementation pending REED-02-01
pub fn route(request: &ReedRequest) -> ReedResult<ReedResponse<String>> {
    read_from_csv(".reed/routes.csv", request)
}

/// Retrieves metadata from ReedBase.
///
/// ## Input
/// - `request`: ReedRequest with key (no language)
///
/// ## Output
/// - `ReedResult<ReedResponse<String>>`: Meta value
///
/// ## Performance
/// - O(1) HashMap lookup (when cache is implemented)
/// - < 100μs typical
///
/// ## Note
/// Placeholder implementation - reads directly from .reed/meta.csv
/// Full ReedBase cache implementation pending REED-02-01
pub fn meta(request: &ReedRequest) -> ReedResult<ReedResponse<String>> {
    read_from_csv(".reed/meta.csv", request)
}

/// Retrieves project configuration from ReedBase.
///
/// ## Input
/// - `request`: ReedRequest with key (no language)
///
/// ## Output
/// - `ReedResult<ReedResponse<String>>`: Config value
///
/// ## Performance
/// - O(1) HashMap lookup (when cache is implemented)
/// - < 100μs typical
///
/// ## Note
/// Placeholder implementation - reads directly from .reed/project.csv
/// Full ReedBase cache implementation pending REED-02-01
pub fn project(request: &ReedRequest) -> ReedResult<ReedResponse<String>> {
    read_from_csv(".reed/project.csv", request)
}

/// Retrieves server configuration from ReedBase.
///
/// ## Input
/// - `request`: ReedRequest with key (no language)
///
/// ## Output
/// - `ReedResult<ReedResponse<String>>`: Config value
///
/// ## Performance
/// - O(1) HashMap lookup (when cache is implemented)
/// - < 100μs typical
///
/// ## Note
/// Placeholder implementation - reads directly from .reed/server.csv
/// Full ReedBase cache implementation pending REED-02-01
pub fn server(request: &ReedRequest) -> ReedResult<ReedResponse<String>> {
    read_from_csv(".reed/server.csv", request)
}

/// Reads value from CSV file (placeholder implementation).
///
/// ## Process
/// 1. Read CSV file line by line
/// 2. Parse pipe-delimited format (key|value|description)
/// 3. Match key with language and environment fallback
/// 4. Return value if found
///
/// ## Note
/// This is a temporary implementation. Full ReedBase with HashMap cache
/// will replace this in REED-02-01 for O(1) performance.
fn read_from_csv(csv_path: &str, request: &ReedRequest) -> ReedResult<ReedResponse<String>> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

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

    // Open CSV file
    let file =
        File::open(csv_path).map_err(|e| crate::reedcms::reedstream::ReedError::IoError {
            operation: "read_csv".to_string(),
            path: csv_path.to_string(),
            reason: e.to_string(),
        })?;

    let reader = BufReader::new(file);

    // Search for key in CSV
    for line_result in reader.lines() {
        let line = line_result.map_err(|e| crate::reedcms::reedstream::ReedError::IoError {
            operation: "read_line".to_string(),
            path: csv_path.to_string(),
            reason: e.to_string(),
        })?;

        // Skip empty lines and comments
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse pipe-delimited format: key|value|description
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 2 {
            continue;
        }

        let file_key = parts[0].trim();
        let value = parts[1].trim();

        // Try exact match with environment
        if file_key == lookup_key {
            return Ok(ReedResponse {
                data: value.to_string(),
                source: lookup_key,
                cached: false,
                timestamp: current_timestamp(),
                metrics: None,
            });
        }

        // Try fallback to base key (without environment)
        if request.environment.is_some() && file_key == base_key {
            return Ok(ReedResponse {
                data: value.to_string(),
                source: base_key,
                cached: false,
                timestamp: current_timestamp(),
                metrics: None,
            });
        }
    }

    // Not found
    Err(not_found(&request.key).with_context(format!(
        "CSV: {}, language={:?}, environment={:?}",
        csv_path, request.language, request.environment
    )))
}
