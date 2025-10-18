// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::log::encoder::encode_log_entry;
    use crate::log::types::LogEntry;
    use crate::log::validator::{append_entry, validate_and_truncate_log, validate_log};
    use crate::registry;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test() -> (TempDir, std::path::PathBuf, TempDir) {
        // Setup registry in separate temp dir
        let registry_dir = TempDir::new().unwrap();
        let base_path = registry_dir.path().to_path_buf();
        registry::init_registry(&base_path).unwrap();
        registry::set_base_path(base_path);
        registry::reload_dictionaries().unwrap();

        // Setup log dir
        let log_dir = TempDir::new().unwrap();
        let log_path = log_dir.path().join("version.log");

        (log_dir, log_path, registry_dir)
    }

    fn create_test_entry(timestamp: u64) -> LogEntry {
        LogEntry::new(
            timestamp,
            "update".to_string(),
            "admin".to_string(),
            timestamp - 100,
            2500,
            15,
            format!("sha256:hash{}", timestamp),
            None,
        )
    }

    #[test]
    fn test_validate_empty_log() {
        let (_log_dir, log_path, _registry_dir) = setup_test();
        let report = validate_log(&log_path).unwrap();

        assert_eq!(report.total_entries, 0);
        assert_eq!(report.valid_entries, 0);
        assert_eq!(report.corrupted_count, 0);
        assert!(report.is_healthy());
    }

    #[test]
    fn test_validate_healthy_log() {
        let (_log_dir, log_path, _registry_dir) = setup_test();

        let entry1 = create_test_entry(1736860900);
        let entry2 = create_test_entry(1736861000);
        let entry3 = create_test_entry(1736861100);

        let encoded1 = encode_log_entry(&entry1).unwrap();
        let encoded2 = encode_log_entry(&entry2).unwrap();
        let encoded3 = encode_log_entry(&entry3).unwrap();

        let content = format!("{}\n{}\n{}\n", encoded1, encoded2, encoded3);
        fs::write(&log_path, content).unwrap();

        let report = validate_log(&log_path).unwrap();

        assert_eq!(report.total_entries, 3);
        assert_eq!(report.valid_entries, 3);
        assert_eq!(report.corrupted_count, 0);
        assert!(report.is_healthy());
    }

    #[test]
    fn test_validate_corrupted_log() {
        let (_log_dir, log_path, _registry_dir) = setup_test();

        let entry1 = create_test_entry(1736860900);
        let entry2 = create_test_entry(1736861000);

        let encoded1 = encode_log_entry(&entry1).unwrap();
        let encoded2 = encode_log_entry(&entry2).unwrap();
        let corrupted = "REED|00000058|1736861100|2|1|1736861000|2500|15|sha256:abc|n/a|BADCRC32";

        let content = format!("{}\n{}\n{}\n", encoded1, encoded2, corrupted);
        fs::write(&log_path, content).unwrap();

        let report = validate_log(&log_path).unwrap();

        assert_eq!(report.total_entries, 3);
        assert_eq!(report.valid_entries, 2);
        assert_eq!(report.corrupted_count, 1);
        assert!(!report.is_healthy());
        assert_eq!(report.corrupted_lines, vec![3]);
    }

    #[test]
    fn test_validate_and_truncate_healthy_log() {
        let (_log_dir, log_path, _registry_dir) = setup_test();

        let entry1 = create_test_entry(1736860900);
        let entry2 = create_test_entry(1736861000);
        let entry3 = create_test_entry(1736861100);

        let encoded1 = encode_log_entry(&entry1).unwrap();
        let encoded2 = encode_log_entry(&entry2).unwrap();
        let encoded3 = encode_log_entry(&entry3).unwrap();

        let content = format!("{}\n{}\n{}\n", encoded1, encoded2, encoded3);
        fs::write(&log_path, content).unwrap();

        let report = validate_and_truncate_log(&log_path).unwrap();

        assert_eq!(report.total_entries, 3);
        assert_eq!(report.valid_entries, 3);
        assert_eq!(report.corrupted_count, 0);
        assert!(!report.truncated);
    }

    #[test]
    fn test_validate_and_truncate_corrupted_log() {
        let (_log_dir, log_path, _registry_dir) = setup_test();

        let entry1 = create_test_entry(1736860900);
        let entry2 = create_test_entry(1736861000);

        let encoded1 = encode_log_entry(&entry1).unwrap();
        let encoded2 = encode_log_entry(&entry2).unwrap();
        let corrupted = "REED|00000058|1736861100|2|1|1736861000|2500|15|sha256:abc|n/a|BADCRC32";

        let content = format!("{}\n{}\n{}\n", encoded1, encoded2, corrupted);
        fs::write(&log_path, content).unwrap();

        let report = validate_and_truncate_log(&log_path).unwrap();

        assert_eq!(report.total_entries, 2);
        assert_eq!(report.valid_entries, 2);
        assert_eq!(report.corrupted_count, 1);
        assert!(report.truncated);

        let new_content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = new_content.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_truncate_corruption_in_middle() {
        let (_log_dir, log_path, _registry_dir) = setup_test();

        let entry1 = create_test_entry(1736860900);
        let entry3 = create_test_entry(1736861100);

        let encoded1 = encode_log_entry(&entry1).unwrap();
        let corrupted = "REED|00000058|1736861000|2|1|1736860900|2500|15|sha256:abc|n/a|BADCRC32";
        let encoded3 = encode_log_entry(&entry3).unwrap();

        let content = format!("{}\n{}\n{}\n", encoded1, corrupted, encoded3);
        fs::write(&log_path, content).unwrap();

        let report = validate_and_truncate_log(&log_path).unwrap();

        assert_eq!(report.total_entries, 1);
        assert_eq!(report.valid_entries, 1);
        assert!(report.truncated);

        let new_content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = new_content.lines().collect();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_append_entry() {
        let (_log_dir, log_path, _registry_dir) = setup_test();

        let entry1 = create_test_entry(1736860900);
        let entry2 = create_test_entry(1736861000);

        let encoded1 = encode_log_entry(&entry1).unwrap();
        let encoded2 = encode_log_entry(&entry2).unwrap();

        append_entry(&log_path, &encoded1).unwrap();
        append_entry(&log_path, &encoded2).unwrap();

        let content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_append_to_new_file() {
        let (_log_dir, log_path, _registry_dir) = setup_test();

        let entry = create_test_entry(1736860900);
        let encoded = encode_log_entry(&entry).unwrap();

        append_entry(&log_path, &encoded).unwrap();

        assert!(log_path.exists());
        let content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_validate_with_empty_lines() {
        let (_log_dir, log_path, _registry_dir) = setup_test();

        let entry1 = create_test_entry(1736860900);
        let entry2 = create_test_entry(1736861000);

        let encoded1 = encode_log_entry(&entry1).unwrap();
        let encoded2 = encode_log_entry(&entry2).unwrap();

        let content = format!("{}\n\n{}\n\n", encoded1, encoded2);
        fs::write(&log_path, content).unwrap();

        let report = validate_log(&log_path).unwrap();

        assert_eq!(report.total_entries, 2);
        assert_eq!(report.valid_entries, 2);
        assert!(report.is_healthy());
    }

    #[test]
    fn test_truncate_all_corrupted() {
        let (_log_dir, log_path, _registry_dir) = setup_test();

        let corrupted1 = "REED|00000058|1736860900|2|1|1736860800|2500|15|sha256:abc|n/a|BADCRC32";
        let corrupted2 = "REED|00000058|1736861000|2|1|1736860900|2500|15|sha256:def|n/a|BADCRC32";

        let content = format!("{}\n{}\n", corrupted1, corrupted2);
        fs::write(&log_path, content).unwrap();

        let report = validate_and_truncate_log(&log_path).unwrap();

        assert_eq!(report.total_entries, 0);
        assert_eq!(report.valid_entries, 0);
        assert!(report.truncated);

        if log_path.exists() {
            let content = fs::read_to_string(&log_path).unwrap_or_default();
            assert!(content.is_empty() || content.trim().is_empty());
        }
    }
}
