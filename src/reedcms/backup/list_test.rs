// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::backup::{create_backup, list_backups};
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
    fn test_list_backups_empty() {
        let test_file = create_test_file("test\n");

        let result = list_backups(&test_file);
        assert!(result.is_ok());

        let backups = result.unwrap();
        assert_eq!(backups.len(), 0);

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_list_backups_single() {
        let test_file = create_test_file("test\n");

        create_backup(&test_file).unwrap();

        let backups = list_backups(&test_file).unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].original_name, "test.csv");
        assert!(backups[0].size > 0);

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_list_backups_multiple() {
        let test_file = create_test_file("test\n");

        create_backup(&test_file).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        create_backup(&test_file).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        create_backup(&test_file).unwrap();

        let backups = list_backups(&test_file).unwrap();
        assert_eq!(backups.len(), 3);

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_list_backups_sorted_newest_first() {
        let test_file = create_test_file("test\n");

        create_backup(&test_file).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        create_backup(&test_file).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        create_backup(&test_file).unwrap();

        let backups = list_backups(&test_file).unwrap();

        // Should be sorted newest first (descending timestamp)
        assert!(backups[0].timestamp > backups[1].timestamp);
        assert!(backups[1].timestamp > backups[2].timestamp);

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_list_backups_filters_other_files() {
        let test_file = create_test_file("test\n");
        let backup_dir = test_file.parent().unwrap().join("backups");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create a backup
        create_backup(&test_file).unwrap();

        // Add unrelated files
        fs::write(backup_dir.join("other.csv.20250101_120000.xz"), "fake").unwrap();
        fs::write(backup_dir.join("test.csv.txt"), "not a backup").unwrap();
        fs::write(backup_dir.join("random.txt"), "random").unwrap();

        let backups = list_backups(&test_file).unwrap();

        // Should only list backups for test.csv
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].original_name, "test.csv");

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_list_backups_backup_info_fields() {
        let test_file = create_test_file("test content\n");

        let backup_path = create_backup(&test_file).unwrap();

        let backups = list_backups(&test_file).unwrap();
        assert_eq!(backups.len(), 1);

        let info = &backups[0];
        assert_eq!(info.path, backup_path);
        assert_eq!(info.original_name, "test.csv");
        assert_eq!(info.timestamp.len(), 15); // YYYYMMDD_HHMMSS
        assert!(info.size > 0);

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_list_backups_performance() {
        let test_file = create_test_file("test\n");
        let backup_dir = test_file.parent().unwrap().join("backups");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create 10 fake backups with unique timestamps
        for i in 0..10 {
            let timestamp = format!("20250101_{:02}0000", i);
            let filename = format!("test.csv.{}.xz", timestamp);
            fs::write(backup_dir.join(&filename), "fake backup").unwrap();
        }

        let start = std::time::Instant::now();
        let backups = list_backups(&test_file).unwrap();
        let duration = start.elapsed();

        assert_eq!(backups.len(), 10);

        // Should complete in < 10ms
        assert!(
            duration.as_millis() < 10,
            "Listing too slow: {:?}",
            duration
        );

        cleanup_test_dir(&test_file);
    }
}
