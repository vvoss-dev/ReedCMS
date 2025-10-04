// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::packager::*;
    use std::fs;
    use std::path::PathBuf;

    fn setup_test_dir() -> PathBuf {
        let test_dir = std::env::temp_dir().join("reed_test_packager");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).unwrap();
        }
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }

    fn cleanup_test_dir(dir: &PathBuf) {
        if dir.exists() {
            fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn test_copy_dir_recursive() {
        let test_dir = setup_test_dir();
        let src_dir = test_dir.join("source");
        let dst_dir = test_dir.join("destination");

        // Create source structure
        fs::create_dir_all(src_dir.join("subdir")).unwrap();
        fs::write(src_dir.join("file1.txt"), "content 1").unwrap();
        fs::write(src_dir.join("file2.txt"), "content 2").unwrap();
        fs::write(src_dir.join("subdir/file3.txt"), "content 3").unwrap();

        // Copy recursively
        let result = copy_dir_recursive(src_dir.to_str().unwrap(), dst_dir.to_str().unwrap());
        assert!(result.is_ok());

        // Verify copied files
        assert!(dst_dir.join("file1.txt").exists());
        assert!(dst_dir.join("file2.txt").exists());
        assert!(dst_dir.join("subdir/file3.txt").exists());

        // Verify content
        let content = fs::read_to_string(dst_dir.join("file1.txt")).unwrap();
        assert_eq!(content, "content 1");

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_copy_dir_recursive_empty_dir() {
        let test_dir = setup_test_dir();
        let src_dir = test_dir.join("empty_source");
        let dst_dir = test_dir.join("empty_destination");

        fs::create_dir_all(&src_dir).unwrap();

        let result = copy_dir_recursive(src_dir.to_str().unwrap(), dst_dir.to_str().unwrap());
        assert!(result.is_ok());
        assert!(dst_dir.exists());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_copy_dir_recursive_nonexistent_source() {
        let result = copy_dir_recursive("/nonexistent/source", "/tmp/destination");
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_archive_sha256() {
        // Create temp file to simulate archive
        let temp_file = std::env::temp_dir().join("reed_test_archive.tar.gz");
        fs::write(&temp_file, "archive content").unwrap();

        let sha256 = calculate_archive_sha256(temp_file.to_str().unwrap()).unwrap();

        // SHA256 should be 64 characters
        assert_eq!(sha256.len(), 64);
        assert!(sha256.chars().all(|c| c.is_ascii_hexdigit()));

        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_calculate_archive_sha256_nonexistent() {
        let result = calculate_archive_sha256("/nonexistent/archive.tar.gz");
        assert!(result.is_err());
    }

    #[test]
    fn test_package_info_structure() {
        let package_info = PackageInfo {
            package_name: "reedcms-v0.1.0-linux-x86_64".to_string(),
            archive_path: "target/release/reedcms-v0.1.0-linux-x86_64.tar.gz".to_string(),
            archive_size: 10_000_000,
            sha256: "a".repeat(64),
        };

        assert!(package_info.package_name.contains("reedcms"));
        assert!(package_info.package_name.contains("v0.1.0"));
        assert!(package_info.archive_path.ends_with(".tar.gz"));
        assert_eq!(package_info.archive_size, 10_000_000);
        assert_eq!(package_info.sha256.len(), 64);
    }

    #[test]
    fn test_create_tar_gz_archive() {
        let test_dir = setup_test_dir();
        let package_dir = test_dir.join("package");

        // Create package directory with files
        fs::create_dir_all(&package_dir).unwrap();
        fs::write(package_dir.join("file1.txt"), "test content 1").unwrap();
        fs::write(package_dir.join("file2.txt"), "test content 2").unwrap();

        // Create archive
        let archive_path = create_tar_gz_archive("test-package", package_dir.to_str().unwrap());

        assert!(archive_path.is_ok());
        let path = archive_path.unwrap();
        assert!(PathBuf::from(&path).exists());
        assert!(path.ends_with(".tar.gz"));

        // Cleanup
        fs::remove_file(&path).unwrap();
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_create_tar_gz_archive_empty_dir() {
        let test_dir = setup_test_dir();
        let package_dir = test_dir.join("empty_package");
        fs::create_dir_all(&package_dir).unwrap();

        let result = create_tar_gz_archive("empty-package", package_dir.to_str().unwrap());

        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(PathBuf::from(&path).exists());

        // Cleanup
        fs::remove_file(&path).unwrap();
        cleanup_test_dir(&test_dir);
    }
}
