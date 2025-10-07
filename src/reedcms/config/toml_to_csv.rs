// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Synchronises Reed.toml configuration to .reed/*.csv files.
//!
//! This module provides the sync functionality that writes Reed.toml values
//! to the appropriate CSV files using the established CLI commands.

use crate::reedcms::config::toml_parser::ReedConfig;
use crate::reedcms::reedbase::set::set;
use crate::reedcms::reedstream::{ReedRequest, ReedResult};
use std::collections::HashMap;
use std::path::Path;

/// Synchronises all values from Reed.toml to .reed/*.csv files.
///
/// ## Input
/// - config: Parsed ReedConfig structure from Reed.toml
///
/// ## Output
/// - Vec<String>: List of all keys that were updated
///
/// ## Process
/// 1. Validates configuration structure
/// 2. Writes [project] values to .reed/project.csv using set_project()
/// 3. Writes [server] values to .reed/server.csv using set_server()
/// 4. Returns list of updated keys for reporting
///
/// ## Error Conditions
/// - CSV file not accessible
/// - Invalid key/value format
/// - Permission denied
///
/// ## Example Usage
/// ```rust
/// let config = parse_reed_toml("Reed.toml")?;
/// let updated_keys = sync_toml_to_csv(&config)?;
/// println!("Updated {} keys", updated_keys.len());
/// ```
pub fn sync_toml_to_csv(config: &ReedConfig) -> ReedResult<Vec<String>> {
    let mut updated_keys = Vec::new();

    // Sync [project] section
    updated_keys.extend(sync_project_config(config)?);

    // Sync [server] section
    updated_keys.extend(sync_server_config(config)?);

    Ok(updated_keys)
}

/// Synchronises [project] section to .reed/project.csv.
fn sync_project_config(config: &ReedConfig) -> ReedResult<Vec<String>> {
    let mut updated = Vec::new();
    let mut cache = HashMap::new();

    // Load existing project.csv if it exists
    let project_csv = ".reed/project.csv";
    if Path::new(project_csv).exists() {
        use crate::reedcms::csv::read_csv;
        let records = read_csv(project_csv)?;
        for record in records {
            cache.insert(record.key, record.value);
        }
    }

    // Project basic settings
    let mut values = vec![
        ("project.name", config.project.name.clone()),
        ("project.url", config.project.url.clone()),
        (
            "project.languages.default",
            config.project.languages.default.clone(),
        ),
        (
            "project.languages.available",
            config.project.languages.available.join(","),
        ),
        (
            "project.languages.fallback_chain",
            config.project.languages.fallback_chain.to_string(),
        ),
        (
            "project.routing.url_prefix",
            config.project.routing.url_prefix.to_string(),
        ),
        (
            "project.routing.trailing_slash",
            config.project.routing.trailing_slash.to_string(),
        ),
        (
            "project.templates.auto_reload",
            config.project.templates.auto_reload.to_string(),
        ),
        (
            "project.templates.cache_templates",
            config.project.templates.cache_templates.to_string(),
        ),
        (
            "project.assets.css_minify",
            config.project.assets.css_minify.to_string(),
        ),
        (
            "project.assets.css_bundle",
            config.project.assets.css_bundle.to_string(),
        ),
        (
            "project.build.clean_before",
            config.project.build.clean_before.to_string(),
        ),
        (
            "project.build.parallel",
            config.project.build.parallel.to_string(),
        ),
    ];

    // Add optional description if present
    if let Some(ref desc) = config.project.description {
        values.push(("project.description", desc.clone()));
    }

    for (key, value) in values {
        let request = ReedRequest {
            key: key.to_string(),
            language: None,
            environment: None,
            context: Some("project".to_string()),
            value: Some(value),
            description: Some(format!("{} from Reed.toml", key)),
        };

        set(request, &mut cache, project_csv)?;
        updated.push(key.to_string());
    }

    Ok(updated)
}

/// Synchronises [server] section to .reed/server.csv.
fn sync_server_config(config: &ReedConfig) -> ReedResult<Vec<String>> {
    let mut updated = Vec::new();
    let mut cache = HashMap::new();

    // Load existing server.csv if it exists
    let server_csv = ".reed/server.csv";
    if Path::new(server_csv).exists() {
        use crate::reedcms::csv::read_csv;
        let records = read_csv(server_csv)?;
        for record in records {
            cache.insert(record.key, record.value);
        }
    }

    // Server global settings
    let mut values = vec![("server.workers", config.server.workers.to_string())];

    // Dev environment settings
    values.extend(vec![
        ("server.dev.domain", config.server.dev.domain.clone()),
        ("server.dev.io", config.server.dev.io.clone()),
        (
            "server.dev.enable_cors",
            config.server.dev.enable_cors.to_string(),
        ),
        (
            "server.dev.allowed_origins",
            config.server.dev.allowed_origins.join(","),
        ),
        (
            "server.dev.enable_rate_limit",
            config.server.dev.enable_rate_limit.to_string(),
        ),
        (
            "server.dev.requests_per_minute",
            config.server.dev.requests_per_minute.to_string(),
        ),
        (
            "server.dev.enable_compression",
            config.server.dev.enable_compression.to_string(),
        ),
        (
            "server.dev.enable_http2",
            config.server.dev.enable_http2.to_string(),
        ),
        (
            "server.dev.keep_alive",
            config.server.dev.keep_alive.to_string(),
        ),
    ]);

    // Prod environment settings
    values.extend(vec![
        ("server.prod.domain", config.server.prod.domain.clone()),
        ("server.prod.io", config.server.prod.io.clone()),
        (
            "server.prod.enable_cors",
            config.server.prod.enable_cors.to_string(),
        ),
        (
            "server.prod.allowed_origins",
            config.server.prod.allowed_origins.join(","),
        ),
        (
            "server.prod.enable_rate_limit",
            config.server.prod.enable_rate_limit.to_string(),
        ),
        (
            "server.prod.requests_per_minute",
            config.server.prod.requests_per_minute.to_string(),
        ),
        (
            "server.prod.enable_compression",
            config.server.prod.enable_compression.to_string(),
        ),
        (
            "server.prod.enable_http2",
            config.server.prod.enable_http2.to_string(),
        ),
        (
            "server.prod.keep_alive",
            config.server.prod.keep_alive.to_string(),
        ),
    ]);

    for (key, value) in values {
        let request = ReedRequest {
            key: key.to_string(),
            language: None,
            environment: None,
            context: Some("server".to_string()),
            value: Some(value),
            description: Some(format!("{} from Reed.toml", key)),
        };

        set(request, &mut cache, server_csv)?;
        updated.push(key.to_string());
    }

    Ok(updated)
}
