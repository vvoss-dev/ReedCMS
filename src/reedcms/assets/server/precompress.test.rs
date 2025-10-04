// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::precompress::*;
    use std::fs;
    use std::path::PathBuf;

    fn setup_test_dir() -> PathBuf {
        let test_dir = std::env::temp_dir().join("reed_test_precompress");
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
    fn test_discover_compressible_assets() {
        let test_dir = setup_test_dir();

        // Create test files
        fs::write(test_dir.join("style.css"), "body { margin: 0; }").unwrap();
        fs::write(test_dir.join("app.js"), "console.log('test');").unwrap();
        fs::write(test_dir.join("data.json"), "{}").unwrap();
        fs::write(test_dir.join("page.html"), "<html></html>").unwrap();
        fs::write(test_dir.join("icon.svg"), "<svg></svg>").unwrap();
        fs::write(test_dir.join("readme.txt"), "README").unwrap();
        fs::write(test_dir.join("docs.md"), "# Docs").unwrap();

        // Non-compressible file
        fs::write(test_dir.join("image.png"), vec![0u8; 100]).unwrap();

        let result = discover_compressible_assets(test_dir.to_str().unwrap());
        assert!(result.is_ok());
        let assets = result.unwrap();

        // Should find 7 compressible files
        assert_eq!(assets.len(), 7);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_discover_compressible_assets_recursive() {
        let test_dir = setup_test_dir();
        let subdir = test_dir.join("css");
        fs::create_dir_all(&subdir).unwrap();

        fs::write(test_dir.join("app.js"), "console.log('test');").unwrap();
        fs::write(subdir.join("style.css"), "body { margin: 0; }").unwrap();

        let result = discover_compressible_assets(test_dir.to_str().unwrap());
        assert!(result.is_ok());
        let assets = result.unwrap();

        // Should find files in subdirectories
        assert_eq!(assets.len(), 2);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_precompress_asset() {
        let test_dir = setup_test_dir();
        let css_file = test_dir.join("style.css");

        // Create CSS file with compressible content
        let content = "body { margin: 0; padding: 0; }".repeat(50);
        fs::write(&css_file, &content).unwrap();

        let result = precompress_asset(&css_file);
        assert!(result.is_ok());

        // Check that .gz and .br files were created
        let gz_file = test_dir.join("style.css.gz");
        let br_file = test_dir.join("style.css.br");

        assert!(gz_file.exists());
        assert!(br_file.exists());

        // Check that compressed files are smaller
        let gz_size = fs::metadata(&gz_file).unwrap().len();
        let br_size = fs::metadata(&br_file).unwrap().len();
        assert!(gz_size < content.len() as u64);
        assert!(br_size < content.len() as u64);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_precompress_asset_skips_if_not_beneficial() {
        let test_dir = setup_test_dir();
        let file = test_dir.join("small.txt");

        // Very small file that won't compress well
        fs::write(&file, "x").unwrap();

        let result = precompress_asset(&file);
        assert!(result.is_ok());

        // Compressed files might not be created if larger than original
        // This is expected behavior

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_precompress_all_assets() {
        let test_dir = setup_test_dir();

        // Create multiple compressible files
        fs::write(test_dir.join("style.css"), "body { margin: 0; }".repeat(50)).unwrap();
        fs::write(test_dir.join("app.js"), "console.log('test');".repeat(50)).unwrap();
        fs::write(test_dir.join("data.json"), r#"{"key": "value"}"#.repeat(50)).unwrap();

        let result = precompress_all_assets(test_dir.to_str().unwrap());
        assert!(result.is_ok());
        let count = result.unwrap();

        // Should have processed 3 files
        assert_eq!(count, 3);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_clean_precompressed_assets() {
        let test_dir = setup_test_dir();

        // Create original and compressed files
        fs::write(test_dir.join("style.css"), "body { margin: 0; }").unwrap();
        fs::write(test_dir.join("style.css.gz"), vec![0u8; 10]).unwrap();
        fs::write(test_dir.join("style.css.br"), vec![0u8; 10]).unwrap();
        fs::write(test_dir.join("app.js"), "console.log('test');").unwrap();
        fs::write(test_dir.join("app.js.gz"), vec![0u8; 10]).unwrap();

        let result = clean_precompressed_assets(test_dir.to_str().unwrap());
        assert!(result.is_ok());
        let deleted = result.unwrap();

        // Should have deleted 3 compressed files
        assert_eq!(deleted, 3);

        // Original files should still exist
        assert!(test_dir.join("style.css").exists());
        assert!(test_dir.join("app.js").exists());

        // Compressed files should be gone
        assert!(!test_dir.join("style.css.gz").exists());
        assert!(!test_dir.join("style.css.br").exists());
        assert!(!test_dir.join("app.js.gz").exists());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_precompress_incremental() {
        use std::thread;
        use std::time::Duration;

        let test_dir = setup_test_dir();
        let css_file = test_dir.join("style.css");

        // Create and compress file
        fs::write(&css_file, "body { margin: 0; }".repeat(50)).unwrap();
        precompress_asset(&css_file).unwrap();

        let gz_file = test_dir.join("style.css.gz");
        let first_gz_mtime = fs::metadata(&gz_file).unwrap().modified().unwrap();

        // Wait and run again without modification
        thread::sleep(Duration::from_millis(10));
        precompress_asset(&css_file).unwrap();

        let second_gz_mtime = fs::metadata(&gz_file).unwrap().modified().unwrap();

        // Should not recompress (mtime unchanged)
        assert_eq!(first_gz_mtime, second_gz_mtime);

        // Now modify original file
        thread::sleep(Duration::from_millis(10));
        fs::write(&css_file, "body { padding: 0; }".repeat(50)).unwrap();
        precompress_asset(&css_file).unwrap();

        let third_gz_mtime = fs::metadata(&gz_file).unwrap().modified().unwrap();

        // Should have recompressed (mtime changed)
        assert_ne!(second_gz_mtime, third_gz_mtime);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_discover_nonexistent_directory() {
        let result = discover_compressible_assets("/nonexistent/directory");
        assert!(result.is_err());
    }

    #[test]
    fn test_precompress_nonexistent_file() {
        let result = precompress_asset(std::path::Path::new("/nonexistent/file.css"));
        assert!(result.is_err());
    }
}
