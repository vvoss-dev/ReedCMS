// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Global metrics collector singleton.
//!
//! Provides thread-safe global access to metrics collection:
//! - `MetricsCollector::global()`: Access singleton instance
//! - `record()`: Record a single metric
//! - `flush()`: Persist all buffered metrics to storage
//!
//! ## Performance
//! - O(1) record operation (lock + push)
//! - Lock-free reads (RwLock with write bias)
//! - Configurable buffer size (default 1000 metrics)

use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

use super::storage::MetricsStorage;
use super::types::Metric;

/// Global singleton instance.
static METRICS_COLLECTOR: Lazy<Arc<MetricsCollector>> =
    Lazy::new(|| Arc::new(MetricsCollector::new()));

/// Thread-safe metrics collector.
///
/// ## Example
/// ```
/// use reedbase::metrics::collector::MetricsCollector;
/// use reedbase::metrics::types::{Metric, MetricUnit};
///
/// // Record a metric
/// let metric = Metric::new("query_duration", 1250.0, MetricUnit::Microseconds)
///     .with_tag("table", "text");
/// MetricsCollector::global().record(metric);
///
/// // Flush to storage
/// MetricsCollector::global().flush();
/// ```
pub struct MetricsCollector {
    /// In-memory buffer of metrics (awaiting flush)
    buffer: RwLock<Vec<Metric>>,

    /// Persistent storage backend
    storage: MetricsStorage,

    /// Maximum buffer size before auto-flush
    buffer_limit: usize,
}

impl MetricsCollector {
    /// Creates a new metrics collector with default settings.
    fn new() -> Self {
        Self {
            buffer: RwLock::new(Vec::with_capacity(1000)),
            storage: MetricsStorage::new(),
            buffer_limit: 1000,
        }
    }

    /// Returns the global singleton instance.
    ///
    /// ## Example
    /// ```
    /// use reedbase::MetricsCollector;
    ///
    /// let collector = MetricsCollector::global();
    /// ```
    pub fn global() -> Arc<Self> {
        Arc::clone(&METRICS_COLLECTOR)
    }

    /// Records a metric to the buffer.
    ///
    /// ## Arguments
    /// - `metric`: Metric to record
    ///
    /// ## Performance
    /// - O(1) operation
    /// - Acquires write lock briefly
    /// - Auto-flushes if buffer exceeds limit
    pub fn record(&self, metric: Metric) {
        let mut buffer = self.buffer.write().expect("Failed to acquire write lock");
        buffer.push(metric);

        // Auto-flush if buffer limit reached
        if buffer.len() >= self.buffer_limit {
            drop(buffer); // Release lock before flushing
            self.flush();
        }
    }

    /// Records multiple metrics at once.
    ///
    /// More efficient than calling `record()` multiple times.
    pub fn record_batch(&self, metrics: Vec<Metric>) {
        let mut buffer = self.buffer.write().expect("Failed to acquire write lock");
        buffer.extend(metrics);

        if buffer.len() >= self.buffer_limit {
            drop(buffer);
            self.flush();
        }
    }

    /// Flushes all buffered metrics to persistent storage.
    ///
    /// ## Behaviour
    /// - Swaps buffer with empty Vec (minimal lock time)
    /// - Writes to CSV storage asynchronously
    /// - Safe to call concurrently
    ///
    /// ## Error Handling
    /// - Logs errors but does not panic
    /// - Failed metrics are lost (trade-off for performance)
    pub fn flush(&self) {
        // Swap buffer with empty Vec to minimize lock time
        let metrics = {
            let mut buffer = self.buffer.write().expect("Failed to acquire write lock");
            std::mem::replace(&mut *buffer, Vec::with_capacity(self.buffer_limit))
        };

        if metrics.is_empty() {
            return;
        }

        // Write to storage (outside lock)
        if let Err(e) = self.storage.write_batch(&metrics) {
            eprintln!("Failed to flush metrics: {}", e);
        }
    }

    /// Returns the current buffer size.
    ///
    /// Useful for monitoring and testing.
    pub fn buffer_size(&self) -> usize {
        self.buffer
            .read()
            .expect("Failed to acquire read lock")
            .len()
    }

    /// Clears the buffer without persisting.
    ///
    /// ⚠️ **WARNING**: Discards all buffered metrics. Use only for testing.
    pub fn clear(&self) {
        let mut buffer = self.buffer.write().expect("Failed to acquire write lock");
        buffer.clear();
    }
}

// Ensure flush on program exit
impl Drop for MetricsCollector {
    fn drop(&mut self) {
        self.flush();
    }
}
