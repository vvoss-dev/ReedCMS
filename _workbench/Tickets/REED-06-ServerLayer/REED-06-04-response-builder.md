# REED-06-04: Response Builder System

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function (CONFIG and SYSTEM use identical logic)
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs` - every topic (auth, login,...) has its own scoped rs service file
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-06-04
- **Title**: HTTP Response Builder with Template Rendering
- **Layer**: Server Layer (REED-06)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-06-02, REED-05-02, REED-05-03

## Summary Reference
- **Section**: Response Builder
- **Lines**: 1010-1013 in project_summary.md
- **Key Concepts**: Template rendering, content negotiation, variant selection, HTTP headers

## Objective
Implement HTTP response builder that orchestrates URL routing, template rendering, variant selection, and content delivery with proper HTTP headers including cache control, content type, and compression.

## Requirements

### Response Flow
```
1. Request arrives → Route resolution (REED-06-02)
2. Layout + Language identified
3. Variant detection (mouse/touch/reader)
4. Context building (REED-05-03)
5. Template rendering (REED-05-02)
6. Response assembly with headers
7. Delivery to client
```

### Implementation (`src/reedcms/response/builder.rs`)

```rust
/// Builds HTTP response for incoming request.
///
/// ## Process
/// 1. Resolve URL to layout + language
/// 2. Detect user agent variant (mouse/touch/reader)
/// 3. Build template context
/// 4. Render template
/// 5. Assemble response with headers
/// 6. Return HttpResponse
///
/// ## Performance
/// - Complete response building: < 50ms
/// - Cached response: < 10ms
///
/// ## Headers Set
/// - Content-Type: text/html; charset=utf-8
/// - Cache-Control: max-age={ttl}
/// - Content-Encoding: gzip (if supported)
/// - X-Layout: {layout}
/// - X-Language: {language}
/// - X-Variant: {variant}
pub async fn build_response(req: HttpRequest) -> Result<HttpResponse, Error> {
    let start_time = std::time::Instant::now();

    // 1. Resolve URL to layout + language
    let route_info = match resolve_url(req.path()) {
        Ok(info) => info,
        Err(_) => return Ok(build_404_response()),
    };

    // 2. Detect variant from User-Agent
    let variant = detect_variant(&req);

    // 3. Build template context
    let context = match build_context(
        &route_info.layout,
        &route_info.language,
        &get_environment(),
        &variant,
    ) {
        Ok(ctx) => ctx,
        Err(e) => return Ok(build_500_response(e)),
    };

    // 4. Render template
    let template_name = format!("{}.{}", route_info.layout, variant);
    let html = match render_template(&template_name, &context) {
        Ok(output) => output,
        Err(e) => return Ok(build_500_response(e)),
    };

    // 5. Assemble response
    let response = HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header(("X-Layout", route_info.layout.as_str()))
        .insert_header(("X-Language", route_info.language.as_str()))
        .insert_header(("X-Variant", variant.as_str()))
        .insert_header(("X-Render-Time", format!("{}ms", start_time.elapsed().as_millis())))
        .insert_header(cache_control_header(&route_info.layout))
        .body(html);

    Ok(response)
}

/// Resolves URL to layout + language using routing system.
fn resolve_url(path: &str) -> ReedResult<RouteInfo> {
    reedcms::routing::resolver::resolve_url(path)
}

/// Detects variant from User-Agent header.
///
/// ## Variant Detection Rules
/// - **Touch**: Mobile/tablet user agents (iOS, Android)
/// - **Reader**: Reader mode or text browsers (Lynx, w3m)
/// - **Mouse**: Desktop browsers (default)
///
/// ## User-Agent Patterns
/// - Touch: "iPhone", "iPad", "Android", "Mobile"
/// - Reader: "Lynx", "w3m", "Reader"
/// - Mouse: Everything else
///
/// ## Performance
/// - Detection: < 1ms (string matching)
fn detect_variant(req: &HttpRequest) -> String {
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // Check for reader mode
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

    // Default to mouse
    "mouse".to_string()
}

/// Builds template context for rendering.
fn build_context(
    layout: &str,
    language: &str,
    environment: &str,
    variant: &str,
) -> ReedResult<Context> {
    reedcms::templates::context::build_context(layout, language, environment, variant)
}

/// Renders template with context.
fn render_template(template_name: &str, context: &Context) -> ReedResult<String> {
    let env = get_template_engine();
    let template = env.get_template(template_name).map_err(|e| ReedError::TemplateError {
        template: template_name.to_string(),
        reason: format!("Template not found: {}", e),
    })?;

    template.render(context).map_err(|e| ReedError::TemplateError {
        template: template_name.to_string(),
        reason: format!("Render error: {}", e),
    })
}

