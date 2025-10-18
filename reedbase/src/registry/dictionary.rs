// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Dictionary lookup operations with HashMap caching.
//!
//! Provides O(1) lookups for action and user code translations.

use crate::error::{ReedError, ReedResult};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Global actions cache (code → name).
static ACTIONS_BY_CODE: OnceLock<RwLock<HashMap<u8, String>>> = OnceLock::new();

/// Global actions reverse cache (name → code).
static ACTIONS_BY_NAME: OnceLock<RwLock<HashMap<String, u8>>> = OnceLock::new();

/// Global users cache (code → username).
static USERS_BY_CODE: OnceLock<RwLock<HashMap<u32, String>>> = OnceLock::new();

/// Global users reverse cache (username → code).
static USERS_BY_NAME: OnceLock<RwLock<HashMap<String, u32>>> = OnceLock::new();

/// Next available user code (auto-increment).
static NEXT_USER_CODE: OnceLock<RwLock<u32>> = OnceLock::new();

/// Base path for dictionaries.
static BASE_PATH: OnceLock<RwLock<PathBuf>> = OnceLock::new();

/// Initialises dictionary caches.
///
/// Must be called before any lookup operations.
fn ensure_initialized() -> ReedResult<()> {
    // Initialize base path if not set
    if BASE_PATH.get().is_none() {
        let _ = BASE_PATH.set(RwLock::new(PathBuf::from(".reed")));
    }

    // Initialize action caches
    if ACTIONS_BY_CODE.get().is_none() {
        let _ = ACTIONS_BY_CODE.set(RwLock::new(HashMap::new()));
        let _ = ACTIONS_BY_NAME.set(RwLock::new(HashMap::new()));
        load_actions_dict()?;
    }

    // Initialize user caches
    if USERS_BY_CODE.get().is_none() {
        let _ = USERS_BY_CODE.set(RwLock::new(HashMap::new()));
        let _ = USERS_BY_NAME.set(RwLock::new(HashMap::new()));
        let _ = NEXT_USER_CODE.set(RwLock::new(1)); // Start from 1 (0 is system)
        load_users_dict()?;
    }

    Ok(())
}

/// Loads actions dictionary into cache.
fn load_actions_dict() -> ReedResult<()> {
    let base_path = BASE_PATH
        .get()
        .ok_or_else(|| ReedError::IoError {
            operation: "get_base_path".to_string(),
            reason: "Base path not initialized".to_string(),
        })?
        .read()
        .expect("RwLock poisoned - cache corrupted");

    let path = base_path.join("registry/actions.dict");
    let content = fs::read_to_string(&path).map_err(|e| ReedError::IoError {
        operation: "read_actions_dict".to_string(),
        reason: e.to_string(),
    })?;

    let mut by_code = ACTIONS_BY_CODE
        .get()
        .expect("Cache not initialized")
        .write()
        .expect("RwLock poisoned");
    let mut by_name = ACTIONS_BY_NAME
        .get()
        .expect("Cache not initialized")
        .write()
        .expect("RwLock poisoned");

    by_code.clear();
    by_name.clear();

    for (line_num, line) in content.lines().enumerate() {
        if line_num == 0 {
            continue; // Skip header
        }

        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 2 {
            continue;
        }

        let code = parts[0]
            .parse::<u8>()
            .map_err(|_| ReedError::DictionaryCorrupted {
                file: "actions".to_string(),
                reason: format!("Invalid code: '{}'", parts[0]),
                line: line_num + 1,
            })?;

        let name = parts[1].to_string();

        by_code.insert(code, name.clone());
        by_name.insert(name.to_lowercase(), code);
    }

    Ok(())
}

