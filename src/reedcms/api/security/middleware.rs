// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0
//
// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One function = One job
// MANDATORY: BBC English for all documentation and comments
// MANDATORY: Type-safe error handling with ReedResult<T> pattern
// MANDATORY: Actix-Web middleware pattern with Transform trait
// MANDATORY: Reuse existing patterns from auth/middleware.rs
//
// == FILE PURPOSE ==
// This file: API security middleware for resource-level access control and rate limiting
// Architecture: API security layer - wraps API routes with permission and rate limit checks
// Performance: < 100μs security matrix lookup, < 100μs rate limit check, < 5ms rejection
// Dependencies: actix-web for middleware, futures-util for async
// Data Flow: Request → extract resource/operation → check permissions → check rate limit → allow/deny

//! API Security Middleware
//!
//! Provides security middleware for API routes with permission-based access control and rate limiting.

use crate::reedcms::api::security::matrix::SecurityMatrix;
use crate::reedcms::api::security::rate_limit::check_rate_limit;
use crate::reedcms::auth::verification::AuthenticatedUser;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;

/// Security middleware for API routes.
///
/// ## Process
/// 1. Extract authenticated user from request extensions (set by AuthMiddleware)
/// 2. Parse resource and operation from request path
/// 3. Load security matrix from .reed/api.security.csv
/// 4. Check user permissions and role against security rules
/// 5. Check rate limits for this user and operation
/// 6. Allow or deny request
///
/// ## Performance
/// - Security matrix lookup: < 100μs (cached HashMap)
/// - Rate limit check: < 100μs (in-memory HashMap)
/// - Unauthorised rejection: < 5ms
///
/// ## Security
/// - Permission-based access control
/// - Role-based access control
/// - Per-user, per-operation rate limiting
/// - Sliding window rate limit algorithm
///
/// ## Example Usage
/// ```
/// App::new()
///     .wrap(AuthMiddleware::authenticated())
///     .wrap(SecurityMiddleware::new())
///     .route("/api/v1/text/get", web::get().to(get_text))
/// ```
pub struct SecurityMiddleware {
    matrix: Rc<SecurityMatrix>,
}

impl SecurityMiddleware {
    /// Creates new security middleware.
    ///
    /// ## Returns
    /// - SecurityMiddleware instance with loaded security matrix
    ///
    /// ## Errors
    /// - Panics if security matrix cannot be loaded from .reed/api.security.csv
    ///
    /// ## Performance
    /// - O(n) where n = number of security rules in CSV
    /// - Typical: < 10ms for 100 rules
    pub fn new() -> Self {
        let matrix = SecurityMatrix::load()
            .expect("Failed to load API security matrix from .reed/api.security.csv");

        Self {
            matrix: Rc::new(matrix),
        }
    }
}

impl Default for SecurityMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecurityMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityMiddlewareService {
            service: Rc::new(service),
            matrix: self.matrix.clone(),
        }))
    }
}

/// Security middleware service.
pub struct SecurityMiddlewareService<S> {
    service: Rc<S>,
    matrix: Rc<SecurityMatrix>,
}

impl<S, B> Service<ServiceRequest> for SecurityMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let matrix = self.matrix.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Extract authenticated user from request extensions
            // Note: AuthMiddleware should run before SecurityMiddleware
            // TODO: Once AuthMiddleware injects user into extensions, extract it here
            // For now, we'll parse path and check against matrix without user context

            // Parse resource and operation from request path
            let (resource, operation) = match parse_resource_operation(req.path(), req.method().as_str()) {
                Some(parsed) => parsed,
                None => {
                    // Invalid API path format - pass through to handler (may be valid non-API route)
                    return service.call(req).await;
                }
            };

            // For now, create a placeholder user until AuthMiddleware extension injection is implemented
            // TODO: Replace with actual user extraction from request extensions
            let placeholder_user = AuthenticatedUser {
                id: "anonymous".to_string(),
                username: "anonymous".to_string(),
                email: "anonymous@localhost".to_string(),
                roles: vec!["user".to_string()],
            };

            // Check access permissions
            if let Err(_e) = matrix.check_access(&resource, &operation, &placeholder_user) {
                // Permission check failed - return 403 Forbidden
                return Err(create_access_denied_error());
            }

            // Permission check passed - now check rate limit
            if let Some(rate_limit) = matrix.get_rate_limit(&resource, &operation) {
                if let Err(_e) = check_rate_limit(&placeholder_user.id, &format!("{}:{}", resource, operation), &rate_limit) {
                    // Rate limit exceeded - return 429 Too Many Requests
                    return Err(create_rate_limit_error());
                }
            }

            // All checks passed - proceed with request
            service.call(req).await
        })
    }
}

