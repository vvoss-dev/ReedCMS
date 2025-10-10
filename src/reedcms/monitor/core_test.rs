// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for ReedMonitor core.

#[cfg(test)]
mod tests {
    use crate::reedcms::monitor::core::ReedMonitor;
    use crate::reedcms::monitor::metrics::Health;
    use std::time::Duration;

    #[test]
    fn test_monitor_creation() {
        let monitor = ReedMonitor::new();
        let snapshot = monitor.get_snapshot();

        assert_eq!(snapshot.total_requests, 0);
        assert_eq!(snapshot.error_rate, 0.0);
    }

    #[test]
    fn test_record_request() {
        let monitor = ReedMonitor::new();

        monitor.record_request("GET", "/test", 200, Duration::from_millis(50));
        monitor.record_request("POST", "/api", 201, Duration::from_millis(30));

        let snapshot = monitor.get_snapshot();
        assert_eq!(snapshot.total_requests, 2);
    }

    #[test]
    fn test_error_rate_calculation() {
        let monitor = ReedMonitor::new();

        // 2 successful, 1 error = 33.3% error rate
        monitor.record_request("GET", "/ok1", 200, Duration::from_millis(10));
        monitor.record_request("GET", "/ok2", 200, Duration::from_millis(10));
        monitor.record_request("GET", "/fail", 500, Duration::from_millis(10));

        let snapshot = monitor.get_snapshot();
        assert_eq!(snapshot.total_requests, 3);
        assert!((snapshot.error_rate - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_reedbase_metrics() {
        let monitor = ReedMonitor::new();

        // 2 hits, 1 miss = 66.7% hit rate
        monitor.record_reedbase_lookup("key1", Duration::from_micros(50), true);
        monitor.record_reedbase_lookup("key2", Duration::from_micros(45), true);
        monitor.record_reedbase_lookup("key3", Duration::from_micros(100), false);

        let snapshot = monitor.get_snapshot();
        assert!((snapshot.reedbase_hit_rate - 0.667).abs() < 0.01);
    }

    #[test]
    fn test_template_metrics() {
        let monitor = ReedMonitor::new();

        monitor.record_template_render("layout1", Duration::from_millis(30));
        monitor.record_template_render("layout2", Duration::from_millis(20));

        let snapshot = monitor.get_snapshot();
        let avg_ms = snapshot.template_avg_time.as_millis();
        assert_eq!(avg_ms, 25);
    }

    #[test]
    fn test_health_status_healthy() {
        let monitor = ReedMonitor::new();

        // Low error rate, fast responses = healthy
        monitor.record_request("GET", "/test", 200, Duration::from_millis(10));
        monitor.record_request("GET", "/test", 200, Duration::from_millis(20));

        let health = monitor.get_health();
        assert_eq!(health.status, Health::Healthy);
    }

    #[test]
    fn test_health_status_degraded() {
        let monitor = ReedMonitor::new();

        // Slow responses = degraded
        monitor.record_request("GET", "/slow", 200, Duration::from_millis(600));

        let health = monitor.get_health();
        assert_eq!(health.status, Health::Degraded);
    }

    #[test]
    fn test_health_status_unhealthy() {
        let monitor = ReedMonitor::new();

        // High error rate = unhealthy (> 5%)
        for _ in 0..10 {
            monitor.record_request("GET", "/fail", 500, Duration::from_millis(10));
        }
        monitor.record_request("GET", "/ok", 200, Duration::from_millis(10));

        let health = monitor.get_health();
        assert_eq!(health.status, Health::Unhealthy);
    }

    #[test]
    fn test_reset_metrics() {
        let monitor = ReedMonitor::new();

        monitor.record_request("GET", "/test", 200, Duration::from_millis(10));
        assert_eq!(monitor.get_snapshot().total_requests, 1);

        monitor.reset();
        assert_eq!(monitor.get_snapshot().total_requests, 0);
    }
}
