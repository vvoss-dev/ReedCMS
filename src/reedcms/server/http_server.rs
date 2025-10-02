// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! HTTP server implementation using Actix-Web.
//!
//! Provides HTTP server foundation with configurable workers and port binding.

use crate::reedcms::reedbase;
use crate::reedcms::reedstream::{ReedError, ReedRequest, ReedResult};
use crate::reedcms::routing::resolver::resolve_url;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};

/// Starts HTTP server on specified port.
///
/// ## Arguments
/// - port: Port number (default: 8333)
/// - workers: Number of worker threads (default: CPU count)
///
/// ## Process
/// 1. Initialize Actix-Web App
/// 2. Configure middleware (Logger, Compress)
/// 3. Register routes
/// 4. Bind to address
/// 5. Start server
///
/// ## Performance
/// - Startup time: < 500ms
/// - Request handling: < 10ms average
/// - Concurrent connections: 10k+
///
/// ## Output
/// ```
/// Starting HTTP server on 127.0.0.1:8333
/// Worker threads: 4
/// Server started successfully
/// ```
pub async fn start_http_server(port: u16, workers: Option<usize>) -> ReedResult<()> {
    let worker_count = workers.unwrap_or_else(num_cpus::get);

    println!("🚀 Starting ReedCMS HTTP server...");
    println!("   Port: {}", port);
    println!("   Workers: {}", worker_count);

    let server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(configure_routes)
    })
    .workers(worker_count)
    .bind(format!("127.0.0.1:{}", port))
    .map_err(|e| ReedError::IoError {
        operation: "bind".to_string(),
        path: format!("127.0.0.1:{}", port),
        reason: e.to_string(),
    })?
    .run();

    println!("✓ Server started successfully");
    println!("  Access at: http://127.0.0.1:{}", port);

    server.await.map_err(|e| ReedError::ConfigError {
        component: "http_server".to_string(),
        reason: format!("Server error: {}", e),
    })
}

/// Configures application routes.
///
/// ## Routes
/// - GET /* → handle_request (catch-all)
///
/// ## Future Routes
/// - Will be extended by REED-06-02 (Routing)
/// - Will be extended by REED-06-03 (Authentication)
/// - Will be extended by REED-06-04 (Response Builder)
fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/{path:.*}").route(web::get().to(handle_request)));
}

/// Main request handler.
///
/// ## Process
/// 1. Resolve URL to layout + language (REED-06-02)
/// 2. Build response with template rendering (REED-06-04)
///
/// ## Current Implementation
/// - URL routing implemented (REED-06-02)
/// - Template rendering placeholder (REED-06-04)
/// - Client detection placeholder (REED-06-05)
async fn handle_request(req: HttpRequest, _path: web::Path<String>) -> HttpResponse {
    let url = req.path();
    println!("Request: {} {}", req.method(), url);

    // Resolve URL to layout + language
    match resolve_url(url) {
        Ok(route_info) => {
            println!(
                "  Resolved: layout={}, language={}, params={:?}",
                route_info.layout, route_info.language, route_info.params
            );

            // Placeholder response until REED-06-04 (Response Builder)
            let html = format!(
                r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="utf-8">
    <title>ReedCMS</title>
</head>
<body>
    <h1>ReedCMS Routing Active</h1>
    <p><strong>Layout:</strong> {}</p>
    <p><strong>Language:</strong> {}</p>
    <p><strong>Params:</strong> {:?}</p>
    <p><em>Template rendering will be added in REED-06-04</em></p>
</body>
</html>"#,
                route_info.language, route_info.layout, route_info.language, route_info.params
            );

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(e) => {
            println!("  Route not found: {}", e);

            // Try to get 404 texts from ReedBase, fallback to hardcoded English
            // Hardcoded fallback ensures error pages work even if text.csv is broken
            let lang = "en"; // TODO: REED-06-05 will detect language from Accept-Language
            let title = get_error_text("error.404.title", lang)
                .unwrap_or_else(|| "404 - Not Found".to_string());
            let message = get_error_text("error.404.message", lang)
                .unwrap_or_else(|| "The requested page does not exist.".to_string());

            // 404 response
            let html = format!(
                r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="utf-8">
    <title>{}</title>
</head>
<body>
    <h1>{}</h1>
    <p>{}</p>
</body>
</html>"#,
                lang, title, title, message
            );

            HttpResponse::NotFound()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
    }
}

/// Gets error text from ReedBase.
///
/// ## Arguments
/// - key: Text key (e.g., "error.404.title")
/// - lang: Language code (e.g., "en", "de")
///
/// ## Returns
/// - Some(text) if found in ReedBase
/// - None if not found (caller should use fallback)
fn get_error_text(key: &str, lang: &str) -> Option<String> {
    let req = ReedRequest {
        key: format!("{}@{}", key, lang),
        language: Some(lang.to_string()),
        environment: None,
        context: Some("http_server".to_string()),
        value: None,
        description: None,
    };

    reedbase::get::text(&req).ok().map(|r| r.data)
}
