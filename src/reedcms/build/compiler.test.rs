// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::compiler::*;
    use std::fs;

    #[test]
    fn test_calculate_sha256() {
        // Create temp file
        let temp_file = std::env::temp_dir().join("reed_test_sha256.txt");
        fs::write(&temp_file, "test content").unwrap();

        let sha256 = calculate_sha256(temp_file.to_str().unwrap()).unwrap();

        // SHA256 should be 64 characters (hex)
        assert_eq!(sha256.len(), 64);
        assert!(sha256.chars().all(|c| c.is_ascii_hexdigit()));

        // Same content should produce same hash
        let sha256_2 = calculate_sha256(temp_file.to_str().unwrap()).unwrap();
        assert_eq!(sha256, sha256_2);

        // Cleanup
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_calculate_md5() {
        // Create temp file
        let temp_file = std::env::temp_dir().join("reed_test_md5.txt");
        fs::write(&temp_file, "test content").unwrap();

        let md5 = calculate_md5(temp_file.to_str().unwrap()).unwrap();

        // MD5 should be 32 characters (hex)
        assert_eq!(md5.len(), 32);
        assert!(md5.chars().all(|c| c.is_ascii_hexdigit()));

        // Same content should produce same hash
        let md5_2 = calculate_md5(temp_file.to_str().unwrap()).unwrap();
        assert_eq!(md5, md5_2);

        // Cleanup
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_calculate_sha256_different_content() {
        let temp_file_1 = std::env::temp_dir().join("reed_test_sha256_1.txt");
        let temp_file_2 = std::env::temp_dir().join("reed_test_sha256_2.txt");

        fs::write(&temp_file_1, "content 1").unwrap();
        fs::write(&temp_file_2, "content 2").unwrap();

        let sha256_1 = calculate_sha256(temp_file_1.to_str().unwrap()).unwrap();
        let sha256_2 = calculate_sha256(temp_file_2.to_str().unwrap()).unwrap();

        // Different content should produce different hashes
        assert_ne!(sha256_1, sha256_2);

        // Cleanup
        fs::remove_file(&temp_file_1).unwrap();
        fs::remove_file(&temp_file_2).unwrap();
    }

    #[test]
    fn test_calculate_sha256_nonexistent_file() {
        let result = calculate_sha256("/nonexistent/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_md5_nonexistent_file() {
        let result = calculate_md5("/nonexistent/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_info_structure() {
        let build_info = BuildInfo {
            version: "0.1.0".to_string(),
            binary_path: "target/release/reedcms".to_string(),
            original_size: 15_000_000,
            compressed_size: Some(6_000_000),
            sha256: "a".repeat(64),
            md5: "b".repeat(32),
            build_time: "2025-02-04T12:00:00Z".to_string(),
            build_duration_secs: 180,
        };

        assert_eq!(build_info.version, "0.1.0");
        assert_eq!(build_info.original_size, 15_000_000);
        assert_eq!(build_info.compressed_size, Some(6_000_000));
        assert_eq!(build_info.sha256.len(), 64);
        assert_eq!(build_info.md5.len(), 32);
    }

    #[test]
    fn test_should_use_upx() {
        // This test just verifies the function doesn't panic
        let _ = should_use_upx();
    }

    #[test]
    fn test_build_info_serialization() {
        let build_info = BuildInfo {
            version: "0.1.0".to_string(),
            binary_path: "target/release/reedcms".to_string(),
            original_size: 15_000_000,
            compressed_size: None,
            sha256: "test_sha".to_string(),
            md5: "test_md5".to_string(),
            build_time: "2025-02-04T12:00:00Z".to_string(),
            build_duration_secs: 180,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&build_info).unwrap();
        assert!(json.contains("0.1.0"));
        assert!(json.contains("test_sha"));
        assert!(json.contains("test_md5"));
    }
}
