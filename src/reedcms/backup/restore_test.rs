// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::backup::{create_backup, restore_backup};
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
    fn test_restore_backup_basic() {
        let original_content = "key1|value1|desc1\nkey2|value2|desc2\n";
        let test_file = create_test_file(original_content);

        // Create backup
        let backup_path = create_backup(&test_file).unwrap();

        // Modify original file
        fs::write(&test_file, "modified content\n").unwrap();

        // Restore from backup
        let result = restore_backup(&backup_path, &test_file);
        assert!(result.is_ok());

        // Verify content is restored
        let restored_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(restored_content, original_content);

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_restore_backup_to_different_location() {
        let original_content = "original data\n";
        let test_file = create_test_file(original_content);

        // Create backup
        let backup_path = create_backup(&test_file).unwrap();

        // Restore to different location
        let restore_target = test_file.parent().unwrap().join("restored.csv");
        restore_backup(&backup_path, &restore_target).unwrap();

        // Verify restored file
        assert!(restore_target.exists());
        let restored_content = fs::read_to_string(&restore_target).unwrap();
        assert_eq!(restored_content, original_content);

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_restore_backup_overwrites_existing() {
        let original_content = "original\n";
        let test_file = create_test_file(original_content);

        let backup_path = create_backup(&test_file).unwrap();

        // Create file with different content
        fs::write(&test_file, "different content\n").unwrap();

        // Restore should overwrite
        restore_backup(&backup_path, &test_file).unwrap();

        let restored_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(restored_content, original_content);

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_restore_backup_empty_file() {
        let test_file = create_test_file("");

        let backup_path = create_backup(&test_file).unwrap();
        fs::write(&test_file, "modified\n").unwrap();

        restore_backup(&backup_path, &test_file).unwrap();

        let restored_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(restored_content, "");

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_restore_backup_large_file() {
        // Create 10KB file
        let mut content = String::new();
        for i in 0..500 {
            content.push_str(&format!("key{}|value{}|description{}\n", i, i, i));
        }

        let test_file = create_test_file(&content);
        let backup_path = create_backup(&test_file).unwrap();

        fs::write(&test_file, "modified\n").unwrap();

        let start = std::time::Instant::now();
        restore_backup(&backup_path, &test_file).unwrap();
        let duration = start.elapsed();

        let restored_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(restored_content, content);

        // Should complete in < 100ms
        assert!(
            duration.as_millis() < 100,
            "Restore too slow: {:?}",
            duration
        );

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_restore_backup_file_not_found() {
        let test_file = create_test_file("test\n");

        let result = restore_backup("/tmp/nonexistent_backup.xz", &test_file);
        assert!(result.is_err());

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_restore_backup_round_trip() {
        let original_content = "key1|value1|desc1\nkey2|value2|desc2\nkey3|value3|desc3\n";
        let test_file = create_test_file(original_content);

        // Create backup
        let backup_path = create_backup(&test_file).unwrap();

        // Modify original
        fs::write(&test_file, "completely different content\n").unwrap();

        // Restore
        restore_backup(&backup_path, &test_file).unwrap();

        // Verify exact match
        let restored = fs::read_to_string(&test_file).unwrap();
        assert_eq!(restored, original_content);
        assert_eq!(restored.len(), original_content.len());

        cleanup_test_dir(&test_file);
    }

    #[test]
    fn test_restore_backup_no_temp_file_left() {
        let test_file = create_test_file("test\n");
        let backup_path = create_backup(&test_file).unwrap();

        restore_backup(&backup_path, &test_file).unwrap();

        // Check no .tmp files left
        let parent = test_file.parent().unwrap();
        let entries: Vec<_> = fs::read_dir(parent)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().to_string_lossy().contains(".tmp"))
            .collect();

        assert_eq!(entries.len(), 0);

        cleanup_test_dir(&test_file);
    }
}
