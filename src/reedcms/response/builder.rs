// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One function = One job
// MANDATORY: BBC English for all documentation and comments
// MANDATORY: Type-safe error handling with ReedResult<T> pattern
//
// == FILE PURPOSE ==
// This file: HTTP response builder orchestrating URL routing, variant detection, and template rendering
// Architecture: Server response layer - complete request-to-response flow
// Performance: < 50ms complete response, < 10ms cached
// Dependencies: routing, templates, reedbase, actix_web
// Data Flow: HttpRequest → URL routing → Variant detection → Context building → Template rendering → HttpResponse

//! Response Builder
//!
//! Orchestrates complete HTTP response building from request to rendered HTML.

use actix_web::{Error, HttpRequest, HttpResponse};
use crate::reedcms::response::cache::cache_control_header;
use crate::reedcms::response::errors::{build_404_response, build_500_response};
use crate::reedcms::routing::resolver::resolve_url;
use crate::reedcms::templates::context::build_context;
use minijinja::Environment;
use std::sync::OnceLock;

/// Builds complete HTTP response for incoming request.
///
/// ## Input
/// - `req`: Actix-Web HTTP request
///
/// ## Output
/// - `Result<HttpResponse, Error>`: Complete HTTP response with headers
///
/// ## Process
/// 1. Resolve URL to layout + language (via REED-06-02 routing)
/// 2. Detect user agent variant (mouse/touch/reader)
/// 3. Build template context (via REED-05-03)
/// 4. Render template (via REED-05-02)
/// 5. Assemble response with headers
/// 6. Return HttpResponse
///
/// ## HTTP Headers Set
/// - `Content-Type`: "text/html; charset=utf-8"
/// - `Cache-Control`: "public/private, max-age={ttl}" (from meta.csv)
/// - `X-Layout`: Layout name (e.g., "knowledge")
/// - `X-Language`: Language code (e.g., "de", "en")
/// - `X-Variant`: Variant name (e.g., "mouse", "touch", "reader")
/// - `X-Render-Time`: Render time in milliseconds (for debugging)
///
/// ## Performance
/// - Complete response: < 50ms
/// - Cached response: < 10ms
/// - Render time included in `X-Render-Time` header
///
/// ## Error Handling
/// - URL not found → 404 response
/// - Context build error → 500 response
/// - Template render error → 500 response
///
/// ## Example Usage
/// ```rust
/// async fn handle_request(req: HttpRequest) -> Result<HttpResponse, Error> {
///     build_response(req).await
/// }
/// ```
pub async fn build_response(req: HttpRequest) -> Result<HttpResponse, Error> {
    let start_time = std::time::Instant::now();

    // 1. Resolve URL to layout + language
    let route_info = match resolve_url(req.path()) {
        Ok(info) => info,
        Err(_) => return Ok(build_404_response()),
    };

    // 2. Detect client info (variant, breakpoint, device_type, etc.)
    let client_info = match crate::reedcms::server::client_detection::detect_client_info(&req, &route_info.language) {
        Ok(info) => info,
        Err(e) => return Ok(build_500_response(e)),
    };

    let variant = client_info.interaction_mode.clone();

    // 3. Build template context with client info
    let context = match build_context(
        &route_info.layout,
        &route_info.language,
        &client_info,
    ) {
        Ok(ctx) => ctx,
        Err(e) => return Ok(build_500_response(e)),
    };

    // 5. Render template
    // Template path: layouts/{layout}/{layout}.jinja
    let template_name = format!("layouts/{}/{}.jinja", route_info.layout, route_info.layout);
    let html = match render_template(&template_name, &context) {
        Ok(output) => output,
        Err(e) => return Ok(build_500_response(e)),
    };

    // 6. Assemble response with headers
    let render_time_ms = start_time.elapsed().as_millis();
    let (cache_name, cache_value) = cache_control_header(&route_info.layout);

    let response = HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header(("X-Layout", route_info.layout.as_str()))
        .insert_header(("X-Language", route_info.language.as_str()))
        .insert_header(("X-Variant", variant.as_str()))
        .insert_header(("X-Render-Time", format!("{}ms", render_time_ms)))
        .insert_header((cache_name, cache_value))
        .body(html);

    Ok(response)
}

