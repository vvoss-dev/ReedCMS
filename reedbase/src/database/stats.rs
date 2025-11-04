// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Query pattern tracking for auto-indexing.
//!
//! Tracks query patterns to automatically create indices when beneficial.

use std::collections::HashMap;

/// Query pattern for tracking.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct QueryPattern {
    /// Table name
    pub table: String,

    /// Column name
    pub column: String,

    /// Operation type ("equals", "range", "like")
    pub operation: String,
}

impl QueryPattern {
    /// Creates a new query pattern.
    pub fn new(table: String, column: String, operation: String) -> Self {
        Self {
            table,
            column,
            operation,
        }
    }
}

/// Tracks query patterns for auto-indexing decisions.
#[derive(Debug)]
pub struct PatternTracker {
    /// Pattern → count mapping
    patterns: HashMap<QueryPattern, usize>,

    /// Patterns that have triggered index creation
    indexed_patterns: HashMap<QueryPattern, bool>,
}

impl PatternTracker {
    /// Creates a new pattern tracker.
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            indexed_patterns: HashMap::new(),
        }
    }

    /// Records a query pattern.
    ///
    /// ## Input
    /// - `pattern`: Query pattern to track
    ///
    /// ## Output
    /// - Current count for this pattern
    pub fn record(&mut self, pattern: QueryPattern) -> usize {
        let count = self.patterns.entry(pattern).or_insert(0);
        *count += 1;
        *count
    }

    /// Checks if a pattern should trigger index creation.
    ///
    /// ## Input
    /// - `pattern`: Query pattern to check
    /// - `threshold`: Minimum count to trigger
    ///
    /// ## Output
    /// - `true` if index should be created
    /// - `false` if not yet
    pub fn should_create_index(&self, pattern: &QueryPattern, threshold: usize) -> bool {
        // Already indexed?
        if self.indexed_patterns.contains_key(pattern) {
            return false;
        }

        // Check count
        self.patterns.get(pattern).copied().unwrap_or(0) >= threshold
    }

    /// Marks a pattern as indexed.
    pub fn mark_indexed(&mut self, pattern: QueryPattern) {
        self.indexed_patterns.insert(pattern, true);
    }

    /// Gets count for a pattern.
    pub fn get_count(&self, pattern: &QueryPattern) -> usize {
        self.patterns.get(pattern).copied().unwrap_or(0)
    }

    /// Gets top N patterns by count.
    pub fn get_top_patterns(&self, n: usize) -> Vec<(QueryPattern, usize)> {
        let mut patterns: Vec<_> = self.patterns.iter().map(|(p, c)| (p.clone(), *c)).collect();
        patterns.sort_by(|a, b| b.1.cmp(&a.1));
        patterns.into_iter().take(n).collect()
    }

    /// Clears all tracked patterns.
    pub fn clear(&mut self) {
        self.patterns.clear();
        self.indexed_patterns.clear();
    }
}

impl Default for PatternTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_tracker_new() {
        let tracker = PatternTracker::new();
        assert_eq!(tracker.patterns.len(), 0);
        assert_eq!(tracker.indexed_patterns.len(), 0);
    }

    #[test]
    fn test_record_pattern() {
        let mut tracker = PatternTracker::new();
        let pattern =
            QueryPattern::new("text".to_string(), "key".to_string(), "equals".to_string());

        assert_eq!(tracker.record(pattern.clone()), 1);
        assert_eq!(tracker.record(pattern.clone()), 2);
        assert_eq!(tracker.record(pattern.clone()), 3);
    }

    #[test]
    fn test_should_create_index() {
        let mut tracker = PatternTracker::new();
        let pattern =
            QueryPattern::new("text".to_string(), "key".to_string(), "equals".to_string());

        // Not yet
        assert!(!tracker.should_create_index(&pattern, 10));

        // Record 10 times
        for _ in 0..10 {
            tracker.record(pattern.clone());
        }

        // Should create now
        assert!(tracker.should_create_index(&pattern, 10));

        // Mark as indexed
        tracker.mark_indexed(pattern.clone());

        // Should not create again
        assert!(!tracker.should_create_index(&pattern, 10));
    }

    #[test]
    fn test_get_count() {
        let mut tracker = PatternTracker::new();
        let pattern =
            QueryPattern::new("text".to_string(), "key".to_string(), "equals".to_string());

        assert_eq!(tracker.get_count(&pattern), 0);

        tracker.record(pattern.clone());
        assert_eq!(tracker.get_count(&pattern), 1);

        tracker.record(pattern.clone());
        assert_eq!(tracker.get_count(&pattern), 2);
    }

    #[test]
    fn test_get_top_patterns() {
        let mut tracker = PatternTracker::new();

        let pattern1 =
            QueryPattern::new("text".to_string(), "key".to_string(), "equals".to_string());
        let pattern2 =
            QueryPattern::new("text".to_string(), "value".to_string(), "like".to_string());
        let pattern3 = QueryPattern::new(
            "routes".to_string(),
            "path".to_string(),
            "equals".to_string(),
        );

        // Record different counts
        for _ in 0..10 {
            tracker.record(pattern1.clone());
        }
        for _ in 0..5 {
            tracker.record(pattern2.clone());
        }
        for _ in 0..20 {
            tracker.record(pattern3.clone());
        }

        let top = tracker.get_top_patterns(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].1, 20); // pattern3
        assert_eq!(top[1].1, 10); // pattern1
    }

    #[test]
    fn test_clear() {
        let mut tracker = PatternTracker::new();
        let pattern =
            QueryPattern::new("text".to_string(), "key".to_string(), "equals".to_string());

        tracker.record(pattern.clone());
        tracker.mark_indexed(pattern.clone());

        assert_eq!(tracker.patterns.len(), 1);
        assert_eq!(tracker.indexed_patterns.len(), 1);

        tracker.clear();

        assert_eq!(tracker.patterns.len(), 0);
        assert_eq!(tracker.indexed_patterns.len(), 0);
    }
}
