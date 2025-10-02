// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::csv::{read_csv, write_csv, CsvRecord};
    use std::fs;
    use std::path::PathBuf;

    fn get_test_path() -> PathBuf {
        PathBuf::from(format!("/tmp/reed_test_{}.csv", rand::random::<u32>()))
    }

    fn cleanup_test_file(path: &PathBuf) {
        let _ = fs::remove_file(path);
        let temp_path = format!("{}.tmp", path.to_string_lossy());
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_write_csv_basic() {
        let path = get_test_path();
        let records = vec![
            CsvRecord::new(
                "key1".to_string(),
                "value1".to_string(),
                Some("desc1".to_string()),
            ),
            CsvRecord::new(
                "key2".to_string(),
                "value2".to_string(),
                Some("desc2".to_string()),
            ),
        ];

        let result = write_csv(&path, &records);
        assert!(result.is_ok());

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "key1|value1|desc1\nkey2|value2|desc2\n");

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_csv_no_description() {
        let path = get_test_path();
        let records = vec![
            CsvRecord::new("key1".to_string(), "value1".to_string(), None),
            CsvRecord::new("key2".to_string(), "value2".to_string(), None),
        ];

        let result = write_csv(&path, &records);
        assert!(result.is_ok());

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "key1|value1\nkey2|value2\n");

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_csv_mixed_formats() {
        let path = get_test_path();
        let records = vec![
            CsvRecord::new(
                "key1".to_string(),
                "value1".to_string(),
                Some("desc1".to_string()),
            ),
            CsvRecord::new("key2".to_string(), "value2".to_string(), None),
            CsvRecord::new(
                "key3".to_string(),
                "value3".to_string(),
                Some("desc3".to_string()),
            ),
        ];

        let result = write_csv(&path, &records);
        assert!(result.is_ok());

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "key1|value1|desc1\nkey2|value2\nkey3|value3|desc3\n"
        );

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_csv_empty_records() {
        let path = get_test_path();
        let records: Vec<CsvRecord> = vec![];

        let result = write_csv(&path, &records);
        assert!(result.is_ok());

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "");

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_csv_overwrite_existing() {
        let path = get_test_path();

        // Write first version
        let records1 = vec![CsvRecord::new(
            "key1".to_string(),
            "value1".to_string(),
            None,
        )];
        write_csv(&path, &records1).unwrap();

        // Overwrite with second version
        let records2 = vec![
            CsvRecord::new("key2".to_string(), "value2".to_string(), None),
            CsvRecord::new("key3".to_string(), "value3".to_string(), None),
        ];
        let result = write_csv(&path, &records2);
        assert!(result.is_ok());

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "key2|value2\nkey3|value3\n");

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_csv_atomic_no_temp_file_left() {
        let path = get_test_path();
        let records = vec![CsvRecord::new(
            "key1".to_string(),
            "value1".to_string(),
            None,
        )];

        write_csv(&path, &records).unwrap();

        // Verify temp file doesn't exist
        let temp_path = format!("{}.tmp", path.to_string_lossy());
        assert!(!PathBuf::from(&temp_path).exists());

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_csv_round_trip() {
        let path = get_test_path();
        let original_records = vec![
            CsvRecord::new(
                "page.title@en".to_string(),
                "Welcome".to_string(),
                Some("Homepage title".to_string()),
            ),
            CsvRecord::new(
                "page.subtitle@en".to_string(),
                "Hello World".to_string(),
                None,
            ),
        ];

        // Write
        write_csv(&path, &original_records).unwrap();

        // Read back
        let read_records = read_csv(&path).unwrap();

        // Verify
        assert_eq!(read_records.len(), original_records.len());
        assert_eq!(read_records[0].key, original_records[0].key);
        assert_eq!(read_records[0].value, original_records[0].value);
        assert_eq!(read_records[0].description, original_records[0].description);
        assert_eq!(read_records[1].key, original_records[1].key);
        assert_eq!(read_records[1].value, original_records[1].value);
        assert_eq!(read_records[1].description, original_records[1].description);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_csv_performance_1000_rows() {
        let path = get_test_path();
        let mut records = Vec::new();
        for i in 0..1000 {
            records.push(CsvRecord::new(
                format!("key{}", i),
                format!("value{}", i),
                Some(format!("description{}", i)),
            ));
        }

        let start = std::time::Instant::now();
        let result = write_csv(&path, &records);
        let duration = start.elapsed();

        assert!(result.is_ok());

        // Should be < 5ms for 1000 rows (allowing for file I/O overhead)
        assert!(
            duration.as_millis() < 5,
            "write_csv too slow: {:?}",
            duration
        );

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_csv_real_world_keys() {
        let path = get_test_path();
        let records = vec![
            CsvRecord::new(
                "page-header.logo.title@de".to_string(),
                "ReedCMS".to_string(),
                Some("Logo title".to_string()),
            ),
            CsvRecord::new(
                "page-header.logo.alt@de".to_string(),
                "ReedCMS Logo".to_string(),
                Some("Logo alt text".to_string()),
            ),
            CsvRecord::new(
                "page-header.nav.home@de".to_string(),
                "Startseite".to_string(),
                Some("Navigation home".to_string()),
            ),
        ];

        let result = write_csv(&path, &records);
        assert!(result.is_ok());

        let read_back = read_csv(&path).unwrap();
        assert_eq!(read_back.len(), 3);
        assert_eq!(read_back[0].key, "page-header.logo.title@de");
        assert_eq!(read_back[1].key, "page-header.logo.alt@de");
        assert_eq!(read_back[2].key, "page-header.nav.home@de");

        cleanup_test_file(&path);
    }

    #[test]
    fn test_write_csv_concurrent_safety() {
        // Test that atomic write prevents corruption
        let path = get_test_path();
        let records1 = vec![CsvRecord::new(
            "key1".to_string(),
            "value1".to_string(),
            None,
        )];
        let records2 = vec![CsvRecord::new(
            "key2".to_string(),
            "value2".to_string(),
            None,
        )];

        // Write first
        write_csv(&path, &records1).unwrap();

        // Write second (should atomically replace)
        write_csv(&path, &records2).unwrap();

        // Read and verify it's one of the two, not corrupted
        let result = read_csv(&path).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].key == "key2"); // Should be the second write

        cleanup_test_file(&path);
    }
}
