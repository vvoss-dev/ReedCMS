// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::reedcms::csv::record::{create_row, parse_row, CsvRecord};

    // CsvRecord::new tests
    #[test]
    fn test_csv_record_new_with_description() {
        let record = CsvRecord::new(
            "page.title@en".to_string(),
            "Welcome".to_string(),
            Some("Homepage title".to_string()),
        );

        assert_eq!(record.key, "page.title@en");
        assert_eq!(record.value, "Welcome");
        assert_eq!(record.description, Some("Homepage title".to_string()));
    }

    #[test]
    fn test_csv_record_new_without_description() {
        let record = CsvRecord::new("page.title@en".to_string(), "Welcome".to_string(), None);

        assert_eq!(record.key, "page.title@en");
        assert_eq!(record.value, "Welcome");
        assert_eq!(record.description, None);
    }

    // parse_row tests
    #[test]
    fn test_parse_row_three_fields() {
        let result = parse_row("page.title@en|Welcome|Homepage title");
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.key, "page.title@en");
        assert_eq!(record.value, "Welcome");
        assert_eq!(record.description, Some("Homepage title".to_string()));
    }

    #[test]
    fn test_parse_row_two_fields() {
        let result = parse_row("page.title@en|Welcome");
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.key, "page.title@en");
        assert_eq!(record.value, "Welcome");
        assert_eq!(record.description, None);
    }

    #[test]
    fn test_parse_row_with_whitespace() {
        let result = parse_row("  page.title@en  |  Welcome  |  Homepage title  ");
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.key, "page.title@en");
        assert_eq!(record.value, "Welcome");
        assert_eq!(record.description, Some("Homepage title".to_string()));
    }

    #[test]
    fn test_parse_row_empty_description() {
        let result = parse_row("page.title@en|Welcome|");
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.key, "page.title@en");
        assert_eq!(record.value, "Welcome");
        assert_eq!(record.description, None);
    }

    #[test]
    fn test_parse_row_one_field_error() {
        let result = parse_row("page.title@en");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_row_empty_key_error() {
        let result = parse_row("|Welcome|Description");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_row_whitespace_key_error() {
        let result = parse_row("   |Welcome|Description");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_row_empty_line_error() {
        let result = parse_row("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_row_with_pipes_in_value() {
        let result = parse_row("key|value|with|pipes|description");
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.key, "key");
        assert_eq!(record.value, "value");
        // Only first 3 fields are parsed
        assert_eq!(record.description, Some("with".to_string()));
    }

    // create_row tests
    #[test]
    fn test_create_row_with_description() {
        let record = CsvRecord::new(
            "page.title@en".to_string(),
            "Welcome".to_string(),
            Some("Homepage title".to_string()),
        );

        let row = create_row(&record);
        assert_eq!(row, "page.title@en|Welcome|Homepage title");
    }

    #[test]
    fn test_create_row_without_description() {
        let record = CsvRecord::new("page.title@en".to_string(), "Welcome".to_string(), None);

        let row = create_row(&record);
        assert_eq!(row, "page.title@en|Welcome");
    }

    // Round-trip tests
    #[test]
    fn test_round_trip_with_description() {
        let original = "page.title@en|Welcome|Homepage title";
        let record = parse_row(original).unwrap();
        let recreated = create_row(&record);
        assert_eq!(recreated, original);
    }

    #[test]
    fn test_round_trip_without_description() {
        let original = "page.title@en|Welcome";
        let record = parse_row(original).unwrap();
        let recreated = create_row(&record);
        assert_eq!(recreated, original);
    }

    // Performance benchmarks
    #[test]
    fn test_parse_row_performance() {
        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = parse_row("page.title@en|Welcome|Homepage title");
        }
        let duration = start.elapsed();

        // Should be < 10μs per operation on average
        // 10,000 operations should complete in < 100ms
        assert!(
            duration.as_millis() < 100,
            "parse_row too slow: {:?}",
            duration
        );
    }

    #[test]
    fn test_create_row_performance() {
        let record = CsvRecord::new(
            "page.title@en".to_string(),
            "Welcome".to_string(),
            Some("Homepage title".to_string()),
        );

        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = create_row(&record);
        }
        let duration = start.elapsed();

        // Should be < 1μs per operation on average
        // 10,000 operations should complete in < 10ms
        assert!(
            duration.as_millis() < 10,
            "create_row too slow: {:?}",
            duration
        );
    }
}
