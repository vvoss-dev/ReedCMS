// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Server configuration loading from .reed/server.csv.
//!
//! Loads server configuration including bind type, address, socket path, and worker count.

use crate::reedcms::reedstream::{ReedRequest, ReedResult};

/// Server configuration structure.
///
/// ## Fields
/// - bind_type: HTTP or Unix socket ("http" or "unix")
/// - bind_address: IP:Port for HTTP binding
/// - socket_path: Path for Unix socket
/// - workers: Number of worker threads
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_type: Option<String>,
    pub bind_address: Option<String>,
    pub socket_path: Option<String>,
    pub workers: Option<usize>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_type: Some("http".to_string()),
            bind_address: Some("127.0.0.1:8333".to_string()),
            socket_path: None,
            workers: None,
        }
    }
}

/// Loads server configuration from .reed/server.csv.
///
/// ## Configuration Keys (environment-aware)
/// - server.{env}.io: "127.0.0.1:8333" or "/var/run/reed.sock"
/// - server.workers: "4"
///
/// ## Performance
/// - Load time: < 10ms
/// - Cached after first load
///
/// ## Output
/// - ServerConfig struct with parsed values
/// - Default values if keys missing
pub fn load_server_config() -> ReedResult<ServerConfig> {
    let mut config = ServerConfig::default();

    // Detect environment
    let env = std::env::var("REED_ENV")
        .ok()
        .or_else(|| crate::reedcms::cli::server_commands::load_env_var("ENVIRONMENT"))
        .unwrap_or_else(|| "prod".to_string())
        .to_lowercase();

    // Load environment-specific io setting
    let io_key = format!("server.{}.io", env);
    if let Ok(io_value) = get_config_value(&io_key) {
        // Parse io value: either "IP:PORT" or "/path/to/socket"
        if io_value.starts_with('/') {
            // Unix socket
            config.bind_type = Some("unix".to_string());
            config.socket_path = Some(io_value);
        } else {
            // HTTP binding
            config.bind_type = Some("http".to_string());
            config.bind_address = Some(io_value);
        }
    }

    // Load workers
    if let Ok(workers_str) = get_config_value("server.workers") {
        if let Ok(workers) = workers_str.parse::<usize>() {
            config.workers = Some(workers);
        }
    }

    Ok(config)
}

/// Gets configuration value from ReedBase.
fn get_config_value(key: &str) -> ReedResult<String> {
    let req = ReedRequest {
        key: key.to_string(),
        language: None,
        environment: None,
        context: None,
        value: None,
        description: None,
    };

    match crate::reedcms::reedbase::get::server(&req) {
        Ok(response) => Ok(response.data),
        Err(e) => Err(e),
    }
}
