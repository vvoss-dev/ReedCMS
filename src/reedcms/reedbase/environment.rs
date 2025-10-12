// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Environment Fallback System
//!
//! Provides intelligent environment-aware key resolution with fallback logic.
//! Enables environment-specific overrides (dev, prod, christmas, easter) whilst
//! maintaining a base fallback chain.
//!
//! ## Fallback Logic
//! ```
//! Lookup order:
//! 1. key@environment (e.g., "title@dev", "title@christmas")
//! 2. key (base key without environment)
//!
//! Examples:
//! - "knowledge.title@dev" → try "knowledge.title@dev" → fallback to "knowledge.title"
//! - "knowledge.title@christmas" → try "@christmas" → fallback to base
//! ```
//!
//! ## Valid Environments
//! - **Deployment**: dev, prod, staging
//! - **Seasonal**: christmas, easter
//! - **Custom**: Any alphanumeric + underscore combination
//!
//! ## Performance
//! - Environment suffix detection: < 1μs
//! - Base key extraction: < 1μs
//! - Fallback lookup: < 10μs total

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::collections::HashMap;

/// Resolves environment-specific key with fallback in nested HashMap.
///
/// ## Input
/// - `cache`: Nested HashMap (language -> key -> value)
/// - `key`: Base key or key with @environment suffix
/// - `language`: Language code (de, en, etc.)
/// - `environment`: Optional environment override
///
/// ## Output
/// - Resolved value from cache
/// - Falls back to base key if environment-specific not found
///
/// ## Performance
/// - O(1) lookup for environment key
/// - O(1) fallback lookup for base key
/// - Total: < 10μs
///
/// ## Example
/// ```rust
/// use std::collections::HashMap;
/// use reedcms::reedbase::environment::resolve_with_fallback;
///
/// let mut cache = HashMap::new();
/// let mut de_map = HashMap::new();
/// de_map.insert("title".to_string(), "Titel".to_string());
/// de_map.insert("title@dev".to_string(), "DEV-Titel".to_string());
/// cache.insert("de".to_string(), de_map);
///
/// // With environment
/// let value = resolve_with_fallback(&cache, "title", "de", Some("dev")).unwrap();
/// assert_eq!(value, "DEV-Titel");
///
/// // Fallback to base
/// let value = resolve_with_fallback(&cache, "title", "de", Some("prod")).unwrap();
/// assert_eq!(value, "Titel");
/// ```
pub fn resolve_with_fallback(
    cache: &HashMap<String, HashMap<String, String>>,
    key: &str,
    language: &str,
    environment: Option<&str>,
) -> ReedResult<String> {
    // Get language-specific map
    let lang_map = cache.get(language).ok_or_else(|| ReedError::NotFound {
        resource: "language".to_string(),
        context: Some(language.to_string()),
    })?;

    // Try with environment suffix first
    if let Some(env) = environment {
        let env_key = build_env_key(key, env);
        if let Some(value) = lang_map.get(&env_key) {
            return Ok(value.clone());
        }
    }

    // Fallback: try without environment
    lang_map
        .get(key)
        .cloned()
        .ok_or_else(|| ReedError::NotFound {
            resource: key.to_string(),
            context: Some(format!("language={}", language)),
        })
}

/// Resolves environment-specific key with fallback in flat HashMap.
///
/// ## Input
/// - `cache`: Flat HashMap (key -> value)
/// - `key`: Base key or key with @environment suffix
/// - `environment`: Optional environment override
///
/// ## Output
/// - Resolved value from cache
/// - Falls back to base key if environment-specific not found
///
/// ## Performance
/// - O(1) lookup for environment key
/// - O(1) fallback lookup for base key
/// - Total: < 5μs
///
/// ## Example
/// ```rust
/// use std::collections::HashMap;
/// use reedcms::reedbase::environment::resolve_flat_with_fallback;
///
/// let mut cache = HashMap::new();
/// cache.insert("port".to_string(), "8333".to_string());
/// cache.insert("port@dev".to_string(), "3000".to_string());
///
/// // With environment
/// let value = resolve_flat_with_fallback(&cache, "port", Some("dev")).unwrap();
/// assert_eq!(value, "3000");
///
/// // Fallback to base
/// let value = resolve_flat_with_fallback(&cache, "port", Some("prod")).unwrap();
/// assert_eq!(value, "8333");
/// ```
pub fn resolve_flat_with_fallback(
    cache: &HashMap<String, String>,
    key: &str,
    environment: Option<&str>,
) -> ReedResult<String> {
    // Try with environment suffix first
    if let Some(env) = environment {
        let env_key = build_env_key(key, env);
        if let Some(value) = cache.get(&env_key) {
            return Ok(value.clone());
        }
    }

    // Fallback: try without environment
    cache.get(key).cloned().ok_or_else(|| ReedError::NotFound {
        resource: key.to_string(),
        context: None,
    })
}