/// Loads users dictionary into cache.
fn load_users_dict() -> ReedResult<()> {
    let base_path = BASE_PATH
        .get()
        .ok_or_else(|| ReedError::IoError {
            operation: "get_base_path".to_string(),
            reason: "Base path not initialized".to_string(),
        })?
        .read()
        .unwrap();

    let path = base_path.join("registry/users.dict");
    let content = fs::read_to_string(&path).map_err(|e| ReedError::IoError {
        operation: "read_users_dict".to_string(),
        reason: e.to_string(),
    })?;

    let mut by_code = USERS_BY_CODE
        .get()
        .expect("Cache not initialized")
        .write()
        .expect("RwLock poisoned");
    let mut by_name = USERS_BY_NAME
        .get()
        .expect("Cache not initialized")
        .write()
        .expect("RwLock poisoned");
    let mut next_code = NEXT_USER_CODE
        .get()
        .expect("Cache not initialized")
        .write()
        .expect("RwLock poisoned");

    by_code.clear();
    by_name.clear();
    *next_code = 1;

    for (line_num, line) in content.lines().enumerate() {
        if line_num == 0 {
            continue; // Skip header
        }

        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 2 {
            continue;
        }

        let code = parts[0]
            .parse::<u32>()
            .map_err(|_| ReedError::DictionaryCorrupted {
                file: "users".to_string(),
                reason: format!("Invalid code: '{}'", parts[0]),
                line: line_num + 1,
            })?;

        let username = parts[1].to_string();

        by_code.insert(code, username.clone());
        by_name.insert(username, code);

        // Track highest code for auto-increment
        if code >= *next_code {
            *next_code = code + 1;
        }
    }

    Ok(())
}

/// Gets action name from code.
///
/// ## Input
/// - `code`: Action code (0-255)
///
/// ## Output
/// - `Result<String>`: Action name
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100ns typical
///
/// ## Error Conditions
/// - UnknownActionCode: Code not in dictionary
///
/// ## Example Usage
/// ```no_run
/// use reedbase::registry::get_action_name;
///
/// let name = get_action_name(2)?; // "update"
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn get_action_name(code: u8) -> ReedResult<String> {
    ensure_initialized()?;

    let cache = ACTIONS_BY_CODE
        .get()
        .expect("Cache not initialized")
        .read()
        .expect("RwLock poisoned");
    cache
        .get(&code)
        .cloned()
        .ok_or(ReedError::UnknownActionCode { code })
}

/// Gets action code from name.
///
/// Reverse lookup: name → code. Case-insensitive.
///
/// ## Input
/// - `name`: Action name (e.g., "update", "UPDATE")
///
/// ## Output
/// - `Result<u8>`: Action code
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100ns typical
///
/// ## Error Conditions
/// - UnknownAction: Name not found
///
/// ## Example Usage
/// ```no_run
/// use reedbase::registry::get_action_code;
///
/// let code = get_action_code("update")?; // 2
/// let code2 = get_action_code("UPDATE")?; // 2 (case-insensitive)
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn get_action_code(name: &str) -> ReedResult<u8> {
    ensure_initialized()?;

    let cache = ACTIONS_BY_NAME
        .get()
        .expect("Cache not initialized")
        .read()
        .expect("RwLock poisoned");
    cache
        .get(&name.to_lowercase())
        .copied()
        .ok_or_else(|| ReedError::UnknownAction {
            name: name.to_string(),
        })
}

/// Gets username from code.
///
/// ## Input
/// - `code`: User code (0-4294967295)
///
/// ## Output
/// - `Result<String>`: Username
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100ns typical
///
/// ## Error Conditions
/// - UnknownUserCode: Code not in dictionary
///
/// ## Example Usage
/// ```no_run
/// use reedbase::registry::get_username;
///
/// let name = get_username(0)?; // "system"
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn get_username(code: u32) -> ReedResult<String> {
    ensure_initialized()?;

    let cache = USERS_BY_CODE
        .get()
        .expect("Cache not initialized")
        .read()
        .expect("RwLock poisoned");
    cache
        .get(&code)
        .cloned()
        .ok_or(ReedError::UnknownUserCode { code })
}

