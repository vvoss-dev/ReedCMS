// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::data_commands::*;
    use crate::reedcms::csv::{write_csv, CsvRecord};
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    /// Creates a test CSV file with sample data.
    fn create_test_csv(path: &str, records: Vec<CsvRecord>) {
        // Ensure directory exists
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).expect("Failed to create test directory");
        }
        write_csv(path, &records).unwrap();
    }

    /// Cleans up test CSV files.
    fn cleanup_test_csv(path: &str) {
        fs::remove_file(path).ok();
    }

    /// Cleans up .reed directory after tests.
    fn cleanup_reed_dir() {
        fs::remove_dir_all(".reed").ok();
    }

    #[test]
    fn test_set_text_success() {
        let csv_path = ".reed/text.csv";
        create_test_csv(csv_path, vec![]);

        let args = vec!["test.key@en".to_string(), "Test Value".to_string()];
        let mut flags = HashMap::new();
        flags.insert("desc".to_string(), "Test description for text".to_string());

        let result = set_text(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("✓ Text set"));
        assert!(response.data.contains("test.key@en"));
        assert!(response.data.contains("Test Value"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_set_text_missing_args() {
        let args = vec!["only_one_arg".to_string()];
        let flags = HashMap::new();

        let result = set_text(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_text_missing_desc_flag() {
        let args = vec!["test.key@en".to_string(), "Test Value".to_string()];
        let flags = HashMap::new();

        let result = set_text(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_text_desc_too_short() {
        let args = vec!["test.key@en".to_string(), "Test Value".to_string()];
        let mut flags = HashMap::new();
        flags.insert("desc".to_string(), "short".to_string());

        let result = set_text(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_route_success() {
        let csv_path = ".reed/routes.csv";
        create_test_csv(csv_path, vec![]);

        let args = vec!["knowledge@en".to_string(), "knowledge".to_string()];
        let mut flags = HashMap::new();
        flags.insert(
            "desc".to_string(),
            "English route for knowledge page".to_string(),
        );

        let result = set_route(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("✓ Route set"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_set_meta_success() {
        let csv_path = ".reed/meta.csv";
        create_test_csv(csv_path, vec![]);

        let args = vec!["cache.ttl".to_string(), "3600".to_string()];
        let mut flags = HashMap::new();
        flags.insert(
            "desc".to_string(),
            "Cache time-to-live in seconds".to_string(),
        );

        let result = set_meta(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("✓ Meta set"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_get_text_success() {
        let csv_path = ".reed/text.csv";
        create_test_csv(
            csv_path,
            vec![CsvRecord {
                key: "test.key@en".to_string(),
                value: "Test Value".to_string(),
                description: Some("Test description".to_string()),
            }],
        );

        let args = vec!["test.key@en".to_string()];
        let result = get_text(&args);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("Test Value"));
        assert!(response.data.contains("source:"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_get_text_missing_key() {
        let csv_path = ".reed/text.csv";
        create_test_csv(csv_path, vec![]);

        let args = vec!["nonexistent.key@en".to_string()];
        let result = get_text(&args);
        assert!(result.is_err());

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_get_text_no_args() {
        let args = vec![];
        let result = get_text(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_route_success() {
        let csv_path = ".reed/routes.csv";
        create_test_csv(
            csv_path,
            vec![CsvRecord {
                key: "knowledge@en".to_string(),
                value: "knowledge".to_string(),
                description: Some("English route".to_string()),
            }],
        );

        let args = vec!["knowledge@en".to_string()];
        let result = get_route(&args);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("knowledge"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_get_meta_success() {
        let csv_path = ".reed/meta.csv";
        create_test_csv(
            csv_path,
            vec![CsvRecord {
                key: "cache.ttl".to_string(),
                value: "3600".to_string(),
                description: Some("TTL".to_string()),
            }],
        );

        let args = vec!["cache.ttl".to_string()];
        let result = get_meta(&args);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("3600"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_list_text_all() {
        let csv_path = ".reed/text.csv";
        create_test_csv(
            csv_path,
            vec![
                CsvRecord {
                    key: "test.key1@en".to_string(),
                    value: "Value 1".to_string(),
                    description: None,
                },
                CsvRecord {
                    key: "test.key2@en".to_string(),
                    value: "Value 2".to_string(),
                    description: None,
                },
                CsvRecord {
                    key: "other.key@de".to_string(),
                    value: "Value 3".to_string(),
                    description: None,
                },
            ],
        );

        let args = vec!["*".to_string()];
        let result = list_text(&args);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("test.key1@en"));
        assert!(response.data.contains("test.key2@en"));
        assert!(response.data.contains("other.key@de"));
        assert!(response.data.contains("(3 entries found)"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_list_text_with_prefix_pattern() {
        let csv_path = ".reed/text.csv";
        create_test_csv(
            csv_path,
            vec![
                CsvRecord {
                    key: "test.key1@en".to_string(),
                    value: "Value 1".to_string(),
                    description: None,
                },
                CsvRecord {
                    key: "test.key2@en".to_string(),
                    value: "Value 2".to_string(),
                    description: None,
                },
                CsvRecord {
                    key: "other.key@de".to_string(),
                    value: "Value 3".to_string(),
                    description: None,
                },
            ],
        );

        let args = vec!["test.*".to_string()];
        let result = list_text(&args);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("test.key1@en"));
        assert!(response.data.contains("test.key2@en"));
        assert!(!response.data.contains("other.key@de"));
        assert!(response.data.contains("(2 entries found)"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_list_text_with_suffix_pattern() {
        let csv_path = ".reed/text.csv";
        create_test_csv(
            csv_path,
            vec![
                CsvRecord {
                    key: "test.key@en".to_string(),
                    value: "Value 1".to_string(),
                    description: None,
                },
                CsvRecord {
                    key: "other.key@en".to_string(),
                    value: "Value 2".to_string(),
                    description: None,
                },
                CsvRecord {
                    key: "test.key@de".to_string(),
                    value: "Value 3".to_string(),
                    description: None,
                },
            ],
        );

        let args = vec!["*@en".to_string()];
        let result = list_text(&args);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("test.key@en"));
        assert!(response.data.contains("other.key@en"));
        assert!(!response.data.contains("@de"));
        assert!(response.data.contains("(2 entries found)"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_list_text_no_matches() {
        let csv_path = ".reed/text.csv";
        create_test_csv(
            csv_path,
            vec![CsvRecord {
                key: "test.key@en".to_string(),
                value: "Value 1".to_string(),
                description: None,
            }],
        );

        let args = vec!["nonexistent.*".to_string()];
        let result = list_text(&args);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("(0 entries found)"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_list_route_success() {
        let csv_path = ".reed/routes.csv";
        create_test_csv(
            csv_path,
            vec![
                CsvRecord {
                    key: "knowledge@en".to_string(),
                    value: "knowledge".to_string(),
                    description: None,
                },
                CsvRecord {
                    key: "knowledge@de".to_string(),
                    value: "wissen".to_string(),
                    description: None,
                },
            ],
        );

        let args = vec!["knowledge*".to_string()];
        let result = list_route(&args);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("knowledge@en"));
        assert!(response.data.contains("knowledge@de"));
        assert!(response.data.contains("(2 entries found)"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_list_meta_success() {
        let csv_path = ".reed/meta.csv";
        create_test_csv(
            csv_path,
            vec![
                CsvRecord {
                    key: "cache.ttl".to_string(),
                    value: "3600".to_string(),
                    description: None,
                },
                CsvRecord {
                    key: "cache.enabled".to_string(),
                    value: "true".to_string(),
                    description: None,
                },
            ],
        );

        let args = vec!["cache.*".to_string()];
        let result = list_meta(&args);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("cache.ttl"));
        assert!(response.data.contains("cache.enabled"));
        assert!(response.data.contains("(2 entries found)"));

        cleanup_test_csv(csv_path);
    }

    #[test]
    fn test_round_trip_set_and_get() {
        let csv_path = ".reed/text.csv";
        create_test_csv(csv_path, vec![]);

        // Set a value
        let set_args = vec![
            "roundtrip.test@en".to_string(),
            "Round Trip Value".to_string(),
        ];
        let mut flags = HashMap::new();
        flags.insert(
            "desc".to_string(),
            "Round trip test description".to_string(),
        );

        let set_result = set_text(&set_args, &flags);
        assert!(set_result.is_ok());

        // Get the same value
        let get_args = vec!["roundtrip.test@en".to_string()];
        let get_result = get_text(&get_args);
        assert!(get_result.is_ok());

        let response = get_result.unwrap();
        assert!(response.data.contains("Round Trip Value"));

        cleanup_test_csv(csv_path);
    }
}
