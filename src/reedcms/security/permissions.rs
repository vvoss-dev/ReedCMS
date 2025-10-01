// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Permission Parsing and Checking
//!
//! Provides Unix-style permission parsing and checking with caching.

use crate::reedcms::reedstream::{validation_error, ReedResult};
use serde::{Deserialize, Serialize};

/// Permission structure for resource access control.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Permission {
    /// Creates a new permission with all flags set to false.
    pub fn new(resource: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            read: false,
            write: false,
            execute: false,
        }
    }

    /// Checks if permission allows specific action.
    pub fn allows(&self, action: &str) -> bool {
        match action {
            "read" | "r" => self.read,
            "write" | "w" => self.write,
            "execute" | "x" => self.execute,
            _ => false,
        }
    }

    /// Checks if this permission matches a resource (with wildcard and hierarchy support).
    ///
    /// ## Matching Rules
    /// - Exact match: "text" matches "text"
    /// - Wildcard: "*" matches any resource
    /// - Hierarchical: "content/blog/*" matches "content/blog/post1"
    pub fn matches_resource(&self, resource: &str) -> bool {
        if self.resource == "*" {
            return true;
        }

        if self.resource == resource {
            return true;
        }

        // Hierarchical matching: content/blog/* matches content/blog/post1
        if self.resource.ends_with("/*") {
            let prefix = &self.resource[..self.resource.len() - 2];
            if resource.starts_with(prefix) {
                return true;
            }
        }

        false
    }
}

/// Parses permission string into Permission structure.
///
/// ## Input
/// - `perm`: Permission string (e.g., "text[rwx]", "route[rw-]", "*[r--]")
///
/// ## Output
/// - `ReedResult<Permission>`: Parsed permission or error
///
/// ## Format
/// - resource[permissions]
/// - permissions: r (read), w (write), x (execute), - (none)
///
/// ## Error Conditions
/// - Returns `ReedError::ValidationError` if format is invalid
/// - Returns `ReedError::ValidationError` if permissions length != 3
///
/// ## Example Usage
/// ```
/// let perm = parse_permission("text[rwx]")?;
/// assert_eq!(perm.resource, "text");
/// assert!(perm.read && perm.write && perm.execute);
/// ```
pub fn parse_permission(perm: &str) -> ReedResult<Permission> {
    let perm = perm.trim();

    // Find bracket positions
    let open_bracket = perm
        .find('[')
        .ok_or_else(|| validation_error("permission", perm, "Missing opening bracket ["))?;

    let close_bracket = perm
        .find(']')
        .ok_or_else(|| validation_error("permission", perm, "Missing closing bracket ]"))?;

    if close_bracket != perm.len() - 1 {
        return Err(validation_error(
            "permission",
            perm,
            "Closing bracket ] must be at end",
        ));
    }

    // Extract resource and permissions
    let resource = &perm[..open_bracket];
    let perms = &perm[open_bracket + 1..close_bracket];

    if resource.is_empty() {
        return Err(validation_error(
            "permission",
            perm,
            "Resource name cannot be empty",
        ));
    }

    if perms.len() != 3 {
        return Err(validation_error(
            "permission",
            perm,
            "Permissions must be exactly 3 characters (rwx format)",
        ));
    }

    let chars: Vec<char> = perms.chars().collect();

    // Parse read permission
    let read = match chars[0] {
        'r' => true,
        '-' => false,
        _ => {
            return Err(validation_error(
                "permission",
                perm,
                "First character must be 'r' or '-'",
            ))
        }
    };

    // Parse write permission
    let write = match chars[1] {
        'w' => true,
        '-' => false,
        _ => {
            return Err(validation_error(
                "permission",
                perm,
                "Second character must be 'w' or '-'",
            ))
        }
    };

    // Parse execute permission
    let execute = match chars[2] {
        'x' => true,
        '-' => false,
        _ => {
            return Err(validation_error(
                "permission",
                perm,
                "Third character must be 'x' or '-'",
            ))
        }
    };

    Ok(Permission {
        resource: resource.to_string(),
        read,
        write,
        execute,
    })
}

/// Validates permission syntax without parsing.
///
/// ## Input
/// - `perm`: Permission string to validate
///
/// ## Output
/// - `ReedResult<()>`: Ok if valid, error otherwise
///
/// ## Example Usage
/// ```
/// validate_permission_syntax("text[rwx]")?;  // Ok
/// validate_permission_syntax("invalid")?;    // Error
/// ```
pub fn validate_permission_syntax(perm: &str) -> ReedResult<()> {
    parse_permission(perm)?;
    Ok(())
}

/// Parses comma-separated permission list.
///
/// ## Input
/// - `perms`: Comma-separated permissions (e.g., "text[rwx],route[rw-]")
///
/// ## Output
/// - `ReedResult<Vec<Permission>>`: Parsed permissions or error
///
/// ## Example Usage
/// ```
/// let perms = parse_permissions("text[rwx],route[rw-],*[r--]")?;
/// assert_eq!(perms.len(), 3);
/// ```
pub fn parse_permissions(perms: &str) -> ReedResult<Vec<Permission>> {
    let mut result = Vec::new();

    for perm in perms.split(',') {
        let perm = perm.trim();
        if !perm.is_empty() {
            result.push(parse_permission(perm)?);
        }
    }

    Ok(result)
}

/// Formats permission back to string representation.
///
/// ## Input
/// - `perm`: Permission to format
///
/// ## Output
/// - String in format "resource[rwx]"
///
/// ## Example Usage
/// ```
/// let perm = Permission { resource: "text".to_string(), read: true, write: true, execute: false };
/// assert_eq!(format_permission(&perm), "text[rw-]");
/// ```
pub fn format_permission(perm: &Permission) -> String {
    let r = if perm.read { 'r' } else { '-' };
    let w = if perm.write { 'w' } else { '-' };
    let x = if perm.execute { 'x' } else { '-' };

    format!("{}[{}{}{}]", perm.resource, r, w, x)
}

/// Formats multiple permissions to comma-separated string.
pub fn format_permissions(perms: &[Permission]) -> String {
    perms
        .iter()
        .map(|p| format_permission(p))
        .collect::<Vec<_>>()
        .join(",")
}
