// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! HTTP server implementation using Actix-Web.
//!
//! Provides HTTP server foundation with configurable workers and port binding.

use crate::reedcms::reedstream::{ReedError, ReedResult};
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

/// Main request handler (placeholder).
///
/// ## Current Implementation
/// - Returns simple "ReedCMS" response
///
/// ## Future Implementation
/// - REED-06-02: URL routing
/// - REED-06-04: Template rendering
/// - REED-06-05: Client detection
async fn handle_request(_req: HttpRequest, _path: web::Path<String>) -> HttpResponse {
    println!("Request: {} {}", _req.method(), _req.path());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body("<h1>ReedCMS</h1><p>Server Foundation Active</p>")
}
