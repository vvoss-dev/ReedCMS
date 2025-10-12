// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Role Inheritance
//!
//! Provides role inheritance resolution with circular dependency detection.

use crate::reedcms::matrix::{read_matrix_csv, MatrixValue};
use crate::reedcms::reedstream::{not_found, validation_error, ReedResult};
use crate::reedcms::security::permissions::{parse_permissions, Permission};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Resolves role inheritance chain.
///
/// ## Input
/// - `role`: Role name to resolve
///
/// ## Output
/// - `ReedResult<Vec<String>>`: Vector of role names from child to parent
///
/// ## Process
/// 1. Start with role
/// 2. Follow inherits chain
/// 3. Detect circular dependencies
/// 4. Build complete inheritance chain
///
/// ## Error Conditions
/// - Returns `ReedError::NotFound` if role doesn't exist
/// - Returns `ReedError::ValidationError` if circular inheritance detected
///
/// ## Example Usage
/// ```
/// let chain = resolve_inheritance("author")?;
/// // Returns: ["author", "editor", "viewer"]
/// ```
pub fn resolve_inheritance(role: &str) -> ReedResult<Vec<String>> {
    let mut chain = Vec::new();
    let mut visited = HashSet::new();
    let mut current = role.to_string();

    loop {
        // Check for circular dependency
        if visited.contains(&current) {
            return Err(validation_error(
                "role",
                role,
                format!("Circular inheritance detected: {}", current),
            ));
        }

        visited.insert(current.clone());
        chain.push(current.clone());

        // Get parent role
        match get_parent_role(&current)? {
            Some(parent) => {
                current = parent;
            }
            None => break,
        }
    }

    Ok(chain)
}

/// Gets parent role name for a role.
///
/// ## Input
/// - `role`: Role name
///
/// ## Output
/// - `ReedResult<Option<String>>`: Parent role name or None if no parent
///
/// ## Error Conditions
/// - Returns `ReedError::NotFound` if role doesn't exist
fn get_parent_role(role: &str) -> ReedResult<Option<String>> {
    let roles_path = Path::new(".reed/roles.matrix.csv");

    if !roles_path.exists() {
        return Err(not_found(role));
    }

    let records = read_matrix_csv(roles_path)?;

    for record in records {
        if let Some(MatrixValue::Single(rolename)) = record.fields.get("rolename") {
            if rolename == role {
                if let Some(MatrixValue::Single(inherits)) = record.fields.get("inherits") {
                    if inherits.is_empty() {
                        return Ok(None);
                    }
                    return Ok(Some(inherits.clone()));
                }
                return Ok(None);
            }
        }
    }

    Err(not_found(role))
}

/// Checks for circular inheritance in role chain.
///
/// ## Input
/// - `role`: Role name to check
///
/// ## Output
/// - `ReedResult<bool>`: true if circular, false otherwise
///
/// ## Example Usage
/// ```
/// if has_circular_inheritance("admin")? {
///     return Err(validation_error("role", "admin", "Circular inheritance"));
/// }
/// ```
pub fn has_circular_inheritance(role: &str) -> ReedResult<bool> {
    match resolve_inheritance(role) {
        Ok(_) => Ok(false),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("Circular inheritance") {
                Ok(true)
            } else {
                Err(e)
            }
        }
    }
}

/// Merges permissions from inheritance chain.
///
/// ## Input
/// - `roles`: Vector of role names (child to parent order)
///
/// ## Output
/// - `ReedResult<Vec<Permission>>`: Merged permissions with child overrides
///
/// ## Rules
/// - Child permissions override parent
/// - Wildcard (*) applies to all resources
/// - More specific resources override general
///
/// ## Example Usage
/// ```
/// let roles = vec!["author".to_string(), "editor".to_string()];
/// let perms = merge_inherited_permissions(&roles)?;
/// ```
pub fn merge_inherited_permissions(roles: &[String]) -> ReedResult<Vec<Permission>> {
    let mut permission_map: HashMap<String, Permission> = HashMap::new();

    // Process from parent to child (reverse order) so child overrides parent
    for role in roles.iter().rev() {
        let role_perms = get_role_permissions(role)?;

        for perm in role_perms {
            // Child permission overrides parent
            permission_map.insert(perm.resource.clone(), perm);
        }
    }

    Ok(permission_map.into_values().collect())
}

/// Gets direct permissions for a role (without inheritance).
///
/// ## Input
/// - `role`: Role name
///
/// ## Output
/// - `ReedResult<Vec<Permission>>`: Role's direct permissions
fn get_role_permissions(role: &str) -> ReedResult<Vec<Permission>> {
    let roles_path = Path::new(".reed/roles.matrix.csv");

    if !roles_path.exists() {
        return Err(not_found(role));
    }

    let records = read_matrix_csv(roles_path)?;

    for record in records {
        if let Some(MatrixValue::Single(rolename)) = record.fields.get("rolename") {
            if rolename == role {
                if let Some(MatrixValue::Single(perms_str)) = record.fields.get("permissions") {
                    return parse_permissions(perms_str);
                }
                return Ok(Vec::new());
            }
        }
    }

    Err(not_found(role))
}

/// Resolves all permissions for a role (including inherited).
///
/// ## Input
/// - `role`: Role name
///
/// ## Output
/// - `ReedResult<Vec<Permission>>`: Complete permission set
///
/// ## Process
/// 1. Resolve inheritance chain
/// 2. Collect permissions from all roles in chain
/// 3. Merge with child overrides
///
/// ## Performance
/// - < 5ms typical (includes file I/O)
///
/// ## Example Usage
/// ```
/// let perms = resolve_role_permissions("admin")?;
/// ```
pub fn resolve_role_permissions(role: &str) -> ReedResult<Vec<Permission>> {
    let chain = resolve_inheritance(role)?;
    merge_inherited_permissions(&chain)
}
