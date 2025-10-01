// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for Permission Parsing and Checking

#[cfg(test)]
mod tests {
    use crate::reedcms::security::permissions::{
        format_permission, format_permissions, parse_permission, parse_permissions,
        validate_permission_syntax, Permission,
    };

    #[test]
    fn test_parse_permission_full() {
        let perm = parse_permission("text[rwx]").unwrap();
        assert_eq!(perm.resource, "text");
        assert!(perm.read);
        assert!(perm.write);
        assert!(perm.execute);
    }

    #[test]
    fn test_parse_permission_read_write() {
        let perm = parse_permission("route[rw-]").unwrap();
        assert_eq!(perm.resource, "route");
        assert!(perm.read);
        assert!(perm.write);
        assert!(!perm.execute);
    }

    #[test]
    fn test_parse_permission_read_only() {
        let perm = parse_permission("content[r--]").unwrap();
        assert_eq!(perm.resource, "content");
        assert!(perm.read);
        assert!(!perm.write);
        assert!(!perm.execute);
    }

    #[test]
    fn test_parse_permission_wildcard() {
        let perm = parse_permission("*[rwx]").unwrap();
        assert_eq!(perm.resource, "*");
        assert!(perm.read && perm.write && perm.execute);
    }

    #[test]
    fn test_parse_permission_hierarchical() {
        let perm = parse_permission("content/blog/*[rw-]").unwrap();
        assert_eq!(perm.resource, "content/blog/*");
        assert!(perm.read && perm.write && !perm.execute);
    }

    #[test]
    fn test_parse_permission_missing_bracket() {
        let result = parse_permission("textrwx");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing opening bracket"));
    }

    #[test]
    fn test_parse_permission_missing_closing_bracket() {
        let result = parse_permission("text[rwx");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing closing bracket"));
    }

    #[test]
    fn test_parse_permission_empty_resource() {
        let result = parse_permission("[rwx]");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Resource name cannot be empty"));
    }

    #[test]
    fn test_parse_permission_wrong_length() {
        let result = parse_permission("text[rw]");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exactly 3 characters"));
    }

    #[test]
    fn test_parse_permission_invalid_char() {
        let result = parse_permission("text[abc]");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("must be 'r' or '-'"));
    }

    #[test]
    fn test_parse_permissions_multiple() {
        let perms = parse_permissions("text[rwx],route[rw-],meta[r--]").unwrap();
        assert_eq!(perms.len(), 3);
        assert_eq!(perms[0].resource, "text");
        assert_eq!(perms[1].resource, "route");
        assert_eq!(perms[2].resource, "meta");
    }

    #[test]
    fn test_parse_permissions_with_spaces() {
        let perms = parse_permissions("text[rwx], route[rw-] , meta[r--]").unwrap();
        assert_eq!(perms.len(), 3);
    }

    #[test]
    fn test_parse_permissions_empty() {
        let perms = parse_permissions("").unwrap();
        assert_eq!(perms.len(), 0);
    }

    #[test]
    fn test_validate_permission_syntax_valid() {
        validate_permission_syntax("text[rwx]").unwrap();
        validate_permission_syntax("route[rw-]").unwrap();
        validate_permission_syntax("*[r--]").unwrap();
    }

    #[test]
    fn test_validate_permission_syntax_invalid() {
        assert!(validate_permission_syntax("invalid").is_err());
        assert!(validate_permission_syntax("text[abc]").is_err());
    }

    #[test]
    fn test_permission_allows() {
        let perm = Permission {
            resource: "text".to_string(),
            read: true,
            write: true,
            execute: false,
        };

        assert!(perm.allows("read"));
        assert!(perm.allows("r"));
        assert!(perm.allows("write"));
        assert!(perm.allows("w"));
        assert!(!perm.allows("execute"));
        assert!(!perm.allows("x"));
        assert!(!perm.allows("invalid"));
    }

    #[test]
    fn test_permission_matches_resource_exact() {
        let perm = Permission {
            resource: "text".to_string(),
            read: true,
            write: false,
            execute: false,
        };

        assert!(perm.matches_resource("text"));
        assert!(!perm.matches_resource("route"));
    }

    #[test]
    fn test_permission_matches_resource_wildcard() {
        let perm = Permission {
            resource: "*".to_string(),
            read: true,
            write: false,
            execute: false,
        };

        assert!(perm.matches_resource("text"));
        assert!(perm.matches_resource("route"));
        assert!(perm.matches_resource("anything"));
    }

    #[test]
    fn test_permission_matches_resource_hierarchical() {
        let perm = Permission {
            resource: "content/blog/*".to_string(),
            read: true,
            write: false,
            execute: false,
        };

        assert!(perm.matches_resource("content/blog/post1"));
        assert!(perm.matches_resource("content/blog/post2"));
        assert!(!perm.matches_resource("content/news/article1"));
        assert!(!perm.matches_resource("text"));
    }

    #[test]
    fn test_format_permission() {
        let perm = Permission {
            resource: "text".to_string(),
            read: true,
            write: true,
            execute: false,
        };

        assert_eq!(format_permission(&perm), "text[rw-]");
    }

    #[test]
    fn test_format_permission_all_denied() {
        let perm = Permission {
            resource: "blocked".to_string(),
            read: false,
            write: false,
            execute: false,
        };

        assert_eq!(format_permission(&perm), "blocked[---]");
    }

    #[test]
    fn test_format_permissions_multiple() {
        let perms = vec![
            Permission {
                resource: "text".to_string(),
                read: true,
                write: true,
                execute: true,
            },
            Permission {
                resource: "route".to_string(),
                read: true,
                write: false,
                execute: false,
            },
        ];

        assert_eq!(format_permissions(&perms), "text[rwx],route[r--]");
    }

    #[test]
    fn test_permission_new() {
        let perm = Permission::new("test");
        assert_eq!(perm.resource, "test");
        assert!(!perm.read);
        assert!(!perm.write);
        assert!(!perm.execute);
    }

    #[test]
    fn test_permission_roundtrip() {
        let original = "text[rwx],route[rw-],meta[r--]";
        let parsed = parse_permissions(original).unwrap();
        let formatted = format_permissions(&parsed);
        assert_eq!(formatted, original);
    }
}
