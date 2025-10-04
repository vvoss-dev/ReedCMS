// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0
//
// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One test = One assertion
// MANDATORY: BBC English for all test names and documentation
// MANDATORY: Test all error paths explicitly
// MANDATORY: Performance assertions for all operations
//
// == FILE PURPOSE ==
// This file: Tests for api_keys.rs API key management operations
// Architecture: Separate test file following KISS principle
// Performance: All tests must complete within defined time limits
// Test Scope: Unit tests for API key generation, verification, revocation

#[cfg(test)]
mod tests {
    use crate::reedcms::api::security::api_keys::{
        generate_random_key, hash_api_key, ApiKeyManager,
    };

    #[test]
    fn test_generate_random_key_format() {
        let key = generate_random_key();

        // Should start with "reed_" prefix
        assert!(key.starts_with("reed_"));

        // Total length should be 37 characters (reed_ = 5, 32 hex chars = 32)
        assert_eq!(key.len(), 37);

        // Should only contain valid characters
        for c in key.chars().skip(5) {
            assert!(c.is_ascii_hexdigit());
        }
    }

    #[test]
    fn test_generate_random_key_uniqueness() {
        let key1 = generate_random_key();
        let key2 = generate_random_key();

        // Two generated keys should be different
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_hash_api_key_deterministic() {
        let key = "reed_test123456789012345678901234";
        let hash1 = hash_api_key(key);
        let hash2 = hash_api_key(key);

        // Same key should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_api_key_different_inputs() {
        let key1 = "reed_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let key2 = "reed_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

        let hash1 = hash_api_key(key1);
        let hash2 = hash_api_key(key2);

        // Different keys should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_api_key_length() {
        let key = "reed_test123";
        let hash = hash_api_key(key);

        // SHA-256 produces 64-character hex string
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_api_key_hex_output() {
        let key = "reed_test123";
        let hash = hash_api_key(key);

        // Hash should only contain hex characters
        for c in hash.chars() {
            assert!(c.is_ascii_hexdigit());
        }
    }

    #[test]
    fn test_api_key_prefix() {
        let key = generate_random_key();
        assert!(
            key.starts_with("reed_"),
            "API key must start with reed_ prefix"
        );
    }

    #[test]
    fn test_api_key_collision_resistance() {
        // Generate 100 keys and ensure all are unique
        let mut keys = std::collections::HashSet::new();

        for _ in 0..100 {
            let key = generate_random_key();
            assert!(keys.insert(key), "Generated duplicate key");
        }

        assert_eq!(keys.len(), 100);
    }

    #[test]
    fn test_hash_empty_string() {
        let hash = hash_api_key("");
        assert_eq!(hash.len(), 64); // Still produces valid SHA-256 hash
    }

    #[test]
    fn test_hash_special_characters() {
        let key = "reed_!@#$%^&*()_+-={}[]|:;<>?,./";
        let hash = hash_api_key(key);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_unicode_characters() {
        let key = "reed_日本語_мир_🌍";
        let hash = hash_api_key(key);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_very_long_input() {
        let key = "reed_".to_string() + &"a".repeat(10000);
        let hash = hash_api_key(&key);
        assert_eq!(hash.len(), 64); // SHA-256 always produces same length
    }
}
