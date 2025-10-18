// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for metrics types module.

#[cfg(test)]
mod tests {
    use crate::metrics::types::{Metric, MetricUnit};

    #[test]
    fn test_metric_creation() {
        let metric = Metric::new("test_metric", 42.0, MetricUnit::Count);

        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.value, 42.0);
        assert_eq!(metric.unit, MetricUnit::Count);
        assert!(metric.tags.is_empty());
        assert!(metric.timestamp > 0);
    }

    #[test]
    fn test_metric_with_tags() {
        let metric = Metric::new("query_duration", 1250.0, MetricUnit::Microseconds)
            .with_tag("table", "text")
            .with_tag("operation", "get");

        assert_eq!(metric.tags.get("table"), Some(&"text".to_string()));
        assert_eq!(metric.tags.get("operation"), Some(&"get".to_string()));
    }

    #[test]
    fn test_metric_format_value() {
        let metric = Metric::new("test", 1250.5, MetricUnit::Microseconds);
        assert_eq!(metric.format_value(), "1250.50μs");

        let metric2 = Metric::new("test", 42.0, MetricUnit::Percent);
        assert_eq!(metric2.format_value(), "42.00%");
    }

    #[test]
    fn test_unit_suffix() {
        assert_eq!(MetricUnit::Microseconds.suffix(), "μs");
        assert_eq!(MetricUnit::Megabytes.suffix(), "MB");
        assert_eq!(MetricUnit::Count.suffix(), "");
    }

    #[test]
    fn test_unit_conversion() {
        // Time conversions
        assert_eq!(MetricUnit::Microseconds.to_base_unit(1.0), 1_000.0);
        assert_eq!(MetricUnit::Milliseconds.to_base_unit(1.0), 1_000_000.0);

        // Size conversions
        assert_eq!(MetricUnit::Kilobytes.to_base_unit(1.0), 1_024.0);
        assert_eq!(MetricUnit::Megabytes.to_base_unit(1.0), 1_048_576.0);

        // No conversion
        assert_eq!(MetricUnit::Count.to_base_unit(42.0), 42.0);
    }
}