/// Gets or creates user code.
///
/// Auto-increments if user doesn't exist. Thread-safe.
///
/// ## Input
/// - `username`: Username to look up or create
///
/// ## Output
/// - `Result<u32>`: User code (existing or new)
///
/// ## Performance
/// - Existing user: < 100ns (cached)
/// - New user: < 10ms (append to CSV + cache update)
///
/// ## Error Conditions
/// - IoError: Cannot write to users.dict
/// - CsvError: CSV corruption
///
/// ## Example Usage
/// ```no_run
/// use reedbase::registry::get_or_create_user_code;
///
/// let code = get_or_create_user_code("alice")?; // 1 (first call)
/// let code2 = get_or_create_user_code("alice")?; // 1 (cached)
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn get_or_create_user_code(username: &str) -> ReedResult<u32> {
    ensure_initialized()?;

    // Check if user exists (read lock)
    {
        let cache = USERS_BY_NAME
            .get()
            .expect("Cache not initialized")
            .read()
            .expect("RwLock poisoned");
        if let Some(&code) = cache.get(username) {
            return Ok(code);
        }
    }

    // User doesn't exist - create new code (write lock)
    let mut by_code = USERS_BY_CODE
        .get()
        .expect("Cache not initialized")
        .write()
        .expect("RwLock poisoned");
    let mut by_name = USERS_BY_NAME
        .get()
        .expect("Cache not initialized")
        .write()
        .expect("RwLock poisoned");
    let mut next_code = NEXT_USER_CODE
        .get()
        .expect("Cache not initialized")
        .write()
        .expect("RwLock poisoned");

    // Double-check after acquiring write lock (another thread may have created it)
    if let Some(&code) = by_name.get(username) {
        return Ok(code);
    }

    // Assign new code
    let new_code = *next_code;
    *next_code += 1;

    // Append to CSV file
    let base_path = BASE_PATH
        .get()
        .expect("Cache not initialized")
        .read()
        .expect("RwLock poisoned");
    let path = base_path.join("registry/users.dict");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before Unix epoch")
        .as_secs();

    let line = format!("{}|{}|{}\n", new_code, username, timestamp);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| ReedError::IoError {
            operation: "append_users_dict".to_string(),
            reason: e.to_string(),
        })?;

    file.write_all(line.as_bytes())
        .map_err(|e| ReedError::IoError {
            operation: "write_users_dict".to_string(),
            reason: e.to_string(),
        })?;

    // Update cache
    by_code.insert(new_code, username.to_string());
    by_name.insert(username.to_string(), new_code);

    Ok(new_code)
}

/// Reloads dictionaries from disk.
///
/// Hot-reload for changes made externally.
///
/// ## Performance
/// - < 10ms for typical dictionary sizes
///
/// ## Error Conditions
/// - IoError: Cannot read dictionary files
/// - DictionaryCorrupted: CSV corruption
///
/// ## Example Usage
/// ```no_run
/// use reedbase::registry::reload_dictionaries;
///
/// reload_dictionaries()?;
/// # Ok::<(), reedbase::ReedError>(())
/// ```
pub fn reload_dictionaries() -> ReedResult<()> {
    // Force re-initialization
    ensure_initialized()?;
    load_actions_dict()?;
    load_users_dict()?;
    Ok(())
}

/// Sets base path for dictionaries.
///
/// Used for testing or non-standard locations.
/// Clears all caches to force reload with new path.
pub fn set_base_path(path: PathBuf) {
    if let Some(base_path_lock) = BASE_PATH.get() {
        if let Ok(mut base_path) = base_path_lock.write() {
            *base_path = path;
        }
    } else {
        let _ = BASE_PATH.set(RwLock::new(path));
    }

    // Clear caches
    if let Some(cache) = ACTIONS_BY_CODE.get() {
        if let Ok(mut c) = cache.write() {
            c.clear();
        }
    }
    if let Some(cache) = ACTIONS_BY_NAME.get() {
        if let Ok(mut c) = cache.write() {
            c.clear();
        }
    }
    if let Some(cache) = USERS_BY_CODE.get() {
        if let Ok(mut c) = cache.write() {
            c.clear();
        }
    }
    if let Some(cache) = USERS_BY_NAME.get() {
        if let Ok(mut c) = cache.write() {
            c.clear();
        }
    }
}
