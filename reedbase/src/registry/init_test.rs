// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for registry initialisation module.

#[cfg(test)]
mod tests {
    use crate::registry::init::{init_registry, validate_dictionaries};
    use std::fs;
    use std::path::Path;

    fn create_temp_dir(name: &str) -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir().join(format!("reedbase_test_registry_{}", name));
        let _ = fs::remove_dir_all(&temp_dir); // Clean up previous runs
        temp_dir
    }

    #[test]
    fn test_init_registry_creates_directories() {
        let temp_dir = create_temp_dir("init_dirs");

        init_registry(&temp_dir).unwrap();

        assert!(temp_dir.join("registry").exists());
        assert!(temp_dir.join("registry/actions.dict").exists());
        assert!(temp_dir.join("registry/users.dict").exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_init_registry_idempotent() {
        let temp_dir = create_temp_dir("init_idempotent");

        // First call
        init_registry(&temp_dir).unwrap();

        // Second call should not fail
        init_registry(&temp_dir).unwrap();

        assert!(temp_dir.join("registry").exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_create_default_action_dict() {
        let temp_dir = create_temp_dir("action_dict");

        init_registry(&temp_dir).unwrap();

        let actions_path = temp_dir.join("registry/actions.dict");
        let content = fs::read_to_string(&actions_path).unwrap();

        // Check header
        assert!(content.contains("code|name|description"));

        // Check some default actions
        assert!(content.contains("0|delete|Delete operation"));
        assert!(content.contains("1|create|Create new entry"));
        assert!(content.contains("2|update|Update existing entry"));

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_create_default_user_dict() {
        let temp_dir = create_temp_dir("user_dict");

        init_registry(&temp_dir).unwrap();

        let users_path = temp_dir.join("registry/users.dict");
        let content = fs::read_to_string(&users_path).unwrap();

        // Check header
        assert!(content.contains("code|username|created_at"));

        // Check system user
        assert!(content.contains("0|system|"));

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_validate_dictionaries_valid() {
        let temp_dir = create_temp_dir("validate_valid");

        init_registry(&temp_dir).unwrap();

        // Should pass validation
        let result = validate_dictionaries(&temp_dir);
        assert!(result.is_ok());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_validate_dictionaries_corrupted_csv() {
        let temp_dir = create_temp_dir("validate_corrupted");

        init_registry(&temp_dir).unwrap();

        // Corrupt the actions.dict file (invalid CSV)
        let actions_path = temp_dir.join("registry/actions.dict");
        fs::write(&actions_path, "code|name|description\ninvalid").unwrap();

        let result = validate_dictionaries(&temp_dir);
        assert!(result.is_err());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_validate_dictionaries_duplicate_code() {
        let temp_dir = create_temp_dir("validate_duplicate");

        init_registry(&temp_dir).unwrap();

        // Add duplicate code to actions.dict
        let actions_path = temp_dir.join("registry/actions.dict");
        let mut content = fs::read_to_string(&actions_path).unwrap();
        content.push_str("0|duplicate|Duplicate code\n");
        fs::write(&actions_path, content).unwrap();

        let result = validate_dictionaries(&temp_dir);
        assert!(result.is_err());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_validate_dictionaries_invalid_action_code() {
        let temp_dir = create_temp_dir("validate_invalid_code");

        init_registry(&temp_dir).unwrap();

        // Add invalid action code (not 0-255)
        let actions_path = temp_dir.join("registry/actions.dict");
        let mut content = fs::read_to_string(&actions_path).unwrap();
        content.push_str("999|invalid|Invalid code\n");
        fs::write(&actions_path, content).unwrap();

        let result = validate_dictionaries(&temp_dir);
        assert!(result.is_err());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
