// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::version::*;

    #[test]
    fn test_get_version() {
        let version = get_version();
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }

    #[test]
    fn test_get_build_metadata() {
        let metadata = get_build_metadata();
        assert_eq!(metadata.name, "reedcms");
        assert!(!metadata.version.is_empty());
        assert!(!metadata.authors.is_empty());
        assert_eq!(metadata.license, "Apache-2.0");
    }

    #[test]
    fn test_get_version_with_suffix() {
        let version_no_suffix = get_version_with_suffix(None);
        assert_eq!(version_no_suffix, get_version());

        let version_beta = get_version_with_suffix(Some("beta"));
        assert!(version_beta.ends_with("-beta"));
        assert!(version_beta.starts_with(get_version()));

        let version_rc = get_version_with_suffix(Some("rc1"));
        assert!(version_rc.ends_with("-rc1"));
    }

    #[test]
    fn test_parse_version() {
        // Valid versions
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("10.20.30"), Some((10, 20, 30)));

        // Invalid versions
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version("1.2.3.4"), None);
        assert_eq!(parse_version("a.b.c"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn test_is_compatible() {
        // Same major version - compatible
        assert!(is_compatible("1.2.3", "1.9.0"));
        assert!(is_compatible("1.0.0", "1.9.9"));
        assert!(is_compatible("2.1.0", "2.5.3"));

        // Different major version - incompatible
        assert!(!is_compatible("1.2.3", "2.0.0"));
        assert!(!is_compatible("0.9.0", "1.0.0"));
        assert!(!is_compatible("2.0.0", "3.0.0"));

        // Invalid versions - incompatible
        assert!(!is_compatible("1.2.3", "invalid"));
        assert!(!is_compatible("invalid", "1.2.3"));
        assert!(!is_compatible("1.2", "1.2.3"));
    }

    #[test]
    fn test_build_metadata_full_version() {
        let metadata = get_build_metadata();
        let full = metadata.full_version();
        assert!(full.contains(&metadata.name));
        assert!(full.contains(&metadata.version));
        assert_eq!(full, format!("{} {}", metadata.name, metadata.version));
    }

    #[test]
    fn test_build_metadata_build_info_string() {
        let metadata = get_build_metadata();
        let info = metadata.build_info_string();

        assert!(info.contains(&metadata.name));
        assert!(info.contains(&metadata.version));
        assert!(info.contains(&metadata.authors));
        assert!(info.contains(&metadata.license));
        assert!(info.contains(&metadata.repository));

        // Check multiline format
        assert!(info.contains('\n'));
    }

    #[test]
    fn test_version_format() {
        let version = get_version();
        // Should be semantic versioning format (x.y.z)
        let parts: Vec<&str> = version.split('.').collect();
        assert_eq!(parts.len(), 3, "Version should be in x.y.z format");

        for part in parts {
            assert!(
                part.parse::<u32>().is_ok(),
                "Version parts should be numbers"
            );
        }
    }
}