/// Gets current environment (DEV/PROD).
fn get_environment() -> String {
    std::env::var("REED_ENV").unwrap_or_else(|_| "PROD".to_string())
}

/// Gets template engine singleton.
fn get_template_engine() -> &'static Environment<'static> {
    use std::sync::OnceLock;
    static ENGINE: OnceLock<Environment<'static>> = OnceLock::new();
    ENGINE.get_or_init(|| {
        reedcms::templates::engine::init_template_engine()
            .expect("Failed to initialise template engine")
    })
}
```

### Cache Control Headers (`src/reedcms/response/cache.rs`)

```rust
/// Generates Cache-Control header based on layout configuration.
///
/// ## Cache TTL Sources
/// 1. Layout-specific: meta.csv → layout.cache.ttl
/// 2. Default: 3600 seconds (1 hour)
///
/// ## Cache-Control Format
/// - Public content: "public, max-age={ttl}"
/// - Private content: "private, max-age={ttl}"
/// - No cache: "no-cache, no-store, must-revalidate"
///
/// ## Examples
/// - Blog posts: max-age=3600 (1 hour)
/// - Static pages: max-age=86400 (24 hours)
/// - User dashboards: no-cache
pub fn cache_control_header(layout: &str) -> (&'static str, String) {
    let ttl = get_cache_ttl(layout);

    if ttl == 0 {
        return ("Cache-Control", "no-cache, no-store, must-revalidate".to_string());
    }

    let cache_type = if is_public_layout(layout) {
        "public"
    } else {
        "private"
    };

    ("Cache-Control", format!("{}, max-age={}", cache_type, ttl))
}

/// Gets cache TTL for layout from meta.csv.
fn get_cache_ttl(layout: &str) -> u64 {
    let key = format!("{}.cache.ttl", layout);
    let req = ReedRequest {
        key,
        language: None,
        environment: None,
        context: None,
    };

    match reedbase::get::meta(&req) {
        Ok(response) => response.data.parse().unwrap_or(3600),
        Err(_) => 3600, // Default 1 hour
    }
}

/// Checks if layout is publicly cacheable.
fn is_public_layout(layout: &str) -> bool {
    let key = format!("{}.cache.public", layout);
    let req = ReedRequest {
        key,
        language: None,
        environment: None,
        context: None,
    };

    match reedbase::get::meta(&req) {
        Ok(response) => response.data == "true",
        Err(_) => true, // Default public
    }
}
```

### Error Responses (`src/reedcms/response/errors.rs`)

```rust
/// Builds 404 Not Found response.
///
/// ## Process
/// 1. Load 404 template
/// 2. Render with minimal context
/// 3. Return with 404 status code
///
/// ## Template
/// - Template: "error.404.mouse" (or variant-specific)
/// - Fallback: Plain text response if template missing
pub fn build_404_response() -> HttpResponse {
    let variant = "mouse"; // Default for errors
    let template_name = format!("error.404.{}", variant);

    match render_error_template(&template_name, 404, "Page Not Found") {
        Ok(html) => HttpResponse::NotFound()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(_) => HttpResponse::NotFound()
            .content_type("text/plain")
            .body("404 - Page Not Found"),
    }
}

/// Builds 500 Internal Server Error response.
///
/// ## Process
/// 1. Log error details
/// 2. Load 500 template
/// 3. Render with error context (in DEV only)
/// 4. Return with 500 status code
///
/// ## Security
/// - DEV: Show error details
/// - PROD: Generic error message only
pub fn build_500_response(error: ReedError) -> HttpResponse {
    // Log error
    eprintln!("Server error: {:?}", error);

    let variant = "mouse";
    let template_name = format!("error.500.{}", variant);

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
            .content_type("text/plain")
            .body("500 - Internal Server Error"),
    }
}

/// Renders error template with context.
fn render_error_template(template_name: &str, code: u16, message: &str) -> ReedResult<String> {
    let env = get_template_engine();

    // Build minimal error context
    let mut context = Context::new();
    context.insert("error_code", &code);
    context.insert("error_message", message);
    context.insert("site_name", "ReedCMS");

    let template = env.get_template(template_name).map_err(|e| ReedError::TemplateError {
        template: template_name.to_string(),
        reason: format!("Error template not found: {}", e),
    })?;

    template.render(&context).map_err(|e| ReedError::TemplateError {
        template: template_name.to_string(),
        reason: format!("Error template render failed: {}", e),
    })
}