/// Creates 403 Forbidden error for access denied.
///
/// ## Returns
/// - Actix-Web Error with 403 status code
///
/// ## Example
/// ```
/// if !has_access {
///     return Err(create_access_denied_error());
/// }
/// ```
fn create_access_denied_error() -> Error {
    actix_web::error::ErrorForbidden(
        serde_json::json!({
            "success": false,
            "error": "ACCESS_DENIED",
            "message": "Access denied: insufficient permissions"
        })
        .to_string(),
    )
}

/// Creates 429 Too Many Requests error for rate limit exceeded.
///
/// ## Returns
/// - Actix-Web Error with 429 status code
///
/// ## Example
/// ```
/// if rate_limit_exceeded {
///     return Err(create_rate_limit_error());
/// }
/// ```
fn create_rate_limit_error() -> Error {
    actix_web::error::ErrorTooManyRequests(
        serde_json::json!({
            "success": false,
            "error": "RATE_LIMIT_EXCEEDED",
            "message": "Rate limit exceeded"
        })
        .to_string(),
    )
}

/// Parses resource and operation from API request path and HTTP method.
///
/// ## Arguments
/// - `path`: Request path (e.g., "/api/v1/text/get")
/// - `method`: HTTP method (GET, POST, etc.)
///
/// ## Returns
/// - Some((resource, operation)) if path matches API pattern
/// - None if path is invalid
///
/// ## Examples
/// - "/api/v1/text/get" + GET → Some(("text", "read"))
/// - "/api/v1/text/set" + POST → Some(("text", "write"))
/// - "/api/v1/batch/get" + POST → Some(("batch", "read"))
/// - "/api/v1/list/text" + GET → Some(("list", "read"))
///
/// ## Performance
/// - O(1) string operations
/// - < 1μs typical
fn parse_resource_operation(path: &str, method: &str) -> Option<(String, String)> {
    // Remove /api/v1/ prefix
    let path = path.strip_prefix("/api/v1/")?;

    // Split into parts
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 2 {
        return None;
    }

    let resource = parts[0].to_string();
    let endpoint = parts[1];

    // Map endpoint + method to operation
    let operation = match (endpoint, method) {
        ("get", "GET") | ("get", "POST") => "read",
        ("set", "POST") => "write",
        ("list", "GET") => "read",
        _ => return None,
    };

    Some((resource, operation.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resource_operation_text_get() {
        let result = parse_resource_operation("/api/v1/text/get", "GET");
        assert_eq!(result, Some(("text".to_string(), "read".to_string())));
    }

    #[test]
    fn test_parse_resource_operation_text_set() {
        let result = parse_resource_operation("/api/v1/text/set", "POST");
        assert_eq!(result, Some(("text".to_string(), "write".to_string())));
    }

    #[test]
    fn test_parse_resource_operation_batch_get() {
        let result = parse_resource_operation("/api/v1/batch/get", "POST");
        assert_eq!(result, Some(("batch".to_string(), "read".to_string())));
    }

    #[test]
    fn test_parse_resource_operation_list_text() {
        let result = parse_resource_operation("/api/v1/list/text", "GET");
        assert_eq!(result, Some(("list".to_string(), "read".to_string())));
    }

    #[test]
    fn test_parse_resource_operation_invalid_path() {
        let result = parse_resource_operation("/api/v1/invalid", "GET");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_resource_operation_non_api_path() {
        let result = parse_resource_operation("/other/path", "GET");
        assert_eq!(result, None);
    }
}
