// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One function = One job
// MANDATORY: BBC English for all documentation and comments
// MANDATORY: Type-safe error handling with ReedResult<T> pattern
//
// == FILE PURPOSE ==
// This file: HTTP Content-Type negotiation based on Accept headers
// Architecture: Server response layer - determines output format (HTML/JSON/Plain)
// Performance: < 1μs per negotiation (string matching only)
// Dependencies: actix_web
// Data Flow: HttpRequest → Accept header → ContentType enum

//! Content Type Negotiation
//!
//! Handles HTTP Accept header negotiation to determine response format.

use actix_web::HttpRequest;

/// Content type enum for response formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// HTML response (text/html)
    Html,
    /// JSON response (application/json)
    Json,
    /// Plain text response (text/plain)
    Plain,
}

impl ContentType {
    /// Returns MIME type string for HTTP Content-Type header.
    ///
    /// ## Output
    /// - `Html`: "text/html; charset=utf-8"
    /// - `Json`: "application/json; charset=utf-8"
    /// - `Plain`: "text/plain; charset=utf-8"
    ///
    /// ## Performance
    /// - O(1) constant time
    ///
    /// ## Example Usage
    /// ```rust
    /// let content_type = ContentType::Html;
    /// assert_eq!(content_type.mime_type(), "text/html; charset=utf-8");
    /// ```
    pub fn mime_type(&self) -> &'static str {
        match self {
            ContentType::Html => "text/html; charset=utf-8",
            ContentType::Json => "application/json; charset=utf-8",
            ContentType::Plain => "text/plain; charset=utf-8",
        }
    }
}

/// Determines response content type based on request Accept header.
///
/// ## Input
/// - `req`: HTTP request with Accept header
///
/// ## Output
/// - `ContentType`: Negotiated content type
///
/// ## Accept Header Processing
/// - `text/html` → `ContentType::Html` (default)
/// - `application/json` → `ContentType::Json`
/// - `text/plain` → `ContentType::Plain`
///
/// ## Fallback Behaviour
/// - Missing Accept header → `Html`
/// - Invalid Accept header → `Html`
/// - Multiple types → First matching type
///
/// ## Performance
/// - < 1μs (simple string matching)
///
/// ## Example Usage
/// ```rust
/// let content_type = negotiate_content_type(&req);
/// match content_type {
///     ContentType::Html => render_html(),
///     ContentType::Json => render_json(),
///     ContentType::Plain => render_plain(),
/// }
/// ```
pub fn negotiate_content_type(req: &HttpRequest) -> ContentType {
    let accept_header = req
        .headers()
        .get("Accept")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("text/html");

    // Check for JSON first (most specific)
    if accept_header.contains("application/json") {
        return ContentType::Json;
    }

    // Check for plain text
    if accept_header.contains("text/plain") {
        return ContentType::Plain;
    }

    // Default to HTML (most common for browsers)
    ContentType::Html
}
