// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Metrics infrastructure for ReedBase observability.
//!
//! Provides lightweight performance monitoring and observability:
//! - **Types**: Metric types and units
//! - **Collector**: Global singleton for recording metrics
//! - **Storage**: CSV-based persistent storage
//! - **Aggregator**: Percentile and statistical calculations
//!
//! ## Quick Start
//!
//! ```rust
//! use reedbase::metrics::{MetricsCollector, Metric, MetricUnit};
//!
//! // Record a metric
//! let metric = Metric::new("query_duration", 1250.0, MetricUnit::Microseconds)
//!     .with_tag("table", "text")
//!     .with_tag("operation", "get");
//!
//! MetricsCollector::global().record(metric);
//!
//! // Flush to storage periodically
//! MetricsCollector::global().flush();
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                   Application Code                      │
//! └────────────────────┬────────────────────────────────────┘
//!                      │
//!                      ▼
//!            ┌─────────────────────┐
//!            │  MetricsCollector   │ ◄── Singleton
//!            │    (In-memory)      │
//!            └──────────┬──────────┘
//!                       │ flush()
//!                       ▼
//!            ┌─────────────────────┐
//!            │  MetricsStorage     │
//!            │   (CSV Backend)     │
//!            └──────────┬──────────┘
//!                       │
//!                       ▼
//!       .reedbase/metrics/query_duration.csv
//!       .reedbase/metrics/cache_hit_rate.csv
//!       .reedbase/metrics/write_latency.csv
//! ```
//!
//! ## Storage Format
//!
//! Each metric is stored in a separate CSV file:
//!
//! ```csv
//! timestamp|value|unit|tags
//! 1730000000000000000|1250.50|μs|table=text,operation=get
//! 1730000001000000000|980.25|μs|table=routes,operation=get
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Record**: O(1) - lock + push to buffer
//! - **Flush**: O(n) - write batched metrics to CSV
//! - **Aggregation**: O(n log n) - sorting for percentiles
//! - **Storage**: Append-only CSV (no seeks)
//!
//! ## Thread Safety
//!
//! - `MetricsCollector` uses `RwLock` for thread-safe access
//! - Multiple threads can record metrics concurrently
//! - Flush operations are synchronized
//! - Storage writes are atomic (temp file + rename)

pub mod aggregator;
pub mod collector;
pub mod storage;
pub mod types;

// Re-export commonly used types for convenience
pub use aggregator::{calculate_stats, p50, p95, p99, MetricStats};
pub use collector::MetricsCollector;
pub use types::{Metric, MetricType, MetricUnit};

#[cfg(test)]
mod integration_tests {
    use super::*;

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
