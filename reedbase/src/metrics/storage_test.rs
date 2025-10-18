// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for metrics storage module.

#[cfg(test)]
mod tests {
    use crate::metrics::storage::MetricsStorage;
    use crate::metrics::types::{Metric, MetricUnit};
    use std::fs;

    #[test]
    fn test_write_and_read_metrics() {
        let temp_dir = std::env::temp_dir().join("reedbase_test_metrics");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up previous test runs

        let storage = MetricsStorage::with_directory(&temp_dir);

        let mut metric = Metric::new("test_metric", 42.5, MetricUnit::Microseconds);
        metric.tags.insert("key".to_string(), "value".to_string());

        storage.write_batch(&[metric.clone()]).unwrap();

        let read_metrics = storage.read_metrics("test_metric").unwrap();
        assert_eq!(read_metrics.len(), 1);
        assert_eq!(read_metrics[0].value, 42.5);
        assert_eq!(read_metrics[0].tags.get("key"), Some(&"value".to_string()));

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    // Note: format_tags and parse_tags are private methods tested indirectly
    // via test_write_and_read_metrics which validates the full CSV round-trip

    #[test]
    fn test_list_metrics() {
        let temp_dir = std::env::temp_dir().join("reedbase_test_list");
        let _ = fs::remove_dir_all(&temp_dir);

        let storage = MetricsStorage::with_directory(&temp_dir);

        storage
            .write_batch(&[Metric::new("metric1", 1.0, MetricUnit::Count)])
            .unwrap();
        storage
            .write_batch(&[Metric::new("metric2", 2.0, MetricUnit::Count)])
            .unwrap();

        let names = storage.list_metrics().unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"metric1".to_string()));
        assert!(names.contains(&"metric2".to_string()));

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
