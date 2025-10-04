// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0
//
// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One function = One job
// MANDATORY: BBC English for all documentation and comments
// MANDATORY: Clear module organisation with public exports
//
// == FILE PURPOSE ==
// This file: Module exports for API security components
// Architecture: API security layer - permission matrix, rate limiting, API keys, middleware
// Dependencies: Actix-Web for middleware, CSV for configuration storage

//! API Security Module
//!
//! Provides comprehensive security for ReedAPI with:
//! - Permission-based access control via security matrix
//! - Role-based access control
//! - Per-user, per-operation rate limiting
//! - API key management with expiration and revocation
//! - Security middleware integration with Actix-Web

pub mod api_keys;
pub mod matrix;
pub mod middleware;
pub mod rate_limit;

// Public exports for convenience
pub use api_keys::{generate_random_key, hash_api_key, ApiKeyManager};
pub use matrix::{RateLimit, RateLimitPeriod, SecurityMatrix, SecurityRule};
pub use middleware::SecurityMiddleware;
pub use rate_limit::{check_rate_limit, cleanup_rate_limits, start_cleanup_thread};