/// Checks if running in DEV environment.
fn is_dev_environment() -> bool {
    std::env::var("REED_ENV")
        .unwrap_or_else(|_| "PROD".to_string())
        .to_uppercase()
        == "DEV"
}
```

### Content Negotiation (`src/reedcms/response/content_type.rs`)

```rust
/// Determines response content type based on request.
///
/// ## Accept Header Processing
/// - text/html → HTML response (default)
/// - application/json → JSON response
/// - text/plain → Plain text response
///
/// ## Usage
/// Enables API endpoints to return different formats:
/// - HTML for browser requests
/// - JSON for API clients
/// - Plain text for debugging
pub fn negotiate_content_type(req: &HttpRequest) -> ContentType {
    let accept_header = req
        .headers()
        .get("Accept")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("text/html");

    if accept_header.contains("application/json") {
        ContentType::Json
    } else if accept_header.contains("text/plain") {
        ContentType::Plain
    } else {
        ContentType::Html
    }
}

/// Content type enum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentType {
    Html,
    Json,
    Plain,
}

impl ContentType {
    /// Returns MIME type string.
    pub fn mime_type(&self) -> &'static str {
        match self {
            ContentType::Html => "text/html; charset=utf-8",
            ContentType::Json => "application/json; charset=utf-8",
            ContentType::Plain => "text/plain; charset=utf-8",
        }
    }
}
```

## Implementation Files

### Primary Implementation
- `src/reedcms/response/builder.rs` - Response builder
- `src/reedcms/response/cache.rs` - Cache control headers
- `src/reedcms/response/errors.rs` - Error responses
- `src/reedcms/response/content_type.rs` - Content negotiation

### Test Files
- `src/reedcms/response/builder.test.rs`
- `src/reedcms/response/cache.test.rs`
- `src/reedcms/response/errors.test.rs`
- `src/reedcms/response/content_type.test.rs`

## File Structure
```
src/reedcms/response/
├── builder.rs              # Response builder
├── builder.test.rs         # Builder tests
├── cache.rs                # Cache headers
├── cache.test.rs           # Cache tests
├── errors.rs               # Error responses
├── errors.test.rs          # Error tests
├── content_type.rs         # Content negotiation
└── content_type.test.rs    # Content type tests
```

## Testing Requirements

### Unit Tests
- [ ] Test URL resolution in response flow
- [ ] Test variant detection from User-Agent
- [ ] Test context building integration
- [ ] Test template rendering
- [ ] Test cache control header generation
- [ ] Test 404 response building
- [ ] Test 500 response building
- [ ] Test content type negotiation

### Integration Tests
- [ ] Test complete response flow (URL → HTML)
- [ ] Test different variants (mouse/touch/reader)
- [ ] Test error responses with templates
- [ ] Test cache headers for different layouts
- [ ] Test content negotiation with different Accept headers
- [ ] Test response with authentication context

### Performance Tests
- [ ] Complete response: < 50ms
- [ ] Cached response: < 10ms
- [ ] Variant detection: < 1ms
- [ ] Error response: < 20ms

### Error Handling Tests
- [ ] Test missing template handling
- [ ] Test invalid route handling
- [ ] Test rendering errors
- [ ] Test context building failures

## Acceptance Criteria
- [ ] Complete request-to-response flow working
- [ ] URL routing integrated (REED-06-02)
- [ ] Template rendering integrated (REED-05-02, REED-05-03)
- [ ] Variant detection functional (mouse/touch/reader)
- [ ] Cache-Control headers correct
- [ ] Custom headers set (X-Layout, X-Language, X-Variant)
- [ ] 404 error responses working
- [ ] 500 error responses working
- [ ] Content negotiation implemented
- [ ] DEV/PROD environment handling
- [ ] All tests pass with 100% coverage
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-06-02 (Routing), REED-05-02 (Template Engine), REED-05-03 (Context Builder)

## Blocks
- REED-07-01 (ReedAPI needs response builder)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1010-1013 in `project_summary.md`

## Notes
Response builder is the final step in request processing, orchestrating all previous layers. Variant detection enables responsive design at the template level rather than client-side. Cache-Control headers are critical for performance in production. Error responses must be secure (no error details in PROD) but helpful in DEV. Custom X-* headers provide debugging information without impacting caching. Template fallback ensures graceful degradation when templates are missing.
