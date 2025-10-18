// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for metrics collector module.

#[cfg(test)]
mod tests {
    use crate::metrics::collector::MetricsCollector;
    use crate::metrics::types::{Metric, MetricUnit};
    use std::sync::Arc;

    #[test]
    fn test_singleton_access() {
        let collector1 = MetricsCollector::global();
        let collector2 = MetricsCollector::global();

        // Both should point to same instance
        assert!(Arc::ptr_eq(&collector1, &collector2));
    }

    #[test]
    fn test_record_metric() {
        let collector = MetricsCollector::global();
        collector.clear(); // Start fresh

        let metric = Metric::new("test_metric", 42.0, MetricUnit::Count);
        collector.record(metric);

        assert_eq!(collector.buffer_size(), 1);
    }

    #[test]
    fn test_record_batch() {
        let collector = MetricsCollector::global();
        collector.clear();

        let metrics = vec![
            Metric::new("metric1", 1.0, MetricUnit::Count),
            Metric::new("metric2", 2.0, MetricUnit::Count),
            Metric::new("metric3", 3.0, MetricUnit::Count),
        ];

        collector.record_batch(metrics);
        assert_eq!(collector.buffer_size(), 3);
    }

    #[test]
    fn test_flush() {
        let collector = MetricsCollector::global();
        collector.clear();

        collector.record(Metric::new("test_flush", 1.0, MetricUnit::Count));

        // Buffer size should be at least 1 (may be more due to concurrent tests)
        let size_before = collector.buffer_size();
        assert!(size_before >= 1);

        collector.flush();
        assert_eq!(collector.buffer_size(), 0);
    }

    #[test]
    fn test_clear() {
        let collector = MetricsCollector::global();

        collector.record(Metric::new("test", 1.0, MetricUnit::Count));
        collector.record(Metric::new("test", 2.0, MetricUnit::Count));

        collector.clear();
        assert_eq!(collector.buffer_size(), 0);
    }
}