/// Tests if key has environment suffix.
///
/// ## Examples
/// - "title@dev" → true
/// - "title@christmas" → true
/// - "title" → false
/// - "page.title@en@dev" → true (last @ counts)
///
/// ## Performance
/// - < 1μs (simple string search)
#[allow(dead_code)]
pub fn has_environment_suffix(key: &str) -> bool {
    key.contains('@')
}

/// Extracts base key from environment-specific key.
///
/// ## Examples
/// - "knowledge.title@dev" → "knowledge.title"
/// - "knowledge.title@christmas" → "knowledge.title"
/// - "knowledge.title" → "knowledge.title"
/// - "page.title@en@dev" → "page.title@en" (removes last @ segment)
///
/// ## Performance
/// - < 1μs (single string operation)
#[allow(dead_code)]
pub fn extract_base_key(key: &str) -> String {
    if let Some((base, _env)) = key.rsplit_once('@') {
        base.to_string()
    } else {
        key.to_string()
    }
}

/// Validates environment name.
///
/// ## Valid Environments
/// - **Deployment**: dev, prod, staging
/// - **Seasonal**: christmas, easter
/// - **Custom**: alphanumeric + underscore (e.g., test_2024, qa-staging)
///
/// ## Invalid Examples
/// - Empty string
/// - Contains spaces
/// - Special chars (except underscore and hyphen)
///
/// ## Performance
/// - < 1μs (simple validation)
#[allow(dead_code)]
pub fn validate_environment(env: &str) -> ReedResult<()> {
    if env.is_empty() {
        return Err(ReedError::ValidationError {
            field: "environment".to_string(),
            value: env.to_string(),
            constraint: "Environment name cannot be empty".to_string(),
        });
    }

    // Check for valid characters: alphanumeric, underscore, hyphen
    if !env
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ReedError::ValidationError {
            field: "environment".to_string(),
            value: env.to_string(),
            constraint: "Only alphanumeric, underscore, and hyphen allowed".to_string(),
        });
    }

    Ok(())
}

/// Builds environment-specific key.
///
/// ## Example
/// - base: "title", env: "dev" → "title@dev"
/// - base: "port", env: "prod" → "port@prod"
///
/// ## Performance
/// - < 1μs (simple string formatting)
pub fn build_env_key(base: &str, environment: &str) -> String {
    format!("{}@{}", base, environment)
}

/// Module identification for environment subsystem.
#[allow(dead_code)]
pub fn subsystem_name() -> &'static str {
    "reedbase::environment"
}

