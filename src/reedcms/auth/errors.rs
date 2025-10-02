// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Error Responses
//!
//! Provides standardised HTTP error responses for authentication failures.

use actix_web::error::Error;

/// Creates 401 Unauthorized error response.
///
/// ## Output
/// - HTTP 401 Unauthorized
/// - JSON error body with message
///
/// ## Use Case
/// - Missing or invalid credentials
/// - Authentication required but not provided
///
/// ## Response Body
/// ```json
/// {
///   "error": "Unauthorized",
///   "message": "Authentication required",
///   "status": 401
/// }
/// ```
///
/// ## Example Usage
/// ```
/// if credentials.is_err() {
///     return Err(create_unauthorized_error());
/// }
/// ```
pub fn create_unauthorized_error() -> Error {
    actix_web::error::ErrorUnauthorized(
        serde_json::json!({
            "error": "Unauthorized",
            "message": "Authentication required",
            "status": 401
        })
        .to_string(),
    )
}

/// Creates 403 Forbidden error response.
///
/// ## Output
/// - HTTP 403 Forbidden
/// - JSON error body with message
///
/// ## Use Case
/// - Valid credentials but insufficient permissions
/// - Role or permission requirement not met
///
/// ## Response Body
/// ```json
/// {
///   "error": "Forbidden",
///   "message": "Insufficient permissions",
///   "status": 403
/// }
/// ```
///
/// ## Example Usage
/// ```
/// if !user.has_role("admin") {
///     return Err(create_forbidden_error());
/// }
/// ```
pub fn create_forbidden_error() -> Error {
    actix_web::error::ErrorForbidden(
        serde_json::json!({
            "error": "Forbidden",
            "message": "Insufficient permissions",
            "status": 403
        })
        .to_string(),
    )
}
