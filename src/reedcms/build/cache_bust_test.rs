// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::cache_bust::*;
    use std::fs;
    use std::path::PathBuf;

    fn setup_test_dir() -> PathBuf {
        let test_dir = std::env::temp_dir().join("reed_test_cache_bust");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).unwrap();
        }
        fs::create_dir_all(test_dir.join("public/css")).unwrap();
        fs::create_dir_all(test_dir.join("public/js")).unwrap();
        test_dir
    }

    fn cleanup_test_dir(dir: &PathBuf) {
        if dir.exists() {
            fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn test_calculate_content_hash() {
        let content = b"body { margin: 0; }";
        let hash = calculate_content_hash(content);

        // Hash should be 8 characters
        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Same content should produce same hash
        let hash2 = calculate_content_hash(content);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_calculate_content_hash_different_content() {
        let hash1 = calculate_content_hash(b"content 1");
        let hash2 = calculate_content_hash(b"content 2");

        // Different content should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_insert_hash_into_filename() {
        // CSS file
        let result = insert_hash_into_filename("knowledge.mouse.css", "a7f3k9s2");
        assert_eq!(result, "knowledge.mouse.a7f3k9s2.css");

        // JS file
        let result = insert_hash_into_filename("blog.touch.js", "b4k7p2m9");
        assert_eq!(result, "blog.touch.b4k7p2m9.js");

        // File without extension
        let result = insert_hash_into_filename("noext", "12345678");
        assert_eq!(result, "noext.12345678");
    }

    #[test]
    fn test_is_already_hashed() {
        // Already hashed files
        assert!(is_already_hashed("knowledge.mouse.a7f3k9s2.css"));
        assert!(is_already_hashed("blog.touch.b4k7p2m9.js"));

        // Not hashed files
        assert!(!is_already_hashed("knowledge.mouse.css"));
        assert!(!is_already_hashed("blog.touch.js"));

        // Invalid hash (not 8 chars)
        assert!(!is_already_hashed("file.abc.css"));
        assert!(!is_already_hashed("file.123.css"));

        // Invalid hash (not hex)
        assert!(!is_already_hashed("file.abcdefgh.css"));
    }

    #[test]
    fn test_asset_manifest_new() {
        let manifest = AssetManifest::new();
        assert!(manifest.entries.is_empty());
    }

    #[test]
    fn test_asset_manifest_insert() {
        let mut manifest = AssetManifest::new();
        manifest
            .entries
            .insert("file.css".to_string(), "file.abc123.css".to_string());

        assert_eq!(manifest.entries.len(), 1);
        assert_eq!(
            manifest.entries.get("file.css"),
            Some(&"file.abc123.css".to_string())
        );
    }

    #[test]
    fn test_write_and_load_manifest() {
        let test_dir = setup_test_dir();
        std::env::set_current_dir(&test_dir).unwrap();

        let mut manifest = AssetManifest::new();
        manifest.entries.insert(
            "knowledge.mouse.css".to_string(),
            "knowledge.mouse.a7f3k9s2.css".to_string(),
        );
        manifest.entries.insert(
            "blog.touch.js".to_string(),
            "blog.touch.b4k7p2m9.js".to_string(),
        );

        // Write manifest
        write_manifest(&manifest).unwrap();

        // Check file exists
        assert!(PathBuf::from("public/asset-manifest.json").exists());

        // Load manifest
        let loaded = load_manifest().unwrap();
        assert_eq!(loaded.entries.len(), 2);
        assert_eq!(
            loaded.entries.get("knowledge.mouse.css"),
            Some(&"knowledge.mouse.a7f3k9s2.css".to_string())
        );

        cleanup_test_dir(&test_dir);
        std::env::set_current_dir("/").unwrap();
    }

    #[test]
    fn test_process_directory() {
        let test_dir = setup_test_dir();
        let css_dir = test_dir.join("public/css");

        // Create test files
        fs::write(css_dir.join("style.css"), "body { margin: 0; }").unwrap();
        fs::write(css_dir.join("theme.css"), "body { padding: 0; }").unwrap();

        // Should skip these
        fs::write(css_dir.join("style.css.map"), "{}").unwrap();
        fs::write(css_dir.join("style.css.gz"), vec![0u8; 10]).unwrap();

        let mut manifest = AssetManifest::new();
        std::env::set_current_dir(&test_dir).unwrap();

        process_directory("public/css", &mut manifest).unwrap();

        // Should have processed 2 CSS files
        assert_eq!(manifest.entries.len(), 2);
        assert!(manifest.entries.contains_key("style.css"));
        assert!(manifest.entries.contains_key("theme.css"));

        // Check files were renamed
        let entries = fs::read_dir(&css_dir).unwrap();
        let filenames: Vec<String> = entries
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();

        // Original files should be renamed
        assert!(!filenames.contains(&"style.css".to_string()));
        assert!(!filenames.contains(&"theme.css".to_string()));

        // Should have hashed filenames
        let hashed_count = filenames.iter().filter(|f| is_already_hashed(f)).count();
        assert_eq!(hashed_count, 2);

        cleanup_test_dir(&test_dir);
        std::env::set_current_dir("/").unwrap();
    }

    #[test]
    fn test_load_manifest_nonexistent() {
        let test_dir = setup_test_dir();
        std::env::set_current_dir(&test_dir).unwrap();

        let result = load_manifest();
        assert!(result.is_err());

        cleanup_test_dir(&test_dir);
        std::env::set_current_dir("/").unwrap();
    }
}
