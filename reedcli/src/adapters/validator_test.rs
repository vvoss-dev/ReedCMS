// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for adapter validator.

#[cfg(test)]
mod tests {
    use crate::adapters::validator::*;
    use crate::types::*;
    use std::collections::HashMap;

    #[test]
    fn test_format_missing_adapter_error() {
        let adapter = Adapter {
            name: "reedbase".to_string(),
            binary: std::path::PathBuf::from("reedbase"),
            description: "Database".to_string(),
            version_requirement: None,
            required: true,
            aliases: HashMap::new(),
            commands: vec![],
            validated: false,
        };

        let msg = format_missing_adapter_error(&adapter);
        assert!(msg.contains("reedbase"));
        assert!(msg.contains("cargo install"));
    }

    #[test]
    fn test_find_binary_in_path_exists() {
        // Test with known binary that should exist
        let result = find_binary_in_path("ls");
        assert!(result.is_ok());
        // On Unix systems, ls should exist
        #[cfg(unix)]
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_find_binary_in_path_not_exists() {
        let result = find_binary_in_path("nonexistent_binary_xyz123").unwrap();
        assert!(result.is_none());
    }
}
