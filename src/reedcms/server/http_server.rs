// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! HTTP server implementation using Actix-Web.
//!
//! Provides HTTP server foundation with configurable workers and port binding.

use crate::reedcms::auth::SiteProtection;
use crate::reedcms::response::builder::build_response;
use crate::reedcms::reedstream::{ReedError, ReedResult};
use crate::reedcms::server::client_detection::is_bot_request;
use crate::reedcms::server::screen_detection::{generate_screen_detection_html, needs_screen_detection};
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};

/// Starts HTTP server on specified port.
///
/// ## Arguments
/// - port: Port number (default: 8333)
/// - workers: Number of worker threads (default: CPU count)
///
/// ## Process
/// 1. Initialise Actix-Web App
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
            .wrap(SiteProtection::new())
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
fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/{path:.*}").route(web::get().to(handle_request)));
}

/// Main request handler.
///
/// ## Process
/// 1. Check if screen detection needed (REED-06-05)
/// 2. Build complete response with template rendering (REED-06-04)
///
/// ## Implementation
/// - Screen detection: ✅ Implemented (REED-06-05)
/// - URL routing: ✅ Implemented (REED-06-02)
/// - Client detection: ✅ Implemented (REED-06-05)
/// - Template rendering: ✅ Implemented (REED-06-04)
async fn handle_request(req: HttpRequest, _path: web::Path<String>) -> HttpResponse {
    let url = req.path();
    println!("Request: {} {}", req.method(), url);

    // 1. Check for screen detection (skip for bots)
    if needs_screen_detection(&req) && !is_bot_request(&req) {
        println!("  First visit - sending screen detection HTML");
        return HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(generate_screen_detection_html());
    }

    // 2. Build complete response with template rendering
    match build_response(req).await {
        Ok(response) => {
            println!("  ✓ Response built successfully");
            response
        }
        Err(e) => {
            println!("  ✗ Response build failed: {}", e);
            HttpResponse::InternalServerError()
                .content_type("text/html; charset=utf-8")
                .body(format!("<h1>500 - Internal Server Error</h1><p>{}</p>", e))
        }
    }
}
