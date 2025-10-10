// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for performance profiler.

#[cfg(test)]
mod tests {
    use crate::reedcms::profiler::core::Profiler;
    use std::time::Duration;

    #[test]
    fn test_profiler_creation() {
        let profiler = Profiler::start("test_operation");
        let report = profiler.finish();

        assert_eq!(report.name, "test_operation");
        assert_eq!(report.spans.len(), 0);
    }

    #[test]
    fn test_single_span() {
        let profiler = Profiler::start("test");

        {
            let _span = profiler.span("operation1");
            std::thread::sleep(Duration::from_millis(10));
        }

        let report = profiler.finish();
        assert_eq!(report.spans.len(), 1);
        assert_eq!(report.spans[0].name, "operation1");
        assert!(report.spans[0].duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_multiple_spans() {
        let profiler = Profiler::start("test");

        {
            let _span1 = profiler.span("op1");
            std::thread::sleep(Duration::from_millis(5));
        }
        {
            let _span2 = profiler.span("op2");
            std::thread::sleep(Duration::from_millis(5));
        }

        let report = profiler.finish();
        assert_eq!(report.spans.len(), 2);
        assert_eq!(report.spans[0].name, "op1");
        assert_eq!(report.spans[1].name, "op2");
    }

    #[test]
    fn test_nested_spans() {
        let profiler = Profiler::start("test");

        {
            let _outer = profiler.span("outer");
            std::thread::sleep(Duration::from_millis(5));

            {
                let _inner = profiler.span("inner");
                std::thread::sleep(Duration::from_millis(5));
            }
        }

        let report = profiler.finish();
        assert_eq!(report.spans.len(), 2);
        assert_eq!(report.spans[0].depth, 0); // outer
        assert_eq!(report.spans[1].depth, 1); // inner (nested)
    }

    #[test]
    fn test_report_format() {
        let profiler = Profiler::start("test_request");

        {
            let _span = profiler.span("routing");
            std::thread::sleep(Duration::from_millis(10));
        }

        let report = profiler.finish();
        let formatted = report.format();

        assert!(formatted.contains("Profile: test_request"));
        assert!(formatted.contains("routing"));
        assert!(formatted.contains("ms"));
        assert!(formatted.contains("%"));
    }

    #[test]
    fn test_bottleneck_detection() {
        let profiler = Profiler::start("test");

        {
            let _fast = profiler.span("fast");
            std::thread::sleep(Duration::from_millis(5));
        }
        {
            let _slow = profiler.span("slow");
            std::thread::sleep(Duration::from_millis(50)); // > 25% of total
        }

        let report = profiler.finish();
        let bottlenecks = report.bottlenecks();

        // Should detect "slow" as bottleneck
        assert!(!bottlenecks.is_empty());
        assert_eq!(bottlenecks[0].name, "slow");
    }

    #[test]
    fn test_report_to_json() {
        let profiler = Profiler::start("test");

        {
            let _span = profiler.span("operation");
            std::thread::sleep(Duration::from_millis(10));
        }

        let report = profiler.finish();
        let json = report.to_json();

        assert_eq!(json["name"], "test");
        assert!(json["total_duration_ms"].as_f64().unwrap() > 0.0);
        assert_eq!(json["spans"].as_array().unwrap().len(), 1);
    }
}
