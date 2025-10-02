// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::backup::{cleanup_old_backups, create_backup, list_backups};
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
    fn test_cleanup_old_backups_none_to_delete() {
        let test_file = create_test_file("test\n");

        // Create 3 backups (well under 32)
        for _ in 0..3 {
            create_backup(&test_file).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        let deleted = cleanup_old_backups(&test_file).unwrap();
        assert_eq!(deleted, 0);

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_cleanup_old_backups_empty_directory() {
        let test_file = create_test_file("test\n");

        let deleted = cleanup_old_backups(&test_file).unwrap();
        assert_eq!(deleted, 0);

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_cleanup_old_backups_basic_functionality() {
        let test_file = create_test_file("test\n");
        let backup_dir = test_file.parent().unwrap().join("backups");
        fs::create_dir_all(&backup_dir).unwrap();

        // Manually create 35 fake backup files with different timestamps
        for i in 0..35 {
            let timestamp = format!("2025010{:1}_{:02}0000", i / 10, i % 10);
            let filename = format!("test.csv.{}.xz", timestamp);
            fs::write(backup_dir.join(&filename), "fake backup").unwrap();
        }

        let backups_before = list_backups(&test_file).unwrap();
        assert_eq!(backups_before.len(), 35);

        let deleted = cleanup_old_backups(&test_file).unwrap();
        assert_eq!(deleted, 3); // 35 - 32 = 3

        let backups_after = list_backups(&test_file).unwrap();
        assert_eq!(backups_after.len(), 32);

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_cleanup_old_backups_keeps_newest() {
        let test_file = create_test_file("test\n");
        let backup_dir = test_file.parent().unwrap().join("backups");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create 40 fake backups
        for i in 0..40 {
            let timestamp = format!("2025010{:1}_{:02}0000", i / 10, i % 10);
            let filename = format!("test.csv.{}.xz", timestamp);
            fs::write(backup_dir.join(&filename), "fake backup").unwrap();
        }

        cleanup_old_backups(&test_file).unwrap();

        let backups = list_backups(&test_file).unwrap();
        assert_eq!(backups.len(), 32);

        // Verify newest are kept (higher timestamp values)
        assert!(backups[0].timestamp >= "20250103".to_string());

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_cleanup_old_backups_performance() {
        let test_file = create_test_file("test\n");
        let backup_dir = test_file.parent().unwrap().join("backups");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create 50 fake backups
        for i in 0..50 {
            let timestamp = format!("2025010{:1}_{:02}0000", i / 10, i % 10);
            let filename = format!("test.csv.{}.xz", timestamp);
            fs::write(backup_dir.join(&filename), "fake backup").unwrap();
        }

        let start = std::time::Instant::now();
        cleanup_old_backups(&test_file).unwrap();
        let duration = start.elapsed();

        // Should complete in < 50ms
        assert!(
            duration.as_millis() < 50,
            "Cleanup too slow: {:?}",
            duration
        );

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_cleanup_old_backups_exactly_32() {
        let test_file = create_test_file("test\n");
        let backup_dir = test_file.parent().unwrap().join("backups");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create exactly 32 backups
        for i in 0..32 {
            let timestamp = format!("20250101_{:02}00{:02}", i / 60, i % 60);
            let filename = format!("test.csv.{}.xz", timestamp);
            fs::write(backup_dir.join(&filename), "fake backup").unwrap();
        }

        let deleted = cleanup_old_backups(&test_file).unwrap();
        assert_eq!(deleted, 0);

        let backups = list_backups(&test_file).unwrap();
        assert_eq!(backups.len(), 32);

        cleanup_test_dir(&test_file);
    }
}
