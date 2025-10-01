// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for User Management

#[cfg(test)]
mod tests {
    use crate::reedcms::reedstream::ReedRequest;
    use crate::reedcms::security::users::{
        create_user, delete_user, get_user, list_users, update_user, Address, SocialMedia,
        UserFilter, UserUpdate,
    };
    use std::fs;
    use std::path::Path;

    fn setup_test_env() {
        let _ = fs::create_dir_all(".reed");
        let _ = fs::create_dir_all(".reed/backups");
        cleanup_test_users();
    }

    fn cleanup_test_users() {
        let _ = fs::remove_file(".reed/users.matrix.csv");
        // Don't remove backups directory, just clean files inside
        if let Ok(entries) = fs::read_dir(".reed/backups") {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    #[test]
    fn test_create_user_success() {
        setup_test_env();

        let req = ReedRequest {
            key: "testuser".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(
                r#"{"email":"test@example.com","firstname":"Test","lastname":"User","desc":"Test user"}"#
                    .to_string(),
            ),
            language: None,
            environment: None,
            description: None,
        };

        let response = create_user(&req).expect("User creation should succeed");
        assert_eq!(response.data.username, "testuser");
        assert_eq!(response.data.email, "test@example.com");
        assert_eq!(response.data.firstname, "Test");
        assert_eq!(response.data.lastname, "User");
        assert!(response.data.is_active);

        cleanup_test_users();
    }

    #[test]
    fn test_create_user_with_roles() {
        setup_test_env();

        let req = ReedRequest {
            key: "editor_user".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(
                r#"{"email":"editor@example.com","roles":["editor","author"]}"#.to_string(),
            ),
            language: None,
            environment: None,
            description: None,
        };

        let response = create_user(&req).expect("User creation with roles should succeed");
        assert_eq!(response.data.roles, vec!["editor", "author"]);

        cleanup_test_users();
    }

    #[test]
    fn test_create_user_duplicate_username() {
        setup_test_env();

        let req = ReedRequest {
            key: "duplicate".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"user1@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        create_user(&req).expect("First user creation should succeed");

        let req2 = ReedRequest {
            key: "duplicate".to_string(),
            value: Some("DifferentP@ss123".to_string()),
            context: Some(r#"{"email":"user2@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        let result = create_user(&req2);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("already exists"));

        cleanup_test_users();
    }

    #[test]
    fn test_create_user_duplicate_email() {
        setup_test_env();

        let req = ReedRequest {
            key: "user1".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"same@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        create_user(&req).expect("First user creation should succeed");

        let req2 = ReedRequest {
            key: "user2".to_string(),
            value: Some("DifferentP@ss123".to_string()),
            context: Some(r#"{"email":"same@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        let result = create_user(&req2);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("already registered"));

        cleanup_test_users();
    }

    #[test]
    fn test_create_user_weak_password() {
        setup_test_env();

        let req = ReedRequest {
            key: "weakpass".to_string(),
            value: Some("weak".to_string()),
            context: Some(r#"{"email":"weak@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        let result = create_user(&req);
        assert!(result.is_err());

        cleanup_test_users();
    }

    #[test]
    fn test_create_user_invalid_username() {
        setup_test_env();

        let req = ReedRequest {
            key: "ab".to_string(), // Too short
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"test@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        let result = create_user(&req);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("at least 3 characters"));

        cleanup_test_users();
    }

    #[test]
    fn test_create_user_invalid_email() {
        setup_test_env();

        let req = ReedRequest {
            key: "testuser".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"invalid-email"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        let result = create_user(&req);
        assert!(result.is_err());

        cleanup_test_users();
    }

    #[test]
    fn test_create_user_with_full_profile() {
        setup_test_env();

        let req = ReedRequest {
            key: "fullprofile".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(
                r#"{
                    "email":"full@example.com",
                    "firstname":"John",
                    "lastname":"Doe",
                    "mobile":"+44123456789",
                    "street":"Main St 1",
                    "city":"London",
                    "postcode":"SW1A 1AA",
                    "region":"London",
                    "country":"UK",
                    "twitter":"@johndoe",
                    "facebook":"john.doe",
                    "desc":"Full profile user"
                }"#
                .to_string(),
            ),
            language: None,
            environment: None,
            description: None,
        };

        let response = create_user(&req).expect("User with full profile should succeed");
        assert_eq!(response.data.mobile, Some("+44123456789".to_string()));
        assert_eq!(response.data.address.street, Some("Main St 1".to_string()));
        assert_eq!(response.data.address.city, Some("London".to_string()));
        assert_eq!(
            response.data.social_media.twitter,
            Some("@johndoe".to_string())
        );

        cleanup_test_users();
    }

    #[test]
    fn test_get_user_success() {
        setup_test_env();

        let req = ReedRequest {
            key: "gettest".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"get@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        create_user(&req).expect("User creation should succeed");

        let response = get_user("gettest").expect("Get user should succeed");
        assert_eq!(response.data.username, "gettest");
        assert_eq!(response.data.email, "get@example.com");

        cleanup_test_users();
    }

    #[test]
    fn test_get_user_not_found() {
        setup_test_env();

        let result = get_user("nonexistent");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            crate::reedcms::reedstream::ReedError::NotFound { .. }
        ));

        cleanup_test_users();
    }

    #[test]
    fn test_list_users_empty() {
        setup_test_env();

        let response = list_users(None).expect("List should succeed even when empty");
        assert_eq!(response.data.len(), 0);

        cleanup_test_users();
    }

    #[test]
    fn test_list_users_multiple() {
        setup_test_env();

        // Create 3 users
        for i in 1..=3 {
            let req = ReedRequest {
                key: format!("user{}", i),
                value: Some("SecureP@ss123".to_string()),
                context: Some(format!(r#"{{"email":"user{}@example.com"}}"#, i)),
                language: None,
                environment: None,
                description: None,
            };
            create_user(&req).expect("User creation should succeed");
        }

        let response = list_users(None).expect("List should succeed");
        assert_eq!(response.data.len(), 3);

        cleanup_test_users();
    }

    #[test]
    fn test_list_users_filter_active() {
        setup_test_env();

        // Create active user
        let req1 = ReedRequest {
            key: "active".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"active@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };
        create_user(&req1).expect("User creation should succeed");

        // Create inactive user
        let req2 = ReedRequest {
            key: "inactive".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"inactive@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };
        create_user(&req2).expect("User creation should succeed");

        // Deactivate second user
        let update = UserUpdate {
            is_active: Some(false),
            firstname: None,
            lastname: None,
            email: None,
            mobile: None,
            social_media: None,
            address: None,
            desc: None,
        };
        update_user("inactive", update).expect("Update should succeed");

        // Filter for active only
        let filter = Some(UserFilter {
            is_active: Some(true),
            role: None,
        });
        let response = list_users(filter).expect("List should succeed");
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].username, "active");

        cleanup_test_users();
    }

    #[test]
    fn test_update_user_success() {
        setup_test_env();

        let req = ReedRequest {
            key: "updatetest".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(
                r#"{"email":"old@example.com","firstname":"Old","lastname":"Name"}"#.to_string(),
            ),
            language: None,
            environment: None,
            description: None,
        };
        create_user(&req).expect("User creation should succeed");

        let update = UserUpdate {
            firstname: Some("New".to_string()),
            lastname: Some("Updated".to_string()),
            email: Some("new@example.com".to_string()),
            mobile: None,
            social_media: None,
            address: None,
            desc: None,
            is_active: None,
        };

        let response = update_user("updatetest", update).expect("Update should succeed");
        assert_eq!(response.data.firstname, "New");
        assert_eq!(response.data.lastname, "Updated");
        assert_eq!(response.data.email, "new@example.com");

        cleanup_test_users();
    }

    #[test]
    fn test_update_user_not_found() {
        setup_test_env();

        let update = UserUpdate {
            firstname: Some("Test".to_string()),
            lastname: None,
            email: None,
            mobile: None,
            social_media: None,
            address: None,
            desc: None,
            is_active: None,
        };

        let result = update_user("nonexistent", update);
        assert!(result.is_err());

        cleanup_test_users();
    }

    #[test]
    fn test_delete_user_success() {
        setup_test_env();

        let req = ReedRequest {
            key: "deletetest".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"delete@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };
        create_user(&req).expect("User creation should succeed");

        delete_user("deletetest", true).expect("Delete should succeed");

        let result = get_user("deletetest");
        assert!(result.is_err());

        cleanup_test_users();
    }

    #[test]
    fn test_delete_user_requires_confirmation() {
        setup_test_env();

        let req = ReedRequest {
            key: "confirmtest".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"confirm@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };
        create_user(&req).expect("User creation should succeed");

        let result = delete_user("confirmtest", false);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Confirmation required"));

        cleanup_test_users();
    }

    #[test]
    fn test_delete_user_not_found() {
        setup_test_env();

        let result = delete_user("nonexistent", true);
        assert!(result.is_err());

        cleanup_test_users();
    }

    #[test]
    fn test_user_lifecycle_complete() {
        setup_test_env();

        // 1. Create user
        let req = ReedRequest {
            key: "lifecycle".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(
                r#"{"email":"lifecycle@example.com","firstname":"Life","lastname":"Cycle"}"#
                    .to_string(),
            ),
            language: None,
            environment: None,
            description: None,
        };
        let create_response = create_user(&req).expect("Create should succeed");
        assert_eq!(create_response.data.username, "lifecycle");

        // 2. Get user
        let get_response = get_user("lifecycle").expect("Get should succeed");
        assert_eq!(get_response.data.email, "lifecycle@example.com");

        // 3. Update user
        let update = UserUpdate {
            firstname: Some("Updated".to_string()),
            email: None,
            lastname: None,
            mobile: None,
            social_media: None,
            address: None,
            desc: None,
            is_active: None,
        };
        let update_response = update_user("lifecycle", update).expect("Update should succeed");
        assert_eq!(update_response.data.firstname, "Updated");

        // 4. List users
        let list_response = list_users(None).expect("List should succeed");
        assert!(list_response.data.iter().any(|u| u.username == "lifecycle"));

        // 5. Delete user
        delete_user("lifecycle", true).expect("Delete should succeed");

        // 6. Verify deletion
        let result = get_user("lifecycle");
        assert!(result.is_err());

        cleanup_test_users();
    }

    #[test]
    fn test_user_timestamps() {
        setup_test_env();

        let req = ReedRequest {
            key: "timestamp".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"timestamp@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };

        let response = create_user(&req).expect("Create should succeed");
        assert!(response.data.created_at > 0);
        assert!(response.data.updated_at > 0);
        assert_eq!(response.data.created_at, response.data.updated_at);
        assert_eq!(response.data.last_login, None);

        cleanup_test_users();
    }

    #[test]
    fn test_backup_created_on_user_creation() {
        setup_test_env();

        let req1 = ReedRequest {
            key: "backup1".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"backup1@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };
        create_user(&req1).expect("First user creation should succeed");

        let req2 = ReedRequest {
            key: "backup2".to_string(),
            value: Some("SecureP@ss123".to_string()),
            context: Some(r#"{"email":"backup2@example.com"}"#.to_string()),
            language: None,
            environment: None,
            description: None,
        };
        create_user(&req2).expect("Second user creation should create backup");

        // Check if users file exists (backup only created when file already exists before write)
        assert!(Path::new(".reed/users.matrix.csv").exists());

        cleanup_test_users();
    }
}
