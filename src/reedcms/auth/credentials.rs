// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Credential Extraction
//!
//! Extracts authentication credentials from HTTP Authorization header.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use actix_web::HttpRequest;
use base64::{engine::general_purpose::STANDARD, Engine};

/// Authentication credentials enum.
#[derive(Debug, Clone)]
pub enum AuthCredentials {
    Basic { username: String, password: String },
    Bearer { token: String },
}

/// Extracts authentication credentials from request.
///
/// ## Input
/// - `req`: HTTP request with Authorization header
///
/// ## Supported Headers
/// - Authorization: Basic {base64}
/// - Authorization: Bearer {token}
///
/// ## Output
/// - `ReedResult<AuthCredentials>`: Parsed credentials
///
/// ## Process
/// 1. Get Authorization header
/// 2. Parse authentication type
/// 3. Extract credentials
/// 4. Decode if necessary
///
/// ## Error Conditions
/// - Returns `ReedError::AuthError` if header is missing
/// - Returns `ReedError::AuthError` if header format is invalid
/// - Returns `ReedError::AuthError` if authentication type is unsupported
///
/// ## Performance
/// - Extraction time: < 1ms
///
/// ## Example Usage
/// ```
/// let credentials = extract_auth_credentials(&req)?;
/// match credentials {
///     AuthCredentials::Basic { username, password } => { /* verify */ },
///     AuthCredentials::Bearer { token } => { /* verify token */ },
/// }
/// ```
pub fn extract_auth_credentials(req: &HttpRequest) -> ReedResult<AuthCredentials> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| ReedError::AuthError {
            user: None,
            action: "extract_credentials".to_string(),
            reason: "Missing Authorization header".to_string(),
        })?;

    let auth_str = auth_header.to_str().map_err(|_| ReedError::AuthError {
        user: None,
        action: "extract_credentials".to_string(),
        reason: "Invalid Authorization header format".to_string(),
    })?;

    // Parse authentication type
    if auth_str.starts_with("Basic ") {
        parse_basic_auth(auth_str)
    } else if auth_str.starts_with("Bearer ") {
        parse_bearer_auth(auth_str)
    } else {
        Err(ReedError::AuthError {
            user: None,
            action: "extract_credentials".to_string(),
            reason: "Unsupported authentication type".to_string(),
        })
    }
}

/// Parses HTTP Basic Auth credentials.
///
/// ## Input
/// - `auth_str`: Authorization header value (e.g., "Basic dXNlcjpwYXNz")
///
/// ## Output
/// - `ReedResult<AuthCredentials>`: Parsed username and password
///
/// ## Format
/// - Authorization: Basic base64(username:password)
///
/// ## Process
/// 1. Extract base64 part
/// 2. Decode base64
/// 3. Split by colon
/// 4. Return username and password
///
/// ## Error Conditions
/// - Returns `ReedError::AuthError` if format is invalid
/// - Returns `ReedError::AuthError` if base64 decoding fails
/// - Returns `ReedError::AuthError` if credentials don't contain colon
///
/// ## Example
/// - Input: "Basic dXNlcjpwYXNz"
/// - Output: AuthCredentials::Basic { username: "user", password: "pass" }
fn parse_basic_auth(auth_str: &str) -> ReedResult<AuthCredentials> {
    let encoded = auth_str
        .strip_prefix("Basic ")
        .ok_or_else(|| ReedError::AuthError {
            user: None,
            action: "parse_basic_auth".to_string(),
            reason: "Invalid Basic auth format".to_string(),
        })?;

    let decoded = STANDARD.decode(encoded).map_err(|_| ReedError::AuthError {
        user: None,
        action: "parse_basic_auth".to_string(),
        reason: "Invalid base64 encoding".to_string(),
    })?;

    let decoded_str = String::from_utf8(decoded).map_err(|_| ReedError::AuthError {
        user: None,
        action: "parse_basic_auth".to_string(),
        reason: "Invalid UTF-8 in credentials".to_string(),
    })?;

    let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(ReedError::AuthError {
            user: None,
            action: "parse_basic_auth".to_string(),
            reason: "Invalid credentials format (expected username:password)".to_string(),
        });
    }

    Ok(AuthCredentials::Basic {
        username: parts[0].to_string(),
        password: parts[1].to_string(),
    })
}

/// Parses Bearer token authentication.
///
/// ## Input
/// - `auth_str`: Authorization header value (e.g., "Bearer abc123")
///
/// ## Output
/// - `ReedResult<AuthCredentials>`: Parsed token
///
/// ## Format
/// - Authorization: Bearer {token}
///
/// ## Process
/// 1. Extract token part
/// 2. Validate token format
/// 3. Return token
///
/// ## Error Conditions
/// - Returns `ReedError::AuthError` if format is invalid
/// - Returns `ReedError::AuthError` if token is empty
///
/// ## Example
/// - Input: "Bearer abc123xyz"
/// - Output: AuthCredentials::Bearer { token: "abc123xyz" }
fn parse_bearer_auth(auth_str: &str) -> ReedResult<AuthCredentials> {
    let token = auth_str
        .strip_prefix("Bearer ")
        .ok_or_else(|| ReedError::AuthError {
            user: None,
            action: "parse_bearer_auth".to_string(),
            reason: "Invalid Bearer auth format".to_string(),
        })?;

    if token.is_empty() {
        return Err(ReedError::AuthError {
            user: None,
            action: "parse_bearer_auth".to_string(),
            reason: "Empty token".to_string(),
        });
    }

    Ok(AuthCredentials::Bearer {
        token: token.to_string(),
    })
}
