// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! ReedAPI HTTP Interface
//!
//! RESTful API for managing ReedBase data via HTTP endpoints.
//!
//! ## Module Structure
//! - `responses` - Standard API response types
//! - `routes` - Route configuration for all endpoints
//! - `get_handlers` - GET endpoint implementations
//! - `set_handlers` - POST endpoint implementations
//! - `batch_handlers` - Batch operation endpoints
//! - `list_handlers` - List operation endpoints
//! - `security` - Security matrix, rate limiting, API keys, middleware
//!
//! ## Endpoints
//! - `/api/v1/text/get` - Retrieve text data
//! - `/api/v1/text/set` - Set text data
//! - `/api/v1/route/get` - Retrieve route data
//! - `/api/v1/route/set` - Set route data
//! - `/api/v1/meta/get` - Retrieve metadata
//! - `/api/v1/meta/set` - Set metadata
//! - `/api/v1/config/get` - Retrieve configuration
//! - `/api/v1/config/set` - Set configuration
//! - `/api/v1/batch/get` - Batch retrieval
//! - `/api/v1/batch/set` - Batch setting
//! - `/api/v1/list/text` - List text keys
//! - `/api/v1/list/routes` - List route keys
//! - `/api/v1/list/layouts` - List layouts
//!
//! ## Authentication
//! All endpoints require authentication via AuthMiddleware:
//! - Basic Auth: `Authorization: Basic <base64(username:password)>`
//! - Bearer Token: `Authorization: Bearer <api_key>`
//!
//! ## Performance
//! - GET operations: O(1) cache lookup, < 10ms average
//! - SET operations: O(1) cache + O(n) CSV write, < 50ms average
//! - Batch operations: O(n) where n is batch size
//! - List operations: O(n) where n is total keys
//!
//! ## Example Usage
//! ```rust
//! use actix_web::App;
//! use reedcms::api::routes::configure_api_routes;
//!
//! let app = App::new()
//!     .configure(configure_api_routes);
//! ```

pub mod batch_handlers;
pub mod get_handlers;
pub mod list_handlers;
pub mod responses;
pub mod routes;
pub mod security;
pub mod set_handlers;

// Re-export commonly used types
pub use responses::{
    ApiBatchResponse, ApiBatchResult, ApiConfigResponse, ApiError, ApiResponse, ApiSuccess,
};

pub use routes::configure_api_routes;

// Re-export security components
pub use security::{check_rate_limit, ApiKeyManager, SecurityMatrix, SecurityMiddleware};
