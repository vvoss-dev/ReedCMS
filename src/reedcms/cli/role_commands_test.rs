// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::super::role_commands::*;
    use std::collections::HashMap;
    use std::fs;

    const TEST_ROLES_FILE: &str = ".reed/roles.matrix.csv";

    /// Cleanup test role data.
    fn cleanup_test_roles() {
        fs::remove_file(TEST_ROLES_FILE).ok();
    }

    #[test]
    fn test_create_role_success() {
        cleanup_test_roles();

        let args = vec!["testrole".to_string()];
        let mut flags = HashMap::new();
        flags.insert(
            "permissions".to_string(),
            "text[rwx],route[rw-]".to_string(),
        );
        flags.insert("desc".to_string(), "Test role description".to_string());

        let result = create_role(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("✓ Role 'testrole' created"));
        assert!(response.data.contains("text[rwx]"));

        cleanup_test_roles();
    }

    #[test]
    fn test_create_role_with_inheritance() {
        cleanup_test_roles();

        // Create parent role first
        let parent_args = vec!["viewer".to_string()];
        let mut parent_flags = HashMap::new();
        parent_flags.insert("permissions".to_string(), "*[r--]".to_string());
        parent_flags.insert("desc".to_string(), "Read-only viewer".to_string());
        create_role(&parent_args, &parent_flags).ok();

        // Create child role
        let args = vec!["editor".to_string()];
        let mut flags = HashMap::new();
        flags.insert("permissions".to_string(), "text[rwx]".to_string());
        flags.insert("desc".to_string(), "Content editor".to_string());
        flags.insert("inherit".to_string(), "viewer".to_string());

        let result = create_role(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("Inherits: viewer"));

        cleanup_test_roles();
    }

    #[test]
    fn test_create_role_missing_rolename() {
        let args = vec![];
        let flags = HashMap::new();

        let result = create_role(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_role_missing_permissions() {
        let args = vec!["testrole".to_string()];
        let mut flags = HashMap::new();
        flags.insert("desc".to_string(), "Test description".to_string());

        let result = create_role(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_role_missing_desc() {
        let args = vec!["testrole".to_string()];
        let mut flags = HashMap::new();
        flags.insert("permissions".to_string(), "text[rwx]".to_string());

        let result = create_role(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_roles_empty() {
        cleanup_test_roles();

        let flags = HashMap::new();
        let result = list_roles(&flags);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.data.contains("No roles found") || response.data.contains("0 role"));

        cleanup_test_roles();
    }

    #[test]
    fn test_list_roles_table_format() {
        cleanup_test_roles();

        // Create test role
        let args = vec!["testrole".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("permissions".to_string(), "text[rwx]".to_string());
        create_flags.insert("desc".to_string(), "Test role".to_string());
        create_role(&args, &create_flags).ok();

        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "table".to_string());

        let result = list_roles(&flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("Role"));

        cleanup_test_roles();
    }

    #[test]
    fn test_list_roles_json_format() {
        cleanup_test_roles();

        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "json".to_string());

        let result = list_roles(&flags);
        assert!(result.is_ok());

        cleanup_test_roles();
    }

    #[test]
    fn test_list_roles_csv_format() {
        cleanup_test_roles();

        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "csv".to_string());

        let result = list_roles(&flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("rolename,permissions"));

        cleanup_test_roles();
    }

    #[test]
    fn test_show_role_missing_rolename() {
        let args = vec![];
        let result = show_role(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_show_role_not_found() {
        cleanup_test_roles();

        let args = vec!["nonexistent".to_string()];
        let result = show_role(&args);
        assert!(result.is_err());

        cleanup_test_roles();
    }

    #[test]
    fn test_show_role_success() {
        cleanup_test_roles();

        // Create test role
        let create_args = vec!["testrole".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("permissions".to_string(), "text[rwx]".to_string());
        create_flags.insert("desc".to_string(), "Test role".to_string());
        create_role(&create_args, &create_flags).ok();

        let args = vec!["testrole".to_string()];
        let result = show_role(&args);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("Role: testrole"));

        cleanup_test_roles();
    }

    #[test]
    fn test_update_role_missing_rolename() {
        let args = vec![];
        let flags = HashMap::new();
        let result = update_role(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_role_no_flags() {
        let args = vec!["testrole".to_string()];
        let flags = HashMap::new();
        let result = update_role(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_role_success() {
        cleanup_test_roles();

        // Create test role
        let create_args = vec!["testrole".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("permissions".to_string(), "text[rwx]".to_string());
        create_flags.insert("desc".to_string(), "Test role".to_string());
        create_role(&create_args, &create_flags).ok();

        // Update role
        let args = vec!["testrole".to_string()];
        let mut flags = HashMap::new();
        flags.insert(
            "permissions".to_string(),
            "text[rwx],route[rw-]".to_string(),
        );
        flags.insert("desc".to_string(), "Updated description".to_string());

        let result = update_role(&args, &flags);
        assert!(result.is_ok());

        cleanup_test_roles();
    }

    #[test]
    fn test_delete_role_missing_rolename() {
        let args = vec![];
        let flags = HashMap::new();
        let result = delete_role(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_role_missing_force() {
        let args = vec!["testrole".to_string()];
        let flags = HashMap::new();
        let result = delete_role(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_role_success() {
        cleanup_test_roles();

        // Create test role
        let create_args = vec!["testrole".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("permissions".to_string(), "text[rwx]".to_string());
        create_flags.insert("desc".to_string(), "Test role".to_string());
        create_role(&create_args, &create_flags).ok();

        // Delete role
        let args = vec!["testrole".to_string()];
        let mut flags = HashMap::new();
        flags.insert("force".to_string(), "true".to_string());

        let result = delete_role(&args, &flags);
        assert!(result.is_ok());

        cleanup_test_roles();
    }

    #[test]
    fn test_manage_permissions_missing_rolename() {
        let args = vec![];
        let flags = HashMap::new();
        let result = manage_permissions(&args, &flags);
        assert!(result.is_err());
    }

    #[test]
    fn test_manage_permissions_show() {
        cleanup_test_roles();

        // Create test role
        let create_args = vec!["testrole".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("permissions".to_string(), "text[rwx]".to_string());
        create_flags.insert("desc".to_string(), "Test role".to_string());
        create_role(&create_args, &create_flags).ok();

        // Show permissions
        let args = vec!["testrole".to_string()];
        let flags = HashMap::new();

        let result = manage_permissions(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("Current permissions"));

        cleanup_test_roles();
    }

    #[test]
    fn test_manage_permissions_add() {
        cleanup_test_roles();

        // Create test role
        let create_args = vec!["testrole".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("permissions".to_string(), "text[rwx]".to_string());
        create_flags.insert("desc".to_string(), "Test role".to_string());
        create_role(&create_args, &create_flags).ok();

        // Add permission
        let args = vec!["testrole".to_string()];
        let mut flags = HashMap::new();
        flags.insert("add".to_string(), "route[rw-]".to_string());

        let result = manage_permissions(&args, &flags);
        assert!(result.is_ok());

        cleanup_test_roles();
    }

    #[test]
    fn test_manage_permissions_remove() {
        cleanup_test_roles();

        // Create test role with multiple permissions
        let create_args = vec!["testrole".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert(
            "permissions".to_string(),
            "text[rwx],route[rw-]".to_string(),
        );
        create_flags.insert("desc".to_string(), "Test role".to_string());
        create_role(&create_args, &create_flags).ok();

        // Remove permission
        let args = vec!["testrole".to_string()];
        let mut flags = HashMap::new();
        flags.insert("remove".to_string(), "route[rw-]".to_string());

        let result = manage_permissions(&args, &flags);
        assert!(result.is_ok());

        cleanup_test_roles();
    }

    #[test]
    fn test_manage_permissions_set() {
        cleanup_test_roles();

        // Create test role
        let create_args = vec!["testrole".to_string()];
        let mut create_flags = HashMap::new();
        create_flags.insert("permissions".to_string(), "text[rwx]".to_string());
        create_flags.insert("desc".to_string(), "Test role".to_string());
        create_role(&create_args, &create_flags).ok();

        // Set permissions (replace all)
        let args = vec!["testrole".to_string()];
        let mut flags = HashMap::new();
        flags.insert(
            "set".to_string(),
            "text[r--],route[r--],content[r--]".to_string(),
        );

        let result = manage_permissions(&args, &flags);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("New permissions"));

        cleanup_test_roles();
    }
}
