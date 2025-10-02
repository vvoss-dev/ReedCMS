// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Unix Socket Server Implementation
//!
//! Provides Unix domain socket server for nginx/apache reverse proxy integration.
//!
//! ## Features
//! - Automatic socket directory creation
//! - Socket file cleanup on startup
//! - Configurable worker threads
//! - Socket permissions (0o666)
//!
//! ## Performance
//! - Unix sockets: faster than TCP for local communication
//! - Zero network stack overhead
//!
//! ## Error Conditions
//! - IoError: Socket file operations fail
//! - ConfigError: Invalid socket path
//!
//! ## Example Usage
//! ```rust
//! use reedcms::server::socket_server::start_socket_server;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     start_socket_server("/tmp/reedcms.sock", Some(4)).await?;
//!     Ok(())
//! }
//! ```

use crate::reedcms::auth::SiteProtection;
use crate::reedcms::reedbase;
use crate::reedcms::reedstream::{ReedError, ReedRequest, ReedResult};
use crate::reedcms::routing::resolver::resolve_url;
use crate::reedcms::server::client_detection::{detect_client_info, is_bot_request};
use crate::reedcms::server::screen_detection::{generate_screen_detection_html, needs_screen_detection};
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Configures routes for the application.
///
/// ## Arguments
/// - cfg: Service configuration
///
/// ## Implementation Note
/// This is a placeholder. REED-06-02 will implement proper routing via routes.csv.
fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/{path:.*}").route(web::get().to(handle_request)));
}

/// Handles incoming HTTP requests.
///
/// ## Arguments
/// - req: HTTP request
/// - path: Requested path
///
/// ## Returns
/// - HTTP response
///
/// ## Implementation Note
/// Screen detection (REED-06-05), URL routing (REED-06-02), and client detection (REED-06-05) implemented.
/// Template rendering will be added in REED-06-04.
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

    // 2. Resolve URL to layout + language
    match resolve_url(url) {
        Ok(route_info) => {
            // 3. Detect client information
            let client_info = match detect_client_info(&req, &route_info.language) {
                Ok(info) => info,
                Err(e) => {
                    println!("  Client detection error: {}", e);
                    return HttpResponse::InternalServerError()
                        .content_type("text/html; charset=utf-8")
                        .body("<h1>500 - Internal Server Error</h1><p>Client detection failed.</p>");
                }
            };

            println!(
                "  Resolved: layout={}, language={}, params={:?}",
                route_info.layout, route_info.language, route_info.params
            );
            println!(
                "  Client: mode={}, device={}, breakpoint={}, bot={}",
                client_info.interaction_mode, client_info.device_type, client_info.breakpoint, client_info.is_bot
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
    <h1>ReedCMS Routing + Client Detection Active (Unix Socket)</h1>
    <h2>Route Information</h2>
    <p><strong>Layout:</strong> {}</p>
    <p><strong>Language:</strong> {}</p>
    <p><strong>Params:</strong> {:?}</p>
    <h2>Client Information</h2>
    <p><strong>Interaction Mode:</strong> {}</p>
    <p><strong>Device Type:</strong> {}</p>
    <p><strong>Breakpoint:</strong> {}</p>
    <p><strong>Is Bot:</strong> {}</p>
    {}
    <p><em>Template rendering will be added in REED-06-04</em></p>
</body>
</html>"#,
                route_info.language,
                route_info.layout,
                route_info.language,
                route_info.params,
                client_info.interaction_mode,
                client_info.device_type,
                client_info.breakpoint,
                client_info.is_bot,
                if let Some(vw) = client_info.viewport_width {
                    format!("<p><strong>Viewport:</strong> {}x{}</p>", vw, client_info.viewport_height.unwrap_or(0))
                } else {
                    String::new()
                }
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

/// Sets Unix socket file permissions.
///
/// ## Arguments
/// - socket_path: Path to socket file
///
/// ## Returns
/// - ReedResult<()>
///
/// ## Error Conditions
/// - IoError: Cannot set permissions
fn set_socket_permissions(socket_path: &str) -> ReedResult<()> {
    let metadata = fs::metadata(socket_path).map_err(|e| ReedError::IoError {
        operation: "metadata".to_string(),
        path: socket_path.to_string(),
        reason: e.to_string(),
    })?;

    let mut permissions = metadata.permissions();
    permissions.set_mode(0o666);

    fs::set_permissions(socket_path, permissions).map_err(|e| ReedError::IoError {
        operation: "set_permissions".to_string(),
        path: socket_path.to_string(),
        reason: e.to_string(),
    })?;

    Ok(())
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
        context: Some("socket_server".to_string()),
        value: None,
        description: None,
    };

    reedbase::get::text(&req).ok().map(|r| r.data)
}


/// Starts Unix socket server.
///
/// ## Arguments
/// - socket_path: Path to Unix socket file
/// - workers: Optional worker thread count (defaults to CPU count)
///
/// ## Returns
/// - ReedResult<()>
///
/// ## Performance
/// - Unix sockets: 20-30% faster than TCP for local communication
/// - Worker threads: One per CPU core by default
///
/// ## Error Conditions
/// - IoError: Socket file operations fail
/// - ConfigError: Invalid socket path
/// - ServerError: Server binding fails
///
/// ## Example Usage
/// ```rust
/// use reedcms::server::socket_server::start_socket_server;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     start_socket_server("/tmp/reedcms.sock", Some(4)).await?;
///     Ok(())
/// }
/// ```
pub async fn start_socket_server(socket_path: &str, workers: Option<usize>) -> ReedResult<()> {
    let worker_count = workers.unwrap_or_else(num_cpus::get);

    println!("🚀 Starting ReedCMS Unix Socket server...");
    println!("   Socket: {}", socket_path);
    println!("   Workers: {}", worker_count);

    // Create socket directory if it doesn't exist
    let socket_dir = Path::new(socket_path)
        .parent()
        .ok_or_else(|| ReedError::ConfigError {
            component: "socket_server".to_string(),
            reason: format!("Invalid socket path: {}", socket_path),
        })?;

    if !socket_dir.exists() {
        fs::create_dir_all(socket_dir).map_err(|e| ReedError::IoError {
            operation: "create_dir_all".to_string(),
            path: socket_dir.to_string_lossy().to_string(),
            reason: e.to_string(),
        })?;
    }

    // Remove existing socket file
    if Path::new(socket_path).exists() {
        fs::remove_file(socket_path).map_err(|e| ReedError::IoError {
            operation: "remove_file".to_string(),
            path: socket_path.to_string(),
            reason: e.to_string(),
        })?;
    }

    // Start server
    let server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(SiteProtection::new())  // Simple htaccess-style site protection
            .configure(configure_routes)
    })
    .workers(worker_count)
    .bind_uds(socket_path)
    .map_err(|e| ReedError::ServerError {
        component: "socket_server".to_string(),
        reason: format!("Failed to bind Unix socket: {}", e),
    })?;

    // Set socket permissions
    set_socket_permissions(socket_path)?;

    println!("✓ Server ready");

    // Run server
    server.run().await.map_err(|e| ReedError::ServerError {
        component: "socket_server".to_string(),
        reason: format!("Server runtime error: {}", e),
    })?;

    Ok(())
}
