// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

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

use crate::reedcms::reedstream::{ReedError, ReedResult};
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
/// This is a placeholder. REED-06-04 will implement proper response building.
async fn handle_request(_req: HttpRequest, _path: web::Path<String>) -> HttpResponse {
    println!("Request: {} {}", _req.method(), _req.path());
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("ReedCMS Server Foundation Active (Unix Socket)")
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
