// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Registry system for ReedBase dictionaries.
//!
//! Provides global lookup tables for efficient integer encoding of frequently-used values:
//! - **Action codes**: Encode operation types (delete, create, update, etc.)
//! - **User codes**: Encode usernames with auto-increment
//!
//! ## Architecture
//!
//! ```text
//! .reed/registry/
//! ├── actions.dict    # code|name|description
//! └── users.dict      # code|username|created_at
//! ```
//!
//! ## Performance
//!
//! - Lookups: O(1) HashMap cached, < 100ns
//! - User creation: < 10ms (CSV append + cache update)
//! - Memory: < 50 KB for typical dictionaries
//!
//! ## Thread Safety
//!
//! - Read operations: Lock-free via `OnceLock`
//! - Write operations: Synchronized via `RwLock`
//! - User creation: Atomic via file locking

pub mod dictionary;
pub mod init;

#[cfg(test)]
mod dictionary_test;
#[cfg(test)]
mod init_test;

// Re-export public API
pub use dictionary::{
    get_action_code, get_action_name, get_or_create_user_code, get_username, reload_dictionaries,
    set_base_path,
};
pub use init::{init_registry, validate_dictionaries};
