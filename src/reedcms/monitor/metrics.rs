// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Metrics storage and aggregation.
//!
//! ## Features
//! - Request metrics (count, duration, status codes)
//! - ReedBase metrics (lookups, hit rate)
//! - Template metrics (render times)
//! - System metrics (memory usage)
//!
//! ## Performance
//! - Metric recording: < 10μs
//! - Aggregation: < 1ms
//! - Memory usage: ~10MB for 24h data

use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

/// Complete metrics storage.
#[derive(Debug, Clone)]
pub struct Metrics {
    pub requests: RequestMetrics,
    pub reedbase: ReedBaseMetrics,
    pub templates: TemplateMetrics,
    pub system: SystemMetrics,
}

impl Metrics {
    /// Creates new metrics instance.
    pub fn new() -> Self {
        Self {
            requests: RequestMetrics::new(),
            reedbase: ReedBaseMetrics::new(),
            templates: TemplateMetrics::new(),
            system: SystemMetrics::new(),
        }
    }

    /// Records request metric.
    pub fn record_request(&mut self, method: &str, path: &str, status: u16, duration: Duration) {
        self.requests.record(method, path, status, duration);
    }

    /// Records ReedBase lookup metric.
    pub fn record_reedbase_lookup(&mut self, key: &str, duration: Duration, hit: bool) {
        self.reedbase.record(key, duration, hit);
    }

    /// Records template render metric.
    pub fn record_template_render(&mut self, template: &str, duration: Duration) {
        self.templates.record(template, duration);
    }

    /// Creates metrics snapshot.
    pub fn snapshot(&self, uptime: Duration) -> MetricsSnapshot {
        MetricsSnapshot {
            uptime,
            total_requests: self.requests.total_count,
            avg_response_time: self.requests.avg_duration(),
            error_rate: self.requests.error_rate(),
            requests_by_path: self.requests.by_path.clone(),
            status_codes: self.requests.status_codes.clone(),
            reedbase_hit_rate: self.reedbase.hit_rate(),
            reedbase_avg_time: self.reedbase.avg_duration(),
            template_avg_time: self.templates.avg_duration(),
            memory_usage: self.system.memory_usage(),
        }
    }
}

/// Request metrics structure.
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub total_count: u64,
    pub durations: Vec<Duration>,
    pub by_path: HashMap<String, u64>,
    pub status_codes: HashMap<u16, u64>,
}

impl RequestMetrics {
    pub fn new() -> Self {
        Self {
            total_count: 0,
            durations: Vec::new(),
            by_path: HashMap::new(),
            status_codes: HashMap::new(),
        }
    }

    pub fn record(&mut self, _method: &str, path: &str, status: u16, duration: Duration) {
        self.total_count += 1;
        self.durations.push(duration);

        // Trim old durations (keep last 10000)
        if self.durations.len() > 10000 {
            self.durations.drain(0..5000);
        }

        // Track by path
        *self.by_path.entry(path.to_string()).or_insert(0) += 1;

        // Track status codes
        *self.status_codes.entry(status).or_insert(0) += 1;
    }

    pub fn avg_duration(&self) -> Duration {
        if self.durations.is_empty() {
            return Duration::from_secs(0);
        }

        let total: Duration = self.durations.iter().sum();
        total / self.durations.len() as u32
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        let errors: u64 = self
            .status_codes
            .iter()
            .filter(|(code, _)| **code >= 400)
            .map(|(_, count)| count)
            .sum();

        errors as f64 / self.total_count as f64
    }
}

/// ReedBase metrics structure.
#[derive(Debug, Clone)]
pub struct ReedBaseMetrics {
    pub total_lookups: u64,
    pub cache_hits: u64,
    pub durations: Vec<Duration>,
}

impl ReedBaseMetrics {
    pub fn new() -> Self {
        Self {
            total_lookups: 0,
            cache_hits: 0,
            durations: Vec::new(),
        }
    }

    pub fn record(&mut self, _key: &str, duration: Duration, hit: bool) {
        self.total_lookups += 1;
        if hit {
            self.cache_hits += 1;
        }
        self.durations.push(duration);

        // Trim old durations
        if self.durations.len() > 10000 {
            self.durations.drain(0..5000);
        }
    }

    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / self.total_lookups as f64
    }

    pub fn avg_duration(&self) -> Duration {
        if self.durations.is_empty() {
            return Duration::from_secs(0);
        }

        let total: Duration = self.durations.iter().sum();
        total / self.durations.len() as u32
    }
}

/// Template metrics structure.
#[derive(Debug, Clone)]
pub struct TemplateMetrics {
    pub render_count: u64,
    pub durations: Vec<Duration>,
}

impl TemplateMetrics {
    pub fn new() -> Self {
        Self {
            render_count: 0,
            durations: Vec::new(),
        }
    }

    pub fn record(&mut self, _template: &str, duration: Duration) {
        self.render_count += 1;
        self.durations.push(duration);

        if self.durations.len() > 10000 {
            self.durations.drain(0..5000);
        }
    }

    pub fn avg_duration(&self) -> Duration {
        if self.durations.is_empty() {
            return Duration::from_secs(0);
        }

        let total: Duration = self.durations.iter().sum();
        total / self.durations.len() as u32
    }
}

/// System metrics structure.
#[derive(Debug, Clone)]
pub struct SystemMetrics;

impl SystemMetrics {
    pub fn new() -> Self {
        Self
    }

    /// Gets RSS memory usage in bytes.
    pub fn memory_usage(&self) -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<u64>() {
                                return kb_val * 1024; // Convert to bytes
                            }
                        }
                    }
                }
            }
        }

        0
    }
}

/// Metrics snapshot for reporting.
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub uptime: Duration,
    pub total_requests: u64,
    pub avg_response_time: Duration,
    pub error_rate: f64,
    pub requests_by_path: HashMap<String, u64>,
    pub status_codes: HashMap<u16, u64>,
    pub reedbase_hit_rate: f64,
    pub reedbase_avg_time: Duration,
    pub template_avg_time: Duration,
    pub memory_usage: u64,
}

/// Health status enum.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Health {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health status structure.
#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: Health,
    pub uptime: Duration,
    pub total_requests: u64,
    pub error_rate: f64,
    pub avg_response_time: Duration,
}
