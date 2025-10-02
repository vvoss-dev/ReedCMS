// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One function = One job
// MANDATORY: BBC English for all documentation and comments
// MANDATORY: Type-safe error handling with ReedResult<T> pattern
//
// == FILE PURPOSE ==
// This file: HTTP error response builders for 404 and 500 errors
// Architecture: Server response layer - generates error pages with templates
// Performance: < 20ms per error response (template rendering)
// Dependencies: actix_web, minijinja, reedstream
// Data Flow: Error trigger → Template render → HttpResponse

//! Error Response Builders
//!
//! Builds HTTP error responses (404, 500) with template rendering and fallbacks.

use crate::reedcms::reedstream::ReedError;
use actix_web::HttpResponse;
use minijinja::{context, Environment};
use std::sync::OnceLock;

/// Builds 404 Not Found HTTP response.
///
/// ## Process
/// 1. Attempt to load `error.404.{variant}` template
/// 2. Render with minimal context
/// 3. Return with 404 status code
/// 4. Fallback to plain text if template missing
///
/// ## Template
/// - Template name: `error.404.mouse` (default variant)
/// - Context: `error_code`, `error_message`, `site_name`
/// - Fallback: Plain text "404 - Page Not Found"
///
/// ## Performance
/// - With template: ~10-20ms
/// - Without template: < 1ms (plain text)
///
/// ## Example Usage
/// ```rust
/// match resolve_url(path) {
///     Ok(route) => build_response(route),
///     Err(_) => build_404_response(),
/// }
/// ```
pub fn build_404_response() -> HttpResponse {
    let variant = "mouse"; // Default for errors
    let template_name = format!("error.404.{}", variant);

    match render_error_template(&template_name, 404, "Page Not Found") {
        Ok(html) => HttpResponse::NotFound()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(_) => HttpResponse::NotFound()
            .content_type("text/plain; charset=utf-8")
            .body("404 - Page Not Found"),
    }
}

/// Builds 500 Internal Server Error HTTP response.
///
/// ## Input
/// - `error`: ReedError that triggered the 500 response
///
/// ## Process
/// 1. Log error details to stderr
/// 2. Attempt to load `error.500.{variant}` template
/// 3. Render with error context (DEV only)
/// 4. Return with 500 status code
/// 5. Fallback to plain text if template missing
///
/// ## Security
/// - **DEV environment**: Show full error details in response
/// - **PROD environment**: Generic "Internal Server Error" message only
///
/// ## Environment Detection
/// - Checks `REED_ENV` environment variable
/// - DEV: `REED_ENV=DEV`
/// - PROD: `REED_ENV=PROD` or unset
///
/// ## Performance
/// - With template: ~10-20ms
/// - Without template: < 1ms (plain text)
///
/// ## Example Usage
/// ```rust
/// match render_template(template_name, context) {
///     Ok(html) => Ok(HttpResponse::Ok().body(html)),
///     Err(e) => Ok(build_500_response(e)),
/// }
/// ```
pub fn build_500_response(error: ReedError) -> HttpResponse {
    // Log error to stderr (always, regardless of environment)
    eprintln!("Server error: {:?}", error);

    let variant = "mouse";
    let template_name = format!("error.500.{}", variant);

    // Show error details only in DEV environment
    let error_message = if is_dev_environment() {
        format!("{:?}", error)
    } else {
        "Internal Server Error".to_string()
    };

    match render_error_template(&template_name, 500, &error_message) {
        Ok(html) => HttpResponse::InternalServerError()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(_) => HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body("500 - Internal Server Error"),
    }
}

/// Renders error template with minimal context.
///
/// ## Input
/// - `template_name`: Template file name (e.g., "error.404.mouse")
/// - `code`: HTTP status code (404, 500)
/// - `message`: Error message to display
///
/// ## Output
/// - `ReedResult<String>`: Rendered HTML or error
///
/// ## Template Context
/// - `error_code`: HTTP status code (404, 500)
/// - `error_message`: Error message text
/// - `site_name`: "ReedCMS" (static)
///
/// ## Error Conditions
/// - Template not found → `ReedError::TemplateError`
/// - Render failure → `ReedError::TemplateError`
///
/// ## Performance
/// - ~10-20ms (template loading + rendering)
fn render_error_template(
    template_name: &str,
    code: u16,
    message: &str,
) -> Result<String, ReedError> {
    let env = get_template_engine();

    // Load template
    let template = env
        .get_template(template_name)
        .map_err(|e| ReedError::TemplateError {
            template: template_name.to_string(),
            reason: format!("Error template not found: {}", e),
        })?;

    // Render with minimal context
    template
        .render(context! {
            error_code => code,
            error_message => message,
            site_name => "ReedCMS",
        })
        .map_err(|e| ReedError::TemplateError {
            template: template_name.to_string(),
            reason: format!("Error template render failed: {}", e),
        })
}

/// Checks if running in DEV environment.
///
/// ## Output
/// - `true`: DEV environment (show error details)
/// - `false`: PROD environment (hide error details)
///
/// ## Environment Variable
/// - Checks `REED_ENV`
/// - DEV: `REED_ENV=DEV`
/// - PROD: `REED_ENV=PROD` or unset
///
/// ## Case Insensitive
/// - Converts to uppercase for comparison
/// - "dev", "DEV", "Dev" all match
///
/// ## Performance
/// - < 1μs (environment variable lookup)
fn is_dev_environment() -> bool {
    std::env::var("REED_ENV")
        .unwrap_or_else(|_| "PROD".to_string())
        .to_uppercase()
        == "DEV"
}

/// Gets template engine singleton.
///
/// ## Output
/// - `&'static Environment<'static>`: Global template engine instance
///
/// ## Initialisation
/// - Uses `OnceLock` for thread-safe lazy initialisation
/// - Initialises on first access with default language "en" and mode "mouse"
/// - Panics if initialisation fails
///
/// ## Note
/// - Template engine needs language and mode at init time
/// - Using defaults: lang="en", mode="mouse"
/// - Filters are added at init time but work for all languages via runtime lookup
///
/// ## Performance
/// - First call: ~1-5ms (initialisation)
/// - Subsequent calls: < 1μs (static reference)
///
/// ## Error Handling
/// - Panics if template engine initialisation fails
/// - Should not happen in production (templates validated at startup)
fn get_template_engine() -> &'static Environment<'static> {
    static ENGINE: OnceLock<Environment<'static>> = OnceLock::new();
    ENGINE.get_or_init(|| {
        // Initialise with defaults - filters handle runtime language
        crate::reedcms::templates::engine::init_template_engine(
            "en".to_string(),
            "mouse".to_string(),
        )
        .expect("Failed to initialise template engine")
    })
}
