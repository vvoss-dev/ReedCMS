// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Reed.toml parser and validator.
//!
//! Parses Reed.toml configuration file and validates structure.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Complete Reed.toml configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub server: ServerConfig,
}

/// Project configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub description: Option<String>,
    pub languages: LanguageConfig,
    #[serde(default)]
    pub routing: RoutingConfig,
    #[serde(default)]
    pub templates: TemplateConfig,
    #[serde(default)]
    pub assets: AssetConfig,
    #[serde(default)]
    pub build: BuildConfig,
}

/// Language configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub default: String,
    pub available: Vec<String>,
    #[serde(default = "default_true")]
    pub fallback_chain: bool,
}

/// Routing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoutingConfig {
    #[serde(default = "default_true")]
    pub url_prefix: bool,
    #[serde(default = "default_true")]
    pub trailing_slash: bool,
}

/// Template configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateConfig {
    #[serde(default = "default_true")]
    pub auto_reload: bool,
    #[serde(default = "default_true")]
    pub cache_templates: bool,
}

/// Asset configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetConfig {
    #[serde(default = "default_true")]
    pub css_minify: bool,
    #[serde(default = "default_true")]
    pub css_bundle: bool,
}

/// Build configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    #[serde(default = "default_true")]
    pub clean_before: bool,
    #[serde(default = "default_true")]
    pub parallel: bool,
}

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    #[serde(default)]
    pub workers: usize,
    #[serde(default)]
    pub dev: ServerEnvironmentConfig,
    #[serde(default)]
    pub prod: ServerEnvironmentConfig,
}

/// Server environment-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerEnvironmentConfig {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub io: String,
    #[serde(default)]
    pub enable_cors: bool,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    #[serde(default)]
    pub enable_rate_limit: bool,
    #[serde(default = "default_rate_limit")]
    pub requests_per_minute: u32,
    #[serde(default = "default_true")]
    pub enable_compression: bool,
    #[serde(default = "default_true")]
    pub enable_http2: bool,
    #[serde(default = "default_keep_alive")]
    pub keep_alive: u64,
}

// Default value functions
fn default_true() -> bool {
    true
}
fn default_rate_limit() -> u32 {
    60
}
fn default_keep_alive() -> u64 {
    75
}

/// Parses Reed.toml file.
///
/// ## Input
/// - path: Path to Reed.toml file
///
/// ## Output
/// - ReedConfig structure
///
/// ## Error Conditions
/// - File not found
/// - TOML parse error
/// - Validation error
///
/// ## Performance
/// - < 5ms for typical config
pub fn parse_reed_toml<P: AsRef<Path>>(path: P) -> ReedResult<ReedConfig> {
    let content = std::fs::read_to_string(&path).map_err(|e| ReedError::IoError {
        operation: "read".to_string(),
        path: path.as_ref().display().to_string(),
        reason: e.to_string(),
    })?;

    let config: ReedConfig = toml::from_str(&content).map_err(|e| ReedError::ParseError {
        input: "Reed.toml".to_string(),
        reason: e.to_string(),
    })?;

    validate_config(&config)?;

    Ok(config)
}

/// Validates Reed.toml configuration.
///
/// ## Input
/// - config: ReedConfig structure
///
/// ## Output
/// - Ok(()) if valid
///
/// ## Error Conditions
/// - Project name: 1-100 characters
/// - Project URL: Valid URL format
/// - Default language: Must be in available languages
/// - Available languages: At least one language
/// - Default port: 1-65535
///
/// ## Performance
/// - < 1ms
pub fn validate_config(config: &ReedConfig) -> ReedResult<()> {
    // Validate project name
    if config.project.name.is_empty() || config.project.name.len() > 100 {
        return Err(ReedError::ValidationError {
            field: "project.name".to_string(),
            value: config.project.name.clone(),
            constraint: "1-100 characters".to_string(),
        });
    }

    // Validate project URL
    if !config.project.url.starts_with("http://") && !config.project.url.starts_with("https://") {
        return Err(ReedError::ValidationError {
            field: "project.url".to_string(),
            value: config.project.url.clone(),
            constraint: "Must start with http:// or https://".to_string(),
        });
    }

    // Validate languages
    if config.project.languages.available.is_empty() {
        return Err(ReedError::ValidationError {
            field: "project.languages.available".to_string(),
            value: "[]".to_string(),
            constraint: "At least one language required".to_string(),
        });
    }

    // Validate default language is in available languages
    if !config
        .project
        .languages
        .available
        .contains(&config.project.languages.default)
    {
        return Err(ReedError::ValidationError {
            field: "project.languages.default".to_string(),
            value: config.project.languages.default.clone(),
            constraint: format!(
                "Must be one of: {}",
                config.project.languages.available.join(", ")
            ),
        });
    }

    Ok(())
}
