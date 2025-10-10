// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CLI commands for debug tools.
//!
//! ## Commands
//! - debug:request {url} - Inspect request
//! - debug:cache [type] [--search term] - View cache
//! - debug:route {url} - Test route
//! - debug:config - Inspect configuration

use crate::reedcms::debug;
use crate::reedcms::reedstream::{ReedError, ReedResponse, ReedResult};
use std::collections::HashMap;

/// CLI command: reed debug:request {url}
///
/// ## Usage
/// ```bash
/// reed debug:request /knowledge
/// ```
pub fn debug_request_handler(
    args: &[String],
    _flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "url".to_string(),
            value: String::new(),
            constraint: "URL argument required".to_string(),
        });
    }

    let url = &args[0];
    let inspection = debug::inspect_request(url);
    let output = inspection.format();

    Ok(ReedResponse::new(output, "debug::request"))
}

/// CLI command: reed debug:cache [type] [--search term]
///
/// ## Usage
/// ```bash
/// reed debug:cache
/// reed debug:cache text
/// reed debug:cache --search "knowledge"
/// ```
pub fn debug_cache_handler(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    let cache_type = args.first().map(|s| s.as_str());
    let search = flags.get("search").map(|s| s.as_str());

    let view = debug::view_cache(cache_type, search);
    let output = view.format();

    Ok(ReedResponse::new(output, "debug::cache"))
}

/// CLI command: reed debug:route {url}
///
/// ## Usage
/// ```bash
/// reed debug:route /knowledge
/// reed debug:route /de/wissen
/// ```
pub fn debug_route_handler(
    args: &[String],
    _flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "url".to_string(),
            value: String::new(),
            constraint: "URL argument required".to_string(),
        });
    }

    let url = &args[0];
    let test = debug::test_route(url);
    let output = test.format();

    Ok(ReedResponse::new(output, "debug::route"))
}

/// CLI command: reed debug:config
///
/// ## Usage
/// ```bash
/// reed debug:config
/// ```
pub fn debug_config_handler(
    _args: &[String],
    _flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    let inspection = debug::inspect_config();
    let output = inspection.format();

    Ok(ReedResponse::new(output, "debug::config"))
}
