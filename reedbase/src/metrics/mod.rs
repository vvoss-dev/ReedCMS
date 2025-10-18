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

#[cfg(test)]
mod aggregator_test;
#[cfg(test)]
mod collector_test;
#[cfg(test)]
mod mod_test;
#[cfg(test)]
mod storage_test;
#[cfg(test)]
mod types_test;

// Re-export commonly used types for convenience
pub use aggregator::{calculate_stats, p50, p95, p99, MetricStats};
pub use collector::MetricsCollector;
pub use types::{Metric, MetricType, MetricUnit};
