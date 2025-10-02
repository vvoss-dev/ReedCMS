// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One function = One job
// MANDATORY: BBC English for all documentation and comments
// MANDATORY: Type-safe error handling with ReedResult<T> pattern
//
// == FILE PURPOSE ==
// This file: HTTP Cache-Control header generation based on layout configuration
// Architecture: Server response layer - determines caching strategy from meta.csv
// Performance: < 1ms per header generation (ReedBase cached lookup)
// Dependencies: reedbase::get::meta
// Data Flow: Layout name → meta.csv lookup → Cache-Control header

//! Cache Control Headers
//!
//! Generates HTTP Cache-Control headers based on layout configuration from meta.csv.

use crate::reedcms::reedbase::get;
use crate::reedcms::reedstream::ReedRequest;

/// Generates Cache-Control header tuple for HTTP response.
///
/// ## Input
/// - `layout`: Layout name (e.g., "knowledge", "agility")
///
/// ## Output
/// - `(&'static str, String)`: ("Cache-Control", "public, max-age=3600")
///
/// ## Cache TTL Sources
/// 1. Layout-specific: `.reed/meta.csv` → `{layout}.cache.ttl`
/// 2. Default fallback: 3600 seconds (1 hour)
///
/// ## Cache-Control Formats
/// - **Public content**: `"public, max-age={ttl}"`
/// - **Private content**: `"private, max-age={ttl}"`
/// - **No cache**: `"no-cache, no-store, must-revalidate"`
///
/// ## Public/Private Determination
/// - Checked via: `.reed/meta.csv` → `{layout}.cache.public`
/// - Values: "true" = public, "false" = private
/// - Default: public (true)
///
/// ## Performance
/// - < 1ms (ReedBase cached lookup)
///
/// ## Example Usage
/// ```rust
/// let (name, value) = cache_control_header("knowledge");
/// // ("Cache-Control", "public, max-age=3600")
///
/// response.insert_header((name, value));
/// ```
///
/// ## Example Configurations
/// - Blog posts: `knowledge.cache.ttl=3600` (1 hour)
/// - Static pages: `agility.cache.ttl=86400` (24 hours)
/// - User dashboards: `dashboard.cache.ttl=0` (no cache)
pub fn cache_control_header(layout: &str) -> (&'static str, String) {
    let ttl = get_cache_ttl(layout);

    // No caching if TTL is 0
    if ttl == 0 {
        return (
            "Cache-Control",
            "no-cache, no-store, must-revalidate".to_string(),
        );
    }

    // Determine public vs private caching
    let cache_type = if is_public_layout(layout) {
        "public"
    } else {
        "private"
    };

    ("Cache-Control", format!("{}, max-age={}", cache_type, ttl))
}

/// Gets cache TTL (Time To Live) for layout from meta.csv.
///
/// ## Input
/// - `layout`: Layout name
///
/// ## Output
/// - `u64`: TTL in seconds (default: 3600)
///
/// ## Lookup Key
/// - Format: `{layout}.cache.ttl`
/// - Example: `knowledge.cache.ttl`
///
/// ## Fallback
/// - Missing key → 3600 seconds (1 hour)
/// - Invalid value → 3600 seconds
///
/// ## Performance
/// - < 1ms (ReedBase cached)
fn get_cache_ttl(layout: &str) -> u64 {
    let key = format!("{}.cache.ttl", layout);
    let req = ReedRequest {
        key,
        language: None,
        environment: None,
        context: None,
        value: None,
        description: None,
    };

    match get::meta(&req) {
        Ok(response) => response.data.parse().unwrap_or(3600),
        Err(_) => 3600, // Default 1 hour
    }
}

/// Checks if layout is publicly cacheable.
///
/// ## Input
/// - `layout`: Layout name
///
/// ## Output
/// - `bool`: true = public cache, false = private cache
///
/// ## Lookup Key
/// - Format: `{layout}.cache.public`
/// - Example: `knowledge.cache.public`
///
/// ## Values
/// - "true" → public caching
/// - "false" → private caching
///
/// ## Fallback
/// - Missing key → true (public)
/// - Invalid value → true (public)
///
/// ## Performance
/// - < 1ms (ReedBase cached)
///
/// ## Use Cases
/// - Public: Blog posts, static pages, documentation
/// - Private: User dashboards, personalised content
fn is_public_layout(layout: &str) -> bool {
    let key = format!("{}.cache.public", layout);
    let req = ReedRequest {
        key,
        language: None,
        environment: None,
        context: None,
        value: None,
        description: None,
    };

    match get::meta(&req) {
        Ok(response) => response.data == "true",
        Err(_) => true, // Default public
    }
}
