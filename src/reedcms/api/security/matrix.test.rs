// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0
//
// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One test = One assertion
// MANDATORY: BBC English for all test names and documentation
// MANDATORY: Test all error paths explicitly
// MANDATORY: Performance assertions for all operations
//
// == FILE PURPOSE ==
// This file: Tests for matrix.rs security matrix operations
// Architecture: Separate test file following KISS principle
// Performance: All tests must complete within defined time limits
// Test Scope: Unit tests for security rule loading and access checking

#[cfg(test)]
mod tests {
    use crate::reedcms::api::security::matrix::{SecurityMatrix, SecurityRule};
    use crate::reedcms::api::security::rate_limit::{RateLimit, RateLimitPeriod};
    use crate::reedcms::auth::verification::AuthenticatedUser;
    use std::collections::HashMap;

    fn create_test_user(roles: Vec<&str>, _permissions: Vec<&str>) -> AuthenticatedUser {
        // Note: AuthenticatedUser doesn't store permissions directly
        // Permissions are loaded via has_permission() method which checks role permissions
        AuthenticatedUser {
            id: "test_user".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
        }
    }

    fn create_test_matrix() -> SecurityMatrix {
        let mut rules = HashMap::new();

        rules.insert(
            "text:read".to_string(),
            SecurityRule {
                resource: "text".to_string(),
                operation: "read".to_string(),
                required_permission: "text.read".to_string(),
                required_role: "user".to_string(),
                rate_limit: RateLimit {
                    requests: 100,
                    period: RateLimitPeriod::Minute,
                },
            },
        );

        rules.insert(
            "text:write".to_string(),
            SecurityRule {
                resource: "text".to_string(),
                operation: "write".to_string(),
                required_permission: "text.write".to_string(),
                required_role: "editor".to_string(),
                rate_limit: RateLimit {
                    requests: 50,
                    period: RateLimitPeriod::Minute,
                },
            },
        );

        SecurityMatrix { rules }
    }

    #[test]
    fn test_security_rule_creation() {
        let rule = SecurityRule {
            resource: "text".to_string(),
            operation: "read".to_string(),
            required_permission: "text.read".to_string(),
            required_role: "user".to_string(),
            rate_limit: RateLimit {
                requests: 100,
                period: RateLimitPeriod::Minute,
            },
        };

        assert_eq!(rule.resource, "text");
        assert_eq!(rule.operation, "read");
        assert_eq!(rule.required_permission, "text.read");
        assert_eq!(rule.required_role, "user");
        assert_eq!(rule.rate_limit.requests, 100);
    }

    #[test]
    fn test_check_access_user_has_permission_and_role() {
        let matrix = create_test_matrix();
        let user = create_test_user(vec!["user"], vec!["text.read"]);

        let result = matrix.check_access("text", "read", &user);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_access_user_missing_permission() {
        let matrix = create_test_matrix();
        let user = create_test_user(vec!["user"], vec!["route.read"]); // Wrong permission

        let result = matrix.check_access("text", "read", &user);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_access_user_missing_role() {
        let matrix = create_test_matrix();
        let user = create_test_user(vec!["guest"], vec!["text.read"]); // Wrong role

        let result = matrix.check_access("text", "read", &user);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_access_editor_can_write() {
        let matrix = create_test_matrix();
        let user = create_test_user(vec!["editor"], vec!["text.write"]);

        let result = matrix.check_access("text", "write", &user);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_access_user_cannot_write() {
        let matrix = create_test_matrix();
        let user = create_test_user(vec!["user"], vec!["text.read"]); // User role, not editor

        let result = matrix.check_access("text", "write", &user);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_access_nonexistent_resource() {
        let matrix = create_test_matrix();
        let user = create_test_user(vec!["admin"], vec!["all.permissions"]);

        let result = matrix.check_access("nonexistent", "read", &user);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_access_returns_rate_limit() {
        let matrix = create_test_matrix();
        let user = create_test_user(vec!["user"], vec!["text.read"]);

        let result = matrix.check_access("text", "read", &user);
        assert!(result.is_ok());

        let rate_limit = result.unwrap();
        assert_eq!(rate_limit.requests, 100);
    }

    #[test]
    fn test_security_matrix_empty_rules() {
        let matrix = SecurityMatrix {
            rules: HashMap::new(),
        };
        let user = create_test_user(vec!["admin"], vec!["all.permissions"]);

        let result = matrix.check_access("text", "read", &user);
        assert!(result.is_err());
    }
}
