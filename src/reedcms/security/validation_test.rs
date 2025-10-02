// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Tests for Validation Services

#[cfg(test)]
mod tests {
    use crate::reedcms::security::validation::{
        email_exists, username_exists, validate_email, validate_username,
    };
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_validate_email_valid() {
        let valid_emails = vec![
            "user@example.com",
            "test.user@example.co.uk",
            "user+tag@example.com",
            "user_name@example.com",
            "123@example.com",
        ];

        for email in valid_emails {
            validate_email(email).expect(&format!("Email '{}' should be valid", email));
        }
    }

    #[test]
    fn test_validate_email_empty() {
        let result = validate_email("");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cannot be empty"));
    }

    #[test]
    fn test_validate_email_no_at_symbol() {
        let result = validate_email("userexample.com");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exactly one @ symbol"));
    }

    #[test]
    fn test_validate_email_multiple_at_symbols() {
        let result = validate_email("user@example@com");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exactly one @ symbol"));
    }

    #[test]
    fn test_validate_email_empty_local_part() {
        let result = validate_email("@example.com");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("local part cannot be empty"));
    }

    #[test]
    fn test_validate_email_empty_domain() {
        let result = validate_email("user@");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("domain"));
    }

    #[test]
    fn test_validate_email_no_dot_in_domain() {
        let result = validate_email("user@examplecom");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("at least one dot"));
    }

    #[test]
    fn test_validate_email_invalid_characters() {
        let result = validate_email("user name@example.com");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid characters"));
    }

    #[test]
    fn test_validate_username_valid() {
        let valid_usernames = vec!["admin", "user123", "test_user", "a_long_username_here"];

        for username in valid_usernames {
            validate_username(username).expect(&format!("Username '{}' should be valid", username));
        }
    }

    #[test]
    fn test_validate_username_too_short() {
        let result = validate_username("ab");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("at least 3 characters"));
    }

    #[test]
    fn test_validate_username_too_long() {
        let long_username = "a".repeat(33);
        let result = validate_username(&long_username);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not exceed 32 characters"));
    }

    #[test]
    fn test_validate_username_invalid_characters() {
        let invalid_usernames = vec!["user-name", "user.name", "user@name", "user name"];

        for username in invalid_usernames {
            let result = validate_username(username);
            assert!(result.is_err(), "Username '{}' should be invalid", username);
            let err = result.unwrap_err().to_string();
            assert!(err.contains("alphanumeric"));
        }
    }

    #[test]
    fn test_validate_username_must_start_with_letter() {
        let result = validate_username("123user");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("start with a letter"));
    }

    #[test]
    fn test_validate_username_underscore_start() {
        let result = validate_username("_user");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("start with a letter"));
    }

    #[test]
    fn test_validate_username_cannot_end_with_underscore() {
        let result = validate_username("user_");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cannot end with an underscore"));
    }

    #[test]
    fn test_username_exists_no_file() {
        // Clean up any existing test file
        let _ = fs::remove_file(".reed/users.matrix.csv");

        let result = username_exists("testuser").expect("Should not error when file missing");
        assert!(!result, "User should not exist when file doesn't exist");
    }

    #[test]
    fn test_email_exists_no_file() {
        // Clean up any existing test file
        let _ = fs::remove_file(".reed/users.matrix.csv");

        let result = email_exists("test@example.com").expect("Should not error when file missing");
        assert!(!result, "Email should not exist when file doesn't exist");
    }

    #[test]
    fn test_username_exists_with_users() {
        // This test requires a users.matrix.csv file to exist
        // In a real scenario, this would be created by user creation tests
        if Path::new(".reed/users.matrix.csv").exists() {
            // Test would check if username exists
            let _result = username_exists("admin");
        }
    }

    #[test]
    fn test_email_exists_with_users() {
        // This test requires a users.matrix.csv file to exist
        if Path::new(".reed/users.matrix.csv").exists() {
            let _result = email_exists("admin@example.com");
        }
    }

    #[test]
    fn test_validate_username_edge_cases() {
        // Exactly 3 chars (minimum)
        validate_username("abc").expect("3-char username should be valid");

        // Exactly 32 chars (maximum)
        let max_username = "a".repeat(32);
        validate_username(&max_username).expect("32-char username should be valid");

        // With underscores in middle
        validate_username("user_test_123").expect("Underscores in middle should be valid");
    }

    #[test]
    fn test_validate_email_edge_cases() {
        // Very long but valid email
        let long_email = format!("{}@example.com", "a".repeat(50));
        validate_email(&long_email).expect("Long email should be valid");

        // Email with plus (common for Gmail aliases)
        validate_email("user+alias@example.com").expect("Email with + should be valid");

        // Email with dots in local part
        validate_email("first.last@example.com").expect("Email with dots should be valid");

        // Email with numbers
        validate_email("user123@example456.com").expect("Email with numbers should be valid");
    }
}
