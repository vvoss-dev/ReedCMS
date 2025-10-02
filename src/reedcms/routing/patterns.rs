// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Pattern matching for dynamic routes.
//!
//! Provides pattern matching and parameter extraction for dynamic URLs.
//!
//! ## Pattern Syntax
//! - `:param` - Named parameter (e.g., `/blog/:slug`)
//! - `*` - Wildcard segment (matches anything)
//! - Literal - Exact match (e.g., `/blog`)
//!
//! ## Examples
//! - Pattern: `/blog/:slug` + URL: `/blog/my-post` → `{ slug: "my-post" }`
//! - Pattern: `/docs/:category/:page` + URL: `/docs/api/intro` → `{ category: "api", page: "intro" }`
//! - Pattern: `/blog/*` + URL: `/blog/anything` → matches
//!
//! ## Performance
//! - O(n) where n=number of segments
//! - Target: < 10ms per pattern match

use std::collections::HashMap;

/// Matches URL against pattern and extracts parameters.
///
/// ## Arguments
/// - pattern: Route pattern with placeholders (e.g., "/blog/:slug")
/// - path: URL path to match (e.g., "/blog/my-post")
///
/// ## Returns
/// - Some(params) if pattern matches, with extracted parameters
/// - None if pattern doesn't match
///
/// ## Pattern Rules
/// - `:param` matches one segment and extracts value
/// - `*` matches one segment without extraction
/// - Literal matches exact text
/// - Segment count must match exactly
///
/// ## Performance
/// - O(n) where n=number of segments
/// - < 10ms typical matching time
///
/// ## Examples
/// ```rust
/// let params = match_pattern("/blog/:slug", "/blog/my-post");
/// // Some({ "slug": "my-post" })
///
/// let params = match_pattern("/docs/:cat/:page", "/docs/api/intro");
/// // Some({ "cat": "api", "page": "intro" })
///
/// let params = match_pattern("/blog/:slug", "/portfolio/work");
/// // None (doesn't match)
/// ```
pub fn match_pattern(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_parts: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
    let path_parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    // Segment count must match
    if pattern_parts.len() != path_parts.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if pattern_part.starts_with(':') {
            // Named parameter - extract value
            let param_name = pattern_part.trim_start_matches(':');
            if param_name.is_empty() {
                // Malformed pattern
                return None;
            }
            params.insert(param_name.to_string(), path_part.to_string());
        } else if *pattern_part == "*" {
            // Wildcard - matches anything, no extraction
            continue;
        } else if pattern_part != path_part {
            // Literal doesn't match
            return None;
        }
    }

    Some(params)
}

/// Checks if pattern is valid.
///
/// ## Arguments
/// - pattern: Pattern to validate
///
/// ## Returns
/// - true if pattern is valid
/// - false if pattern has syntax errors
///
/// ## Validation Rules
/// - Must not be empty
/// - Parameters must not be empty (`:` alone is invalid)
/// - Must not contain consecutive slashes
///
/// ## Examples
/// - "/blog/:slug" → true
/// - "/docs/:cat/:page" → true
/// - "/blog/*" → true
/// - "/blog/:" → false (empty parameter)
/// - "" → false (empty pattern)
pub fn is_valid_pattern(pattern: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }

    let parts: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();

    for part in parts {
        if part.is_empty() {
            // Consecutive slashes or trailing slash
            return false;
        }

        if part.starts_with(':') && part.len() == 1 {
            // Empty parameter name
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_simple_parameter() {
        let result = match_pattern("/blog/:slug", "/blog/my-post");
        assert!(result.is_some());
        let params = result.unwrap();
        assert_eq!(params.get("slug"), Some(&"my-post".to_string()));
    }

    #[test]
    fn test_match_multiple_parameters() {
        let result = match_pattern("/docs/:category/:page", "/docs/api/intro");
        assert!(result.is_some());
        let params = result.unwrap();
        assert_eq!(params.get("category"), Some(&"api".to_string()));
        assert_eq!(params.get("page"), Some(&"intro".to_string()));
    }

    #[test]
    fn test_match_wildcard() {
        let result = match_pattern("/blog/*", "/blog/anything");
        assert!(result.is_some());
        let params = result.unwrap();
        assert!(params.is_empty());
    }

    #[test]
    fn test_no_match_different_literal() {
        let result = match_pattern("/blog/:slug", "/portfolio/work");
        assert!(result.is_none());
    }

    #[test]
    fn test_no_match_different_segment_count() {
        let result = match_pattern("/blog/:slug", "/blog/category/post");
        assert!(result.is_none());
    }

    #[test]
    fn test_valid_pattern() {
        assert!(is_valid_pattern("/blog/:slug"));
        assert!(is_valid_pattern("/docs/:cat/:page"));
        assert!(is_valid_pattern("/blog/*"));
    }

    #[test]
    fn test_invalid_pattern() {
        assert!(!is_valid_pattern(""));
        assert!(!is_valid_pattern("/blog/:"));
        assert!(!is_valid_pattern("//blog"));
    }
}
