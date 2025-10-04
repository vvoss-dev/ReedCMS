// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::compression::*;

    #[test]
    fn test_compress_gzip_success() {
        let data = b"Hello, World! This is a test string that should compress well.";
        let result = compress_gzip(data);
        assert!(result.is_ok());
        let compressed = result.unwrap();
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_compress_brotli_success() {
        let data = b"Hello, World! This is a test string that should compress well.";
        let result = compress_brotli(data);
        assert!(result.is_ok());
        let compressed = result.unwrap();
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_brotli_better_than_gzip() {
        let data = b"body { margin: 0; padding: 0; } body { margin: 0; padding: 0; }".repeat(10);
        let gzip = compress_gzip(&data).unwrap();
        let brotli = compress_brotli(&data).unwrap();
        // Brotli should compress better than gzip for repetitive text
        assert!(brotli.len() <= gzip.len());
    }

    #[test]
    fn test_get_compression_method_brotli() {
        let accept = "gzip, deflate, br";
        assert_eq!(
            get_compression_method(accept),
            Some(CompressionMethod::Brotli)
        );
    }

    #[test]
    fn test_get_compression_method_gzip() {
        let accept = "gzip, deflate";
        assert_eq!(
            get_compression_method(accept),
            Some(CompressionMethod::Gzip)
        );
    }

    #[test]
    fn test_get_compression_method_none() {
        let accept = "deflate";
        assert_eq!(get_compression_method(accept), None);
    }

    #[test]
    fn test_get_compression_method_case_insensitive() {
        assert_eq!(
            get_compression_method("GZIP, BR"),
            Some(CompressionMethod::Brotli)
        );
        assert_eq!(
            get_compression_method("GzIp"),
            Some(CompressionMethod::Gzip)
        );
    }

    #[test]
    fn test_compress_with_method_gzip() {
        let data = b"Test data for compression";
        let result = compress_with_method(data, CompressionMethod::Gzip);
        assert!(result.is_ok());
        let compressed = result.unwrap();
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_compress_with_method_brotli() {
        let data = b"Test data for compression";
        let result = compress_with_method(data, CompressionMethod::Brotli);
        assert!(result.is_ok());
        let compressed = result.unwrap();
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_compress_empty_data() {
        let data = b"";
        let gzip = compress_gzip(data);
        let brotli = compress_brotli(data);
        assert!(gzip.is_ok());
        assert!(brotli.is_ok());
    }

    #[test]
    fn test_compress_small_data() {
        // Small data might not compress well
        let data = b"x";
        let gzip = compress_gzip(data).unwrap();
        let brotli = compress_brotli(data).unwrap();
        // Just verify it doesn't error
        assert!(!gzip.is_empty());
        assert!(!brotli.is_empty());
    }
}
