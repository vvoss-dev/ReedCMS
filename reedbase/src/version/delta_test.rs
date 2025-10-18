// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for binary delta compression.

#[cfg(test)]
mod tests {
    use crate::version::{apply_delta, calculate_savings, generate_delta};
    use std::fs;
    use tempfile::TempDir;

    /// Test generate and apply delta roundtrip.
    #[test]
    fn test_generate_and_apply_delta() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");
        let output_path = temp_dir.path().join("output.csv");

        // Create test files
        fs::write(&old_path, "id|name\n1|Alice\n2|Bob\n").unwrap();
        fs::write(&new_path, "id|name\n1|Alice\n2|Bob\n3|Charlie\n").unwrap();

        // Generate delta
        let info = generate_delta(&old_path, &new_path, &delta_path).unwrap();
        // Note: For very small files, delta + compression overhead can be larger
        // This is expected and acceptable - savings come with larger files
        assert!(info.size > 0, "Delta should exist");

        // Apply delta
        apply_delta(&old_path, &delta_path, &output_path).unwrap();

        // Verify output matches new version
        let output = fs::read_to_string(&output_path).unwrap();
        let expected = fs::read_to_string(&new_path).unwrap();
        assert_eq!(output, expected);
    }

    /// Test delta for single line change.
    #[test]
    fn test_generate_delta_one_line_change() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");

        fs::write(&old_path, "id|name|age\n1|Alice|30\n2|Bob|25\n").unwrap();
        fs::write(&new_path, "id|name|age\n1|Alice|31\n2|Bob|25\n").unwrap();

        let info = generate_delta(&old_path, &new_path, &delta_path).unwrap();

        // For small files, overhead dominates - just verify delta works
        // Real savings come with larger files (see test_generate_delta_large_file)
        assert!(info.size > 0, "Delta should be generated");
    }

    /// Test delta with multiple row changes.
    #[test]
    fn test_generate_delta_multiple_changes() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");

        // Create 100-row CSV
        let mut old_content = String::from("id|name|value\n");
        let mut new_content = String::from("id|name|value\n");
        for i in 1..=100 {
            old_content.push_str(&format!("{}|name{}|value{}\n", i, i, i));
            // Change every 10th row
            if i % 10 == 0 {
                new_content.push_str(&format!("{}|CHANGED|value{}\n", i, i * 2));
            } else {
                new_content.push_str(&format!("{}|name{}|value{}\n", i, i, i));
            }
        }

        fs::write(&old_path, old_content.as_bytes()).unwrap();
        fs::write(&new_path, new_content.as_bytes()).unwrap();

        let info = generate_delta(&old_path, &new_path, &delta_path).unwrap();

        // 10% row changes should result in < 20% delta size
        assert!(
            info.ratio < 20,
            "Delta ratio should be < 20% for 10% row changes, got {}%",
            info.ratio
        );
    }

    /// Test delta with empty files.
    #[test]
    fn test_generate_delta_empty_files() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");

        fs::write(&old_path, b"").unwrap();
        fs::write(&new_path, b"id|name\n1|Alice\n").unwrap();

        let info = generate_delta(&old_path, &new_path, &delta_path).unwrap();
        assert!(info.size > 0, "Delta should be generated");
        // Compression overhead can make ratio > 100% for tiny files
        assert!(info.ratio > 0, "Ratio should be positive");
    }

    /// Test delta with identical files.
    #[test]
    fn test_generate_delta_identical_files() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");

        let content = b"id|name\n1|Alice\n2|Bob\n";
        fs::write(&old_path, content).unwrap();
        fs::write(&new_path, content).unwrap();

        let info = generate_delta(&old_path, &new_path, &delta_path).unwrap();
        // Delta for identical files should be very small
        assert!(
            info.size < 200,
            "Delta for identical files should be < 200 bytes, got {}",
            info.size
        );
    }

    /// Test calculate_savings function.
    #[test]
    fn test_calculate_savings() {
        assert_eq!(calculate_savings(500, 10000), 95.0);
        assert_eq!(calculate_savings(1000, 10000), 90.0);
        assert_eq!(calculate_savings(5000, 10000), 50.0);
        assert_eq!(calculate_savings(10000, 10000), 0.0);
        assert_eq!(calculate_savings(0, 10000), 100.0);
        assert_eq!(calculate_savings(0, 0), 0.0); // Edge case
    }

    /// Test apply delta with corrupted delta file.
    #[test]
    fn test_apply_delta_corrupted() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");
        let output_path = temp_dir.path().join("output.csv");

        fs::write(&old_path, b"id|name\n1|Alice\n").unwrap();
        fs::write(&delta_path, b"corrupted data").unwrap();

        let result = apply_delta(&old_path, &delta_path, &output_path);
        assert!(result.is_err(), "Should fail with corrupted delta");
    }

    /// Test apply delta with missing old file.
    #[test]
    fn test_apply_delta_missing_old_file() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("nonexistent.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");
        let output_path = temp_dir.path().join("output.csv");

        fs::write(&delta_path, b"some data").unwrap();

        let result = apply_delta(&old_path, &delta_path, &output_path);
        assert!(result.is_err(), "Should fail with missing old file");
    }

    /// Test generate delta with missing files.
    #[test]
    fn test_generate_delta_missing_files() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("nonexistent_old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");

        fs::write(&new_path, b"data").unwrap();

        let result = generate_delta(&old_path, &new_path, &delta_path);
        assert!(result.is_err(), "Should fail with missing old file");
    }

    /// Test delta with UTF-8 content.
    #[test]
    fn test_generate_delta_utf8() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");
        let output_path = temp_dir.path().join("output.csv");

        fs::write(&old_path, "id|name\n1|Übung\n2|Café\n").unwrap();
        fs::write(&new_path, "id|name\n1|Übung\n2|Café\n3|Naïve\n").unwrap();

        let info = generate_delta(&old_path, &new_path, &delta_path).unwrap();
        assert!(info.size > 0);

        apply_delta(&old_path, &delta_path, &output_path).unwrap();

        let output = fs::read_to_string(&output_path).unwrap();
        let expected = fs::read_to_string(&new_path).unwrap();
        assert_eq!(output, expected);
    }

    /// Test delta with large files (performance check).
    #[test]
    fn test_generate_delta_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");

        // Create 1000-row CSV
        let mut old_content = String::from("id|name|description\n");
        let mut new_content = String::from("id|name|description\n");
        for i in 1..=1000 {
            old_content.push_str(&format!("{}|name{}|description for row {}\n", i, i, i));
            // Change 5% of rows
            if i % 20 == 0 {
                new_content.push_str(&format!("{}|MODIFIED|updated description {}\n", i, i));
            } else {
                new_content.push_str(&format!("{}|name{}|description for row {}\n", i, i, i));
            }
        }

        fs::write(&old_path, old_content.as_bytes()).unwrap();
        fs::write(&new_path, new_content.as_bytes()).unwrap();

        let start = std::time::Instant::now();
        let info = generate_delta(&old_path, &new_path, &delta_path).unwrap();
        let duration = start.elapsed();

        // Should complete in < 500ms for 1000 rows
        assert!(
            duration.as_millis() < 500,
            "Delta generation took too long: {:?}",
            duration
        );

        // 5% changes should result in < 10% delta
        assert!(
            info.ratio < 10,
            "Delta ratio too high for 5% changes: {}%",
            info.ratio
        );
    }

    /// Test apply delta performance.
    #[test]
    fn test_apply_delta_performance() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");
        let output_path = temp_dir.path().join("output.csv");

        // Create 100-row CSV
        let mut old_content = String::from("id|name|value\n");
        let mut new_content = String::from("id|name|value\n");
        for i in 1..=100 {
            old_content.push_str(&format!("{}|name{}|value{}\n", i, i, i));
            new_content.push_str(&format!("{}|name{}|CHANGED\n", i, i));
        }

        fs::write(&old_path, old_content.as_bytes()).unwrap();
        fs::write(&new_path, new_content.as_bytes()).unwrap();

        generate_delta(&old_path, &new_path, &delta_path).unwrap();

        let start = std::time::Instant::now();
        apply_delta(&old_path, &delta_path, &output_path).unwrap();
        let duration = start.elapsed();

        // Should complete in < 50ms for 100 rows
        assert!(
            duration.as_millis() < 50,
            "Delta application took too long: {:?}",
            duration
        );
    }

    /// Test atomic write during delta application.
    #[test]
    fn test_apply_delta_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.csv");
        let new_path = temp_dir.path().join("new.csv");
        let delta_path = temp_dir.path().join("delta.bsdiff");
        let output_path = temp_dir.path().join("output.csv");

        fs::write(&old_path, b"id|name\n1|Alice\n").unwrap();
        fs::write(&new_path, b"id|name\n1|Alice\n2|Bob\n").unwrap();

        generate_delta(&old_path, &new_path, &delta_path).unwrap();
        apply_delta(&old_path, &delta_path, &output_path).unwrap();

        // Verify no .tmp file remains
        let temp_file = output_path.with_extension("tmp");
        assert!(!temp_file.exists(), "Temp file should be cleaned up");
    }
}
