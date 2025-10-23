// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for file locking functionality.

#[cfg(test)]
mod tests {
    use crate::concurrent::lock::{acquire_lock, is_locked, wait_for_unlock};
    use crate::error::ReedError;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_acquire_lock_success() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let lock = acquire_lock(base_path, "test_table", Duration::from_secs(5)).unwrap();
        assert!(is_locked(base_path, "test_table").unwrap());

        drop(lock);
        assert!(!is_locked(base_path, "test_table").unwrap());
    }

    #[test]
    fn test_acquire_lock_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let _lock1 = acquire_lock(base_path, "test_table", Duration::from_secs(5)).unwrap();

        let result = acquire_lock(base_path, "test_table", Duration::from_millis(500));
        assert!(matches!(result, Err(ReedError::LockTimeout { .. })));
    }

    #[test]
    fn test_is_locked_false() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        assert!(!is_locked(base_path, "test_table").unwrap());
    }

    #[test]
    fn test_is_locked_true() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let _lock = acquire_lock(base_path, "test_table", Duration::from_secs(5)).unwrap();
        assert!(is_locked(base_path, "test_table").unwrap());
    }

    #[test]
    fn test_wait_for_unlock_success() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        let lock = acquire_lock(&base_path, "test_table", Duration::from_secs(5)).unwrap();

        let base_path_clone = base_path.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            drop(lock);
        });

        wait_for_unlock(&base_path, "test_table", Duration::from_secs(5)).unwrap();
        assert!(!is_locked(&base_path, "test_table").unwrap());
    }

    #[test]
    fn test_wait_for_unlock_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let _lock = acquire_lock(base_path, "test_table", Duration::from_secs(5)).unwrap();

        let result = wait_for_unlock(base_path, "test_table", Duration::from_millis(200));
        assert!(matches!(result, Err(ReedError::LockTimeout { .. })));
    }

    #[test]
    fn test_multiple_locks_different_tables() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let _lock1 = acquire_lock(base_path, "table1", Duration::from_secs(5)).unwrap();
        let _lock2 = acquire_lock(base_path, "table2", Duration::from_secs(5)).unwrap();

        assert!(is_locked(base_path, "table1").unwrap());
        assert!(is_locked(base_path, "table2").unwrap());
    }

    #[test]
    fn test_lock_raii_drop() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        {
            let _lock = acquire_lock(base_path, "test_table", Duration::from_secs(5)).unwrap();
            assert!(is_locked(base_path, "test_table").unwrap());
        } // Lock dropped here

        assert!(!is_locked(base_path, "test_table").unwrap());
    }

    #[test]
    fn test_lock_contention() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        let _lock1 = acquire_lock(&base_path, "test_table", Duration::from_secs(5)).unwrap();

        let base_path_clone = base_path.clone();
        let handle = std::thread::spawn(move || {
            // This should timeout
            acquire_lock(&base_path_clone, "test_table", Duration::from_millis(300))
        });

        let result = handle.join().unwrap();
        assert!(matches!(result, Err(ReedError::LockTimeout { .. })));
    }

    #[test]
    fn test_is_locked_nonexistent_table() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Should return false for non-existent table
        assert!(!is_locked(base_path, "nonexistent").unwrap());
    }
}
