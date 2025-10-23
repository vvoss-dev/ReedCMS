// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for write queue functionality.

#[cfg(test)]
mod tests {
    use crate::concurrent::queue::{
        count_pending, get_next_pending, queue_write, remove_from_queue,
    };
    use crate::concurrent::types::{CsvRow, PendingWrite, WriteOperation};
    use crate::error::ReedError;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_queue_and_get_write() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let write = PendingWrite {
            rows: vec![CsvRow::new("1", vec!["Alice"])],
            timestamp: 1736860900000000000,
            operation: WriteOperation::Insert,
        };

        let queue_id = queue_write(base_path, "test_table", write.clone()).unwrap();

        let (id, retrieved) = get_next_pending(base_path, "test_table").unwrap().unwrap();
        assert_eq!(id, queue_id);
        assert_eq!(retrieved.timestamp, write.timestamp);
        assert_eq!(retrieved.rows.len(), 1);
        assert_eq!(retrieved.rows[0].key, "1");
    }

    #[test]
    fn test_remove_from_queue() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let write = PendingWrite {
            rows: vec![],
            timestamp: 1736860900000000000,
            operation: WriteOperation::Insert,
        };

        let queue_id = queue_write(base_path, "test_table", write).unwrap();
        assert_eq!(count_pending(base_path, "test_table").unwrap(), 1);

        remove_from_queue(base_path, "test_table", &queue_id).unwrap();
        assert_eq!(count_pending(base_path, "test_table").unwrap(), 0);
    }

    #[test]
    fn test_count_pending_empty() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        assert_eq!(count_pending(base_path, "test_table").unwrap(), 0);
    }

    #[test]
    fn test_count_pending_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let write = PendingWrite {
            rows: vec![],
            timestamp: 1736860900000000000,
            operation: WriteOperation::Insert,
        };

        queue_write(base_path, "test_table", write.clone()).unwrap();
        assert_eq!(count_pending(base_path, "test_table").unwrap(), 1);

        queue_write(base_path, "test_table", write.clone()).unwrap();
        assert_eq!(count_pending(base_path, "test_table").unwrap(), 2);
    }

    #[test]
    fn test_queue_full() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let write = PendingWrite {
            rows: vec![],
            timestamp: 1736860900000000000,
            operation: WriteOperation::Insert,
        };

        // Queue 100 writes (max)
        for _ in 0..100 {
            queue_write(base_path, "test_table", write.clone()).unwrap();
        }

        // 101st write should fail
        let result = queue_write(base_path, "test_table", write);
        assert!(matches!(result, Err(ReedError::QueueFull { .. })));
    }

    #[test]
    fn test_get_next_pending_fifo() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let write1 = PendingWrite {
            rows: vec![],
            timestamp: 1736860900000000000,
            operation: WriteOperation::Insert,
        };

        let write2 = PendingWrite {
            rows: vec![],
            timestamp: 1736861000000000000,
            operation: WriteOperation::Insert,
        };

        queue_write(base_path, "test_table", write1).unwrap();
        thread::sleep(Duration::from_millis(10)); // Ensure different creation times
        queue_write(base_path, "test_table", write2).unwrap();

        let (_, first) = get_next_pending(base_path, "test_table").unwrap().unwrap();
        assert_eq!(first.timestamp, 1736860900000000000); // Older one first
    }

    #[test]
    fn test_get_next_pending_empty() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let result = get_next_pending(base_path, "test_table").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_queue_different_operations() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let insert = PendingWrite {
            rows: vec![CsvRow::new("1", vec!["Alice"])],
            timestamp: 1736860900000000000,
            operation: WriteOperation::Insert,
        };

        let update = PendingWrite {
            rows: vec![CsvRow::new("1", vec!["Bob"])],
            timestamp: 1736861000000000000,
            operation: WriteOperation::Update,
        };

        let delete = PendingWrite {
            rows: vec![CsvRow::new("1", vec![])],
            timestamp: 1736862000000000000,
            operation: WriteOperation::Delete,
        };

        queue_write(base_path, "test_table", insert).unwrap();
        queue_write(base_path, "test_table", update).unwrap();
        queue_write(base_path, "test_table", delete).unwrap();

        assert_eq!(count_pending(base_path, "test_table").unwrap(), 3);
    }

    #[test]
    fn test_queue_multiple_rows() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let write = PendingWrite {
            rows: vec![
                CsvRow::new("1", vec!["Alice", "alice@example.com"]),
                CsvRow::new("2", vec!["Bob", "bob@example.com"]),
                CsvRow::new("3", vec!["Charlie", "charlie@example.com"]),
            ],
            timestamp: 1736860900000000000,
            operation: WriteOperation::Insert,
        };

        queue_write(base_path, "test_table", write.clone()).unwrap();

        let (_, retrieved) = get_next_pending(base_path, "test_table").unwrap().unwrap();
        assert_eq!(retrieved.rows.len(), 3);
        assert_eq!(retrieved.rows[0].key, "1");
        assert_eq!(retrieved.rows[1].key, "2");
        assert_eq!(retrieved.rows[2].key, "3");
    }

    #[test]
    fn test_queue_different_tables() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let write = PendingWrite {
            rows: vec![],
            timestamp: 1736860900000000000,
            operation: WriteOperation::Insert,
        };

        queue_write(base_path, "table1", write.clone()).unwrap();
        queue_write(base_path, "table2", write.clone()).unwrap();

        assert_eq!(count_pending(base_path, "table1").unwrap(), 1);
        assert_eq!(count_pending(base_path, "table2").unwrap(), 1);
    }

    #[test]
    fn test_remove_from_empty_queue() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let result = remove_from_queue(base_path, "test_table", "nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_queue_preserves_order() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Queue multiple writes
        for i in 0..5 {
            let write = PendingWrite {
                rows: vec![],
                timestamp: 1736860900000000000 + (i * 1000),
                operation: WriteOperation::Insert,
            };
            queue_write(base_path, "test_table", write).unwrap();
            thread::sleep(Duration::from_millis(10));
        }

        // Verify FIFO order
        let mut timestamps = Vec::new();
        while let Some((id, write)) = get_next_pending(base_path, "test_table").unwrap() {
            timestamps.push(write.timestamp);
            remove_from_queue(base_path, "test_table", &id).unwrap();
        }

        // Should be in ascending order (oldest first)
        for i in 0..timestamps.len() - 1 {
            assert!(timestamps[i] < timestamps[i + 1]);
        }
    }
}
