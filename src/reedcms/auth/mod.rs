// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Authentication Module
//!
//! Provides HTTP authentication middleware with Basic Auth and Bearer token support.

pub mod credentials;
pub mod errors;
pub mod middleware;
pub mod rate_limit;
pub mod verification;

pub use credentials::{extract_auth_credentials, AuthCredentials};
pub use errors::{create_forbidden_error, create_unauthorized_error};
pub use middleware::{AuthMiddleware, AuthMiddlewareService};
pub use rate_limit::{clear_failed_logins, is_rate_limited, record_failed_login};
pub use verification::{verify_credentials, AuthenticatedUser};
