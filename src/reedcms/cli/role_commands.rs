// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Role management CLI commands.
//!
//! Provides flag-based role CRUD operations, permission management,
//! and inheritance configuration using the Security Layer API.

use crate::reedcms::reedstream::{
    current_timestamp, ReedError, ReedRequest, ReedResponse, ReedResult,
};
use crate::reedcms::security::permissions::{format_permissions, parse_permissions, Permission};
use crate::reedcms::security::roles::{
    create_role as create_role_service, delete_role as delete_role_service, get_role,
    list_roles as list_roles_service, update_role as update_role_service, RoleInfo, RoleUpdate,
};
use serde_json::json;
use std::collections::HashMap;

/// Creates new role with permissions and optional inheritance.
///
/// ## Arguments
/// - args[0]: rolename
///
/// ## Flags (required)
/// - --permissions: Unix-style permissions (e.g., "text[rwx],route[rw-]")
/// - --desc: Role description
/// - --inherit: Parent role name (optional)
///
/// ## Example
/// ```bash
/// reed role:create editor --permissions "text[rwx],route[rw-]" --desc "Content editor"
/// reed role:create admin --permissions "*[rwx]" --inherit "editor" --desc "Administrator"
/// ```
pub fn create_role(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: "role:create".to_string(),
            reason: "Requires rolename argument".to_string(),
        });
    }

    let rolename = &args[0];

    // Validate required flags
    let permissions_str = flags
        .get("permissions")
        .ok_or_else(|| ReedError::InvalidCommand {
            command: "role:create".to_string(),
            reason: "--permissions flag is required".to_string(),
        })?;

    let desc = flags.get("desc").ok_or_else(|| ReedError::InvalidCommand {
        command: "role:create".to_string(),
        reason: "--desc flag is required".to_string(),
    })?;

    // Build context JSON
    let context = json!({
        "desc": desc,
        "inherits": flags.get("inherit").unwrap_or(&String::new()),
    });

    // Build ReedRequest for Security API
    let request = ReedRequest {
        key: rolename.clone(),
        language: None,
        environment: None,
        context: Some(context.to_string()),
        value: Some(permissions_str.clone()),
        description: Some(desc.clone()),
    };

    // Call Security API
    let response = create_role_service(&request)?;
    let role = response.data;

    // Format output
    let mut output = format!(
        "✓ Role '{}' created successfully\n  Permissions: {}",
        rolename,
        format_permissions(&role.permissions)
    );

    if let Some(parent) = &role.inherits {
        if !parent.is_empty() {
            output.push_str(&format!("\n  Inherits: {}", parent));
        }
    }

    Ok(ReedResponse {
        data: output,
        source: "cli::role_commands::create_role".to_string(),
        cached: false,
        timestamp: response.timestamp,
        metrics: None,
    })
}

