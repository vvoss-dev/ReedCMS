// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

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
    let path = url.trim_start_matches('/').trim_end_matches('/');

    // Empty path defaults to landing page
    if path.is_empty() {
        return resolve_landing_page();
    }

    // Extract language prefix if present (e.g., "de", "en")
    let (language, remaining_path) = extract_language_prefix(path);

    // If only language prefix (e.g., "/de" or "/en"), return landing page for that language
    if remaining_path.is_empty() {
        return Ok(RouteInfo {
            layout: "landing".to_string(),
            language,
            params: HashMap::new(),
        });
    }

    // Try exact match first (with language filtering)
    if let Some(route_info) = lookup_exact_route(&remaining_path, &language)? {
        return Ok(route_info);
    }

    // Try pattern matching for dynamic routes
    if let Some(route_info) = lookup_pattern_route(&remaining_path)? {
        return Ok(route_info);
    }

    // 404 - Not found
    Err(ReedError::NotFound {
        resource: url.to_string(),
        context: Some("Route not found in routes.csv".to_string()),
    })
}

/// Extracts language prefix from path.
///
/// ## Arguments
/// - path: URL path without leading/trailing slashes
///
/// ## Returns
/// - (language, remaining_path) tuple
///
/// ## Examples
/// - "de" → ("de", "")
/// - "de/wissen" → ("de", "wissen")
/// - "en/knowledge/api" → ("en", "knowledge/api")
/// - "wissen" → ("de", "wissen") // defaults to "de"
fn extract_language_prefix(path: &str) -> (String, String) {
    let parts: Vec<&str> = path.splitn(2, '/').collect();

    if parts.is_empty() {
        return ("de".to_string(), String::new());
    }

    // Check if first segment is a known language code
    let first = parts[0];
    if first == "de" || first == "en" {
        let remaining = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            String::new()
        };
        return (first.to_string(), remaining);
    }

    // No language prefix, default to "de" and treat entire path as route
    ("de".to_string(), path.to_string())
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
/// - language: Language code to filter routes (e.g., "de", "en")
///
/// ## Returns
/// - Some(RouteInfo) if route found for the given language
/// - None if no match
///
/// ## Performance
/// - O(n) linear scan through all routes
/// - Future optimization: Build reverse index (route → layout@lang)
///
/// ## Implementation Note
/// Reads routes.csv directly and scans for matching value in the specified language.
/// Format in CSV: `key|value|comment` where key = layout@lang, value = route
/// Example: `knowledge@de|wissen|German knowledge route`
///
/// ## Language Filtering
/// Only matches routes where the key ends with @{language}.
/// This ensures /de/wissen matches knowledge@de (not knowledge@en).
fn lookup_exact_route(path: &str, language: &str) -> ReedResult<Option<RouteInfo>> {
    // Read routes.csv directly
    let csv_path = ".reed/routes.csv";
    let entries = match read_csv(csv_path) {
        Ok(e) => e,
        Err(_) => return Ok(None),
    };

    // Scan through routes to find matching value in the correct language
    for record in entries {
        let route_value = record.value.trim();

        // Check if route value matches path
        if route_value == path {
            // Parse key: "layout@lang" → (layout, lang)
            if let Some((layout, lang)) = parse_route_key(&record.key) {
                // Only return if language matches
                if lang == language {
                    return Ok(Some(RouteInfo {
                        layout,
                        language: lang,
                        params: HashMap::new(),
                    }));
                }
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