/// Health check for environment fallback system.
///
/// ## Tests
/// - Environment suffix detection
/// - Base key extraction
/// - Key building
/// - Environment validation
///
/// ## Returns
/// - Ok: System healthy
/// - Err: Specific component failure
#[allow(dead_code)]
pub fn health_check() -> ReedResult<String> {
    // Test environment suffix detection
    if !has_environment_suffix("test@dev") {
        return Err(ReedError::ConfigError {
            component: "environment::has_environment_suffix".to_string(),
            reason: "Environment suffix detection failed".to_string(),
        });
    }

    if has_environment_suffix("test") {
        return Err(ReedError::ConfigError {
            component: "environment::has_environment_suffix".to_string(),
            reason: "False positive in suffix detection".to_string(),
        });
    }

    // Test base key extraction
    if extract_base_key("test@dev") != "test" {
        return Err(ReedError::ConfigError {
            component: "environment::extract_base_key".to_string(),
            reason: "Base key extraction failed".to_string(),
        });
    }

    // Test key building
    if build_env_key("test", "dev") != "test@dev" {
        return Err(ReedError::ConfigError {
            component: "environment::build_env_key".to_string(),
            reason: "Environment key building failed".to_string(),
        });
    }

    // Test environment validation
    if validate_environment("dev").is_err() {
        return Err(ReedError::ConfigError {
            component: "environment::validate_environment".to_string(),
            reason: "Valid environment rejected".to_string(),
        });
    }

    if validate_environment("invalid name").is_ok() {
        return Err(ReedError::ConfigError {
            component: "environment::validate_environment".to_string(),
            reason: "Invalid environment accepted".to_string(),
        });
    }

    Ok("Environment fallback system healthy".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_environment_suffix() {
        assert!(has_environment_suffix("title@dev"));
        assert!(has_environment_suffix("title@prod"));
        assert!(has_environment_suffix("title@christmas"));
        assert!(has_environment_suffix("page.title@en@dev"));
        assert!(!has_environment_suffix("title"));
        assert!(!has_environment_suffix("page.title"));
    }

    #[test]
    fn test_extract_base_key() {
        assert_eq!(extract_base_key("title@dev"), "title");
        assert_eq!(extract_base_key("knowledge.title@dev"), "knowledge.title");
        assert_eq!(
            extract_base_key("knowledge.title@christmas"),
            "knowledge.title"
        );
        assert_eq!(extract_base_key("title"), "title");
        assert_eq!(extract_base_key("page.title@en@dev"), "page.title@en");
    }

    #[test]
    fn test_validate_environment() {
        // Valid environments
        assert!(validate_environment("dev").is_ok());
        assert!(validate_environment("prod").is_ok());
        assert!(validate_environment("staging").is_ok());
        assert!(validate_environment("christmas").is_ok());
        assert!(validate_environment("easter").is_ok());
        assert!(validate_environment("test_2024").is_ok());
        assert!(validate_environment("qa-staging").is_ok());

        // Invalid environments
        assert!(validate_environment("").is_err());
        assert!(validate_environment("invalid name").is_err());
        assert!(validate_environment("env@dev").is_err());
        assert!(validate_environment("env|prod").is_err());
    }

    #[test]
    fn test_build_env_key() {
        assert_eq!(build_env_key("title", "dev"), "title@dev");
        assert_eq!(
            build_env_key("knowledge.title", "prod"),
            "knowledge.title@prod"
        );
        assert_eq!(build_env_key("port", "christmas"), "port@christmas");
    }

    #[test]
    fn test_resolve_with_fallback() {
        let mut cache = HashMap::new();
        let mut de_map = HashMap::new();
        de_map.insert("title".to_string(), "Titel".to_string());
        de_map.insert("title@dev".to_string(), "DEV-Titel".to_string());
        de_map.insert("title@christmas".to_string(), "Weihnachtstitel".to_string());
        cache.insert("de".to_string(), de_map);

        // Environment key exists
        let value = resolve_with_fallback(&cache, "title", "de", Some("dev")).unwrap();
        assert_eq!(value, "DEV-Titel");

        // Environment key doesn't exist, fallback to base
        let value = resolve_with_fallback(&cache, "title", "de", Some("prod")).unwrap();
        assert_eq!(value, "Titel");

        // Seasonal theme
        let value = resolve_with_fallback(&cache, "title", "de", Some("christmas")).unwrap();
        assert_eq!(value, "Weihnachtstitel");

        // No environment specified
        let value = resolve_with_fallback(&cache, "title", "de", None).unwrap();
        assert_eq!(value, "Titel");

        // Key doesn't exist
        assert!(resolve_with_fallback(&cache, "nonexistent", "de", None).is_err());

        // Language doesn't exist
        assert!(resolve_with_fallback(&cache, "title", "fr", None).is_err());
    }

    #[test]
    fn test_resolve_flat_with_fallback() {
        let mut cache = HashMap::new();
        cache.insert("port".to_string(), "8333".to_string());
        cache.insert("port@dev".to_string(), "3000".to_string());
        cache.insert("host@prod".to_string(), "0.0.0.0".to_string());

        // Environment key exists
        let value = resolve_flat_with_fallback(&cache, "port", Some("dev")).unwrap();
        assert_eq!(value, "3000");

        // Environment key doesn't exist, fallback to base
        let value = resolve_flat_with_fallback(&cache, "port", Some("prod")).unwrap();
        assert_eq!(value, "8333");

        // No environment specified
        let value = resolve_flat_with_fallback(&cache, "port", None).unwrap();
        assert_eq!(value, "8333");

        // Key doesn't exist
        assert!(resolve_flat_with_fallback(&cache, "nonexistent", None).is_err());
    }

    #[test]
    fn test_health_check() {
        let result = health_check();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Environment fallback system healthy");
    }
}