/// Lists all roles with optional formatting.
///
/// ## Flags
/// - --format: Output format (table, json, csv) - default: table
/// - --show-permissions: Include full permission details
///
/// ## Example
/// ```bash
/// reed role:list
/// reed role:list --format json
/// reed role:list --show-permissions
/// ```
pub fn list_roles(flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>> {
    let format = flags.get("format").map(|s| s.as_str()).unwrap_or("table");
    let show_permissions = flags.contains_key("show-permissions");

    // Get roles from Security API
    let response = list_roles_service()?;
    let roles: Vec<&RoleInfo> = response.data.iter().collect();

    // Format output
    let output = match format {
        "json" => format_role_json(&roles),
        "csv" => format_role_csv(&roles),
        _ => format_role_table(&roles, show_permissions),
    };

    Ok(ReedResponse {
        data: output,
        source: "cli::role_commands::list_roles".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Shows detailed role information.
///
/// ## Arguments
/// - args[0]: rolename
///
/// ## Example
/// ```bash
/// reed role:show editor
/// ```
pub fn show_role(args: &[String]) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: "role:show".to_string(),
            reason: "Requires rolename argument".to_string(),
        });
    }

    let rolename = &args[0];
    let response = get_role(rolename)?;

    let output = format_role_details(&response.data);

    Ok(ReedResponse {
        data: output,
        source: "cli::role_commands::show_role".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Updates role properties.
///
/// ## Arguments
/// - args[0]: rolename
///
/// ## Flags (at least one required)
/// - --permissions: New permissions (replaces all)
/// - --inherit: New parent role
/// - --desc: New description
///
/// ## Example
/// ```bash
/// reed role:update editor --permissions "text[rwx],route[rwx],content[rw-]"
/// reed role:update admin --inherit "superuser"
/// reed role:update viewer --desc "Read-only access"
/// ```
pub fn update_role(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: "role:update".to_string(),
            reason: "Requires rolename argument".to_string(),
        });
    }

    if flags.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: "role:update".to_string(),
            reason: "Requires at least one update flag (--permissions, --inherit, --desc)"
                .to_string(),
        });
    }

    let rolename = &args[0];

    // Parse permissions if provided
    let permissions = if let Some(perms_str) = flags.get("permissions") {
        Some(parse_permissions(perms_str)?)
    } else {
        None
    };

    // Build update
    let update = RoleUpdate {
        permissions,
        inherits: flags.get("inherit").cloned(),
        desc: flags.get("desc").cloned(),
        is_active: None,
    };

    // Call Security API
    update_role_service(rolename, update)?;

    let output = format!("✓ Role '{}' updated successfully", rolename);

    Ok(ReedResponse {
        data: output,
        source: "cli::role_commands::update_role".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Deletes role with --force confirmation.
///
/// ## Arguments
/// - args[0]: rolename
///
/// ## Flags
/// - --force: Required to confirm deletion
///
/// ## Example
/// ```bash
/// reed role:delete oldrole --force
/// ```
pub fn delete_role(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: "role:delete".to_string(),
            reason: "Requires rolename argument".to_string(),
        });
    }

    if !flags.contains_key("force") {
        return Err(ReedError::InvalidCommand {
            command: "role:delete".to_string(),
            reason: "--force flag required to confirm deletion".to_string(),
        });
    }

    let rolename = &args[0];

    // Call Security API (confirm=true because --force flag present)
    delete_role_service(rolename, true)?;

    let output = format!("✓ Role '{}' deleted successfully", rolename);

    Ok(ReedResponse {
        data: output,
        source: "cli::role_commands::delete_role".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Manages role permissions (show/add/remove/set).
///
/// ## Arguments
/// - args[0]: rolename
///
/// ## Flags (mutually exclusive)
/// - --add: Add permissions (preserves existing)
/// - --remove: Remove permissions
/// - --set: Set permissions (replaces all)
/// - No flags: Show current permissions
///
/// ## Example
/// ```bash
/// reed role:permissions editor                          # show
/// reed role:permissions editor --add "cms[rw-]"         # add
/// reed role:permissions editor --remove "content[rwx]"  # remove
/// reed role:permissions editor --set "text[rwx],route[rw-]"  # replace
/// ```
pub fn manage_permissions(
    args: &[String],
    flags: &HashMap<String, String>,
) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: "role:permissions".to_string(),
            reason: "Requires rolename argument".to_string(),
        });
    }

    let rolename = &args[0];
    let response = get_role(rolename)?;
    let role = response.data;

    // No flags: show current permissions
    if flags.is_empty() {
        let output = format!(
            "Current permissions for '{}':\n  {}",
            rolename,
            format_permissions(&role.permissions)
                .split(", ")
                .map(|p| format!("- {}", p))
                .collect::<Vec<_>>()
                .join("\n  ")
        );
        return Ok(ReedResponse {
            data: output,
            source: "cli::role_commands::manage_permissions".to_string(),
            cached: false,
            timestamp: current_timestamp(),
            metrics: None,
        });
    }

    // Calculate new permissions
    let mut new_permissions = role.permissions.clone();

    if let Some(add_str) = flags.get("add") {
        let add_perms = parse_permissions(add_str)?;
        for perm in add_perms {
            if !new_permissions.iter().any(|p| p.resource == perm.resource) {
                new_permissions.push(perm);
            }
        }
    } else if let Some(remove_str) = flags.get("remove") {
        let remove_perms = parse_permissions(remove_str)?;
        new_permissions.retain(|p| !remove_perms.iter().any(|rp| rp.resource == p.resource));
    } else if let Some(set_str) = flags.get("set") {
        new_permissions = parse_permissions(set_str)?;
    }

    // Update role with new permissions
    let update = RoleUpdate {
        permissions: Some(new_permissions.clone()),
        inherits: None,
        desc: None,
        is_active: None,
    };

    update_role_service(rolename, update)?;

    let output = format!(
        "✓ Permissions updated for '{}'\n  New permissions:\n  {}",
        rolename,
        format_permissions(&new_permissions)
            .split(", ")
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n  ")
    );

    Ok(ReedResponse {
        data: output,
        source: "cli::role_commands::manage_permissions".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

// Output formatting functions

/// Formats role list as ASCII table.
fn format_role_table(roles: &[&RoleInfo], show_permissions: bool) -> String {
    if roles.is_empty() {
        return "No roles found".to_string();
    }

    let mut output = String::new();

    if show_permissions {
        output.push_str(&format!(
            "{:<15} {:<40} {:<15} {:<10}\n",
            "Role", "Permissions", "Inherits", "Status"
        ));
        output.push_str(&"-".repeat(82));
        output.push('\n');

        for role in roles {
            let perms_str = format_permissions(&role.permissions);
            let inherits_str = role.inherits.as_deref().unwrap_or("");
            let status = if role.is_active { "active" } else { "inactive" };

            output.push_str(&format!(
                "{:<15} {:<40} {:<15} {:<10}\n",
                role.rolename, perms_str, inherits_str, status
            ));
        }
    } else {
        output.push_str(&format!(
            "{:<15} {:<15} {:<10}\n",
            "Role", "Inherits", "Status"
        ));
        output.push_str(&"-".repeat(42));
        output.push('\n');

        for role in roles {
            let inherits_str = role.inherits.as_deref().unwrap_or("");
            let status = if role.is_active { "active" } else { "inactive" };

            output.push_str(&format!(
                "{:<15} {:<15} {:<10}\n",
                role.rolename, inherits_str, status
            ));
        }
    }

    output.push_str(&format!("\n{} role(s) total", roles.len()));
    output
}

/// Formats role list as JSON.
fn format_role_json(roles: &[&RoleInfo]) -> String {
    let role_array: Vec<_> = roles
        .iter()
        .map(|r| {
            json!({
                "rolename": r.rolename,
                "permissions": format_permissions(&r.permissions),
                "inherits": r.inherits,
                "desc": r.desc,
                "is_active": r.is_active,
                "created_at": r.created_at,
            })
        })
        .collect();

    serde_json::to_string_pretty(&role_array).unwrap_or_else(|_| "[]".to_string())
}

/// Formats role list as CSV.
fn format_role_csv(roles: &[&RoleInfo]) -> String {
    let mut output = String::new();
    output.push_str("rolename,permissions,inherits,status\n");

    for role in roles {
        let perms_str = format_permissions(&role.permissions).replace(",", ";");
        let inherits_str = role.inherits.as_deref().unwrap_or("");
        let status = if role.is_active { "active" } else { "inactive" };

        output.push_str(&format!(
            "{},{},{},{}\n",
            role.rolename, perms_str, inherits_str, status
        ));
    }

    output
}

/// Formats single role details.
fn format_role_details(role: &RoleInfo) -> String {
    let mut output = String::new();
    output.push_str(&format!("Role: {}\n", role.rolename));
    output.push_str(&format!("Description: {}\n", role.desc));
    output.push_str("\nPermissions:\n");

    for perm in &role.permissions {
        output.push_str(&format!(
            "  - {}[{}{}{}]: {} access to {}\n",
            perm.resource,
            if perm.read { "r" } else { "-" },
            if perm.write { "w" } else { "-" },
            if perm.execute { "x" } else { "-" },
            describe_permissions(perm),
            perm.resource
        ));
    }

    if let Some(inherits) = &role.inherits {
        if !inherits.is_empty() {
            output.push_str(&format!("\nInherits: {}\n", inherits));
        }
    }

    output.push_str(&format!(
        "\nStatus: {}\n",
        if role.is_active { "active" } else { "inactive" }
    ));
    output.push_str(&format!("Created: {}\n", role.created_at));
    output.push_str(&format!("Updated: {}\n", role.updated_at));

    output
}

/// Describes permission flags in human-readable format.
fn describe_permissions(perm: &Permission) -> &'static str {
    match (perm.read, perm.write, perm.execute) {
        (true, true, true) => "Full",
        (true, true, false) => "Read/write",
        (true, false, false) => "Read-only",
        (false, true, false) => "Write-only",
        (false, false, true) => "Execute-only",
        _ => "Custom",
    }
}
