// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for registry dictionary module.

#[cfg(test)]
mod tests {
    use crate::registry::dictionary::{
        get_action_code, get_action_name, get_or_create_user_code, get_username,
        reload_dictionaries, set_base_path,
    };
    use crate::registry::init::init_registry;
    use std::fs;
    use std::sync::Mutex;

    // Global lock to prevent test interference (singleton caches are global)
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn setup_test_registry(name: &str) -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir().join(format!("reedbase_dict_test_{}", name));
        let _ = fs::remove_dir_all(&temp_dir);

        init_registry(&temp_dir).unwrap();
        set_base_path(temp_dir.clone());

        // Force reload to use new path
        let _ = reload_dictionaries();

        temp_dir
    }

    #[test]
    fn test_get_action_name_valid_code() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("action_name");

        let name = get_action_name(2).unwrap();
        assert_eq!(name, "update");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_action_name_unknown_code() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("unknown_code");

        let result = get_action_name(255);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_action_code_valid_name() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("action_code");

        let code = get_action_code("update").unwrap();
        assert_eq!(code, 2);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_action_code_case_insensitive() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("case_insensitive");

        let code1 = get_action_code("update").unwrap();
        let code2 = get_action_code("UPDATE").unwrap();
        let code3 = get_action_code("Update").unwrap();

        assert_eq!(code1, code2);
        assert_eq!(code2, code3);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_action_code_unknown_name() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("unknown_name");

        let result = get_action_code("nonexistent");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_username_system_user() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("system_user");

        let name = get_username(0).unwrap();
        assert_eq!(name, "system");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_username_unknown_code() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("unknown_user");

        let result = get_username(9999);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_create_new_user_code() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("create_user");

        let code = get_or_create_user_code("alice").unwrap();
        assert_eq!(code, 2); // Third user (0=system, 1=admin, 2=alice)

        // Verify it was written to file
        let users_path = temp_dir.join("registry/users.dict");
        let content = fs::read_to_string(&users_path).unwrap();
        assert!(content.contains("2|alice|"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_create_user_idempotent() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("user_idempotent");

        let code1 = get_or_create_user_code("bob").unwrap();
        let code2 = get_or_create_user_code("bob").unwrap();

        assert_eq!(code1, code2);

        // Should only appear once in file
        let users_path = temp_dir.join("registry/users.dict");
        let content = fs::read_to_string(&users_path).unwrap();
        let count = content.matches("|bob|").count();
        assert_eq!(count, 1);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_multiple_users_auto_increment() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("auto_increment");

        let code1 = get_or_create_user_code("user1").unwrap();
        let code2 = get_or_create_user_code("user2").unwrap();
        let code3 = get_or_create_user_code("user3").unwrap();

        assert_eq!(code1, 2); // 0=system, 1=admin, 2=user1
        assert_eq!(code2, 3); // 3=user2
        assert_eq!(code3, 4); // 4=user3

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_reload_dictionaries() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("reload");

        // Get initial action name
        let name1 = get_action_name(0).unwrap();
        assert_eq!(name1, "delete");

        // Manually modify actions.dict
        let actions_path = temp_dir.join("registry/actions.dict");
        let mut content = fs::read_to_string(&actions_path).unwrap();
        content = content.replace("0|delete|", "0|remove|");
        fs::write(&actions_path, content).unwrap();

        // Reload
        reload_dictionaries().unwrap();

        // Should reflect the change
        let name2 = get_action_name(0).unwrap();
        assert_eq!(name2, "remove");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_action_code_roundtrip() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = setup_test_registry("roundtrip");

        let original_name = "update";
        let code = get_action_code(original_name).unwrap();
        let name = get_action_name(code).unwrap();

        assert_eq!(name, original_name);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
