// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Language detection for ReedCMS routing.
//!
//! Detects user language from URL path and Accept-Language header.
//!
//! ## Detection Order
//! 1. URL path prefix (e.g., /en/page, /de/wissen)
//! 2. Accept-Language HTTP header
//! 3. Default language from project config
//!
//! ## Performance
//! - URL detection: O(1) string operation
//! - Header parsing: O(n) where n=header length
//! - Target: < 1ms per detection

use crate::reedcms::reedbase;
use crate::reedcms::reedstream::{ReedRequest, ReedResult};
use actix_web::HttpRequest;

/// Detects language from HTTP request.
///
/// ## Arguments
/// - req: Actix-Web HTTP request
///
/// ## Returns
/// - Language code (e.g., "de", "en")
///
/// ## Detection Order
/// 1. URL path: /en/page → "en"
/// 2. Accept-Language header: "de-DE,de;q=0.9,en;q=0.8" → "de"
/// 3. Default from config or fallback to "de"
///
/// ## Performance
/// - < 1ms typical detection time
///
/// ## Example Usage
/// ```rust
/// let lang = detect_language(&req);
/// // "de" or "en"
/// ```
pub fn detect_language(req: &HttpRequest) -> String {
    // Try URL path first
    if let Some(lang) = extract_language_from_path(req.path()) {
        return lang;
    }

    // Try Accept-Language header
    if let Some(lang) = parse_accept_language_header(req) {
        return lang;
    }

    // Fall back to default
    get_default_language().unwrap_or_else(|| "de".to_string())
}

/// Extracts language from URL path.
///
/// ## Arguments
/// - path: URL path (e.g., "/en/knowledge", "/de/wissen")
///
/// ## Returns
/// - Some(language) if valid language prefix found
/// - None if no language prefix
///
/// ## Examples
/// - "/en/knowledge" → Some("en")
/// - "/de/wissen" → Some("de")
/// - "/knowledge" → None
/// - "/portfolio" → None
///
/// ## Performance
/// - O(1) string operations
pub fn extract_language_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    // Check if first segment is 2-letter language code
    if !parts.is_empty() && parts[0].len() == 2 {
        let potential_lang = parts[0];

        // Validate against supported languages
        if is_valid_language_code(potential_lang) {
            return Some(potential_lang.to_string());
        }
    }

    None
}

/// Parses Accept-Language HTTP header.
///
/// ## Arguments
/// - req: HTTP request
///
/// ## Returns
/// - Some(language) if header present and parseable
/// - None if header missing or invalid
///
/// ## Examples
/// - "de-DE,de;q=0.9,en;q=0.8" → Some("de")
/// - "en-US,en;q=0.9" → Some("en")
/// - Missing header → None
///
/// ## Performance
/// - O(n) where n=header length
/// - Parses only first language preference
pub fn parse_accept_language_header(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Accept-Language")
        .and_then(|h| h.to_str().ok())
        .and_then(|header_value| {
            // Parse "de-DE,de;q=0.9,en;q=0.8"
            header_value.split(',').next().and_then(|first_pref| {
                // Extract language from "de-DE" or "de;q=0.9"
                let lang_part = first_pref.split(';').next().unwrap_or(first_pref);
                let lang_code = lang_part.split('-').next().unwrap_or(lang_part);
                let lang_code = lang_code.trim();

                // Validate language code
                if is_valid_language_code(lang_code) {
                    Some(lang_code.to_string())
                } else {
                    None
                }
            })
        })
}

/// Checks if language code is valid.
///
/// ## Arguments
/// - code: Language code to validate (e.g., "de", "en")
///
/// ## Returns
/// - true if language is supported
/// - false if not supported
///
/// ## Performance
/// - O(n) where n=number of supported languages
/// - With cache: O(1)
pub fn is_valid_language_code(code: &str) -> bool {
    let valid_languages = get_supported_languages().unwrap_or_default();
    valid_languages.contains(&code.to_string())
}

/// Gets supported languages from project config.
///
/// ## Returns
/// - Vec of supported language codes
/// - Default: ["de", "en"]
///
/// ## Configuration
/// - Key: project.languages
/// - Format: "de,en,fr"
///
/// ## Performance
/// - O(1) with ReedBase cache
pub fn get_supported_languages() -> ReedResult<Vec<String>> {
    let req = ReedRequest {
        key: "project.languages".to_string(),
        language: None,
        environment: None,
        context: Some("routing".to_string()),
        value: None,
        description: None,
    };

    match reedbase::get::project(&req) {
        Ok(response) => Ok(response
            .data
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()),
        Err(_) => {
            // Default fallback
            Ok(vec!["de".to_string(), "en".to_string()])
        }
    }
}

/// Gets default language from project config.
///
/// ## Returns
/// - Some(language) if configured
/// - None if not configured
///
/// ## Configuration
/// - Key: project.default_language
/// - Default: "de"
///
/// ## Performance
/// - O(1) with ReedBase cache
pub fn get_default_language() -> Option<String> {
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
