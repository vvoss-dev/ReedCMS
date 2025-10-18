// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::log::decoder::{
        decode_log_entries, decode_log_entry, filter_by_action, filter_by_time_range,
        filter_by_user,
    };
    use crate::log::encoder::encode_log_entry;
    use crate::log::types::LogEntry;
    use crate::registry;
    use crate::ReedError;
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
    fn test_decode_log_entry_roundtrip() {
        let _temp_dir = setup_registry();
        let entry = create_test_entry();
        let encoded = encode_log_entry(&entry).unwrap();
        let decoded = decode_log_entry(&encoded).unwrap();

        assert_eq!(decoded.timestamp, entry.timestamp);
        assert_eq!(decoded.action, entry.action);
        assert_eq!(decoded.user, entry.user);
        assert_eq!(decoded.base_version, entry.base_version);
        assert_eq!(decoded.size, entry.size);
        assert_eq!(decoded.rows, entry.rows);
        assert_eq!(decoded.hash, entry.hash);
        assert_eq!(decoded.frame_id, entry.frame_id);
    }

    #[test]
    fn test_decode_entry_with_frame_id() {
        let _temp_dir = setup_registry();
        let uuid = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let mut entry = create_test_entry();
        entry.frame_id = Some(uuid);

        let encoded = encode_log_entry(&entry).unwrap();
        let decoded = decode_log_entry(&encoded).unwrap();
        assert_eq!(decoded.frame_id, Some(uuid));
    }

    #[test]
    fn test_decode_invalid_magic() {
        let _temp_dir = setup_registry();
        let line = "FAKE|00000058|1736860900|2|1|1736860800|2500|15|sha256:abc|n/a|A1B2C3D4";
        let result = decode_log_entry(line);
        assert!(matches!(result, Err(ReedError::CorruptedLogEntry { .. })));
    }

    #[test]
    fn test_decode_invalid_crc32() {
        let _temp_dir = setup_registry();
        let entry = create_test_entry();
        let mut encoded = encode_log_entry(&entry).unwrap();
        let len = encoded.len();
        encoded.replace_range(len - 8.., "FFFFFFFF");

        let result = decode_log_entry(&encoded);
        assert!(matches!(result, Err(ReedError::CorruptedLogEntry { .. })));
    }

    #[test]
    fn test_decode_invalid_length() {
        let line = "REED|99999999|1736860900|2|1|1736860800|2500|15|sha256:abc|n/a|A1B2C3D4";
        let result = decode_log_entry(line);
        assert!(matches!(result, Err(ReedError::CorruptedLogEntry { .. })));
    }

    #[test]
    fn test_decode_invalid_field_count() {
        let line = "REED|00000020|1736860900|2|1";
        let result = decode_log_entry(line);
        assert!(matches!(result, Err(ReedError::ParseError { .. })));
    }

    #[test]
    fn test_decode_invalid_timestamp() {
        let line = "REED|00000058|invalid|2|1|1736860800|2500|15|sha256:abc|n/a|A1B2C3D4";
        let result = decode_log_entry(line);
        // Will fail on length check first (line is shorter than 58), or ParseError if it passes
        assert!(matches!(
            result,
            Err(ReedError::ParseError { .. }) | Err(ReedError::CorruptedLogEntry { .. })
        ));
    }

    #[test]
    fn test_decode_old_format_without_crc32() {
        let _temp_dir = setup_registry();
        let line = "1736860900|2|1|1736860800|2500|15|sha256:abc123|n/a";
        let result = decode_log_entry(line);
        if let Err(ref e) = result {
            eprintln!("Decode error: {:?}", e);
        }
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.timestamp, 1736860900);
        assert_eq!(entry.size, 2500);
    }

    #[test]
    fn test_decode_old_format_without_frame_id() {
        let _temp_dir = setup_registry();
        let line = "1736860900|2|1|1736860800|2500|15|sha256:abc123";
        let result = decode_log_entry(line);
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.timestamp, 1736860900);
        assert_eq!(entry.frame_id, None);
    }

    #[test]
    fn test_decode_multiple_entries() {
        let _temp_dir = setup_registry();
        let entry1 = create_test_entry();
        let mut entry2 = create_test_entry();
        entry2.timestamp = 1736861000;

        let encoded1 = encode_log_entry(&entry1).unwrap();
        let encoded2 = encode_log_entry(&entry2).unwrap();
        let content = format!("{}\n{}", encoded1, encoded2);

        let entries = decode_log_entries(&content).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].timestamp, 1736860900);
        assert_eq!(entries[1].timestamp, 1736861000);
    }

    #[test]
    fn test_decode_empty_lines() {
        let _temp_dir = setup_registry();
        let entry = create_test_entry();
        let encoded = encode_log_entry(&entry).unwrap();
        let content = format!("{}\n\n{}\n", encoded, encoded);

        let entries = decode_log_entries(&content).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_filter_by_action() {
        let entry1 = LogEntry::new(
            1736860900,
            "update".to_string(),
            "admin".to_string(),
            0,
            0,
            0,
            "".to_string(),
            None,
        );

        let entry2 = LogEntry::new(
            1736861000,
            "delete".to_string(),
            "admin".to_string(),
            0,
            0,
            0,
            "".to_string(),
            None,
        );

        let entry3 = LogEntry::new(
            1736861100,
            "update".to_string(),
            "editor".to_string(),
            0,
            0,
            0,
            "".to_string(),
            None,
        );

        let entries = vec![entry1, entry2, entry3];
        let updates = filter_by_action(&entries, "update");

        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].action, "update");
        assert_eq!(updates[1].action, "update");
    }

    #[test]
    fn test_filter_by_user() {
        let entry1 = LogEntry::new(
            1736860900,
            "update".to_string(),
            "admin".to_string(),
            0,
            0,
            0,
            "".to_string(),
            None,
        );

        let entry2 = LogEntry::new(
            1736861000,
            "update".to_string(),
            "editor".to_string(),
            0,
            0,
            0,
            "".to_string(),
            None,
        );

        let entry3 = LogEntry::new(
            1736861100,
            "delete".to_string(),
            "admin".to_string(),
            0,
            0,
            0,
            "".to_string(),
            None,
        );

        let entries = vec![entry1, entry2, entry3];
        let admin_actions = filter_by_user(&entries, "admin");

        assert_eq!(admin_actions.len(), 2);
        assert_eq!(admin_actions[0].user, "admin");
        assert_eq!(admin_actions[1].user, "admin");
    }

    #[test]
    fn test_filter_by_time_range() {
        let entry1 = LogEntry::new(
            1736860900,
            "update".to_string(),
            "admin".to_string(),
            0,
            0,
            0,
            "".to_string(),
            None,
        );

        let entry2 = LogEntry::new(
            1736861000,
            "update".to_string(),
            "admin".to_string(),
            0,
            0,
            0,
            "".to_string(),
            None,
        );

        let entry3 = LogEntry::new(
            1736861100,
            "update".to_string(),
            "admin".to_string(),
            0,
            0,
            0,
            "".to_string(),
            None,
        );

        let entries = vec![entry1, entry2, entry3];
        let filtered = filter_by_time_range(&entries, 1736860950, 1736861050);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].timestamp, 1736861000);
    }

    #[test]
    fn test_filter_empty_results() {
        let _temp_dir = setup_registry();
        let entries = vec![create_test_entry()];

        let by_action = filter_by_action(&entries, "nonexistent");
        assert_eq!(by_action.len(), 0);

        let by_user = filter_by_user(&entries, "nonexistent");
        assert_eq!(by_user.len(), 0);

        let by_time = filter_by_time_range(&entries, 0, 100);
        assert_eq!(by_time.len(), 0);
    }
}
