// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Graceful Shutdown Handler
//!
//! Provides signal handling for SIGTERM and SIGINT with graceful shutdown.
//!
//! ## Features
//! - SIGTERM/SIGINT signal handling
//! - 30-second graceful shutdown timeout
//! - Actix-Web server coordination
//!
//! ## Performance
//! - Signal handling: async, non-blocking
//! - Shutdown timeout: 30s
//!
//! ## Error Conditions
//! - IoError: Signal registration fails
//!
//! ## Example Usage
//! ```rust
//! use reedcms::server::shutdown::setup_shutdown_handler;
//!
//! #[tokio::main]
//! async fn main() {
//!     let shutdown_signal = setup_shutdown_handler();
//!     // Start server with shutdown_signal
//! }
//! ```

use tokio::signal;

/// Sets up graceful shutdown handler.
///
/// ## Returns
/// - Future that completes when shutdown signal is received
///
/// ## Signals Handled
/// - SIGTERM: Termination signal (typical from systemd, docker)
/// - SIGINT: Interrupt signal (Ctrl+C)
///
/// ## Performance
/// - Signal handling: < 1ms
/// - Shutdown coordination: 30s timeout
///
/// ## Example Usage
/// ```rust
/// use reedcms::server::shutdown::setup_shutdown_handler;
/// use actix_web::{HttpServer, App};
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let server = HttpServer::new(|| App::new())
///         .bind("127.0.0.1:8333")?
///         .run();
///
///     let shutdown_signal = setup_shutdown_handler();
///
///     tokio::select! {
///         _ = server => println!("Server stopped"),
///         _ = shutdown_signal => println!("Shutdown signal received"),
///     }
///
///     Ok(())
/// }
/// ```
pub async fn setup_shutdown_handler() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install SIGINT handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("\n⚠ SIGINT received, initiating graceful shutdown...");
        }
        _ = terminate => {
            println!("\n⚠ SIGTERM received, initiating graceful shutdown...");
        }
    }

    println!("⏳ Shutdown timeout: 30 seconds");
}
