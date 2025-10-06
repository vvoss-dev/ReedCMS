// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CLI commands for Reed.toml configuration management.
//!
//! Provides commands to sync, initialise, show, and validate Reed.toml configuration.

use crate::reedcms::config::{parse_reed_toml, sync_toml_to_csv, validate_config};
use crate::reedcms::reedstream::{ReedError, ReedResponse, ReedResult};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Synchronises Reed.toml configuration to .reed/*.csv files.
///
/// ⚠️  WARNING: This OVERWRITES current CSV values with Reed.toml values!
/// Use 'reed config:export' first to backup current configuration.
///
/// ## Usage
/// ```bash
/// reed config:sync
/// reed config:sync --file=custom.toml
/// reed config:sync --force  # Skip confirmation prompt
/// ```
///
/// ## Flags
/// - --file: Path to Reed.toml file (default: Reed.toml in project root)
/// - --force: Skip confirmation prompt
///
/// ## Output
/// - Success message with number of updated keys
/// - List of all updated keys
///
/// ## Error Conditions
/// - Reed.toml not found
/// - Invalid TOML syntax
/// - Validation errors
/// - CSV write errors
pub fn config_sync(
    _args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    // Get file path from flag or use default
    let file_path = flags.get("file").map(|s| s.as_str()).unwrap_or("Reed.toml");

    // Check if file exists
    if !Path::new(file_path).exists() {
        return Err(ReedError::FileNotFound {
            path: file_path.to_string(),
            reason: "Reed.toml configuration file not found".to_string(),
        });
    }

    // Show warning unless --force is set
    if !flags.contains_key("force") {
        let warning = format!(
            "\n⚠️  WARNING: This will OVERWRITE current configuration in CSV files!\n\
             ⚠️  Current values will be replaced with Reed.toml values.\n\
             ⚠️  Run 'reed config:export' first to backup current config.\n\n\
             Reading from: {}\n\
             Writing to: .reed/project.csv, .reed/server.csv\n\n\
             Continue? (y/N): ",
            file_path
        );

        // In non-interactive mode, abort
        return Err(ReedError::ConfigError {
            component: "config:sync".to_string(),
            reason: format!("{}Use --force flag to skip confirmation", warning),
        });
    }

    // Parse configuration
    let config = parse_reed_toml(file_path)?;

    // Validate configuration
    validate_config(&config)?;

    // Sync to CSV files
    let updated_keys = sync_toml_to_csv(&config)?;

    // Build response message
    let mut message = format!(
        "✓ Successfully synchronised {} configuration values from Reed.toml → CSV\n\n",
        updated_keys.len()
    );
    message.push_str("Updated keys:\n");
    for key in &updated_keys {
        message.push_str(&format!("  - {}\n", key));
    }

    Ok(ReedResponse::new(message, "config:sync"))
}

/// Initialises a new Reed.toml configuration file.
///
/// ## Usage
/// ```bash
/// reed config:init
/// reed config:init --file=custom.toml
/// ```
///
/// ## Flags
/// - --file: Path to create Reed.toml (default: Reed.toml in project root)
/// - --force: Overwrite existing file
///
/// ## Output
/// - Success message with file path
///
/// ## Error Conditions
/// - File already exists (without --force)
/// - Permission denied
pub fn config_init(
    _args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    // Get file path from flag or use default
    let file_path = flags.get("file").map(|s| s.as_str()).unwrap_or("Reed.toml");

    // Check if file exists and --force not set
    if Path::new(file_path).exists() && !flags.contains_key("force") {
        return Err(ReedError::ConfigError {
            component: "config:init".to_string(),
            reason: format!(
                "Reed.toml already exists at: {}. Use --force to overwrite",
                file_path
            ),
        });
    }

    // Read template content
    let template = include_str!("../templates/Reed.toml.template");

    // Write file
    fs::write(file_path, template).map_err(|e| ReedError::WriteError {
        path: file_path.to_string(),
        reason: format!("Failed to write Reed.toml: {}", e),
    })?;

    let message = format!("✓ Successfully created Reed.toml at: {}", file_path);

    Ok(ReedResponse::new(message, "config:init"))
}

/// Displays current Reed.toml configuration.
///
/// ## Usage
/// ```bash
/// reed config:show
/// reed config:show --file=custom.toml
/// reed config:show --section=project
/// ```
///
/// ## Flags
/// - --file: Path to Reed.toml file (default: Reed.toml in project root)
/// - --section: Show only specific section (project, server, dev)
///
/// ## Output
/// - Formatted configuration display
///
/// ## Error Conditions
/// - Reed.toml not found
/// - Invalid TOML syntax
pub fn config_show(
    _args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    // Get file path from flag or use default
    let file_path = flags.get("file").map(|s| s.as_str()).unwrap_or("Reed.toml");

    // Check if file exists
    if !Path::new(file_path).exists() {
        return Err(ReedError::FileNotFound {
            path: file_path.to_string(),
            reason: "Reed.toml configuration file not found".to_string(),
        });
    }

    // Parse configuration
    let config = parse_reed_toml(file_path)?;

    // Build display message
    let section = flags.get("section").map(|s| s.as_str());
    let mut message = format!("Reed.toml Configuration ({})\n\n", file_path);

    match section {
        Some("project") => {
            message.push_str(&format_project_section(&config));
        }
        Some("server") => {
            message.push_str(&format_server_section(&config));
        }
        None => {
            message.push_str(&format_project_section(&config));
            message.push('\n');
            message.push_str(&format_server_section(&config));
        }
        Some(unknown) => {
            return Err(ReedError::ConfigError {
                component: "config:show".to_string(),
                reason: format!(
                    "Unknown section: {}. Valid sections: project, server",
                    unknown
                ),
            });
        }
    }

    Ok(ReedResponse::new(message, "config:show"))
}

/// Validates Reed.toml configuration without syncing.
///
/// ## Usage
/// ```bash
/// reed config:validate
/// reed config:validate --file=custom.toml
/// ```
///
/// ## Flags
/// - --file: Path to Reed.toml file (default: Reed.toml in project root)
///
/// ## Output
/// - Success message if valid
/// - Detailed error messages if invalid
///
/// ## Error Conditions
/// - Reed.toml not found
/// - Invalid TOML syntax
/// - Validation errors (invalid values, missing required fields)
pub fn config_validate(
    _args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    // Get file path from flag or use default
    let file_path = flags.get("file").map(|s| s.as_str()).unwrap_or("Reed.toml");

    // Check if file exists
    if !Path::new(file_path).exists() {
        return Err(ReedError::FileNotFound {
            path: file_path.to_string(),
            reason: "Reed.toml configuration file not found".to_string(),
        });
    }

    // Parse configuration
    let config = parse_reed_toml(file_path)?;

    // Validate configuration
    validate_config(&config)?;

    let message = format!("✓ Reed.toml configuration is valid ({})", file_path);

    Ok(ReedResponse::new(message, "config:validate"))
}

/// Exports current CSV configuration to Reed.toml file.
///
/// This reads values from .reed/*.csv and writes them to Reed.toml.
/// Use this to backup or version-control your current configuration.
///
/// ## Usage
/// ```bash
/// reed config:export
/// reed config:export --file=backup.toml
/// reed config:export --force  # Overwrite existing file
/// ```
///
/// ## Flags
/// - --file: Output file path (default: Reed.toml in project root)
/// - --force: Overwrite existing file without prompt
///
/// ## Output
/// - Success message with file path and number of exported values
///
/// ## Error Conditions
/// - CSV files not found
/// - File already exists (without --force)
/// - Write permission denied
pub fn config_export(
    _args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    let file_path = flags.get("file").map(|s| s.as_str()).unwrap_or("Reed.toml");

    // Check if file exists and --force not set
    if Path::new(file_path).exists() && !flags.contains_key("force") {
        return Err(ReedError::ConfigError {
            component: "config:export".to_string(),
            reason: format!(
                "File already exists: {}. Use --force to overwrite",
                file_path
            ),
        });
    }

    // Read values from CSV files
    use crate::reedcms::csv::read_csv;
    let mut project_values = HashMap::new();
    let mut server_values = HashMap::new();

    // Read project.csv
    if Path::new(".reed/project.csv").exists() {
        let records = read_csv(".reed/project.csv")?;
        for record in records {
            project_values.insert(record.key, record.value);
        }
    }

    // Read server.csv
    if Path::new(".reed/server.csv").exists() {
        let records = read_csv(".reed/server.csv")?;
        for record in records {
            server_values.insert(record.key, record.value);
        }
    }

    // Build TOML content
    let mut toml_content = String::from(
        "# Reed.toml - Exported Configuration\n\
         # Generated by: reed config:export\n\
         # Source: .reed/*.csv files\n\n",
    );

    // Export [project] section
    if !project_values.is_empty() {
        toml_content.push_str("[project]\n");
        if let Some(name) = project_values.get("project.name") {
            toml_content.push_str(&format!("name = \"{}\"\n", name));
        }
        if let Some(url) = project_values.get("project.url") {
            toml_content.push_str(&format!("url = \"{}\"\n", url));
        }
        if let Some(desc) = project_values.get("project.description") {
            toml_content.push_str(&format!("description = \"{}\"\n", desc));
        }
        toml_content.push('\n');

        // Languages
        toml_content.push_str("[project.languages]\n");
        if let Some(default) = project_values.get("project.languages.default") {
            toml_content.push_str(&format!("default = \"{}\"\n", default));
        }
        if let Some(available) = project_values.get("project.languages.available") {
            let langs: Vec<&str> = available.split(',').collect();
            toml_content.push_str(&format!("available = {:?}\n", langs));
        }
        if let Some(fallback) = project_values.get("project.languages.fallback_chain") {
            toml_content.push_str(&format!("fallback_chain = {}\n", fallback));
        }
        toml_content.push('\n');

        // Routing
        if project_values.contains_key("project.routing.url_prefix") {
            toml_content.push_str("[project.routing]\n");
            if let Some(prefix) = project_values.get("project.routing.url_prefix") {
                toml_content.push_str(&format!("url_prefix = {}\n", prefix));
            }
            if let Some(slash) = project_values.get("project.routing.trailing_slash") {
                toml_content.push_str(&format!("trailing_slash = {}\n", slash));
            }
            toml_content.push('\n');
        }

        // Templates
        if project_values.contains_key("project.templates.auto_reload") {
            toml_content.push_str("[project.templates]\n");
            if let Some(reload) = project_values.get("project.templates.auto_reload") {
                toml_content.push_str(&format!("auto_reload = {}\n", reload));
            }
            if let Some(cache) = project_values.get("project.templates.cache_templates") {
                toml_content.push_str(&format!("cache_templates = {}\n", cache));
            }
            toml_content.push('\n');
        }

        // Assets
        if project_values.contains_key("project.assets.css_minify") {
            toml_content.push_str("[project.assets]\n");
            if let Some(minify) = project_values.get("project.assets.css_minify") {
                toml_content.push_str(&format!("css_minify = {}\n", minify));
            }
            if let Some(bundle) = project_values.get("project.assets.css_bundle") {
                toml_content.push_str(&format!("css_bundle = {}\n", bundle));
            }
            toml_content.push('\n');
        }

        // Build
        if project_values.contains_key("project.build.clean_before") {
            toml_content.push_str("[project.build]\n");
            if let Some(clean) = project_values.get("project.build.clean_before") {
                toml_content.push_str(&format!("clean_before = {}\n", clean));
            }
            if let Some(parallel) = project_values.get("project.build.parallel") {
                toml_content.push_str(&format!("parallel = {}\n", parallel));
            }
            toml_content.push('\n');
        }
    }

    // Export [server] section
    if !server_values.is_empty() {
        toml_content.push_str("[server]\n");
        if let Some(workers) = server_values.get("server.workers") {
            toml_content.push_str(&format!("workers = {}\n", workers));
        }
        toml_content.push('\n');

        // Security
        if server_values.contains_key("server.security.enable_cors") {
            toml_content.push_str("[server.security]\n");
            if let Some(cors) = server_values.get("server.security.enable_cors") {
                toml_content.push_str(&format!("enable_cors = {}\n", cors));
            }
            if let Some(origins) = server_values.get("server.security.allowed_origins") {
                if origins.is_empty() {
                    toml_content.push_str("allowed_origins = []\n");
                } else {
                    let origins_list: Vec<&str> = origins.split(',').collect();
                    toml_content.push_str(&format!("allowed_origins = {:?}\n", origins_list));
                }
            }
            if let Some(rate) = server_values.get("server.security.enable_rate_limit") {
                toml_content.push_str(&format!("enable_rate_limit = {}\n", rate));
            }
            if let Some(rpm) = server_values.get("server.security.requests_per_minute") {
                toml_content.push_str(&format!("requests_per_minute = {}\n", rpm));
            }
            toml_content.push('\n');
        }

        // Performance
        if server_values.contains_key("server.performance.enable_compression") {
            toml_content.push_str("[server.performance]\n");
            if let Some(comp) = server_values.get("server.performance.enable_compression") {
                toml_content.push_str(&format!("enable_compression = {}\n", comp));
            }
            if let Some(http2) = server_values.get("server.performance.enable_http2") {
                toml_content.push_str(&format!("enable_http2 = {}\n", http2));
            }
            if let Some(keep) = server_values.get("server.performance.keep_alive") {
                toml_content.push_str(&format!("keep_alive = {}\n", keep));
            }
        }
    }

    // Write to file
    fs::write(file_path, toml_content).map_err(|e| ReedError::WriteError {
        path: file_path.to_string(),
        reason: format!("Failed to write Reed.toml: {}", e),
    })?;

    let total_values = project_values.len() + server_values.len();
    let message = format!(
        "✓ Successfully exported {} configuration values from CSV → Reed.toml\n\
         Output file: {}",
        total_values, file_path
    );

    Ok(ReedResponse::new(message, "config:export"))
}

// Helper functions for formatting output

fn format_project_section(config: &crate::reedcms::config::toml_parser::ReedConfig) -> String {
    let description = config.project.description.as_deref().unwrap_or("(not set)");
    format!(
        "[project]
  name: {}
  url: {}
  description: {}

  [project.languages]
    default: {}
    available: {}
    fallback_chain: {}

  [project.routing]
    url_prefix: {}
    trailing_slash: {}

  [project.templates]
    auto_reload: {}
    cache_templates: {}

  [project.assets]
    css_minify: {}
    css_bundle: {}

  [project.build]
    clean_before: {}
    parallel: {}",
        config.project.name,
        config.project.url,
        description,
        config.project.languages.default,
        config.project.languages.available.join(", "),
        config.project.languages.fallback_chain,
        config.project.routing.url_prefix,
        config.project.routing.trailing_slash,
        config.project.templates.auto_reload,
        config.project.templates.cache_templates,
        config.project.assets.css_minify,
        config.project.assets.css_bundle,
        config.project.build.clean_before,
        config.project.build.parallel,
    )
}

fn format_server_section(config: &crate::reedcms::config::toml_parser::ReedConfig) -> String {
    format!(
        "[server]
  workers: {} (0 = auto)

  Note: Server binding controlled by ENVIRONMENT in .env:
    - ENVIRONMENT=dev  → localhost:8333 (HTTP)
    - ENVIRONMENT=prod → /tmp/reed.sock (Unix socket)

  [server.security]
    enable_cors: {}
    allowed_origins: {}
    enable_rate_limit: {}
    requests_per_minute: {}

  [server.performance]
    enable_compression: {}
    enable_http2: {}
    keep_alive: {}",
        config.server.workers,
        config.server.security.enable_cors,
        config.server.security.allowed_origins.join(", "),
        config.server.security.enable_rate_limit,
        config.server.security.requests_per_minute,
        config.server.performance.enable_compression,
        config.server.performance.enable_http2,
        config.server.performance.keep_alive,
    )
}
