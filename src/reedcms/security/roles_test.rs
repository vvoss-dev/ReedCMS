// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for Role Management

#[cfg(test)]
mod tests {
    use crate::reedcms::reedstream::ReedRequest;
    use crate::reedcms::security::permissions::parse_permissions;
    use crate::reedcms::security::roles::{
        create_role, delete_role, get_role, list_roles, update_role, RoleUpdate,
    };
    use std::fs;

    fn setup_test_env() {
        let _ = fs::create_dir_all(".reed");
        let _ = fs::create_dir_all(".reed/backups");
        cleanup_test_roles();
    }

    fn cleanup_test_roles() {
        let _ = fs::remove_file(".reed/roles.matrix.csv");
        if let Ok(entries) = fs::read_dir(".reed/backups") {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    #[test]
    fn test_create_role_success() {
        setup_test_env();

        let req = ReedRequest {
            key: "editor".to_string(),
            value: Some("text[rwx],route[rw-]".to_string()),
            context: Some(r#"{"desc":"Content editor"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        let response = create_role(&req).unwrap();
        assert_eq!(response.data.rolename, "editor");
        assert_eq!(response.data.permissions.len(), 2);
        assert!(response.data.is_active);

        cleanup_test_roles();
    }

    #[test]
    fn test_create_role_with_inheritance() {
        setup_test_env();

        // Create parent role first
        let req1 = ReedRequest {
            key: "viewer".to_string(),
            value: Some("text[r--]".to_string()),
            context: Some(r#"{"desc":"Viewer"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req1).unwrap();

        // Create child role
        let req2 = ReedRequest {
            key: "editor".to_string(),
            value: Some("text[rw-]".to_string()),
            context: Some(r#"{"inherits":"viewer","desc":"Editor"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        let response = create_role(&req2).unwrap();
        assert_eq!(response.data.inherits, Some("viewer".to_string()));

        cleanup_test_roles();
    }

    #[test]
    fn test_create_role_duplicate() {
        setup_test_env();

        let req = ReedRequest {
            key: "editor".to_string(),
            value: Some("text[rwx]".to_string()),
            context: None,
            language: None,
            environment: None,
            description: None,
        };

        create_role(&req).unwrap();

        let result = create_role(&req);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("already exists"));

        cleanup_test_roles();
    }

    #[test]
    fn test_create_role_invalid_permissions() {
        setup_test_env();

        let req = ReedRequest {
            key: "invalid".to_string(),
            value: Some("invalid_permission".to_string()),
            context: None,
            language: None,
            environment: None,
            description: None,
        };

        let result = create_role(&req);
        assert!(result.is_err());

        cleanup_test_roles();
    }

    #[test]
    fn test_create_role_parent_not_found() {
        setup_test_env();

        let req = ReedRequest {
            key: "child".to_string(),
            value: Some("text[r--]".to_string()),
            context: Some(r#"{"inherits":"nonexistent"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        let result = create_role(&req);
        assert!(result.is_err());

        cleanup_test_roles();
    }

    #[test]
    fn test_create_role_circular_inheritance() {
        setup_test_env();

        // Create role1
        let req1 = ReedRequest {
            key: "role1".to_string(),
            value: Some("text[r--]".to_string()),
            context: Some(r#"{"inherits":"role2"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        // Create role2 first (pointing to role1 will create cycle)
        let req2 = ReedRequest {
            key: "role2".to_string(),
            value: Some("text[r--]".to_string()),
            context: None,
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req2).unwrap();

        let result = create_role(&req1);
        // This should fail because we check for circular inheritance
        assert!(result.is_err());

        cleanup_test_roles();
    }

    #[test]
    fn test_get_role_success() {
        setup_test_env();

        let req = ReedRequest {
            key: "editor".to_string(),
            value: Some("text[rwx]".to_string()),
            context: Some(r#"{"desc":"Editor role"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req).unwrap();

        let response = get_role("editor").unwrap();
        assert_eq!(response.data.rolename, "editor");
        assert_eq!(response.data.desc, "Editor role");

        cleanup_test_roles();
    }

    #[test]
    fn test_get_role_not_found() {
        setup_test_env();

        let result = get_role("nonexistent");
        assert!(result.is_err());

        cleanup_test_roles();
    }

    #[test]
    fn test_get_role_with_resolved_permissions() {
        setup_test_env();

        // Create parent
        let req1 = ReedRequest {
            key: "viewer".to_string(),
            value: Some("text[r--],route[r--]".to_string()),
            context: None,
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req1).unwrap();

        // Create child
        let req2 = ReedRequest {
            key: "editor".to_string(),
            value: Some("text[rw-]".to_string()),
            context: Some(r#"{"inherits":"viewer"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req2).unwrap();

        let response = get_role("editor").unwrap();

        // Should have resolved permissions from both roles
        assert!(response.data.permissions.len() >= 2);
        assert!(response
            .data
            .permissions
            .iter()
            .any(|p| p.resource == "text"));
        assert!(response
            .data
            .permissions
            .iter()
            .any(|p| p.resource == "route"));

        cleanup_test_roles();
    }

    #[test]
    fn test_list_roles_empty() {
        setup_test_env();

        let response = list_roles().unwrap();
        assert_eq!(response.data.len(), 0);

        cleanup_test_roles();
    }

    #[test]
    fn test_list_roles_multiple() {
        setup_test_env();

        for i in 1..=3 {
            let req = ReedRequest {
                key: format!("role{}", i),
                value: Some("text[r--]".to_string()),
                context: None,
                language: None,
                environment: None,
                description: None,
            };
            create_role(&req).unwrap();
        }

        let response = list_roles().unwrap();
        assert_eq!(response.data.len(), 3);

        cleanup_test_roles();
    }

    #[test]
    fn test_update_role_permissions() {
        setup_test_env();

        let req = ReedRequest {
            key: "editor".to_string(),
            value: Some("text[rw-]".to_string()),
            context: None,
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req).unwrap();

        let update = RoleUpdate {
            permissions: Some(parse_permissions("text[rwx],route[rwx]").unwrap()),
            inherits: None,
            desc: None,
            is_active: None,
        };

        let response = update_role("editor", update).unwrap();
        assert!(response
            .data
            .permissions
            .iter()
            .any(|p| p.resource == "text" && p.execute));
        assert!(response
            .data
            .permissions
            .iter()
            .any(|p| p.resource == "route"));

        cleanup_test_roles();
    }

    #[test]
    fn test_update_role_inheritance() {
        setup_test_env();

        // Create roles
        let req1 = ReedRequest {
            key: "viewer".to_string(),
            value: Some("text[r--]".to_string()),
            context: None,
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req1).unwrap();

        let req2 = ReedRequest {
            key: "editor".to_string(),
            value: Some("text[rw-]".to_string()),
            context: None,
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req2).unwrap();

        // Update editor to inherit from viewer
        let update = RoleUpdate {
            permissions: None,
            inherits: Some("viewer".to_string()),
            desc: None,
            is_active: None,
        };

        let response = update_role("editor", update).unwrap();
        assert_eq!(response.data.inherits, Some("viewer".to_string()));

        cleanup_test_roles();
    }

    #[test]
    fn test_update_role_not_found() {
        setup_test_env();

        let update = RoleUpdate {
            permissions: None,
            inherits: None,
            desc: Some("Updated".to_string()),
            is_active: None,
        };

        let result = update_role("nonexistent", update);
        assert!(result.is_err());

        cleanup_test_roles();
    }

    #[test]
    fn test_delete_role_success() {
        setup_test_env();

        let req = ReedRequest {
            key: "temp".to_string(),
            value: Some("text[r--]".to_string()),
            context: None,
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req).unwrap();

        delete_role("temp", true).unwrap();

        let result = get_role("temp");
        assert!(result.is_err());

        cleanup_test_roles();
    }

    #[test]
    fn test_delete_role_requires_confirmation() {
        setup_test_env();

        let req = ReedRequest {
            key: "temp".to_string(),
            value: Some("text[r--]".to_string()),
            context: None,
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req).unwrap();

        let result = delete_role("temp", false);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Confirmation required"));

        cleanup_test_roles();
    }

    #[test]
    fn test_delete_role_with_dependents() {
        setup_test_env();

        // Create parent
        let req1 = ReedRequest {
            key: "parent".to_string(),
            value: Some("text[r--]".to_string()),
            context: None,
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req1).unwrap();

        // Create child that inherits from parent
        let req2 = ReedRequest {
            key: "child".to_string(),
            value: Some("text[rw-]".to_string()),
            context: Some(r#"{"inherits":"parent"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req2).unwrap();

        // Try to delete parent - should fail
        let result = delete_role("parent", true);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("other roles inherit"));

        cleanup_test_roles();
    }

    #[test]
    fn test_delete_role_not_found() {
        setup_test_env();

        let result = delete_role("nonexistent", true);
        assert!(result.is_err());

        cleanup_test_roles();
    }

    #[test]
    fn test_role_lifecycle_complete() {
        setup_test_env();

        // 1. Create
        let req = ReedRequest {
            key: "lifecycle".to_string(),
            value: Some("text[rw-]".to_string()),
            context: Some(r#"{"desc":"Test role"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };
        create_role(&req).unwrap();

        // 2. Get
        let get_resp = get_role("lifecycle").unwrap();
        assert_eq!(get_resp.data.rolename, "lifecycle");

        // 3. Update
        let update = RoleUpdate {
            permissions: Some(parse_permissions("text[rwx]").unwrap()),
            inherits: None,
            desc: Some("Updated role".to_string()),
            is_active: None,
        };
        let update_resp = update_role("lifecycle", update).unwrap();
        assert_eq!(update_resp.data.desc, "Updated role");

        // 4. List
        let list_resp = list_roles().unwrap();
        assert!(list_resp.data.iter().any(|r| r.rolename == "lifecycle"));

        // 5. Delete
        delete_role("lifecycle", true).unwrap();

        // 6. Verify deletion
        let result = get_role("lifecycle");
        assert!(result.is_err());

        cleanup_test_roles();
    }
}
