// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::reedbase::init::init;
    use crate::reedcms::reedstream::ReedRequest;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_csv(content: &str) -> PathBuf {
        let path = PathBuf::from(format!("/tmp/reed_init_test_{}.csv", rand::random::<u32>()));
        fs::write(&path, content).unwrap();
        path
    }

    fn cleanup_test_file(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_init_basic() {
        let csv_path = create_test_csv("key1|value1\nkey2|value2\n");

        let request = ReedRequest {
            key: String::new(),
            language: None,
            environment: None,
            context: Some("test".to_string()),
            value: Some(csv_path.to_string_lossy().to_string()),
            description: None,
        };

        let result = init(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data.get("key1"), Some(&"value1".to_string()));
        assert_eq!(response.data.get("key2"), Some(&"value2".to_string()));
        assert!(!response.cached);

        cleanup_test_file(&csv_path);
    }

    #[test]
    fn test_init_with_descriptions() {
        let csv_path = create_test_csv("key1|value1|desc1\nkey2|value2|desc2\n");

        let request = ReedRequest {
            key: String::new(),
            language: None,
            environment: None,
            context: Some("test".to_string()),
            value: Some(csv_path.to_string_lossy().to_string()),
            description: None,
        };

        let result = init(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 2);

        cleanup_test_file(&csv_path);
    }

    #[test]
    fn test_init_empty_file() {
        let csv_path = create_test_csv("");

        let request = ReedRequest {
            key: String::new(),
            language: None,
            environment: None,
            context: Some("test".to_string()),
            value: Some(csv_path.to_string_lossy().to_string()),
            description: None,
        };

        let result = init(request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 0);

        cleanup_test_file(&csv_path);
    }

    #[test]
    fn test_init_large_file() {
        let mut content = String::new();
        for i in 0..1000 {
            content.push_str(&format!("key{}|value{}\n", i, i));
        }

        let csv_path = create_test_csv(&content);

        let request = ReedRequest {
            key: String::new(),
            language: None,
            environment: None,
            context: Some("test".to_string()),
            value: Some(csv_path.to_string_lossy().to_string()),
            description: None,
        };

        let start = std::time::Instant::now();
        let result = init(request);
        let duration = start.elapsed();

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), 1000);

        // Should complete in < 10ms
        assert!(duration.as_millis() < 10, "Init too slow: {:?}", duration);

        cleanup_test_file(&csv_path);
    }

    #[test]
    fn test_init_no_path_provided() {
        let request = ReedRequest {
            key: String::new(),
            language: None,
            environment: None,
            context: Some("test".to_string()),
            value: None, // Missing path
            description: None,
        };

        let result = init(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_file_not_found() {
        let request = ReedRequest {
            key: String::new(),
            language: None,
            environment: None,
            context: Some("test".to_string()),
            value: Some("/tmp/nonexistent_reed_file_12345.csv".to_string()),
            description: None,
        };

        let result = init(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_response_fields() {
        let csv_path = create_test_csv("key1|value1\n");

        let request = ReedRequest {
            key: String::new(),
            language: None,
            environment: None,
            context: Some("test".to_string()),
            value: Some(csv_path.to_string_lossy().to_string()),
            description: None,
        };

        let response = init(request).unwrap();

        assert_eq!(response.source, csv_path.to_string_lossy().to_string());
        assert!(!response.cached);
        assert!(response.timestamp > 0);
        assert!(response.metrics.is_none());

        cleanup_test_file(&csv_path);
    }
}
