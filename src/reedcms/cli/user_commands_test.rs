// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::user_commands::*;
    use std::collections::HashMap;
    use std::fs;

    const TEST_USERS_FILE: &str = ".reed/users.matrix.csv";

    /// Cleanup test user data.
    fn cleanup_test_users() {
        fs::remove_file(TEST_USERS_FILE).ok();
    }

    #[test]
    fn test_create_user_success() {
        cleanup_test_users();

        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("email".to_string(), "test@example.com".to_string());
        flags.insert("password".to_string(), "SecurePassword123!".to_string());
        flags.insert("roles".to_string(), "editor".to_string());

        let result = create_user(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("✓ User 'testuser' created"));
        assert!(response.data.contains("test@example.com"));

        cleanup_test_users();
    }

    #[test]
    fn test_create_user_missing_username() {
        let args = vec![];
        let flags = HashMap::new();

        let result = create_user(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_user_missing_email() {
        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("password".to_string(), "SecurePassword123!".to_string());
        flags.insert("roles".to_string(), "editor".to_string());

        let result = create_user(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_user_missing_password() {
        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("email".to_string(), "test@example.com".to_string());
        flags.insert("roles".to_string(), "editor".to_string());

        let result = create_user(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_user_missing_roles() {
        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("email".to_string(), "test@example.com".to_string());
        flags.insert("password".to_string(), "SecurePassword123!".to_string());

        let result = create_user(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_user_weak_password() {
        cleanup_test_users();

        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("email".to_string(), "test@example.com".to_string());
        flags.insert("password".to_string(), "weak".to_string());
        flags.insert("roles".to_string(), "editor".to_string());

        let result = create_user(&args, &flags);
        assert!(result.is_err());

        cleanup_test_users();
    }

    #[test]
    fn test_create_user_with_optional_fields() {
        cleanup_test_users();

        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("email".to_string(), "test@example.com".to_string());
        flags.insert("password".to_string(), "SecurePassword123!".to_string());
        flags.insert("roles".to_string(), "editor,author".to_string());
        flags.insert("firstname".to_string(), "John".to_string());
        flags.insert("lastname".to_string(), "Doe".to_string());
        flags.insert("mobile".to_string(), "+44123456789".to_string());

        let result = create_user(&args, &flags);
        assert!(result.is_ok());

        cleanup_test_users();
    }

    #[test]
    fn test_list_users_empty() {
        cleanup_test_users();

        let flags = HashMap::new();
        let result = list_users(&flags);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.data.contains("No users found") || response.data.contains("0 user"));

        cleanup_test_users();
    }

    #[test]
    fn test_list_users_table_format() {
        cleanup_test_users();

        // Create test user first
        let args = vec!["testuser".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("email".to_string(), "test@example.com".to_string());
        create_flags.insert("password".to_string(), "SecurePassword123!".to_string());
        create_flags.insert("roles".to_string(), "editor".to_string());
        create_user(&args, &create_flags).ok();

        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "table".to_string());

        let result = list_users(&flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("Username"));
        assert!(response.data.contains("Email"));

        cleanup_test_users();
    }

    #[test]
    fn test_list_users_json_format() {
        cleanup_test_users();

        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "json".to_string());

        let result = list_users(&flags);
        assert!(result.is_ok());

        cleanup_test_users();
    }

    #[test]
    fn test_list_users_csv_format() {
        cleanup_test_users();

        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "csv".to_string());

        let result = list_users(&flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("username,email"));

        cleanup_test_users();
    }

    #[test]
    fn test_show_user_missing_username() {
        let args = vec![];
        let result = show_user(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_show_user_not_found() {
        cleanup_test_users();

        let args = vec!["nonexistent".to_string()];
        let result = show_user(&args);
        assert!(result.is_err());

        cleanup_test_users();
    }

    #[test]
    fn test_show_user_success() {
        cleanup_test_users();

        // Create test user
        let create_args = vec!["testuser".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("email".to_string(), "test@example.com".to_string());
        create_flags.insert("password".to_string(), "SecurePassword123!".to_string());
        create_flags.insert("roles".to_string(), "editor".to_string());
        create_user(&create_args, &create_flags).ok();

        let args = vec!["testuser".to_string()];
        let result = show_user(&args);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("User: testuser"));
        assert!(response.data.contains("test@example.com"));

        cleanup_test_users();
    }

    #[test]
    fn test_update_user_missing_username() {
        let args = vec![];
        let flags = HashMap::new();
        let result = update_user(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_user_no_flags() {
        let args = vec!["testuser".to_string()];
        let flags = HashMap::new();
        let result = update_user(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_user_success() {
        cleanup_test_users();

        // Create test user
        let create_args = vec!["testuser".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("email".to_string(), "test@example.com".to_string());
        create_flags.insert("password".to_string(), "SecurePassword123!".to_string());
        create_flags.insert("roles".to_string(), "editor".to_string());
        create_user(&create_args, &create_flags).ok();

        // Update user
        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("email".to_string(), "newemail@example.com".to_string());
        flags.insert("firstname".to_string(), "John".to_string());

        let result = update_user(&args, &flags);
        assert!(result.is_ok());

        cleanup_test_users();
    }

    #[test]
    fn test_delete_user_missing_username() {
        let args = vec![];
        let flags = HashMap::new();
        let result = delete_user(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_user_missing_force() {
        let args = vec!["testuser".to_string()];
        let flags = HashMap::new();
        let result = delete_user(&args, &flags);
        assert!(result.is_err());
        // Should require --force flag
    }

    #[test]
    fn test_delete_user_success() {
        cleanup_test_users();

        // Create test user
        let create_args = vec!["testuser".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("email".to_string(), "test@example.com".to_string());
        create_flags.insert("password".to_string(), "SecurePassword123!".to_string());
        create_flags.insert("roles".to_string(), "editor".to_string());
        create_user(&create_args, &create_flags).ok();

        // Delete user
        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("force".to_string(), "true".to_string());

        let result = delete_user(&args, &flags);
        assert!(result.is_ok());

        cleanup_test_users();
    }

    #[test]
    fn test_change_password_missing_username() {
        let args = vec![];
        let flags = HashMap::new();
        let result = change_password(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_change_password_missing_new() {
        let args = vec!["testuser".to_string()];
        let flags = HashMap::new();
        let result = change_password(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_change_password_weak() {
        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("new".to_string(), "weak".to_string());

        let result = change_password(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_manage_roles_missing_username() {
        let args = vec![];
        let flags = HashMap::new();
        let result = manage_roles(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_manage_roles_show() {
        cleanup_test_users();

        // Create test user
        let create_args = vec!["testuser".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("email".to_string(), "test@example.com".to_string());
        create_flags.insert("password".to_string(), "SecurePassword123!".to_string());
        create_flags.insert("roles".to_string(), "editor".to_string());
        create_user(&create_args, &create_flags).ok();

        // Show roles
        let args = vec!["testuser".to_string()];
        let flags = HashMap::new();

        let result = manage_roles(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("Current roles"));

        cleanup_test_users();
    }

    #[test]
    fn test_manage_roles_add() {
        cleanup_test_users();

        // Create test user
        let create_args = vec!["testuser".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("email".to_string(), "test@example.com".to_string());
        create_flags.insert("password".to_string(), "SecurePassword123!".to_string());
        create_flags.insert("roles".to_string(), "editor".to_string());
        create_user(&create_args, &create_flags).ok();

        // Add role
        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("add".to_string(), "author".to_string());

        let result = manage_roles(&args, &flags);
        assert!(result.is_ok());

        cleanup_test_users();
    }

    #[test]
    fn test_manage_roles_remove() {
        cleanup_test_users();

        // Create test user with multiple roles
        let create_args = vec!["testuser".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("email".to_string(), "test@example.com".to_string());
        create_flags.insert("password".to_string(), "SecurePassword123!".to_string());
        create_flags.insert("roles".to_string(), "editor,author".to_string());
        create_user(&create_args, &create_flags).ok();

        // Remove role
        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("remove".to_string(), "author".to_string());

        let result = manage_roles(&args, &flags);
        assert!(result.is_ok());

        cleanup_test_users();
    }

    #[test]
    fn test_manage_roles_set() {
        cleanup_test_users();

        // Create test user
        let create_args = vec!["testuser".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("email".to_string(), "test@example.com".to_string());
        create_flags.insert("password".to_string(), "SecurePassword123!".to_string());
        create_flags.insert("roles".to_string(), "editor".to_string());
        create_user(&create_args, &create_flags).ok();

        // Set roles (replace all)
        let args = vec!["testuser".to_string()];
        let mut flags = HashMap::new();
        flags.insert("set".to_string(), "admin,editor,author".to_string());

        let result = manage_roles(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("admin"));
        assert!(response.data.contains("editor"));
        assert!(response.data.contains("author"));

        cleanup_test_users();
    }
}
