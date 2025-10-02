// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Role Management
//!
//! Provides role CRUD operations with inheritance support.

use crate::reedcms::backup::create_backup;
use crate::reedcms::matrix::{read_matrix_csv, write_matrix_csv, MatrixRecord, MatrixValue};
use crate::reedcms::reedstream::{
    current_timestamp, not_found, validation_error, ReedRequest, ReedResponse, ReedResult,
};
use crate::reedcms::security::inheritance::{has_circular_inheritance, resolve_role_permissions};
use crate::reedcms::security::permissions::{parse_permissions, Permission};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Role information structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub rolename: String,
    pub permissions: Vec<Permission>,
    pub inherits: Option<String>,
    pub desc: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
}

/// Role update structure for partial updates.
#[derive(Debug, Clone)]
pub struct RoleUpdate {
    pub permissions: Option<Vec<Permission>>,
    pub inherits: Option<String>,
    pub desc: Option<String>,
    pub is_active: Option<bool>,
}

/// Creates new role with permission validation.
///
/// ## Input
/// - `req.key`: Role name (alphanumeric + underscore)
/// - `req.value`: Comma-separated permissions (text[rwx],route[rw-])
/// - `req.context`: JSON with inherits and desc fields
///
/// ## Validation
/// - Role name uniqueness
/// - Permission syntax validation
/// - Parent role existence check (if inherits specified)
/// - Circular inheritance detection
///
/// ## Output
/// - `ReedResult<ReedResponse<RoleInfo>>`: Created role info
///
/// ## Error Conditions
/// - Returns `ReedError::ValidationError` if role exists
/// - Returns `ReedError::ValidationError` if permission syntax invalid
/// - Returns `ReedError::NotFound` if parent role doesn't exist
/// - Returns `ReedError::ValidationError` if circular inheritance
///
/// ## Example Usage
/// ```
/// let req = ReedRequest {
///     key: "editor".to_string(),
///     value: Some("text[rwx],route[rw-]".to_string()),
///     context: Some(r#"{"inherits":"viewer","desc":"Content editor"}"#.to_string()),
///     ..Default::default()
/// };
/// let response = create_role(&req)?;
/// ```
pub fn create_role(req: &ReedRequest) -> ReedResult<ReedResponse<RoleInfo>> {
    let rolename = &req.key;
    let perms_str = req
        .value
        .as_ref()
        .ok_or_else(|| validation_error("permissions", "", "Permissions are required"))?;

    // Validate role name
    if rolename.is_empty() {
        return Err(validation_error(
            "rolename",
            rolename,
            "Role name cannot be empty",
        ));
    }

    // Check role uniqueness
    if role_exists(rolename)? {
        return Err(validation_error(
            "rolename",
            rolename,
            "Role already exists",
        ));
    }

    // Parse and validate permissions
    let permissions = parse_permissions(perms_str)?;

    // Parse context for inherits and desc
    let context_data: serde_json::Value = if let Some(ctx) = &req.context {
        serde_json::from_str(ctx)
            .map_err(|e| validation_error("context", ctx, &format!("Invalid JSON: {}", e)))?
    } else {
        serde_json::json!({})
    };

    let inherits = context_data["inherits"].as_str().map(|s| s.to_string());
    let desc = context_data["desc"].as_str().unwrap_or("").to_string();

    // Validate parent role exists
    if let Some(ref parent) = inherits {
        if !role_exists(parent)? {
            return Err(not_found(parent));
        }
    }

    // Build role record
    let timestamp = current_timestamp();
    let mut fields = HashMap::new();

    fields.insert(
        "rolename".to_string(),
        MatrixValue::Single(rolename.clone()),
    );
    fields.insert(
        "permissions".to_string(),
        MatrixValue::Single(perms_str.clone()),
    );
    fields.insert(
        "inherits".to_string(),
        MatrixValue::Single(inherits.clone().unwrap_or_default()),
    );
    fields.insert("desc".to_string(), MatrixValue::Single(desc.clone()));
    fields.insert(
        "created_at".to_string(),
        MatrixValue::Single(timestamp.to_string()),
    );
    fields.insert(
        "updated_at".to_string(),
        MatrixValue::Single(timestamp.to_string()),
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
        description: Some(format!("Role: {}", rolename)),
    };

    // Write to file (with backup)
    let roles_path = Path::new(".reed/roles.matrix.csv");
    if roles_path.exists() {
        create_backup(roles_path)?;
    }

    let mut records = if roles_path.exists() {
        read_matrix_csv(roles_path)?
    } else {
        Vec::new()
    };
    records.push(record);

    write_matrix_csv(roles_path, &records, &[])?;

    // Check for circular inheritance after creation
    if has_circular_inheritance(rolename)? {
        // Rollback: remove the role
        records.pop();
        write_matrix_csv(roles_path, &records, &[])?;

        return Err(validation_error(
            "inherits",
            inherits.as_deref().unwrap_or(""),
            "Circular inheritance detected",
        ));
    }

    let role_info = RoleInfo {
        rolename: rolename.clone(),
        permissions,
        inherits,
        desc,
        created_at: timestamp,
        updated_at: timestamp,
        is_active: true,
    };

    Ok(ReedResponse {
        data: role_info,
        source: "security::roles::create_role".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Retrieves role with resolved permissions (including inherited).
///
/// ## Input
/// - `rolename`: Role name to retrieve
///
/// ## Output
/// - `ReedResult<ReedResponse<RoleInfo>>`: Role information with resolved permissions
///
/// ## Performance
/// - < 10ms (includes inheritance resolution)
///
/// ## Example Usage
/// ```
/// let response = get_role("editor")?;
/// println!("Role has {} permissions", response.data.permissions.len());
/// ```
pub fn get_role(rolename: &str) -> ReedResult<ReedResponse<RoleInfo>> {
    let roles_path = Path::new(".reed/roles.matrix.csv");

    if !roles_path.exists() {
        return Err(not_found(rolename));
    }

    let records = read_matrix_csv(roles_path)?;

    for record in records {
        if let Some(MatrixValue::Single(role)) = record.fields.get("rolename") {
            if role == rolename {
                // Resolve permissions including inherited
                let resolved_perms = resolve_role_permissions(rolename)?;

                let role_info = matrix_record_to_role_info(&record, resolved_perms)?;

                return Ok(ReedResponse {
                    data: role_info,
                    source: "security::roles::get_role".to_string(),
                    cached: false,
                    timestamp: current_timestamp(),
                    metrics: None,
                });
            }
        }
    }

    Err(not_found(rolename))
}

/// Lists all roles.
///
/// ## Output
/// - `ReedResult<ReedResponse<Vec<RoleInfo>>>`: List of all roles
///
/// ## Example Usage
/// ```
/// let response = list_roles()?;
/// println!("Found {} roles", response.data.len());
/// ```
pub fn list_roles() -> ReedResult<ReedResponse<Vec<RoleInfo>>> {
    let roles_path = Path::new(".reed/roles.matrix.csv");

    if !roles_path.exists() {
        return Ok(ReedResponse {
            data: Vec::new(),
            source: "security::roles::list_roles".to_string(),
            cached: false,
            timestamp: current_timestamp(),
            metrics: None,
        });
    }

    let records = read_matrix_csv(roles_path)?;
    let mut roles = Vec::new();

    for record in records {
        if let Some(MatrixValue::Single(rolename)) = record.fields.get("rolename") {
            let resolved_perms = resolve_role_permissions(rolename)?;
            let role_info = matrix_record_to_role_info(&record, resolved_perms)?;
            roles.push(role_info);
        }
    }

    Ok(ReedResponse {
        data: roles,
        source: "security::roles::list_roles".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Updates role permissions or inheritance.
///
/// ## Input
/// - `rolename`: Role name to update
/// - `updates`: Partial update structure
///
/// ## Output
/// - `ReedResult<ReedResponse<RoleInfo>>`: Updated role information
///
/// ## Error Conditions
/// - Returns `ReedError::NotFound` if role doesn't exist
/// - Returns `ReedError::ValidationError` if circular inheritance created
///
/// ## Example Usage
/// ```
/// let updates = RoleUpdate {
///     permissions: Some(parse_permissions("text[rwx],route[rwx]")?),
///     inherits: None,
///     desc: None,
///     is_active: None,
/// };
/// let response = update_role("editor", updates)?;
/// ```
pub fn update_role(rolename: &str, updates: RoleUpdate) -> ReedResult<ReedResponse<RoleInfo>> {
    let roles_path = Path::new(".reed/roles.matrix.csv");

    if !roles_path.exists() {
        return Err(not_found(rolename));
    }

    // Backup before modification
    create_backup(roles_path)?;

    let mut records = read_matrix_csv(roles_path)?;
    let mut found = false;

    for record in &mut records {
        if let Some(MatrixValue::Single(role)) = record.fields.get("rolename") {
            if role == rolename {
                found = true;

                // Apply updates
                if let Some(ref perms) = updates.permissions {
                    let perms_str =
                        crate::reedcms::security::permissions::format_permissions(perms);
                    record
                        .fields
                        .insert("permissions".to_string(), MatrixValue::Single(perms_str));
                }

                if let Some(ref inherits) = updates.inherits {
                    // Validate parent exists
                    if !inherits.is_empty() && !role_exists(inherits)? {
                        return Err(not_found(inherits));
                    }
                    record.fields.insert(
                        "inherits".to_string(),
                        MatrixValue::Single(inherits.clone()),
                    );
                }

                if let Some(ref desc) = updates.desc {
                    record
                        .fields
                        .insert("desc".to_string(), MatrixValue::Single(desc.clone()));
                }

                if let Some(is_active) = updates.is_active {
                    record.fields.insert(
                        "is_active".to_string(),
                        MatrixValue::Single(is_active.to_string()),
                    );
                }

                // Update timestamp
                record.fields.insert(
                    "updated_at".to_string(),
                    MatrixValue::Single(current_timestamp().to_string()),
                );

                break;
            }
        }
    }

    if !found {
        return Err(not_found(rolename));
    }

    // Save and check for circular inheritance
    write_matrix_csv(roles_path, &records, &[])?;

    if has_circular_inheritance(rolename)? {
        // Rollback from backup
        let backup_files = std::fs::read_dir(".reed/backups")
            .map_err(|e| {
                crate::reedcms::reedstream::io_error("read_dir", ".reed/backups", e.to_string())
            })?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("roles.matrix.csv")
            })
            .max_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));

        if let Some(backup) = backup_files {
            std::fs::copy(&backup.path(), roles_path).map_err(|e| {
                crate::reedcms::reedstream::io_error(
                    "restore",
                    roles_path.to_string_lossy().to_string(),
                    e.to_string(),
                )
            })?;
        }

        return Err(validation_error(
            "inherits",
            updates.inherits.as_deref().unwrap_or(""),
            "Circular inheritance detected",
        ));
    }

    // Get updated role info
    let resolved_perms = resolve_role_permissions(rolename)?;
    let records = read_matrix_csv(roles_path)?;
    let mut updated_role = None;

    for record in records {
        if let Some(MatrixValue::Single(role)) = record.fields.get("rolename") {
            if role == rolename {
                updated_role = Some(matrix_record_to_role_info(&record, resolved_perms)?);
                break;
            }
        }
    }

    Ok(ReedResponse {
        data: updated_role.unwrap(),
        source: "security::roles::update_role".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Deletes role with dependency check.
///
/// ## Input
/// - `rolename`: Role name to delete
/// - `confirm`: Must be true to actually delete
///
/// ## Output
/// - `ReedResult<ReedResponse<()>>`: Success confirmation
///
/// ## Error Conditions
/// - Returns `ReedError::NotFound` if role doesn't exist
/// - Returns `ReedError::ValidationError` if confirm is false
/// - Returns `ReedError::ValidationError` if role has dependents
///
/// ## Example Usage
/// ```
/// delete_role("oldRole", true)?;
/// ```
pub fn delete_role(rolename: &str, confirm: bool) -> ReedResult<ReedResponse<()>> {
    if !confirm {
        return Err(validation_error(
            "confirm",
            "false",
            "Confirmation required to delete role",
        ));
    }

    let roles_path = Path::new(".reed/roles.matrix.csv");

    if !roles_path.exists() {
        return Err(not_found(rolename));
    }

    // Check for dependent roles
    if has_dependent_roles(rolename)? {
        return Err(validation_error(
            "rolename",
            rolename,
            "Cannot delete role: other roles inherit from it",
        ));
    }

    // Backup before modification
    create_backup(roles_path)?;

    let mut records = read_matrix_csv(roles_path)?;
    let original_len = records.len();

    records.retain(|record| {
        if let Some(MatrixValue::Single(role)) = record.fields.get("rolename") {
            role != rolename
        } else {
            true
        }
    });

    if records.len() == original_len {
        return Err(not_found(rolename));
    }

    write_matrix_csv(roles_path, &records, &[])?;

    Ok(ReedResponse {
        data: (),
        source: "security::roles::delete_role".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Helper: Check if role exists.
fn role_exists(rolename: &str) -> ReedResult<bool> {
    let roles_path = Path::new(".reed/roles.matrix.csv");

    if !roles_path.exists() {
        return Ok(false);
    }

    let records = read_matrix_csv(roles_path)?;

    for record in records {
        if let Some(MatrixValue::Single(role)) = record.fields.get("rolename") {
            if role == rolename {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Helper: Check if any roles inherit from this role.
fn has_dependent_roles(rolename: &str) -> ReedResult<bool> {
    let roles_path = Path::new(".reed/roles.matrix.csv");

    if !roles_path.exists() {
        return Ok(false);
    }

    let records = read_matrix_csv(roles_path)?;

    for record in records {
        if let Some(MatrixValue::Single(inherits)) = record.fields.get("inherits") {
            if inherits == rolename {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Helper: Convert MatrixRecord to RoleInfo.
fn matrix_record_to_role_info(
    record: &MatrixRecord,
    resolved_perms: Vec<Permission>,
) -> ReedResult<RoleInfo> {
    let rolename = if let Some(MatrixValue::Single(r)) = record.fields.get("rolename") {
        r.clone()
    } else {
        return Err(validation_error("rolename", "", "Missing rolename field"));
    };

    let get_string = |field: &str| -> String {
        if let Some(MatrixValue::Single(s)) = record.fields.get(field) {
            s.clone()
        } else {
            String::new()
        }
    };

    let inherits_str = get_string("inherits");
    let inherits = if inherits_str.is_empty() {
        None
    } else {
        Some(inherits_str)
    };

    let created_at = get_string("created_at").parse().unwrap_or(0);
    let updated_at = get_string("updated_at").parse().unwrap_or(0);
    let is_active = get_string("is_active") == "true";

    Ok(RoleInfo {
        rolename,
        permissions: resolved_perms,
        inherits,
        desc: get_string("desc"),
        created_at,
        updated_at,
        is_active,
    })
}
