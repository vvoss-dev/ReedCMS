// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Configuration inspector for debugging.
//!
//! ## Features
//! - Display project configuration
//! - Show server settings
//! - List environment variables
//!
//! ## CLI Usage
//! ```bash
//! reed debug:config
//! ```

use std::collections::HashMap;

/// Configuration inspection data.
#[derive(Debug, Clone)]
pub struct ConfigInspection {
    pub project: HashMap<String, String>,
    pub server: HashMap<String, String>,
    pub environment: HashMap<String, String>,
}

impl ConfigInspection {
    /// Creates new config inspection.
    pub fn new() -> Self {
        let mut project = HashMap::new();
        project.insert("name".to_string(), "ReedCMS".to_string());
        project.insert("version".to_string(), "0.1.0".to_string());

        let mut server = HashMap::new();
        server.insert("port".to_string(), "8333".to_string());
        server.insert("workers".to_string(), "4".to_string());

        let mut environment = HashMap::new();
        environment.insert(
            "ENVIRONMENT".to_string(),
            std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
        );
        environment.insert(
            "REED_PROFILE".to_string(),
            std::env::var("REED_PROFILE").unwrap_or_else(|_| "false".to_string()),
        );
        environment.insert(
            "REED_SLOW_THRESHOLD".to_string(),
            std::env::var("REED_SLOW_THRESHOLD").unwrap_or_else(|_| "100".to_string()),
        );

        Self {
            project,
            server,
            environment,
        }
    }

    /// Formats configuration.
    pub fn format(&self) -> String {
        let mut output = String::from("⚙️  Configuration Inspector\n\n");

        output.push_str("Project Configuration:\n");
        for (key, value) in &self.project {
            output.push_str(&format!("  {}: {}\n", key, value));
        }

        output.push_str("\nServer Configuration:\n");
        for (key, value) in &self.server {
            output.push_str(&format!("  {}: {}\n", key, value));
        }

        output.push_str("\nEnvironment Variables:\n");
        for (key, value) in &self.environment {
            output.push_str(&format!("  {}: {}\n", key, value));
        }

        output.push_str("\nConfiguration Files:\n");
        output.push_str("  - .reed/project.csv (runtime configuration)\n");
        output.push_str("  - .reed/server.csv (server settings)\n");
        output.push_str("  - Reed.toml (bootstrap configuration)\n");
        output.push_str("  - .env (environment control)\n");

        output.push_str("\nNote: For complete configuration, see reed config:show command.\n");

        output
    }
}

impl Default for ConfigInspection {
    fn default() -> Self {
        Self::new()
    }
}

/// Inspects configuration.
///
/// ## Returns
/// Configuration inspection data
pub fn inspect_config() -> ConfigInspection {
    ConfigInspection::new()
}