/// Detects variant from User-Agent header.
///
/// ## Input
/// - `req`: HTTP request with User-Agent header
///
/// ## Output
/// - `String`: Variant name ("mouse", "touch", or "reader")
///
/// ## Variant Detection Rules
/// - **Reader**: Text browsers (Lynx, w3m) or reader mode
/// - **Touch**: Mobile/tablet devices (iPhone, iPad, Android, Mobile)
/// - **Mouse**: Desktop browsers (default fallback)
///
/// ## User-Agent Patterns
/// - Reader: "Lynx", "w3m", "Reader"
/// - Touch: "iPhone", "iPad", "Android", "Mobile"
/// - Mouse: Everything else (default)
///
/// ## Priority
/// 1. Reader (highest priority - accessibility)
/// 2. Touch (mobile devices)
/// 3. Mouse (default)
///
/// ## Performance
/// - < 1ms (simple string matching)
///
/// ## Example User-Agents
/// - Reader: "Lynx/2.8.9rel.1"
/// - Touch: "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0)"
/// - Mouse: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"
///
/// ## Fallback Behaviour
/// - Missing User-Agent header → "mouse"
/// - Invalid User-Agent header → "mouse"
fn detect_variant(req: &HttpRequest) -> String {
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // Check for reader mode (highest priority for accessibility)
    if user_agent.contains("Lynx")
        || user_agent.contains("w3m")
        || user_agent.contains("Reader")
    {
        return "reader".to_string();
    }

    // Check for touch devices
    if user_agent.contains("iPhone")
        || user_agent.contains("iPad")
        || user_agent.contains("Android")
        || user_agent.contains("Mobile")
    {
        return "touch".to_string();
    }

    // Default to mouse (desktop browsers)
    "mouse".to_string()
}

/// Renders template with context using MiniJinja engine.
///
/// ## Input
/// - `template_name`: Template file name (e.g., "layouts/knowledge/knowledge.jinja")
/// - `context`: Template context (HashMap with client.lang)
///
/// ## Output
/// - `Result<String, ReedError>`: Rendered HTML or error
///
/// ## Process
/// 1. Get base template engine singleton (fast)
/// 2. Clone environment (cheap - only metadata)
/// 3. Add request-specific filters (text, route with language from context)
/// 4. Load template by name
/// 5. Render with context
/// 6. Return HTML string
///
/// ## Performance
/// - Singleton access: < 1μs
/// - Environment clone: ~1-2ms (metadata only)
/// - Filter registration: < 1ms
/// - Template loading: ~5-10ms
/// - Total: ~10-15ms per request
///
/// ## Legacy Pattern
/// This follows the successful legacy approach:
/// - Base environment as singleton (templates loaded once)
/// - Clone per request (cheap operation)
/// - Filters added to clone with request language
///
/// ## Error Conditions
/// - Template not found → `ReedError::TemplateError`
/// - Render failure → `ReedError::TemplateError`
fn render_template(
    template_name: &str,
    context: &std::collections::HashMap<String, serde_json::Value>,
) -> Result<String, crate::reedcms::reedstream::ReedError> {
    use crate::reedcms::reedstream::ReedError;
    use crate::reedcms::filters;

    // Get language and variant from context
    let lang = context
        .get("client")
        .and_then(|c| c.get("lang"))
        .and_then(|l| l.as_str())
        .unwrap_or("en");

    let variant = context
        .get("client")
        .and_then(|c| c.get("interaction_mode"))
        .and_then(|v| v.as_str())
        .unwrap_or("mouse");

    // Get base environment singleton (fast)
    let base_env = get_template_engine();

    // Clone environment (cheap - only metadata)
    let mut env = base_env.clone();

    // Add ONLY request-specific filters (those that need language)
    // Meta/config filters and organism/molecule/atom functions are already in base env
    env.add_filter(
        "text",
        filters::text::make_text_filter(lang.to_string()),
    );
    env.add_filter(
        "route",
        filters::route::make_route_filter(lang.to_string()),
    );

    // Load template
    let template = env
        .get_template(template_name)
        .map_err(|e| ReedError::TemplateError {
            template: template_name.to_string(),
            reason: format!("Template not found: {}", e),
        })?;

    // Render with context
    template
        .render(context)
        .map_err(|e| ReedError::TemplateError {
            template: template_name.to_string(),
            reason: format!("Render error: {}", e),
        })
}

/// Gets base template engine singleton.
///
/// ## Output
/// - `&'static Environment<'static>`: Global base template engine
///
/// ## Initialization
/// - Initializes once with base configuration (no filters/functions)
/// - Templates loaded once
/// - Filters/functions added per request via clone
///
/// ## Performance
/// - First call: ~50-100ms (template loading)
/// - Subsequent calls: < 1μs (static reference)
fn get_template_engine() -> &'static Environment<'static> {
    static ENGINE: OnceLock<Environment<'static>> = OnceLock::new();
    ENGINE.get_or_init(|| {
        use minijinja::{AutoEscape, UndefinedBehavior};

        let mut env = Environment::new();

        // Set template loader
        env.set_loader(crate::reedcms::templates::engine::template_loader);

        // Configure auto-escape for HTML
        env.set_auto_escape_callback(|name| {
            if name.ends_with(".jinja") || name.ends_with(".html") {
                AutoEscape::Html
            } else {
                AutoEscape::None
            }
        });

        // Enable strict mode (undefined variables error)
        env.set_undefined_behavior(UndefinedBehavior::Strict);

        env
    })
}
