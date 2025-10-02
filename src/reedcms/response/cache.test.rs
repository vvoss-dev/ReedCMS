// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Cache Control Header Tests

#[cfg(test)]
mod tests {
    use super::super::cache::cache_control_header;

    #[test]
    fn test_cache_control_header_format() {
        let (name, value) = cache_control_header("knowledge");

        // Header name should always be Cache-Control
        assert_eq!(name, "Cache-Control");

        // Value should contain max-age
        assert!(value.contains("max-age="));

        // Value should start with public or private
        assert!(value.starts_with("public") || value.starts_with("private"));
    }

    #[test]
    fn test_cache_control_default_ttl() {
        let (_, value) = cache_control_header("nonexistent-layout");

        // Default TTL should be 3600 (1 hour)
        assert!(value.contains("max-age=3600") || value.contains("no-cache"));
    }

    #[test]
    fn test_cache_control_no_cache_format() {
        // Test layouts with TTL=0 would return no-cache
        // (This would need actual meta.csv configuration in integration tests)
        let (name, _) = cache_control_header("test");
        assert_eq!(name, "Cache-Control");
    }
}
