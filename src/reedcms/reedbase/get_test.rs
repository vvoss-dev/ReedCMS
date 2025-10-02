// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::reedbase::get;
    use crate::reedcms::reedstream::ReedRequest;
    use std::collections::HashMap;

    #[test]
    fn test_get_basic() {
        let mut cache = HashMap::new();
        cache.insert("page.title".to_string(), "Welcome".to_string());

        let request = ReedRequest {
            key: "page.title".to_string(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: None,
            description: None,
        };

        let result = get(request, &cache);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data, "Welcome");
        assert!(response.cached);
    }

    #[test]
    fn test_get_with_language() {
        let mut cache = HashMap::new();
        cache.insert("page.title@en".to_string(), "Welcome".to_string());
        cache.insert("page.title@de".to_string(), "Willkommen".to_string());

        let request = ReedRequest {
            key: "page.title".to_string(),
            language: Some("de".to_string()),
            environment: None,
            context: Some("text".to_string()),
            value: None,
            description: None,
        };

        let result = get(request, &cache);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data, "Willkommen");
    }

    #[test]
    fn test_get_with_environment_fallback() {
        let mut cache = HashMap::new();
        cache.insert("page.title@en".to_string(), "Welcome".to_string());
        // No @dev version exists

        let request = ReedRequest {
            key: "page.title".to_string(),
            language: Some("en".to_string()),
            environment: Some("dev".to_string()),
            context: Some("text".to_string()),
            value: None,
            description: None,
        };

        let result = get(request, &cache);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data, "Welcome");
        assert_eq!(response.source, "page.title@en");
    }

    #[test]
    fn test_get_environment_specific() {
        let mut cache = HashMap::new();
        cache.insert("page.title@en".to_string(), "Welcome".to_string());
        cache.insert("page.title@en@dev".to_string(), "Welcome (DEV)".to_string());

        let request = ReedRequest {
            key: "page.title".to_string(),
            language: Some("en".to_string()),
            environment: Some("dev".to_string()),
            context: Some("text".to_string()),
            value: None,
            description: None,
        };

        let result = get(request, &cache);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data, "Welcome (DEV)");
        assert_eq!(response.source, "page.title@en@dev");
    }

    #[test]
    fn test_get_not_found() {
        let cache = HashMap::new();

        let request = ReedRequest {
            key: "nonexistent.key".to_string(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: None,
            description: None,
        };

        let result = get(request, &cache);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_performance() {
        let mut cache = HashMap::new();
        for i in 0..10000 {
            cache.insert(format!("key{}", i), format!("value{}", i));
        }

        let request = ReedRequest {
            key: "key5000".to_string(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: None,
            description: None,
        };

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = get(request.clone(), &cache);
        }
        let duration = start.elapsed();

        // 1000 lookups should complete in < 1ms (O(1) performance)
        assert!(duration.as_micros() < 1000, "Get too slow: {:?}", duration);
    }

    #[test]
    fn test_get_response_fields() {
        let mut cache = HashMap::new();
        cache.insert("key1".to_string(), "value1".to_string());

        let request = ReedRequest {
            key: "key1".to_string(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: None,
            description: None,
        };

        let response = get(request, &cache).unwrap();

        assert_eq!(response.data, "value1");
        assert_eq!(response.source, "key1");
        assert!(response.cached);
        assert!(response.timestamp > 0);
        assert!(response.metrics.is_none());
    }
}
