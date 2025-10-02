// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::csv::read_csv;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_file(content: &str) -> PathBuf {
        let path = PathBuf::from(format!("/tmp/reed_test_{}.csv", rand::random::<u32>()));
        fs::write(&path, content).unwrap();
        path
    }

    fn cleanup_test_file(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_read_csv_basic() {
        let path = create_test_file("key1|value1|desc1\nkey2|value2|desc2");

        let result = read_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);

        assert_eq!(records[0].key, "key1");
        assert_eq!(records[0].value, "value1");
        assert_eq!(records[0].description, Some("desc1".to_string()));

        assert_eq!(records[1].key, "key2");
        assert_eq!(records[1].value, "value2");
        assert_eq!(records[1].description, Some("desc2".to_string()));

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_csv_no_description() {
        let path = create_test_file("key1|value1\nkey2|value2");

        let result = read_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);

        assert_eq!(records[0].key, "key1");
        assert_eq!(records[0].value, "value1");
        assert_eq!(records[0].description, None);

        assert_eq!(records[1].key, "key2");
        assert_eq!(records[1].value, "value2");
        assert_eq!(records[1].description, None);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_csv_skip_empty_lines() {
        let path = create_test_file("key1|value1\n\nkey2|value2\n\n");

        let result = read_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_csv_skip_comments() {
        let path = create_test_file("# Comment line\nkey1|value1\n# Another comment\nkey2|value2");

        let result = read_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_csv_mixed_formats() {
        let path = create_test_file("key1|value1|desc1\nkey2|value2\nkey3|value3|desc3");

        let result = read_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 3);

        assert_eq!(records[0].description, Some("desc1".to_string()));
        assert_eq!(records[1].description, None);
        assert_eq!(records[2].description, Some("desc3".to_string()));

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_csv_file_not_found() {
        let result = read_csv("/tmp/nonexistent_reed_file_12345.csv");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_csv_invalid_format() {
        let path = create_test_file("key1|value1\ninvalid_line_without_pipe\nkey2|value2");

        let result = read_csv(&path);
        assert!(result.is_err());

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_csv_empty_file() {
        let path = create_test_file("");

        let result = read_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 0);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_csv_only_comments() {
        let path = create_test_file("# Comment 1\n# Comment 2\n# Comment 3");

        let result = read_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 0);

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_csv_whitespace_handling() {
        let path = create_test_file("  key1  |  value1  |  desc1  \n  key2  |  value2  ");

        let result = read_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 2);

        assert_eq!(records[0].key, "key1");
        assert_eq!(records[0].value, "value1");
        assert_eq!(records[0].description, Some("desc1".to_string()));

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_csv_performance_1000_rows() {
        let mut content = String::new();
        for i in 0..1000 {
            content.push_str(&format!("key{}|value{}|description{}\n", i, i, i));
        }

        let path = create_test_file(&content);

        let start = std::time::Instant::now();
        let result = read_csv(&path);
        let duration = start.elapsed();

        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 1000);

        // Should be < 5ms for 1000 rows (allowing for file I/O overhead)
        assert!(
            duration.as_millis() < 5,
            "read_csv too slow: {:?}",
            duration
        );

        cleanup_test_file(&path);
    }

    #[test]
    fn test_read_csv_real_world_keys() {
        let path = create_test_file(
            "page-header.logo.title@de|ReedCMS|Logo title\n\
             page-header.logo.alt@de|ReedCMS Logo|Logo alt text\n\
             page-header.nav.home@de|Startseite|Navigation home\n\
             footer.copyright@de|© 2025|Copyright text",
        );

        let result = read_csv(&path);
        assert!(result.is_ok());

        let records = result.unwrap();
        assert_eq!(records.len(), 4);

        assert_eq!(records[0].key, "page-header.logo.title@de");
        assert_eq!(records[1].key, "page-header.logo.alt@de");
        assert_eq!(records[2].key, "page-header.nav.home@de");
        assert_eq!(records[3].key, "footer.copyright@de");

        cleanup_test_file(&path);
    }
}
