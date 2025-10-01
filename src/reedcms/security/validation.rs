// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Validation Services
//!
//! Provides email, username, and uniqueness validation for user management.

use crate::reedcms::matrix::{read_matrix_csv, MatrixValue};
use crate::reedcms::reedstream::{validation_error, ReedResult};
use std::path::Path;

/// Validates email format using basic RFC 5322 pattern.
///
/// ## Input
/// - `email`: Email address to validate
///
/// ## Output
/// - `ReedResult<()>`: Ok if valid, error otherwise
///
/// ## Validation Rules
/// - Must contain exactly one @ symbol
/// - Local part (before @) must not be empty
/// - Domain part (after @) must contain at least one dot
/// - Domain part must not be empty
/// - Only alphanumeric, dots, hyphens, underscores, and plus allowed
///
/// ## Error Conditions
/// - Returns `ReedError::ValidationError` if format is invalid
///
/// ## Example Usage
/// ```
/// validate_email("user@example.com")?;     // Ok
/// validate_email("invalid.email")?;        // Error: no @ symbol
/// validate_email("@example.com")?;         // Error: empty local part
/// ```
pub fn validate_email(email: &str) -> ReedResult<()> {
    if email.is_empty() {
        return Err(validation_error("email", email, "Email cannot be empty"));
    }

    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(validation_error(
            "email",
            email,
            "Email must contain exactly one @ symbol",
        ));
    }

    let local = parts[0];
    let domain = parts[1];

    if local.is_empty() {
        return Err(validation_error(
            "email",
            email,
            "Email local part cannot be empty",
        ));
    }

    if domain.is_empty() || !domain.contains('.') {
        return Err(validation_error(
            "email",
            email,
            "Email domain must contain at least one dot",
        ));
    }

    // Check for valid characters (alphanumeric, dot, hyphen, underscore, plus)
    let valid_chars = |c: char| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '+';
    if !local.chars().all(valid_chars)
        || !domain
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
    {
        return Err(validation_error(
            "email",
            email,
            "Email contains invalid characters",
        ));
    }

    Ok(())
}

/// Validates username format and availability.
///
/// ## Input
/// - `username`: Username to validate
///
/// ## Output
/// - `ReedResult<()>`: Ok if valid, error otherwise
///
/// ## Validation Rules
/// - Length: 3-32 characters
/// - Allowed characters: alphanumeric and underscore only
/// - Must start with a letter
/// - Cannot end with underscore
///
/// ## Error Conditions
/// - Returns `ReedError::ValidationError` if format is invalid
///
/// ## Example Usage
/// ```
/// validate_username("admin")?;       // Ok
/// validate_username("user_123")?;    // Ok
/// validate_username("ab")?;          // Error: too short
/// validate_username("123user")?;     // Error: must start with letter
/// ```
pub fn validate_username(username: &str) -> ReedResult<()> {
    if username.len() < 3 {
        return Err(validation_error(
            "username",
            username,
            "Username must be at least 3 characters",
        ));
    }

    if username.len() > 32 {
        return Err(validation_error(
            "username",
            username,
            "Username must not exceed 32 characters",
        ));
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(validation_error(
            "username",
            username,
            "Username can only contain alphanumeric characters and underscores",
        ));
    }

    let first_char = username.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() {
        return Err(validation_error(
            "username",
            username,
            "Username must start with a letter",
        ));
    }

    if username.ends_with('_') {
        return Err(validation_error(
            "username",
            username,
            "Username cannot end with an underscore",
        ));
    }

    Ok(())
}

/// Checks if username already exists in users database.
///
/// ## Input
/// - `username`: Username to check
///
/// ## Output
/// - `ReedResult<bool>`: true if exists, false if available
///
/// ## Performance
/// - Reads .reed/users.matrix.csv
/// - O(n) scan through all users
/// - < 50ms for typical user count
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if file cannot be read
/// - Returns `ReedError::CsvError` if file is malformed
///
/// ## Example Usage
/// ```
/// if username_exists("admin")? {
///     return Err(validation_error("username", "admin", "Username already taken"));
/// }
/// ```
pub fn username_exists(username: &str) -> ReedResult<bool> {
    let users_path = Path::new(".reed/users.matrix.csv");

    // If file doesn't exist, no users exist yet
    if !users_path.exists() {
        return Ok(false);
    }

    let records = read_matrix_csv(users_path)?;

    for record in records {
        if let Some(MatrixValue::Single(existing_username)) = record.fields.get("username") {
            if existing_username == username {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Checks if email already exists in users database.
///
/// ## Input
/// - `email`: Email address to check
///
/// ## Output
/// - `ReedResult<bool>`: true if exists, false if available
///
/// ## Performance
/// - Reads .reed/users.matrix.csv
/// - O(n) scan through all users
/// - < 50ms for typical user count
///
/// ## Error Conditions
/// - Returns `ReedError::IoError` if file cannot be read
/// - Returns `ReedError::CsvError` if file is malformed
///
/// ## Example Usage
/// ```
/// if email_exists("admin@example.com")? {
///     return Err(validation_error("email", email, "Email already registered"));
/// }
/// ```
pub fn email_exists(email: &str) -> ReedResult<bool> {
    let users_path = Path::new(".reed/users.matrix.csv");

    // If file doesn't exist, no users exist yet
    if !users_path.exists() {
        return Ok(false);
    }

    let records = read_matrix_csv(users_path)?;

    for record in records {
        if let Some(MatrixValue::Single(existing_email)) = record.fields.get("email") {
            if existing_email == email {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
