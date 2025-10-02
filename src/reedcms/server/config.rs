// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

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
/// ## Configuration Keys
/// - server.bind_type: "http" or "unix"
/// - server.bind_address: "127.0.0.1:8333"
/// - server.socket_path: "/var/run/reedcms/web.sock"
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

    // Load bind_type
    if let Ok(bind_type) = get_config_value("server.bind_type") {
        config.bind_type = Some(bind_type);
    }

    // Load bind_address
    if let Ok(bind_address) = get_config_value("server.bind_address") {
        config.bind_address = Some(bind_address);
    }

    // Load socket_path
    if let Ok(socket_path) = get_config_value("server.socket_path") {
        config.socket_path = Some(socket_path);
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
