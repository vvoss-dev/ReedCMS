// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Simple Site Protection
//!
//! Provides htaccess-style site-wide authentication using server.auth.* configuration.

use crate::reedcms::auth::credentials::extract_auth_credentials;
use crate::reedcms::auth::errors::create_unauthorized_error;
use crate::reedcms::reedbase::get::server;
use crate::reedcms::reedstream::{ReedError, ReedRequest};
use crate::reedcms::security::passwords::verify_password;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;

/// Simple site protection middleware for htaccess-style authentication.
///
/// ## Purpose
/// Protects entire website with single username/password from .reed/server.csv
///
/// ## Configuration Keys
/// - `server.auth.enabled`: true/false to enable/disable protection
/// - `server.auth.username`: Username for site access
/// - `server.auth.password`: Argon2 hashed password
///
/// ## Use Cases
/// - Staging servers
/// - Development environments
/// - Preview deployments
/// - Beta testing access
///
/// ## Difference from CMS User Auth
/// - Single username/password (not multi-user)
/// - No roles or permissions
/// - Simple pass/fail check
/// - Site-wide protection (not per-route)
///
/// ## Performance
/// - Config check: < 1ms (cached ReedBase lookup)
/// - Auth verification: ~100ms (Argon2)
/// - Bypass when disabled: < 1μs
///
/// ## Example Usage
/// ```rust
/// use actix_web::{App, HttpServer};
/// use reedcms::auth::SiteProtection;
///
/// HttpServer::new(|| {
///     App::new()
///         .wrap(SiteProtection::new())  // Protects entire site
///         .route("/", web::get().to(index))
/// })
/// ```
pub struct SiteProtection;

impl SiteProtection {
    /// Creates new site protection middleware.
    ///
    /// Reads configuration from .reed/server.csv:
    /// - server.auth.enabled = true/false
    /// - server.auth.username = username
    /// - server.auth.password = $argon2id$...
    pub fn new() -> Self {
        Self
    }
}

impl Default for SiteProtection {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for SiteProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SiteProtectionService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SiteProtectionService {
            service: Rc::new(service),
        }))
    }
}

/// Site protection middleware service.
pub struct SiteProtectionService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SiteProtectionService<S>
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
        let service = self.service.clone();

        Box::pin(async move {
            // Check if site protection is enabled
            let enabled = is_site_protection_enabled();

            if !enabled {
                // Protection disabled - allow all requests
                return service.call(req).await;
            }

            // Extract authorization header
            let auth_result = extract_auth_credentials(req.request());

            match auth_result {
                Ok(credentials) => {
                    // Verify credentials against server.auth.* config
                    match verify_site_credentials(&credentials).await {
                        Ok(true) => {
                            // Valid credentials - allow request
                            service.call(req).await
                        }
                        Ok(false) => {
                            // Invalid credentials
                            Err(create_unauthorized_error())
                        }
                        Err(_) => {
                            // Error during verification (e.g., missing config)
                            Err(create_unauthorized_error())
                        }
                    }
                }
                Err(_) => {
                    // No authentication provided
                    Err(create_unauthorized_error())
                }
            }
        })
    }
}

/// Checks if site protection is enabled.
///
/// ## Input
/// - None (reads from .reed/server.csv via ReedBase)
///
/// ## Output
/// - `bool`: true if server.auth.enabled = "true", false otherwise
///
/// ## Performance
/// - Lookup: < 1ms (ReedBase cached)
///
/// ## Default Behaviour
/// - Missing key: false (protection disabled)
/// - Invalid value: false (protection disabled)
fn is_site_protection_enabled() -> bool {
    let req = ReedRequest {
        key: "auth.enabled".to_string(),
        language: None,
        environment: None,
        context: None,
        value: None,
        description: None,
    };

    match server(&req) {
        Ok(response) => response.data == "true",
        Err(_) => false, // Default: disabled
    }
}

/// Verifies site credentials against server.auth.* configuration.
///
/// ## Input
/// - `credentials`: Parsed credentials from Authorization header
///
/// ## Output
/// - `Result<bool, ReedError>`: true if valid, false if invalid
///
/// ## Process
/// 1. Load server.auth.username from .reed/server.csv
/// 2. Load server.auth.password from .reed/server.csv
/// 3. Verify username matches
/// 4. Verify password with Argon2
///
/// ## Performance
/// - Config lookup: < 1ms (ReedBase cached)
/// - Password verification: ~100ms (Argon2)
///
/// ## Security
/// - Constant-time password comparison via Argon2
/// - No user enumeration (same error for invalid username/password)
async fn verify_site_credentials(
    credentials: &crate::reedcms::auth::credentials::AuthCredentials,
) -> Result<bool, ReedError> {
    use crate::reedcms::auth::credentials::AuthCredentials;

    match credentials {
        AuthCredentials::Basic { username, password } => {
            // Load configured username
            let req_user = ReedRequest {
                key: "auth.username".to_string(),
                language: None,
                environment: None,
                context: None,
                value: None,
                description: None,
            };

            let configured_username = match server(&req_user) {
                Ok(response) => response.data,
                Err(_) => return Ok(false), // No username configured
            };

            // Check username match
            if username != &configured_username {
                return Ok(false);
            }

            // Load configured password hash
            let req_pass = ReedRequest {
                key: "auth.password".to_string(),
                language: None,
                environment: None,
                context: None,
                value: None,
                description: None,
            };

            let configured_password_hash = match server(&req_pass) {
                Ok(response) => response.data,
                Err(_) => return Ok(false), // No password configured
            };

            // Verify password with Argon2
            match verify_password(password, &configured_password_hash) {
                Ok(valid) => Ok(valid),
                Err(_) => Ok(false), // Verification error = invalid
            }
        }
        AuthCredentials::Bearer { token: _ } => {
            // Bearer tokens not supported for site protection
            Ok(false)
        }
    }
}
