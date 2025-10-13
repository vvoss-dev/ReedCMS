// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for adapter registry.

#[cfg(test)]
mod tests {
    use crate::adapters::registry::*;

    #[test]
    fn test_version_parsing() {
        let result = parse_version("1.2.3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (1, 2, 3));
    }

    #[test]
    fn test_version_comparison() {
        let a = (1, 2, 3);
        let b = (1, 2, 4);
        assert_eq!(compare_versions(&a, &b), -1);
        assert_eq!(compare_versions(&b, &a), 1);
        assert_eq!(compare_versions(&a, &a), 0);
    }

    #[test]
    fn test_version_matches_greater_equal() {
        assert!(version_matches("1.2.3", ">=1.2.0").unwrap());
        assert!(version_matches("1.2.3", ">=1.2.3").unwrap());
        assert!(!version_matches("1.2.3", ">=1.2.4").unwrap());
    }

    #[test]
    fn test_version_matches_exact() {
        assert!(version_matches("1.2.3", "=1.2.3").unwrap());
        assert!(version_matches("1.2.3", "1.2.3").unwrap());
        assert!(!version_matches("1.2.3", "=1.2.4").unwrap());
    }

    #[test]
    fn test_build_command_index() {
        use crate::types::*;
        use std::collections::HashMap;

        let mut adapters = HashMap::new();
        adapters.insert(
            "test".to_string(),
            Adapter {
                name: "test".to_string(),
                binary: std::path::PathBuf::from("test"),
                description: "".to_string(),
                version_requirement: None,
                required: false,
                aliases: HashMap::new(),
                commands: vec!["cmd1".to_string(), "cmd2".to_string()],
                validated: false,
            },
        );

        let index = build_command_index(&adapters).unwrap();
        assert!(index.commands.contains_key("cmd1"));
        assert!(index.commands.contains_key("cmd2"));
    }
}
