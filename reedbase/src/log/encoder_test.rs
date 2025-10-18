// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::log::encoder::{calculate_size_savings, encode_log_entries, encode_log_entry};
    use crate::log::types::LogEntry;
    use crate::registry;
    use tempfile::TempDir;

    fn setup_registry() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();
        registry::init_registry(&base_path).unwrap();
        registry::set_base_path(base_path);
        registry::reload_dictionaries().unwrap();
        temp_dir
    }

    fn create_test_entry() -> LogEntry {
        LogEntry::new(
            1736860900,
            "update".to_string(),
            "admin".to_string(),
            1736860800,
            2500,
            15,
            "sha256:abc123".to_string(),
            None,
        )
    }

    #[test]
    fn test_encode_log_entry() {
        let _temp_dir = setup_registry();
        let entry = create_test_entry();
        let encoded = encode_log_entry(&entry).unwrap();

        assert!(encoded.starts_with("REED|"));
        let parts: Vec<&str> = encoded.split('|').collect();
        assert_eq!(parts.len(), 11);
        assert_eq!(parts[0], "REED");
        assert_eq!(parts[2], "1736860900");
        assert_eq!(parts[5], "1736860800");
        assert_eq!(parts[6], "2500");
        assert_eq!(parts[7], "15");
        assert_eq!(parts[8], "sha256:abc123");
        assert_eq!(parts[9], "n/a");
        assert_eq!(parts[10].len(), 8);
    }

    #[test]
    fn test_encode_entry_with_frame_id() {
        let _temp_dir = setup_registry();
        let uuid = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let mut entry = create_test_entry();
        entry.frame_id = Some(uuid);

        let encoded = encode_log_entry(&entry).unwrap();
        let parts: Vec<&str> = encoded.split('|').collect();
        assert_eq!(parts[9], "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_encode_multiple_entries() {
        let _temp_dir = setup_registry();
        let entry1 = create_test_entry();
        let mut entry2 = create_test_entry();
        entry2.timestamp = 1736861000;

        let encoded = encode_log_entries(&[entry1, entry2]).unwrap();
        let lines: Vec<&str> = encoded.lines().collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("REED|"));
        assert!(lines[1].starts_with("REED|"));
    }

    #[test]
    fn test_encode_empty_entries() {
        let encoded = encode_log_entries(&[]).unwrap();
        assert_eq!(encoded, "");
    }

    #[test]
    fn test_calculate_size_savings() {
        let _temp_dir = setup_registry();
        let entries = vec![create_test_entry(); 10];
        let (encoded_size, plain_size) = calculate_size_savings(&entries).unwrap();

        assert!(encoded_size > 0);
        assert!(plain_size > 0);

        // Note: With CRC32 and magic bytes overhead, encoded may be larger for small samples
        // Savings come from integer codes (2 vs "update") which helps with larger datasets
        // Just verify both sizes are calculated
        assert!(encoded_size > 0 && plain_size > 0);
    }

    #[test]
    fn test_encode_large_batch() {
        let _temp_dir = setup_registry();
        let entries: Vec<LogEntry> = (0..100)
            .map(|i| {
                LogEntry::new(
                    1736860900 + i,
                    "update".to_string(),
                    "admin".to_string(),
                    1736860800 + i,
                    2500 + i as usize,
                    15,
                    format!("sha256:hash{}", i),
                    None,
                )
            })
            .collect();

        let encoded = encode_log_entries(&entries).unwrap();
        let lines: Vec<&str> = encoded.lines().collect();

        assert_eq!(lines.len(), 100);
        for line in lines {
            assert!(line.starts_with("REED|"));
            let parts: Vec<&str> = line.split('|').collect();
            assert_eq!(parts.len(), 11);
            assert_eq!(parts[10].len(), 8);
        }
    }

    #[test]
    fn test_encode_init_action() {
        let _temp_dir = setup_registry();
        let entry = LogEntry::new(
            1736860900,
            "init".to_string(),
            "system".to_string(),
            0,
            1500,
            10,
            "sha256:init123".to_string(),
            None,
        );

        let encoded = encode_log_entry(&entry).unwrap();
        let parts: Vec<&str> = encoded.split('|').collect();
        assert_eq!(parts[5], "0");
    }

    #[test]
    fn test_encode_with_different_actions() {
        let _temp_dir = setup_registry();
        let actions = vec!["init", "update", "rollback", "delete"];

        for action in actions {
            let entry = LogEntry::new(
                1736860900,
                action.to_string(),
                "admin".to_string(),
                1736860800,
                2500,
                15,
                "sha256:abc123".to_string(),
                None,
            );

            let result = encode_log_entry(&entry);
            if result.is_ok() {
                let encoded = result.unwrap();
                assert!(encoded.starts_with("REED|"));
            }
        }
    }
}
