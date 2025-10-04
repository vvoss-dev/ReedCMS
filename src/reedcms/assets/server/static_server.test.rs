// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::static_server::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_detect_mime_type_css() {
        assert_eq!(detect_mime_type(Path::new("style.css")), "text/css");
    }

    #[test]
    fn test_detect_mime_type_js() {
        assert_eq!(
            detect_mime_type(Path::new("app.js")),
            "application/javascript"
        );
    }

    #[test]
    fn test_detect_mime_type_images() {
        assert_eq!(detect_mime_type(Path::new("logo.png")), "image/png");
        assert_eq!(detect_mime_type(Path::new("photo.jpg")), "image/jpeg");
        assert_eq!(detect_mime_type(Path::new("photo.jpeg")), "image/jpeg");
        assert_eq!(detect_mime_type(Path::new("icon.svg")), "image/svg+xml");
        assert_eq!(detect_mime_type(Path::new("anim.gif")), "image/gif");
        assert_eq!(detect_mime_type(Path::new("modern.webp")), "image/webp");
    }

    #[test]
    fn test_detect_mime_type_fonts() {
        assert_eq!(detect_mime_type(Path::new("font.woff")), "font/woff");
        assert_eq!(detect_mime_type(Path::new("font.woff2")), "font/woff2");
        assert_eq!(detect_mime_type(Path::new("font.ttf")), "font/ttf");
        assert_eq!(detect_mime_type(Path::new("font.otf")), "font/otf");
    }

    #[test]
    fn test_detect_mime_type_documents() {
        assert_eq!(detect_mime_type(Path::new("doc.pdf")), "application/pdf");
        assert_eq!(detect_mime_type(Path::new("readme.txt")), "text/plain");
        assert_eq!(detect_mime_type(Path::new("docs.md")), "text/markdown");
        assert_eq!(detect_mime_type(Path::new("data.json")), "application/json");
        assert_eq!(detect_mime_type(Path::new("page.html")), "text/html");
    }

    #[test]
    fn test_detect_mime_type_unknown() {
        assert_eq!(
            detect_mime_type(Path::new("file.xyz")),
            "application/octet-stream"
        );
        assert_eq!(
            detect_mime_type(Path::new("noextension")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_get_cache_control_css_js() {
        assert_eq!(
            get_cache_control(Path::new("app.css")),
            "public, max-age=31536000, immutable"
        );
        assert_eq!(
            get_cache_control(Path::new("bundle.js")),
            "public, max-age=31536000, immutable"
        );
    }

    #[test]
    fn test_get_cache_control_images() {
        assert_eq!(
            get_cache_control(Path::new("logo.png")),
            "public, max-age=2592000"
        );
        assert_eq!(
            get_cache_control(Path::new("photo.jpg")),
            "public, max-age=2592000"
        );
        assert_eq!(
            get_cache_control(Path::new("icon.svg")),
            "public, max-age=2592000"
        );
    }

    #[test]
    fn test_get_cache_control_fonts() {
        assert_eq!(
            get_cache_control(Path::new("font.woff2")),
            "public, max-age=31536000"
        );
        assert_eq!(
            get_cache_control(Path::new("font.ttf")),
            "public, max-age=31536000"
        );
    }

    #[test]
    fn test_get_cache_control_other() {
        assert_eq!(
            get_cache_control(Path::new("doc.pdf")),
            "public, max-age=86400"
        );
        assert_eq!(
            get_cache_control(Path::new("data.json")),
            "public, max-age=86400"
        );
    }

    #[test]
    fn test_validate_path_safe() {
        // Create temp directory for testing
        let temp_dir = std::env::temp_dir().join("reed_test_validate");
        fs::create_dir_all(&temp_dir).unwrap();
        let test_file = temp_dir.join("test.txt");
        fs::write(&test_file, "test").unwrap();

        let result = validate_path("test.txt", temp_dir.to_str().unwrap());
        assert!(result.is_ok());

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_validate_path_traversal() {
        let temp_dir = std::env::temp_dir().join("reed_test_traversal");
        fs::create_dir_all(&temp_dir).unwrap();

        // Attempt path traversal
        let result = validate_path("../../../etc/passwd", temp_dir.to_str().unwrap());
        assert!(result.is_err());

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_generate_etag() {
        // Create temp file
        let temp_file = std::env::temp_dir().join("reed_test_etag.txt");
        fs::write(&temp_file, "test content").unwrap();

        let result = generate_etag(&temp_file);
        assert!(result.is_ok());
        let etag = result.unwrap();

        // ETag should be quoted hex string
        assert!(etag.starts_with('"'));
        assert!(etag.ends_with('"'));
        assert!(etag.len() > 2);

        // Cleanup
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_generate_etag_consistent() {
        // Create temp file
        let temp_file = std::env::temp_dir().join("reed_test_etag_consistent.txt");
        fs::write(&temp_file, "test content").unwrap();

        let etag1 = generate_etag(&temp_file).unwrap();
        let etag2 = generate_etag(&temp_file).unwrap();

        // Same file should produce same ETag
        assert_eq!(etag1, etag2);

        // Cleanup
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_generate_etag_changes_on_modification() {
        use std::thread;
        use std::time::Duration;

        // Create temp file
        let temp_file = std::env::temp_dir().join("reed_test_etag_modify.txt");
        fs::write(&temp_file, "original content").unwrap();

        let etag1 = generate_etag(&temp_file).unwrap();

        // Wait to ensure mtime changes
        thread::sleep(Duration::from_millis(10));

        // Modify file
        fs::write(&temp_file, "modified content").unwrap();
        let etag2 = generate_etag(&temp_file).unwrap();

        // ETag should be different after modification
        assert_ne!(etag1, etag2);

        // Cleanup
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_generate_etag_nonexistent_file() {
        let result = generate_etag(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }
}
