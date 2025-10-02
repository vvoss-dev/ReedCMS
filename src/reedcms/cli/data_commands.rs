// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CLI Data Commands
//!
//! Provides CLI command handlers for ReedBase data operations:
//! - set:text, set:route, set:meta, set:server, set:project
//! - get:text, get:route, get:meta, get:server, get:project
//! - list:text, list:route, list:meta

use crate::reedcms::csv::read_csv;
// ReedBase functions accessed via full path
use crate::reedcms::reedstream::{ReedError, ReedRequest, ReedResponse, ReedResult};
use std::collections::HashMap;

/// Sets text content via CLI.
///
/// ## Arguments
/// - args[0]: key@lang (e.g., "knowledge.title@en")
/// - args[1]: value
/// - flags["desc"]: Description (mandatory)
///
/// ## Output
/// - Success message with key and value
///
/// ## Performance
/// - < 50ms including CSV write
///
/// ## Error Conditions
/// - Missing arguments
/// - Missing --desc flag
/// - Invalid key format
///
/// ## Example Usage
/// ```bash
/// reed set:text knowledge.title@en "Knowledge Base" --desc "Main page title"
/// ```
pub fn set_text(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    // Validate arguments
    if args.len() < 2 {
        return Err(ReedError::InvalidCommand {
            command: "set:text".to_string(),
            reason: "Requires 2 arguments: key@lang value".to_string(),
        });
    }

    // Validate --desc flag
    let description = flags.get("desc").ok_or_else(|| ReedError::InvalidCommand {
        command: "set:text".to_string(),
        reason: "--desc flag is mandatory (e.g., --desc \"Page title\")".to_string(),
    })?;

    if description.len() < 10 {
        return Err(ReedError::ValidationError {
            field: "desc".to_string(),
            value: description.clone(),
            constraint: "Description must be at least 10 characters".to_string(),
        });
    }

    // Parse key@lang
    let full_key = &args[0];
    let value = &args[1];

    // Load cache from CSV
    let csv_path = ".reed/text.csv";
    let records = read_csv(csv_path)?;
    let mut cache: HashMap<String, String> =
        records.into_iter().map(|r| (r.key, r.value)).collect();

    // Build request
    let request = ReedRequest {
        key: full_key.clone(),
        language: None,
        environment: None,
        context: Some("text".to_string()),
        value: Some(value.clone()),
        description: Some(description.clone()),
    };

    // Call ReedBase set
    let response = crate::reedcms::reedbase::set::set(request, &mut cache, csv_path)?;

    // Format output
    let output = format!(
        "✓ Text set: {} = \"{}\"\n  Description: {}",
        full_key, value, description
    );

    Ok(ReedResponse {
        data: output,
        source: "cli::data_commands::set_text".to_string(),
        cached: false,
        timestamp: response.timestamp,
        metrics: None,
    })
}

/// Sets route mapping via CLI.
///
/// ## Arguments
/// - args[0]: key@lang (e.g., "knowledge@en")
/// - args[1]: route (e.g., "knowledge")
/// - flags["desc"]: Description (mandatory)
pub fn set_route(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.len() < 2 {
        return Err(ReedError::InvalidCommand {
            command: "set:route".to_string(),
            reason: "Requires 2 arguments: key@lang route".to_string(),
        });
    }

    let description = flags.get("desc").ok_or_else(|| ReedError::InvalidCommand {
        command: "set:route".to_string(),
        reason: "--desc flag is mandatory".to_string(),
    })?;

    if description.len() < 10 {
        return Err(ReedError::ValidationError {
            field: "desc".to_string(),
            value: description.clone(),
            constraint: "Description must be at least 10 characters".to_string(),
        });
    }

    let full_key = &args[0];
    let route = &args[1];

    let csv_path = ".reed/routes.csv";
    let records = read_csv(csv_path)?;
    let mut cache: HashMap<String, String> =
        records.into_iter().map(|r| (r.key, r.value)).collect();

    let request = ReedRequest {
        key: full_key.clone(),
        language: None,
        environment: None,
        context: Some("route".to_string()),
        value: Some(route.clone()),
        description: Some(description.clone()),
    };

    let response = crate::reedcms::reedbase::set::set(request, &mut cache, csv_path)?;

    let output = format!(
        "✓ Route set: {} → \"{}\"\n  Description: {}",
        full_key, route, description
    );

    Ok(ReedResponse {
        data: output,
        source: "cli::data_commands::set_route".to_string(),
        cached: false,
        timestamp: response.timestamp,
        metrics: None,
    })
}

