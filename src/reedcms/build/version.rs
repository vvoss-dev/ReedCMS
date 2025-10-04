// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Version management and build metadata.
//!
//! This module provides version information and build metadata for ReedCMS.

use serde::Serialize;

/// Gets current ReedCMS version.
///
/// ## Input
/// - None
///
/// ## Output
/// - `&str`: Version string from Cargo.toml
///
/// ## Performance
/// - O(1) constant lookup
///
/// ## Example Usage
/// ```rust
/// let version = get_version();
/// println!("ReedCMS v{}", version);
/// ```
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Gets full build metadata.
///
/// ## Input
/// - None
///
/// ## Output
/// - `BuildMetadata`: Complete build information
///
/// ## Performance
/// - O(1) constant lookups
///
/// ## Example Usage
/// ```rust
/// let metadata = get_build_metadata();
/// println!("{}", metadata.full_version());
/// ```
pub fn get_build_metadata() -> BuildMetadata {
    BuildMetadata {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: env!("CARGO_PKG_NAME").to_string(),
        authors: env!("CARGO_PKG_AUTHORS").to_string(),
        description: env!("CARGO_PKG_DESCRIPTION").to_string(),
        repository: env!("CARGO_PKG_REPOSITORY").to_string(),
        license: env!("CARGO_PKG_LICENSE").to_string(),
        rust_version: option_env!("CARGO_PKG_RUST_VERSION")
            .unwrap_or("unknown")
            .to_string(),
    }
}

/// Gets version with optional build suffix.
///
/// ## Input
/// - `suffix`: Optional build suffix (e.g., "beta", "rc1")
///
/// ## Output
/// - `String`: Version with suffix if provided
///
/// ## Example Usage
/// ```rust
/// let version = get_version_with_suffix(Some("beta"));
/// assert_eq!(version, "0.1.0-beta");
/// ```
pub fn get_version_with_suffix(suffix: Option<&str>) -> String {
    if let Some(s) = suffix {
        format!("{}-{}", get_version(), s)
    } else {
        get_version().to_string()
    }
}

/// Parses semantic version into components.
///
/// ## Input
/// - `version`: Version string (e.g., "1.2.3")
///
/// ## Output
/// - `Option<(u32, u32, u32)>`: Major, minor, patch numbers
///
/// ## Example Usage
/// ```rust
/// let (major, minor, patch) = parse_version("1.2.3").unwrap();
/// assert_eq!((major, minor, patch), (1, 2, 3));
/// ```
pub fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;

    Some((major, minor, patch))
}

/// Checks if version is compatible with another version.
///
/// ## Input
/// - `version_a`: First version string
/// - `version_b`: Second version string
///
/// ## Output
/// - `bool`: true if major versions match (SemVer compatibility)
///
/// ## Example Usage
/// ```rust
/// assert!(is_compatible("1.2.3", "1.9.0"));
/// assert!(!is_compatible("1.2.3", "2.0.0"));
/// ```
pub fn is_compatible(version_a: &str, version_b: &str) -> bool {
    match (parse_version(version_a), parse_version(version_b)) {
        (Some((major_a, _, _)), Some((major_b, _, _))) => major_a == major_b,
        _ => false,
    }
}

/// Build metadata structure.
#[derive(Debug, Clone, Serialize)]
pub struct BuildMetadata {
    pub version: String,
    pub name: String,
    pub authors: String,
    pub description: String,
    pub repository: String,
    pub license: String,
    pub rust_version: String,
}

impl BuildMetadata {
    /// Gets full version string with package name.
    ///
    /// ## Output
    /// - `String`: "name version" format
    ///
    /// ## Example
    /// - "reedcms 0.1.0"
    pub fn full_version(&self) -> String {
        format!("{} {}", self.name, self.version)
    }

    /// Gets build information as formatted string.
    ///
    /// ## Output
    /// - `String`: Multi-line build information
    ///
    /// ## Example
    /// ```
    /// ReedCMS 0.1.0
    /// Authors: Vivian Voss <ask@vvoss.dev>
    /// License: Apache-2.0
    /// ```
    pub fn build_info_string(&self) -> String {
        format!(
            "{} {}\nAuthors: {}\nLicense: {}\nRepository: {}",
            self.name, self.version, self.authors, self.license, self.repository
        )
    }
}
