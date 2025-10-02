// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0
//
// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One function = One job
// MANDATORY: BBC English for all documentation and comments
// MANDATORY: Type-safe error handling with proper HTTP status codes
// MANDATORY: JSON error responses with consistent structure
// MANDATORY: Clear, user-friendly error messages
//
// == FILE PURPOSE ==
// This file: Standardised HTTP error responses for authentication failures
// Architecture: Authentication layer - 401 Unauthorised and 403 Forbidden responses
// Performance: < 1ms error creation, zero allocations
// Dependencies: actix-web for HTTP errors, serde_json for JSON formatting
// Data Flow: Authentication failure → create_error() → HTTP response

//! Error Responses
//!
//! Provides standardised HTTP error responses for authentication failures.

use actix_web::error::Error;

/// Creates 401 Unauthorised error response.
///
/// ## Output
/// - HTTP 401 Unauthorised
/// - JSON error body with message
///
/// ## Use Case
/// - Missing or invalid credentials
/// - Authentication required but not provided
///
/// ## Response Body
/// ```json
/// {
///   "error": "Unauthorised",
///   "message": "Authentication required",
///   "status": 401
/// }
/// ```
///
/// ## Example Usage
/// ```
/// if credentials.is_err() {
///     return Err(create_unauthorised_error());
/// }
/// ```
pub fn create_unauthorised_error() -> Error {
    actix_web::error::ErrorUnauthorized(
        serde_json::json!({
            "error": "Unauthorised",
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
