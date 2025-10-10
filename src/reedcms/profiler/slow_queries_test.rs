// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for slow query tracker.

#[cfg(test)]
mod tests {
    use crate::reedcms::profiler::slow_queries::SlowQueryTracker;
    use std::time::Duration;

    #[test]
    fn test_tracker_creation() {
        let tracker = SlowQueryTracker::new();
        assert_eq!(tracker.count(), 0);
    }

    #[test]
    fn test_record_slow_query() {
        let tracker = SlowQueryTracker::new();

        // Should be recorded (> 100ms default threshold)
        tracker.record(
            "database_query",
            Duration::from_millis(150),
            "SELECT * FROM users".to_string(),
        );

        assert_eq!(tracker.count(), 1);
    }

    #[test]
    fn test_fast_query_not_recorded() {
        let tracker = SlowQueryTracker::new();

        // Should NOT be recorded (< 100ms threshold)
        tracker.record(
            "database_query",
            Duration::from_millis(50),
            "SELECT id FROM cache".to_string(),
        );

        assert_eq!(tracker.count(), 0);
    }

    #[test]
    fn test_multiple_slow_queries() {
        let tracker = SlowQueryTracker::new();

        tracker.record("query1", Duration::from_millis(120), "Query 1".to_string());
        tracker.record("query2", Duration::from_millis(150), "Query 2".to_string());
        tracker.record("query3", Duration::from_millis(180), "Query 3".to_string());

        assert_eq!(tracker.count(), 3);
    }

    #[test]
    fn test_ring_buffer_limit() {
        let tracker = SlowQueryTracker::new();

        // Record 105 slow queries
        for i in 0..105 {
            tracker.record("query", Duration::from_millis(110), format!("Query {}", i));
        }

        // Should keep only last 100
        assert_eq!(tracker.count(), 100);
    }

    #[test]
    fn test_get_by_operation() {
        let tracker = SlowQueryTracker::new();

        tracker.record("reedbase", Duration::from_millis(120), "key1".to_string());
        tracker.record(
            "template",
            Duration::from_millis(150),
            "layout1".to_string(),
        );
        tracker.record("reedbase", Duration::from_millis(130), "key2".to_string());

        let reedbase_queries = tracker.get_by_operation("reedbase");
        assert_eq!(reedbase_queries.len(), 2);

        let template_queries = tracker.get_by_operation("template");
        assert_eq!(template_queries.len(), 1);
    }

    #[test]
    fn test_get_all_queries() {
        let tracker = SlowQueryTracker::new();

        tracker.record("op1", Duration::from_millis(120), "ctx1".to_string());
        tracker.record("op2", Duration::from_millis(150), "ctx2".to_string());

        let all_queries = tracker.get_slow_queries();
        assert_eq!(all_queries.len(), 2);
    }

    #[test]
    fn test_clear_queries() {
        let tracker = SlowQueryTracker::new();

        tracker.record("query", Duration::from_millis(120), "test".to_string());
        assert_eq!(tracker.count(), 1);

        tracker.clear();
        assert_eq!(tracker.count(), 0);
    }

    #[test]
    fn test_query_contains_context() {
        let tracker = SlowQueryTracker::new();

        tracker.record(
            "csv_read",
            Duration::from_millis(120),
            ".reed/text.csv".to_string(),
        );

        let queries = tracker.get_slow_queries();
        assert_eq!(queries[0].operation, "csv_read");
        assert_eq!(queries[0].context, ".reed/text.csv");
    }
}
