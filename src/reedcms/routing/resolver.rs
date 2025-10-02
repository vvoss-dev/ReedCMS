// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! URL resolver for ReedCMS routing.
//!
//! Resolves incoming URLs to layout + language combinations via .reed/routes.csv.
//!
//! ## Route Format
//! CSV format: `layout@lang|route|comment`
//! Example: `knowledge@de|wissen|German knowledge section`
//!
//! ## Resolution Process
//! 1. Strip leading slash from URL
//! 2. Try exact route match (reverse lookup: route → layout@lang)
//! 3. Try pattern matching for dynamic routes
//! 4. Return 404 if no match found
//!
//! ## Performance
//! - Exact match: O(n) linear scan through routes (future: reverse index)
//! - Pattern match: O(n×m) where n=patterns, m=segments
//! - Target: < 5ms per resolution

use crate::reedcms::csv::read_csv;
use crate::reedcms::reedbase;
use crate::reedcms::reedstream::{ReedError, ReedRequest, ReedResult};
use std::collections::HashMap;

/// Route information structure.
///
/// Contains resolved layout, language, and extracted URL parameters.
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub layout: String,
    pub language: String,
    pub params: HashMap<String, String>,
}

/// Resolves URL to layout and language.
///
/// ## Arguments
/// - url: Request URL path (e.g., "/wissen", "/blog", "/knowledge/agility")
///
/// ## Returns
/// - RouteInfo with layout, language, and optional params
///
/// ## Performance
/// - O(n) for exact match (linear scan through routes)
/// - < 5ms typical resolution time
///
/// ## Error Conditions
/// - NotFound: URL does not match any route
///
/// ## Example Usage
/// ```rust
/// let info = resolve_url("/wissen")?;
/// // RouteInfo { layout: "knowledge", language: "de", params: {} }
///
/// let info = resolve_url("/")?;
/// // RouteInfo { layout: "landing", language: "de", params: {} }
/// ```
pub fn resolve_url(url: &str) -> ReedResult<RouteInfo> {
    // Strip leading slash
    let path = url.trim_start_matches('/');

    // Empty path defaults to landing page
    if path.is_empty() {
        return resolve_landing_page();
    }

    // Try exact match first
    if let Some(route_info) = lookup_exact_route(path)? {
        return Ok(route_info);
    }

    // Try pattern matching for dynamic routes
    if let Some(route_info) = lookup_pattern_route(path)? {
        return Ok(route_info);
    }

    // 404 - Not found
    Err(ReedError::NotFound {
        resource: url.to_string(),
        context: Some("Route not found in routes.csv".to_string()),
    })
}

/// Resolves landing page (root URL).
///
/// ## Returns
/// - RouteInfo for landing page with default language
///
/// ## Performance
/// - O(1) with default fallback
fn resolve_landing_page() -> ReedResult<RouteInfo> {
    // Try to get default language from config
    let default_lang = get_default_language().unwrap_or_else(|| "de".to_string());

    Ok(RouteInfo {
        layout: "landing".to_string(),
        language: default_lang,
        params: HashMap::new(),
    })
}

/// Looks up exact route match via reverse lookup.
///
/// ## Arguments
/// - path: URL path without leading slash (e.g., "wissen", "blog")
///
/// ## Returns
/// - Some(RouteInfo) if route found
/// - None if no match
///
/// ## Performance
/// - O(n) linear scan through all routes
/// - Future optimization: Build reverse index (route → layout@lang)
///
/// ## Implementation Note
/// Reads routes.csv directly and scans for matching value.
/// Format in CSV: `key|value|comment` where key = layout@lang, value = route
/// Example: `knowledge@de|wissen|German knowledge route`
fn lookup_exact_route(path: &str) -> ReedResult<Option<RouteInfo>> {
    // Read routes.csv directly
    let csv_path = ".reed/routes.csv";
    let entries = match read_csv(csv_path) {
        Ok(e) => e,
        Err(_) => return Ok(None),
    };

    // Scan through routes to find matching value
    for record in entries {
        let route_value = record.value.trim();

        // Check if route value matches path
        if route_value == path {
            // Parse key: "layout@lang" → (layout, lang)
            if let Some((layout, lang)) = parse_route_key(&record.key) {
                return Ok(Some(RouteInfo {
                    layout,
                    language: lang,
                    params: HashMap::new(),
                }));
            }
        }
    }

    Ok(None)
}

/// Looks up pattern-based route match.
///
/// ## Arguments
/// - path: URL path without leading slash
///
/// ## Returns
/// - Some(RouteInfo) with extracted params if pattern matches
/// - None if no pattern matches
///
/// ## Performance
/// - O(n×m) where n=patterns, m=segments
/// - Patterns are checked in definition order
///
/// ## Pattern Examples
/// - `blog/*` → matches `/blog/my-post`
/// - `docs/:category/:page` → extracts category and page params
///
/// ## Implementation Note
/// Currently returns None. Pattern matching will be implemented
/// when dynamic routes are needed (not required for initial release).
fn lookup_pattern_route(_path: &str) -> ReedResult<Option<RouteInfo>> {
    // Pattern matching not yet implemented
    // Will be added when dynamic routes are needed
    Ok(None)
}

/// Parses route key into layout and language.
///
/// ## Arguments
/// - key: Route key in format "layout@lang" (e.g., "knowledge@de")
///
/// ## Returns
/// - Some((layout, language)) if valid format
/// - None if malformed
///
/// ## Examples
/// - "knowledge@de" → Some(("knowledge", "de"))
/// - "blog@en" → Some(("blog", "en"))
/// - "landing@de" → Some(("landing", "de"))
/// - "invalid" → None
fn parse_route_key(key: &str) -> Option<(String, String)> {
    if let Some(pos) = key.rfind('@') {
        let layout = &key[..pos];
        let lang = &key[pos + 1..];

        if !layout.is_empty() && !lang.is_empty() {
            return Some((layout.to_string(), lang.to_string()));
        }
    }

    None
}

/// Gets default language from project config.
///
/// ## Returns
/// - Some(language) if configured
/// - None if not configured (defaults to "de" in caller)
///
/// ## Performance
/// - O(1) ReedBase lookup with cache
fn get_default_language() -> Option<String> {
    let req = ReedRequest {
        key: "project.default_language".to_string(),
        language: None,
        environment: None,
        context: Some("routing".to_string()),
        value: None,
        description: None,
    };

    reedbase::get::project(&req).ok().map(|r| r.data)
}