/// Sets metadata via CLI.
///
/// ## Arguments
/// - args[0]: key (e.g., "knowledge.cache.ttl")
/// - args[1]: value
/// - flags["desc"]: Description (mandatory)
pub fn set_meta(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.len() < 2 {
        return Err(ReedError::InvalidCommand {
            command: "set:meta".to_string(),
            reason: "Requires 2 arguments: key value".to_string(),
        });
    }

    let description = flags.get("desc").ok_or_else(|| ReedError::InvalidCommand {
        command: "set:meta".to_string(),
        reason: "--desc flag is mandatory".to_string(),
    })?;

    if description.len() < 10 {
        return Err(ReedError::ValidationError {
            field: "desc".to_string(),
            value: description.clone(),
            constraint: "Description must be at least 10 characters".to_string(),
        });
    }

    let key = &args[0];
    let value = &args[1];

    let csv_path = ".reed/meta.csv";
    let records = read_csv(csv_path)?;
    let mut cache: HashMap<String, String> =
        records.into_iter().map(|r| (r.key, r.value)).collect();

    let request = ReedRequest {
        key: key.clone(),
        language: None,
        environment: None,
        context: Some("meta".to_string()),
        value: Some(value.clone()),
        description: Some(description.clone()),
    };

    let response = crate::reedcms::reedbase::set::set(request, &mut cache, csv_path)?;

    let output = format!(
        "✓ Meta set: {} = \"{}\"\n  Description: {}",
        key, value, description
    );

    Ok(ReedResponse {
        data: output,
        source: "cli::data_commands::set_meta".to_string(),
        cached: false,
        timestamp: response.timestamp,
        metrics: None,
    })
}

/// Gets text content via CLI.
///
/// ## Arguments
/// - args[0]: key@lang (e.g., "knowledge.title@en")
///
/// ## Output
/// - Retrieved value with source indication
///
/// ## Performance
/// - < 10ms
///
/// ## Example Usage
/// ```bash
/// reed get:text knowledge.title@en
/// ```
pub fn get_text(args: &[String]) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: "get:text".to_string(),
            reason: "Requires 1 argument: key@lang".to_string(),
        });
    }

    let full_key = &args[0];

    // Load cache from CSV
    let csv_path = ".reed/text.csv";
    let records = read_csv(csv_path)?;
    let cache: HashMap<String, String> = records.into_iter().map(|r| (r.key, r.value)).collect();

    // Build request
    let request = ReedRequest {
        key: full_key.clone(),
        language: None,
        environment: None,
        context: Some("text".to_string()),
        value: None,
        description: None,
    };

    // Call ReedBase get
    let response = crate::reedcms::reedbase::get::get(request, &cache)?;

    // Format output
    let output = format!("{}\n(source: {})", response.data, response.source);

    Ok(ReedResponse {
        data: output,
        source: "cli::data_commands::get_text".to_string(),
        cached: false,
        timestamp: response.timestamp,
        metrics: None,
    })
}

/// Gets route mapping via CLI.
///
/// ## Arguments
/// - args[0]: key@lang (e.g., "knowledge@en")
pub fn get_route(args: &[String]) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: "get:route".to_string(),
            reason: "Requires 1 argument: key@lang".to_string(),
        });
    }

    let full_key = &args[0];
    let csv_path = ".reed/routes.csv";
    let records = read_csv(csv_path)?;
    let cache: HashMap<String, String> = records.into_iter().map(|r| (r.key, r.value)).collect();

    let request = ReedRequest {
        key: full_key.clone(),
        language: None,
        environment: None,
        context: Some("route".to_string()),
        value: None,
        description: None,
    };

    let response = crate::reedcms::reedbase::get::get(request, &cache)?;
    let output = format!("{}\n(source: {})", response.data, response.source);

    Ok(ReedResponse {
        data: output,
        source: "cli::data_commands::get_route".to_string(),
        cached: false,
        timestamp: response.timestamp,
        metrics: None,
    })
}

/// Gets metadata via CLI.
///
/// ## Arguments
/// - args[0]: key (e.g., "knowledge.cache.ttl")
pub fn get_meta(args: &[String]) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: "get:meta".to_string(),
            reason: "Requires 1 argument: key".to_string(),
        });
    }

    let key = &args[0];
    let csv_path = ".reed/meta.csv";
    let records = read_csv(csv_path)?;
    let cache: HashMap<String, String> = records.into_iter().map(|r| (r.key, r.value)).collect();

    let request = ReedRequest {
        key: key.clone(),
        language: None,
        environment: None,
        context: Some("meta".to_string()),
        value: None,
        description: None,
    };

    let response = crate::reedcms::reedbase::get::get(request, &cache)?;
    let output = format!("{}\n(source: {})", response.data, response.source);

    Ok(ReedResponse {
        data: output,
        source: "cli::data_commands::get_meta".to_string(),
        cached: false,
        timestamp: response.timestamp,
        metrics: None,
    })
}

