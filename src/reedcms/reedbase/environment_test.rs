// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for Environment Fallback System
//!
//! Tests all environment-aware key resolution functions:
//! - Environment suffix detection
//! - Base key extraction
//! - Environment key building
//! - Environment validation
//! - Fallback resolution (nested and flat)
//! - Health check functionality

#[cfg(test)]
mod tests {
    use crate::reedcms::reedbase::environment::*;
    use std::collections::HashMap;

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

    #[test]
    fn test_fallback_chain_priority() {
        let mut cache = HashMap::new();
        let mut en_map = HashMap::new();

        // Setup: base value + environment override
        en_map.insert("message".to_string(), "Hello".to_string());
        en_map.insert("message@dev".to_string(), "Hello Dev".to_string());
        en_map.insert(
            "message@christmas".to_string(),
            "Merry Christmas".to_string(),
        );
        cache.insert("en".to_string(), en_map);

        // Priority 1: Environment-specific value takes precedence
        let value = resolve_with_fallback(&cache, "message", "en", Some("dev")).unwrap();
        assert_eq!(value, "Hello Dev");

        // Priority 2: Fallback to base when environment not found
        let value = resolve_with_fallback(&cache, "message", "en", Some("staging")).unwrap();
        assert_eq!(value, "Hello"); // Falls back to base

        // Priority 3: Seasonal overrides work
        let value = resolve_with_fallback(&cache, "message", "en", Some("christmas")).unwrap();
        assert_eq!(value, "Merry Christmas");
    }

    #[test]
    fn test_environment_validation_edge_cases() {
        // Edge case: Numbers only
        assert!(validate_environment("2024").is_ok());

        // Edge case: Underscore prefix/suffix
        assert!(validate_environment("_test").is_ok());
        assert!(validate_environment("test_").is_ok());

        // Edge case: Hyphen prefix/suffix
        assert!(validate_environment("-staging").is_ok());
        assert!(validate_environment("staging-").is_ok());

        // Edge case: Mixed case (should be valid)
        assert!(validate_environment("DevEnv").is_ok());

        // Edge case: Special characters that should fail
        assert!(validate_environment("env.test").is_err());
        assert!(validate_environment("env:test").is_err());
        assert!(validate_environment("env/test").is_err());
    }
}
