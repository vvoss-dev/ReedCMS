// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Registry initialisation and validation.
//!
//! Handles creation of default dictionaries and integrity validation.

use crate::error::{ReedError, ReedResult};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Initialises registry system.
///
/// Creates directory structure and default dictionaries if they don't exist.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory (e.g., `.reed`)
///
/// ## Output
/// - `Result<()>`: Success or error
///
/// ## Performance
/// - First run: < 20ms (creates files)
/// - Subsequent runs: < 1ms (files exist)
///
/// ## Error Conditions
/// - IoError: Cannot create directories
/// - PermissionDenied: Insufficient permissions
///
/// ## Example Usage
/// ```no_run
/// use reedbase::registry::init_registry;
/// use std::path::Path;
///
/// init_registry(Path::new(".reed"))?;
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn init_registry(base_path: &Path) -> ReedResult<()> {
    let registry_dir = base_path.join("registry");

    // Create directory if it doesn't exist
    if !registry_dir.exists() {
        fs::create_dir_all(&registry_dir).map_err(|e| ReedError::IoError {
            operation: "create_registry_dir".to_string(),
            reason: e.to_string(),
        })?;
    }

    // Create actions.dict if missing
    let actions_path = registry_dir.join("actions.dict");
    if !actions_path.exists() {
        create_default_action_dict(&actions_path)?;
    }

    // Create users.dict if missing
    let users_path = registry_dir.join("users.dict");
    if !users_path.exists() {
        create_default_user_dict(&users_path)?;
    }

    Ok(())
}

/// Creates default actions dictionary.
///
/// ## Performance
/// - < 5ms
///
/// ## Error Conditions
/// - IoError: Cannot write file
fn create_default_action_dict(path: &Path) -> ReedResult<()> {
    let content = "\
code|name|description
0|delete|Delete operation
1|create|Create new entry
2|update|Update existing entry
3|rollback|Rollback to previous version
4|compact|Compact/cleanup old versions
5|init|Initialise table
6|snapshot|Full snapshot (periodic)
7|automerge|Automatic merge of concurrent writes
8|conflict|Conflict detected
9|resolve|Manual conflict resolution
";

    fs::write(path, content).map_err(|e| ReedError::IoError {
        operation: "write_actions_dict".to_string(),
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Creates default users dictionary.
///
/// Creates users.dict with system user (code 0).
///
/// ## Performance
/// - < 5ms
///
/// ## Error Conditions
/// - IoError: Cannot write file
fn create_default_user_dict(path: &Path) -> ReedResult<()> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before Unix epoch")
        .as_secs();

    let content = format!(
        "code|username|created_at\n0|system|{}\n1|admin|{}\n",
        timestamp, timestamp
    );

    fs::write(path, content).map_err(|e| ReedError::IoError {
        operation: "write_users_dict".to_string(),
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Validates dictionary integrity.
///
/// Checks CSV format and code uniqueness.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
///
/// ## Output
/// - `Result<()>`: Success if valid
///
/// ## Performance
/// - < 10ms for typical dictionaries
///
/// ## Error Conditions
/// - DictionaryCorrupted: Invalid CSV format
/// - DuplicateCode: Code collision detected
///
/// ## Example Usage
/// ```no_run
/// use reedbase::registry::validate_dictionaries;
/// use std::path::Path;
///
/// validate_dictionaries(Path::new(".reed"))?;
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn validate_dictionaries(base_path: &Path) -> ReedResult<()> {
    let registry_dir = base_path.join("registry");

    // Validate actions.dict
    validate_dict_file(&registry_dir.join("actions.dict"), "actions")?;

    // Validate users.dict
    validate_dict_file(&registry_dir.join("users.dict"), "users")?;

    Ok(())
}

/// Validates a single dictionary file.
fn validate_dict_file(path: &Path, dict_type: &str) -> ReedResult<()> {
    let content = fs::read_to_string(path).map_err(|e| ReedError::IoError {
        operation: format!("read_{}_dict", dict_type),
        reason: e.to_string(),
    })?;

    let mut seen_codes = std::collections::HashSet::new();

    for (line_num, line) in content.lines().enumerate() {
        // Skip header
        if line_num == 0 {
            continue;
        }

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse CSV line
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 2 {
            return Err(ReedError::DictionaryCorrupted {
                file: dict_type.to_string(),
                reason: format!(
                    "Invalid CSV format: expected at least 2 columns, got {}",
                    parts.len()
                ),
                line: line_num + 1,
            });
        }

        // Check code uniqueness
        let code = parts[0];
        if !seen_codes.insert(code.to_string()) {
            return Err(ReedError::DuplicateCode {
                code: code.to_string(),
                file: dict_type.to_string(),
            });
        }

        // Validate code is numeric
        if dict_type == "actions" {
            parts[0]
                .parse::<u8>()
                .map_err(|_| ReedError::DictionaryCorrupted {
                    file: dict_type.to_string(),
                    reason: format!("Invalid action code: '{}' (must be 0-255)", parts[0]),
                    line: line_num + 1,
                })?;
        } else if dict_type == "users" {
            parts[0]
                .parse::<u32>()
                .map_err(|_| ReedError::DictionaryCorrupted {
                    file: dict_type.to_string(),
                    reason: format!(
                        "Invalid user code: '{}' (must be unsigned integer)",
                        parts[0]
                    ),
                    line: line_num + 1,
                })?;
        }
    }

    Ok(())
}