/// Lists text keys matching pattern.
///
/// ## Arguments
/// - args[0]: Optional pattern (default: "*")
///
/// ## Output
/// - List of matching keys with count
///
/// ## Performance
/// - < 100ms for 1000 entries
///
/// ## Example Usage
/// ```bash
/// reed list:text "knowledge.*"
/// ```
pub fn list_text(args: &[String]) -> ReedResult<ReedResponse<String>> {
    let pattern = if args.is_empty() {
        "*".to_string()
    } else {
        args[0].clone()
    };

    let csv_path = ".reed/text.csv";
    let records = read_csv(csv_path)?;

    // Filter keys by pattern
    let matching_keys: Vec<String> = records
        .iter()
        .filter(|r| matches_pattern(&r.key, &pattern))
        .map(|r| r.key.clone())
        .collect();

    // Format output
    let mut output = String::new();
    for key in &matching_keys {
        output.push_str(key);
        output.push('\n');
    }
    output.push_str(&format!("({} entries found)", matching_keys.len()));

    Ok(ReedResponse {
        data: output,
        source: "cli::data_commands::list_text".to_string(),
        cached: false,
        timestamp: crate::reedcms::reedstream::current_timestamp(),
        metrics: None,
    })
}

/// Lists route keys matching pattern.
///
/// ## Arguments
/// - args[0]: Optional pattern (default: "*")
pub fn list_route(args: &[String]) -> ReedResult<ReedResponse<String>> {
    let pattern = if args.is_empty() {
        "*".to_string()
    } else {
        args[0].clone()
    };

    let csv_path = ".reed/routes.csv";
    let records = read_csv(csv_path)?;

    let matching_keys: Vec<String> = records
        .iter()
        .filter(|r| matches_pattern(&r.key, &pattern))
        .map(|r| r.key.clone())
        .collect();

    let mut output = String::new();
    for key in &matching_keys {
        output.push_str(key);
        output.push('\n');
    }
    output.push_str(&format!("({} entries found)", matching_keys.len()));

    Ok(ReedResponse {
        data: output,
        source: "cli::data_commands::list_route".to_string(),
        cached: false,
        timestamp: crate::reedcms::reedstream::current_timestamp(),
        metrics: None,
    })
}

/// Lists meta keys matching pattern.
///
/// ## Arguments
/// - args[0]: Optional pattern (default: "*")
pub fn list_meta(args: &[String]) -> ReedResult<ReedResponse<String>> {
    let pattern = if args.is_empty() {
        "*".to_string()
    } else {
        args[0].clone()
    };

    let csv_path = ".reed/meta.csv";
    let records = read_csv(csv_path)?;

    let matching_keys: Vec<String> = records
        .iter()
        .filter(|r| matches_pattern(&r.key, &pattern))
        .map(|r| r.key.clone())
        .collect();

    let mut output = String::new();
    for key in &matching_keys {
        output.push_str(key);
        output.push('\n');
    }
    output.push_str(&format!("({} entries found)", matching_keys.len()));

    Ok(ReedResponse {
        data: output,
        source: "cli::data_commands::list_meta".to_string(),
        cached: false,
        timestamp: crate::reedcms::reedstream::current_timestamp(),
        metrics: None,
    })
}

/// Matches a key against a glob-style pattern.
///
/// ## Supported Patterns
/// - "*" - matches all
/// - "prefix.*" - matches keys starting with prefix
/// - "*.suffix" - matches keys ending with suffix
/// - "exact" - exact match only
fn matches_pattern(key: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.starts_with('*') && pattern.ends_with('*') {
        // *middle* - contains
        let middle = &pattern[1..pattern.len() - 1];
        return key.contains(middle);
    }

    if pattern.starts_with('*') {
        // *.suffix - ends with
        let suffix = &pattern[1..];
        return key.ends_with(suffix);
    }

    if pattern.ends_with('*') {
        // prefix.* - starts with
        let prefix = &pattern[..pattern.len() - 1];
        return key.starts_with(prefix);
    }

    // Exact match
    key == pattern
}
