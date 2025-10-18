// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Core metric types for ReedBase observability.
//!
//! Provides fundamental types for collecting and representing metrics:
//! - `Metric`: Individual metric measurement with tags
//! - `MetricType`: Classification of metric behaviour
//! - `MetricUnit`: Unit of measurement with display formatting

use std::collections::HashMap;

/// A single metric measurement.
///
/// ## Example
/// ```
/// use reedbase::metrics::types::{Metric, MetricUnit};
///
/// let metric = Metric::new("query_duration", 1250.0, MetricUnit::Microseconds)
///     .with_tag("table", "text")
///     .with_tag("operation", "get");
/// ```
#[derive(Debug, Clone)]
pub struct Metric {
    /// Metric name (e.g., "query_duration", "cache_hit_rate")
    pub name: String,

    /// Numeric value of the measurement
    pub value: f64,

    /// Unit of measurement
    pub unit: MetricUnit,

    /// Optional tags for filtering/aggregation (e.g., table=text, operation=get)
    pub tags: HashMap<String, String>,

    /// Unix timestamp in nanoseconds (when metric was recorded)
    pub timestamp: u64,
}

impl Metric {
    /// Creates a new metric with the current timestamp.
    ///
    /// ## Arguments
    /// - `name`: Metric identifier
    /// - `value`: Numeric measurement
    /// - `unit`: Unit of measurement
    ///
    /// ## Returns
    /// A new `Metric` instance with empty tags and current timestamp
    pub fn new(name: impl Into<String>, value: f64, unit: MetricUnit) -> Self {
        Self {
            name: name.into(),
            value,
            unit,
            tags: HashMap::new(),
            timestamp: Self::now_nanos(),
        }
    }

    /// Adds a tag to the metric (builder pattern).
    ///
    /// ## Arguments
    /// - `key`: Tag name
    /// - `value`: Tag value
    ///
    /// ## Returns
    /// Self with tag added (for chaining)
    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }

    /// Adds multiple tags at once.
    pub fn with_tags(mut self, tags: HashMap<String, String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Gets current time as nanoseconds since Unix epoch.
    fn now_nanos() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before Unix epoch")
            .as_nanos() as u64
    }

    /// Formats the metric value with unit suffix.
    ///
    /// ## Example
    /// ```
    /// use reedbase::metrics::types::{Metric, MetricUnit};
    ///
    /// let metric = Metric::new("test", 1250.0, MetricUnit::Microseconds);
    /// let formatted = metric.format_value();
    /// assert_eq!(formatted, "1250.00μs");
    /// ```
    pub fn format_value(&self) -> String {
        format!("{:.2}{}", self.value, self.unit.suffix())
    }
}

/// Classification of metric behaviour.
///
/// Determines how metrics are aggregated and interpreted:
/// - **Counter**: Monotonically increasing value (e.g., total requests)
/// - **Gauge**: Point-in-time value that can go up/down (e.g., active connections)
/// - **Histogram**: Distribution of values (e.g., request latencies)
/// - **Timer**: Duration measurements (special case of histogram)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Monotonically increasing counter (resets on restart)
    Counter,

    /// Point-in-time measurement (can increase or decrease)
    Gauge,

    /// Distribution of values (for percentile calculation)
    Histogram,

    /// Duration measurement (histogram with time unit)
    Timer,
}

/// Unit of measurement for metric values.
///
/// Provides display formatting and semantic meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricUnit {
    // Time units
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,

    // Data size units
    Bytes,
    Kilobytes,
    Megabytes,

    // Dimensionless units
    Count,
    Percent,
}

impl MetricUnit {
    /// Returns the display suffix for the unit.
    ///
    /// ## Example
    /// ```
    /// use reedbase::metrics::types::MetricUnit;
    ///
    /// assert_eq!(MetricUnit::Microseconds.suffix(), "μs");
    /// assert_eq!(MetricUnit::Megabytes.suffix(), "MB");
    /// ```
    pub fn suffix(&self) -> &'static str {
        match self {
            Self::Nanoseconds => "ns",
            Self::Microseconds => "μs",
            Self::Milliseconds => "ms",
            Self::Seconds => "s",
            Self::Bytes => "B",
            Self::Kilobytes => "KB",
            Self::Megabytes => "MB",
            Self::Count => "",
            Self::Percent => "%",
        }
    }

    /// Converts value to base unit (nanoseconds for time, bytes for size).
    ///
    /// Used for aggregation across different unit scales.
    pub fn to_base_unit(&self, value: f64) -> f64 {
        match self {
            // Time: convert to nanoseconds
            Self::Nanoseconds => value,
            Self::Microseconds => value * 1_000.0,
            Self::Milliseconds => value * 1_000_000.0,
            Self::Seconds => value * 1_000_000_000.0,

            // Size: convert to bytes
            Self::Bytes => value,
            Self::Kilobytes => value * 1_024.0,
            Self::Megabytes => value * 1_048_576.0,

            // Dimensionless: no conversion
            Self::Count | Self::Percent => value,
        }
    }
}
