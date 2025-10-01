// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::backup::create_backup;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_file(content: &str) -> PathBuf {
        let dir = PathBuf::from(format!("/tmp/reed_backup_test_{}", rand::random::<u32>()));
        fs::create_dir_all(&dir).unwrap();

        let file_path = dir.join("test.csv");
        fs::write(&file_path, content).unwrap();
        file_path
    }

    fn cleanup_test_dir(path: &PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn test_create_backup_basic() {
        let test_file = create_test_file("key1|value1|desc1\nkey2|value2|desc2\n");

        let result = create_backup(&test_file);
        assert!(result.is_ok());

        let backup_path = result.unwrap();
        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains(".xz"));
        assert!(backup_path.to_string_lossy().contains("test.csv"));

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_create_backup_creates_backup_dir() {
        let test_file = create_test_file("test content\n");
        let backup_dir = test_file.parent().unwrap().join("backups");

        assert!(!backup_dir.exists());

        create_backup(&test_file).unwrap();

        assert!(backup_dir.exists());
        assert!(backup_dir.is_dir());

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_create_backup_compression_ratio() {
        // Create a file with repetitive content (compresses well)
        let mut content = String::new();
        for i in 0..1000 {
            content.push_str(&format!("key{}|value{}|description{}\n", i, i, i));
        }

        let test_file = create_test_file(&content);
        let original_size = fs::metadata(&test_file).unwrap().len();

        let backup_path = create_backup(&test_file).unwrap();
        let compressed_size = fs::metadata(&backup_path).unwrap().len();

        // XZ should compress at least 3x for this repetitive data
        let compression_ratio = original_size as f64 / compressed_size as f64;
        assert!(
            compression_ratio > 3.0,
            "Compression ratio too low: {:.2}x",
            compression_ratio
        );

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_create_backup_timestamp_format() {
        let test_file = create_test_file("test\n");

        let backup_path = create_backup(&test_file).unwrap();
        let filename = backup_path.file_name().unwrap().to_string_lossy();

        // Format: test.csv.YYYYMMDD_HHMMSS.xz
        assert!(filename.starts_with("test.csv."));
        assert!(filename.ends_with(".xz"));

        // Extract timestamp part
        let parts: Vec<&str> = filename.split('.').collect();
        assert!(parts.len() >= 4); // test, csv, timestamp, xz

        let timestamp = parts[2];
        assert_eq!(timestamp.len(), 15); // YYYYMMDD_HHMMSS
        assert!(timestamp.contains('_'));

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_create_backup_multiple_backups() {
        let test_file = create_test_file("test\n");

        // Create first backup
        let backup1 = create_backup(&test_file).unwrap();

        // Wait a moment to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Create second backup
        let backup2 = create_backup(&test_file).unwrap();

        assert_ne!(backup1, backup2);
        assert!(backup1.exists());
        assert!(backup2.exists());

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_create_backup_empty_file() {
        let test_file = create_test_file("");

        let result = create_backup(&test_file);
        assert!(result.is_ok());

        let backup_path = result.unwrap();
        assert!(backup_path.exists());

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_create_backup_large_file() {
        // Create 10KB file
        let mut content = String::new();
        for i in 0..500 {
            content.push_str(&format!("key{}|value{}|description{}\n", i, i, i));
        }

        let test_file = create_test_file(&content);

        let start = std::time::Instant::now();
        let result = create_backup(&test_file);
        let duration = start.elapsed();

        assert!(result.is_ok());

        // Should complete in < 100ms
        assert!(
            duration.as_millis() < 100,
            "Backup too slow: {:?}",
            duration
        );

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_create_backup_file_not_found() {
        let result = create_backup("/tmp/nonexistent_reed_file_12345.csv");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_backup_no_temp_file_left() {
        let test_file = create_test_file("test\n");

        create_backup(&test_file).unwrap();

        let backup_dir = test_file.parent().unwrap().join("backups");
        let entries: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        // Should only have one file (the backup, no .tmp file)
        assert_eq!(entries.len(), 1);
        assert!(entries[0].path().to_string_lossy().ends_with(".xz"));

        cleanup_test_dir(&test_file);
    }
}
