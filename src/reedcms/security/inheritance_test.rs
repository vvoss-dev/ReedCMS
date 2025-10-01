// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tests for Role Inheritance

#[cfg(test)]
mod tests {
    use crate::reedcms::matrix::{write_matrix_csv, MatrixRecord, MatrixValue};
    use crate::reedcms::security::inheritance::{
        has_circular_inheritance, merge_inherited_permissions, resolve_inheritance,
        resolve_role_permissions,
    };
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    fn setup_test_env() {
        let _ = fs::create_dir_all(".reed");
        let _ = fs::create_dir_all(".reed/backups");
        cleanup_test_roles();
    }

    fn cleanup_test_roles() {
        let _ = fs::remove_file(".reed/roles.matrix.csv");
    }

    fn create_test_role(
        rolename: &str,
        permissions: &str,
        inherits: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let roles_path = Path::new(".reed/roles.matrix.csv");

        let mut fields = HashMap::new();
        fields.insert(
            "rolename".to_string(),
            MatrixValue::Single(rolename.to_string()),
        );
        fields.insert(
            "permissions".to_string(),
            MatrixValue::Single(permissions.to_string()),
        );
        fields.insert(
            "inherits".to_string(),
            MatrixValue::Single(inherits.to_string()),
        );
        fields.insert("desc".to_string(), MatrixValue::Single(String::new()));
        fields.insert(
            "created_at".to_string(),
            MatrixValue::Single("0".to_string()),
        );
        fields.insert(
            "updated_at".to_string(),
            MatrixValue::Single("0".to_string()),
        );
        fields.insert(
            "is_active".to_string(),
            MatrixValue::Single("true".to_string()),
        );

        let record = MatrixRecord {
            fields,
            field_order: vec![
                "rolename",
                "permissions",
                "inherits",
                "desc",
                "created_at",
                "updated_at",
                "is_active",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            description: None,
        };

        let mut records = if roles_path.exists() {
            crate::reedcms::matrix::read_matrix_csv(roles_path)?
        } else {
            Vec::new()
        };

        records.push(record);
        write_matrix_csv(roles_path, &records, &[])?;

        Ok(())
    }

    #[test]
    fn test_resolve_inheritance_single() {
        setup_test_env();

        create_test_role("viewer", "text[r--]", "").unwrap();

        let chain = resolve_inheritance("viewer").unwrap();
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0], "viewer");

        cleanup_test_roles();
    }

    #[test]
    fn test_resolve_inheritance_chain() {
        setup_test_env();

        create_test_role("viewer", "text[r--]", "").unwrap();
        create_test_role("editor", "text[rw-]", "viewer").unwrap();
        create_test_role("admin", "*[rwx]", "editor").unwrap();

        let chain = resolve_inheritance("admin").unwrap();
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0], "admin");
        assert_eq!(chain[1], "editor");
        assert_eq!(chain[2], "viewer");

        cleanup_test_roles();
    }

    #[test]
    fn test_resolve_inheritance_not_found() {
        setup_test_env();

        let result = resolve_inheritance("nonexistent");
        assert!(result.is_err());

        cleanup_test_roles();
    }

    #[test]
    fn test_has_circular_inheritance_none() {
        setup_test_env();

        create_test_role("viewer", "text[r--]", "").unwrap();
        create_test_role("editor", "text[rw-]", "viewer").unwrap();

        let result = has_circular_inheritance("editor").unwrap();
        assert!(!result);

        cleanup_test_roles();
    }

    #[test]
    fn test_has_circular_inheritance_direct() {
        setup_test_env();

        create_test_role("role1", "text[r--]", "role1").unwrap();

        let result = has_circular_inheritance("role1").unwrap();
        assert!(result);

        cleanup_test_roles();
    }

    #[test]
    fn test_has_circular_inheritance_indirect() {
        setup_test_env();

        create_test_role("role1", "text[r--]", "role3").unwrap();
        create_test_role("role2", "text[r--]", "role1").unwrap();
        create_test_role("role3", "text[r--]", "role2").unwrap();

        let result = has_circular_inheritance("role1").unwrap();
        assert!(result);

        cleanup_test_roles();
    }

    #[test]
    fn test_merge_inherited_permissions_simple() {
        setup_test_env();

        create_test_role("viewer", "text[r--],route[r--]", "").unwrap();
        create_test_role("editor", "text[rw-]", "viewer").unwrap();

        let roles = vec!["editor".to_string(), "viewer".to_string()];
        let merged = merge_inherited_permissions(&roles).unwrap();

        // Child permission (editor) should override parent (viewer) for "text"
        let text_perm = merged.iter().find(|p| p.resource == "text").unwrap();
        assert!(text_perm.read && text_perm.write && !text_perm.execute);

        // Parent permission for "route" should be inherited
        let route_perm = merged.iter().find(|p| p.resource == "route").unwrap();
        assert!(route_perm.read && !route_perm.write && !route_perm.execute);

        cleanup_test_roles();
    }

    #[test]
    fn test_merge_inherited_permissions_wildcard() {
        setup_test_env();

        create_test_role("viewer", "text[r--]", "").unwrap();
        create_test_role("admin", "*[rwx]", "viewer").unwrap();

        let roles = vec!["admin".to_string(), "viewer".to_string()];
        let merged = merge_inherited_permissions(&roles).unwrap();

        // Wildcard permission should be included
        assert!(merged.iter().any(|p| p.resource == "*"));

        // Parent text permission should also be included (child doesn't override)
        assert!(merged.iter().any(|p| p.resource == "text"));

        cleanup_test_roles();
    }

    #[test]
    fn test_resolve_role_permissions_with_inheritance() {
        setup_test_env();

        create_test_role("viewer", "text[r--],route[r--]", "").unwrap();
        create_test_role("editor", "text[rw-],meta[rw-]", "viewer").unwrap();

        let perms = resolve_role_permissions("editor").unwrap();

        // Should have text (overridden), meta (from editor), route (inherited)
        assert!(perms.iter().any(|p| p.resource == "text"));
        assert!(perms.iter().any(|p| p.resource == "meta"));
        assert!(perms.iter().any(|p| p.resource == "route"));

        // Text should have rw- from editor, not r-- from viewer
        let text_perm = perms.iter().find(|p| p.resource == "text").unwrap();
        assert!(text_perm.read && text_perm.write && !text_perm.execute);

        cleanup_test_roles();
    }

    #[test]
    fn test_resolve_role_permissions_no_inheritance() {
        setup_test_env();

        create_test_role("standalone", "text[rwx]", "").unwrap();

        let perms = resolve_role_permissions("standalone").unwrap();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0].resource, "text");

        cleanup_test_roles();
    }

    #[test]
    fn test_inheritance_chain_ordering() {
        setup_test_env();

        create_test_role("base", "text[r--]", "").unwrap();
        create_test_role("middle", "route[r--]", "base").unwrap();
        create_test_role("top", "meta[r--]", "middle").unwrap();

        let chain = resolve_inheritance("top").unwrap();
        assert_eq!(chain, vec!["top", "middle", "base"]);

        let perms = resolve_role_permissions("top").unwrap();
        assert_eq!(perms.len(), 3);
        assert!(perms.iter().any(|p| p.resource == "text"));
        assert!(perms.iter().any(|p| p.resource == "route"));
        assert!(perms.iter().any(|p| p.resource == "meta"));

        cleanup_test_roles();
    }
}
