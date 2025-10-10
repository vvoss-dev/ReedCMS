// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedMonitor core metrics collection system.
//!
//! ## Features
//! - In-memory time-series metrics
//! - Thread-safe recording
//! - Rolling window (24 hours)
//! - Aggregated statistics
//!
//! ## Performance
//! - Metric recording: < 10μs
//! - Metric retrieval: < 1ms
//! - Memory usage: ~10MB for 24h data

use super::metrics::{Health, HealthStatus, Metrics, MetricsSnapshot};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// ReedMonitor core metrics collection system.
///
/// ## Usage
/// ```rust
/// let monitor = ReedMonitor::new();
/// monitor.record_request("GET", "/knowledge", 200, Duration::from_millis(45));
/// let snapshot = monitor.get_snapshot();
/// ```
pub struct ReedMonitor {
    metrics: Arc<RwLock<Metrics>>,
    start_time: Instant,
}

impl ReedMonitor {
    /// Creates new ReedMonitor instance.
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Metrics::new())),
            start_time: Instant::now(),
        }
    }

    /// Records request metric.
    ///
    /// ## Arguments
    /// - `method`: HTTP method (GET, POST, etc.)
    /// - `path`: Request path
    /// - `status`: Response status code
    /// - `duration`: Request duration
    ///
    /// ## Performance
    /// - < 10μs per call
    ///
    /// ## Example
    /// ```rust
    /// monitor.record_request("GET", "/knowledge", 200, Duration::from_millis(45));
    /// ```
    pub fn record_request(&self, method: &str, path: &str, status: u16, duration: Duration) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.record_request(method, path, status, duration);
        }
    }

    /// Records ReedBase lookup metric.
    ///
    /// ## Arguments
    /// - `key`: Lookup key
    /// - `duration`: Lookup duration
    /// - `hit`: Whether cache hit occurred
    ///
    /// ## Example
    /// ```rust
    /// monitor.record_reedbase_lookup("knowledge.title", Duration::from_micros(50), true);
    /// ```
    pub fn record_reedbase_lookup(&self, key: &str, duration: Duration, hit: bool) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.record_reedbase_lookup(key, duration, hit);
        }
    }

    /// Records template render metric.
    ///
    /// ## Arguments
    /// - `template`: Template name
    /// - `duration`: Render duration
    ///
    /// ## Example
    /// ```rust
    /// monitor.record_template_render("knowledge.mouse.jinja", Duration::from_millis(32));
    /// ```
    pub fn record_template_render(&self, template: &str, duration: Duration) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.record_template_render(template, duration);
        }
    }

    /// Gets current metrics snapshot.
    ///
    /// ## Returns
    /// Complete metrics snapshot with aggregated statistics
    ///
    /// ## Performance
    /// - < 1ms per call
    pub fn get_snapshot(&self) -> MetricsSnapshot {
        let metrics = self.metrics.read().unwrap();
        metrics.snapshot(self.start_time.elapsed())
    }

    /// Gets system health status.
    ///
    /// ## Returns
    /// Health status based on error rate and response time
    ///
    /// ## Thresholds
    /// - Unhealthy: error rate > 5%
    /// - Degraded: avg response time > 500ms
    /// - Healthy: otherwise
    pub fn get_health(&self) -> HealthStatus {
        let snapshot = self.get_snapshot();

        let status = if snapshot.error_rate > 0.05 {
            Health::Unhealthy
        } else if snapshot.avg_response_time > Duration::from_millis(500) {
            Health::Degraded
        } else {
            Health::Healthy
        };

        HealthStatus {
            status,
            uptime: snapshot.uptime,
            total_requests: snapshot.total_requests,
            error_rate: snapshot.error_rate,
            avg_response_time: snapshot.avg_response_time,
        }
    }

    /// Resets metrics (for testing).
    #[cfg(test)]
    pub fn reset(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            *metrics = Metrics::new();
        }
    }
}

impl Default for ReedMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Gets global ReedMonitor instance.
///
/// ## Usage
/// ```rust
/// global_monitor().record_request("GET", "/", 200, Duration::from_millis(10));
/// ```
pub fn global_monitor() -> &'static ReedMonitor {
    use std::sync::OnceLock;
    static MONITOR: OnceLock<ReedMonitor> = OnceLock::new();
    MONITOR.get_or_init(ReedMonitor::new)
}
