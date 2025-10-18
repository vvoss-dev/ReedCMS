// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for metrics module.

#[cfg(test)]
mod integration_tests {
    use crate::metrics::{calculate_stats, Metric, MetricUnit, MetricsCollector};

    #[test]
    fn test_end_to_end_metric_recording() {
        let collector = MetricsCollector::global();
        collector.clear();

        // Record metrics
        let metric1 = Metric::new("test_metric", 100.0, MetricUnit::Microseconds)
            .with_tag("operation", "get");
        let metric2 = Metric::new("test_metric", 200.0, MetricUnit::Microseconds)
            .with_tag("operation", "set");

        collector.record(metric1);
        collector.record(metric2);

        assert_eq!(collector.buffer_size(), 2);

        // Flush to storage
        collector.flush();
        assert_eq!(collector.buffer_size(), 0);
    }

    #[test]
    fn test_metric_aggregation() {
        let values = vec![100.0, 200.0, 300.0, 400.0, 500.0];

        let stats = calculate_stats(&values).unwrap();

        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 300.0);
        assert_eq!(stats.min, 100.0);
        assert_eq!(stats.max, 500.0);
        assert_eq!(stats.p50, 300.0);
    }
}
