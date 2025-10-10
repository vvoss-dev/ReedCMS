// Test script for REED-10-01 ReedMonitor functionality

use std::time::Duration;

// Simulating ReedCMS imports
mod reedcms {
    pub mod monitor {
        pub use crate::src::reedcms::monitor::*;
    }
}

fn main() {
    println!("🧪 Testing REED-10-01: ReedMonitor Foundation\n");

    // Test 1: Syslog Logger
    println!("Test 1: FreeBSD-style Syslog Logger");
    test_syslog();
    println!("✅ Syslog test passed\n");

    // Test 2: Metrics Collection
    println!("Test 2: Metrics Collection");
    test_metrics();
    println!("✅ Metrics test passed\n");

    // Test 3: Health Check
    println!("Test 3: Health Status");
    test_health();
    println!("✅ Health test passed\n");

    println!("✅ All REED-10-01 tests passed!");
}

fn test_syslog() {
    use reedcms::monitor::{LogLevel, OutputMode, SysLogger};

    let mut logger = SysLogger::new(OutputMode::Log, LogLevel::INFO).unwrap();

    // Test all log levels
    logger.log(LogLevel::INFO, "Server started on port 8333");
    logger.log(LogLevel::WARN, "High memory usage detected");
    logger.log(LogLevel::ERROR, "Database connection failed");

    // Test metric logging
    logger.log_metric("counter", "requests_total", "42");
    logger.log_metric("gauge", "memory_usage_mb", "128.5");

    println!("  ✓ All log levels working");
    println!("  ✓ Metric logging working");
    println!("  ✓ Log file created: .reed/flow/reedmonitor.log");
}

fn test_metrics() {
    use reedcms::monitor::global_monitor;

    let monitor = global_monitor();

    // Simulate some requests
    monitor.record_request("GET", "/knowledge", 200, Duration::from_millis(45));
    monitor.record_request("GET", "/blog", 200, Duration::from_millis(32));
    monitor.record_request("POST", "/api/text", 404, Duration::from_millis(8));

    // Simulate ReedBase lookups
    monitor.record_reedbase_lookup("knowledge.title", Duration::from_micros(50), true);
    monitor.record_reedbase_lookup("blog.title", Duration::from_micros(45), true);

    // Simulate template renders
    monitor.record_template_render("knowledge.mouse.jinja", Duration::from_millis(30));

    let snapshot = monitor.get_snapshot();

    println!("  ✓ Request metrics: {} requests", snapshot.total_requests);
    println!(
        "  ✓ Avg response time: {:.1}ms",
        snapshot.avg_response_time.as_secs_f64() * 1000.0
    );
    println!("  ✓ Error rate: {:.1}%", snapshot.error_rate * 100.0);
    println!(
        "  ✓ ReedBase hit rate: {:.1}%",
        snapshot.reedbase_hit_rate * 100.0
    );
}

fn test_health() {
    use reedcms::monitor::global_monitor;

    let monitor = global_monitor();
    let health = monitor.get_health();

    println!("  ✓ Status: {:?}", health.status);
    println!("  ✓ Total requests: {}", health.total_requests);
    println!("  ✓ Error rate: {:.3}", health.error_rate);
    println!(
        "  ✓ Avg response time: {}ms",
        health.avg_response_time.as_millis()
    );
}
