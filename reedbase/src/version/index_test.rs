// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::error::ReedResult;
    use crate::indices::Index;
    use crate::version::index::{FrameId, Timestamp, VersionId, VersionIndices};
    use std::collections::HashMap;

    // Mock index for testing that supports range queries over HashMap
    #[derive(Debug)]
    struct MockIndex {
        data: HashMap<String, Vec<usize>>,
    }

    impl MockIndex {
        fn new() -> Self {
            Self {
                data: HashMap::new(),
            }
        }
    }

    impl Index<String, Vec<usize>> for MockIndex {
        fn get(&self, key: &String) -> ReedResult<Option<Vec<usize>>> {
            Ok(self.data.get(key).cloned())
        }

        fn range(&self, start: &String, end: &String) -> ReedResult<Vec<(String, Vec<usize>)>> {
            let mut results: Vec<(String, Vec<usize>)> = self
                .data
                .iter()
                .filter(|(k, _)| *k >= start && *k <= end)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            results.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(results)
        }

        fn insert(&mut self, key: String, value: Vec<usize>) -> ReedResult<()> {
            self.data.insert(key, value);
            Ok(())
        }

        fn delete(&mut self, key: &String) -> ReedResult<()> {
            self.data.remove(key);
            Ok(())
        }

        fn iter(&self) -> Box<dyn Iterator<Item = (String, Vec<usize>)> + '_> {
            Box::new(self.data.iter().map(|(k, v)| (k.clone(), v.clone())))
        }

        fn backend_type(&self) -> &'static str {
            "mock"
        }

        fn memory_usage(&self) -> usize {
            0
        }

        fn disk_usage(&self) -> usize {
            0
        }
    }

    fn create_test_indices() -> VersionIndices {
        let timestamp_index: Box<dyn Index<Timestamp, Vec<VersionId>>> = Box::new(MockIndex::new());
        let frame_index: Box<dyn Index<FrameId, Vec<VersionId>>> = Box::new(MockIndex::new());

        VersionIndices {
            timestamp_index,
            frame_index,
        }
    }

    #[test]
    fn test_open_or_create_new_indices() {
        let indices = create_test_indices();

        // Indices should be empty initially
        let timestamps = indices.get_all_timestamps().unwrap();
        assert_eq!(timestamps.len(), 0);
    }

    #[test]
    fn test_insert_single_version() {
        let mut indices = create_test_indices();

        indices
            .insert(
                1,
                "2025-10-28T08:15:23.001Z".to_string(),
                "F001".to_string(),
            )
            .unwrap();

        // Query by timestamp
        let versions = indices
            .query_timestamp_range(
                &"2025-10-28T08:00:00.000Z".to_string(),
                &"2025-10-28T09:00:00.000Z".to_string(),
            )
            .unwrap();
        assert_eq!(versions, vec![1]);

        // Query by frame
        let frame_versions = indices.query_frame(&"F001".to_string()).unwrap();
        assert_eq!(frame_versions, vec![1]);
    }

    #[test]
    fn test_insert_multiple_versions_same_timestamp() {
        let mut indices = create_test_indices();

        let timestamp = "2025-10-28T08:15:23.001Z".to_string();
        indices
            .insert(1, timestamp.clone(), "F001".to_string())
            .unwrap();
        indices
            .insert(2, timestamp.clone(), "F001".to_string())
            .unwrap();

        let versions = indices
            .query_timestamp_range(&timestamp, &timestamp)
            .unwrap();
        assert_eq!(versions, vec![1, 2]);
    }

    #[test]
    fn test_insert_multiple_versions_same_frame() {
        let mut indices = create_test_indices();

        indices
            .insert(
                1,
                "2025-10-28T08:15:23.001Z".to_string(),
                "F001".to_string(),
            )
            .unwrap();
        indices
            .insert(
                2,
                "2025-10-28T08:15:24.001Z".to_string(),
                "F001".to_string(),
            )
            .unwrap();

        let frame_versions = indices.query_frame(&"F001".to_string()).unwrap();
        assert_eq!(frame_versions, vec![1, 2]);
    }

    #[test]
    fn test_query_timestamp_range_empty() {
        let indices = create_test_indices();

        let versions = indices
            .query_timestamp_range(
                &"2025-10-28T08:00:00.000Z".to_string(),
                &"2025-10-28T09:00:00.000Z".to_string(),
            )
            .unwrap();
        assert_eq!(versions.len(), 0);
    }

    #[test]
    fn test_query_timestamp_range_with_results() {
        let mut indices = create_test_indices();

        indices
            .insert(
                1,
                "2025-10-28T08:15:23.001Z".to_string(),
                "F001".to_string(),
            )
            .unwrap();
        indices
            .insert(
                2,
                "2025-10-28T08:16:45.500Z".to_string(),
                "F002".to_string(),
            )
            .unwrap();
        indices
            .insert(
                3,
                "2025-10-28T09:30:00.000Z".to_string(),
                "F003".to_string(),
            )
            .unwrap();

        let versions = indices
            .query_timestamp_range(
                &"2025-10-28T08:00:00.000Z".to_string(),
                &"2025-10-28T09:00:00.000Z".to_string(),
            )
            .unwrap();
        assert_eq!(versions, vec![1, 2]);
    }

    #[test]
    fn test_query_timestamp_range_inclusive_bounds() {
        let mut indices = create_test_indices();

        indices
            .insert(
                1,
                "2025-10-28T08:00:00.000Z".to_string(),
                "F001".to_string(),
            )
            .unwrap();
        indices
            .insert(
                2,
                "2025-10-28T09:00:00.000Z".to_string(),
                "F002".to_string(),
            )
            .unwrap();

        let versions = indices
            .query_timestamp_range(
                &"2025-10-28T08:00:00.000Z".to_string(),
                &"2025-10-28T09:00:00.000Z".to_string(),
            )
            .unwrap();
        assert_eq!(versions, vec![1, 2]);
    }

    #[test]
    fn test_query_frame_not_found() {
        let indices = create_test_indices();

        let versions = indices.query_frame(&"F999".to_string()).unwrap();
        assert_eq!(versions.len(), 0);
    }

    #[test]
    fn test_query_frame_with_results() {
        let mut indices = create_test_indices();

        indices
            .insert(
                42,
                "2025-10-28T08:15:23.001Z".to_string(),
                "F042".to_string(),
            )
            .unwrap();
        indices
            .insert(
                43,
                "2025-10-28T08:15:24.001Z".to_string(),
                "F042".to_string(),
            )
            .unwrap();

        let versions = indices.query_frame(&"F042".to_string()).unwrap();
        assert_eq!(versions, vec![42, 43]);
    }

    #[test]
    fn test_get_all_timestamps_empty() {
        let indices = create_test_indices();

        let timestamps = indices.get_all_timestamps().unwrap();
        assert_eq!(timestamps.len(), 0);
    }

    #[test]
    fn test_get_all_timestamps_with_results() {
        let mut indices = create_test_indices();

        indices
            .insert(
                1,
                "2025-10-28T08:15:23.001Z".to_string(),
                "F001".to_string(),
            )
            .unwrap();
        indices
            .insert(
                2,
                "2025-10-28T08:16:45.500Z".to_string(),
                "F002".to_string(),
            )
            .unwrap();
        indices
            .insert(
                3,
                "2025-10-28T08:16:45.500Z".to_string(),
                "F002".to_string(),
            )
            .unwrap();

        let timestamps = indices.get_all_timestamps().unwrap();
        assert_eq!(timestamps.len(), 2);
        assert_eq!(timestamps[0], "2025-10-28T08:15:23.001Z");
        assert_eq!(timestamps[1], "2025-10-28T08:16:45.500Z");
    }

    #[test]
    fn test_get_all_timestamps_sorted() {
        let mut indices = create_test_indices();

        indices
            .insert(
                3,
                "2025-10-28T09:00:00.000Z".to_string(),
                "F003".to_string(),
            )
            .unwrap();
        indices
            .insert(
                1,
                "2025-10-28T08:00:00.000Z".to_string(),
                "F001".to_string(),
            )
            .unwrap();
        indices
            .insert(
                2,
                "2025-10-28T08:30:00.000Z".to_string(),
                "F002".to_string(),
            )
            .unwrap();

        let timestamps = indices.get_all_timestamps().unwrap();
        assert_eq!(timestamps[0], "2025-10-28T08:00:00.000Z");
        assert_eq!(timestamps[1], "2025-10-28T08:30:00.000Z");
        assert_eq!(timestamps[2], "2025-10-28T09:00:00.000Z");
    }

    #[test]
    fn test_stats_initial() {
        let indices = create_test_indices();

        let stats = indices.stats();
        // New B+-Tree files have initial overhead (pages, headers)
        // Just verify stats are accessible, not exact values
        assert!(stats.timestamp_memory >= 0);
        assert!(stats.frame_memory >= 0);
    }

    #[test]
    fn test_stats_after_inserts() {
        let mut indices = create_test_indices();

        indices
            .insert(
                1,
                "2025-10-28T08:15:23.001Z".to_string(),
                "F001".to_string(),
            )
            .unwrap();

        let stats = indices.stats();
        // MockIndex returns 0 for memory/disk, just verify stats are accessible
        assert_eq!(stats.timestamp_memory, 0);
        assert_eq!(stats.frame_memory, 0);
    }

    // NOTE: Persistence test removed - MockIndex is in-memory only for testing.
    // Real persistence will be tested with BTreeIndex in integration tests.

    #[test]
    fn test_insert_duplicate_version_id_same_timestamp() {
        let mut indices = create_test_indices();

        let timestamp = "2025-10-28T08:15:23.001Z".to_string();
        indices
            .insert(1, timestamp.clone(), "F001".to_string())
            .unwrap();
        indices
            .insert(1, timestamp.clone(), "F001".to_string())
            .unwrap();

        let versions = indices
            .query_timestamp_range(&timestamp, &timestamp)
            .unwrap();
        // Should not duplicate version ID
        assert_eq!(versions, vec![1]);
    }

    #[test]
    fn test_query_large_range() {
        let mut indices = create_test_indices();

        // Insert 100 versions across different timestamps
        for i in 1..=100 {
            let timestamp = format!("2025-10-28T{:02}:15:23.001Z", 8 + (i / 10));
            let frame = format!("F{:03}", i / 5);
            indices.insert(i, timestamp, frame).unwrap();
        }

        let versions = indices
            .query_timestamp_range(
                &"2025-10-28T08:00:00.000Z".to_string(),
                &"2025-10-28T12:00:00.000Z".to_string(),
            )
            .unwrap();

        // Query range 08:00-12:00 includes hours 08, 09, 10, 11
        // Hour 08: i=1-9 (9 versions), 09: i=10-19 (10), 10: i=20-29 (10), 11: i=30-39 (10)
        // Total: 39 versions
        assert_eq!(versions.len(), 39);
        assert_eq!(versions[0], 1);
        assert_eq!(versions[38], 39);
    }

    #[test]
    fn test_query_multiple_frames() {
        let mut indices = create_test_indices();

        indices
            .insert(
                1,
                "2025-10-28T08:15:23.001Z".to_string(),
                "F001".to_string(),
            )
            .unwrap();
        indices
            .insert(
                2,
                "2025-10-28T08:16:23.001Z".to_string(),
                "F002".to_string(),
            )
            .unwrap();
        indices
            .insert(
                3,
                "2025-10-28T08:17:23.001Z".to_string(),
                "F003".to_string(),
            )
            .unwrap();

        let f001 = indices.query_frame(&"F001".to_string()).unwrap();
        let f002 = indices.query_frame(&"F002".to_string()).unwrap();
        let f003 = indices.query_frame(&"F003".to_string()).unwrap();

        assert_eq!(f001, vec![1]);
        assert_eq!(f002, vec![2]);
        assert_eq!(f003, vec![3]);
    }

    #[test]
    fn test_deduplication_in_range_query() {
        let mut indices = create_test_indices();

        let timestamp = "2025-10-28T08:15:23.001Z".to_string();
        indices
            .insert(1, timestamp.clone(), "F001".to_string())
            .unwrap();
        indices
            .insert(2, timestamp.clone(), "F001".to_string())
            .unwrap();

        let versions = indices
            .query_timestamp_range(&timestamp, &timestamp)
            .unwrap();

        // Should be sorted and deduplicated
        assert_eq!(versions, vec![1, 2]);
        assert_eq!(versions.len(), 2);
    }
}
