// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::csv::read_csv;
    use crate::reedcms::reedbase::set;
    use crate::reedcms::reedstream::ReedRequest;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_csv(content: &str) -> PathBuf {
        let dir = PathBuf::from(format!("/tmp/reed_set_test_{}", rand::random::<u32>()));
        fs::create_dir_all(&dir).unwrap();

        let file_path = dir.join("test.csv");
        fs::write(&file_path, content).unwrap();
        file_path
    }

    fn cleanup_test_dir(path: &PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn test_set_basic() {
        let csv_path = create_test_csv("key1|value1\n");
        let mut cache = HashMap::new();
        cache.insert("key1".to_string(), "value1".to_string());

        let request = ReedRequest {
            key: "key2".to_string(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: Some("value2".to_string()),
            description: Some("New key".to_string()),
        };

        let result = set(request, &mut cache, &csv_path.to_string_lossy().to_string());
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data, "value2");
        assert!(!response.cached);

        // Verify cache updated
        assert_eq!(cache.get("key2"), Some(&"value2".to_string()));

        cleanup_test_dir(&csv_path);
    }

    #[test]
    fn test_set_updates_existing() {
        let csv_path = create_test_csv("key1|value1\n");
        let mut cache = HashMap::new();
        cache.insert("key1".to_string(), "value1".to_string());

        let request = ReedRequest {
            key: "key1".to_string(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: Some("new_value".to_string()),
            description: None,
        };

        let result = set(request, &mut cache, &csv_path.to_string_lossy().to_string());
        assert!(result.is_ok());

        // Verify cache updated
        assert_eq!(cache.get("key1"), Some(&"new_value".to_string()));

        cleanup_test_dir(&csv_path);
    }

    #[test]
    fn test_set_creates_backup() {
        let csv_path = create_test_csv("key1|value1\n");
        let mut cache = HashMap::new();
        cache.insert("key1".to_string(), "value1".to_string());

        let request = ReedRequest {
            key: "key2".to_string(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: Some("value2".to_string()),
            description: None,
        };

        set(request, &mut cache, &csv_path.to_string_lossy().to_string()).unwrap();

        // Verify backup was created
        let backup_dir = csv_path.parent().unwrap().join("backups");
        assert!(backup_dir.exists());

        let entries: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert!(entries.len() > 0);
        assert!(entries[0].path().to_string_lossy().ends_with(".xz"));

        cleanup_test_dir(&csv_path);
    }

    #[test]
    fn test_set_persists_to_csv() {
        let csv_path = create_test_csv("key1|value1\n");
        let mut cache = HashMap::new();
        cache.insert("key1".to_string(), "value1".to_string());

        let request = ReedRequest {
            key: "key2".to_string(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: Some("value2".to_string()),
            description: None,
        };

        set(request, &mut cache, &csv_path.to_string_lossy().to_string()).unwrap();

        // Read CSV and verify
        let records = read_csv(&csv_path).unwrap();
        assert_eq!(records.len(), 2);

        let has_key2 = records
            .iter()
            .any(|r| r.key == "key2" && r.value == "value2");
        assert!(has_key2);

        cleanup_test_dir(&csv_path);
    }

    #[test]
    fn test_set_no_value_provided() {
        let csv_path = create_test_csv("key1|value1\n");
        let mut cache = HashMap::new();

        let request = ReedRequest {
            key: "key2".to_string(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: None, // Missing value
            description: None,
        };

        let result = set(request, &mut cache, &csv_path.to_string_lossy().to_string());
        assert!(result.is_err());

        cleanup_test_dir(&csv_path);
    }

    #[test]
    fn test_set_performance() {
        let csv_path = create_test_csv("");
        let mut cache = HashMap::new();

        // Pre-populate cache with 1000 items
        for i in 0..1000 {
            cache.insert(format!("key{}", i), format!("value{}", i));
        }

        let request = ReedRequest {
            key: "new_key".to_string(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: Some("new_value".to_string()),
            description: None,
        };

        let start = std::time::Instant::now();
        set(request, &mut cache, &csv_path.to_string_lossy().to_string()).unwrap();
        let duration = start.elapsed();

        // Should complete in < 10ms for 1000 records
        assert!(duration.as_millis() < 10, "Set too slow: {:?}", duration);

        cleanup_test_dir(&csv_path);
    }

    #[test]
    fn test_set_sorted_output() {
        let csv_path = create_test_csv("");
        let mut cache = HashMap::new();

        // Insert in random order
        cache.insert("zebra".to_string(), "z".to_string());
        cache.insert("alpha".to_string(), "a".to_string());
        cache.insert("beta".to_string(), "b".to_string());

        let request = ReedRequest {
            key: "gamma".to_string(),
            language: None,
            environment: None,
            context: Some("text".to_string()),
            value: Some("g".to_string()),
            description: None,
        };

        set(request, &mut cache, &csv_path.to_string_lossy().to_string()).unwrap();

        // Read CSV and verify sorted order
        let records = read_csv(&csv_path).unwrap();
        assert_eq!(records[0].key, "alpha");
        assert_eq!(records[1].key, "beta");
        assert_eq!(records[2].key, "gamma");
        assert_eq!(records[3].key, "zebra");

        cleanup_test_dir(&csv_path);
    }
}
