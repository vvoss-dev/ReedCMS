// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! API route configuration for ReedAPI HTTP Interface.
//!
//! This module configures all API endpoints under the `/api/v1` scope.
//! All endpoints require authentication via AuthMiddleware from REED-06-03.
//!
//! ## Endpoint Structure
//! - `/api/v1/text/*` - Text data operations
//! - `/api/v1/route/*` - Route data operations
//! - `/api/v1/meta/*` - Metadata operations
//! - `/api/v1/config/*` - Configuration operations
//! - `/api/v1/batch/*` - Batch operations
//! - `/api/v1/list/*` - List operations
//!
//! ## Authentication
//! All endpoints require valid authentication:
//! - Basic Auth: `Authorization: Basic <base64(username:password)>`
//! - Bearer Token: `Authorization: Bearer <api_key>`
//!
//! ## Performance
//! - Route configuration: O(1) setup time
//! - Request routing: O(1) lookup
//!
//! ## Example Usage
//! ```rust
//! use actix_web::App;
//! use crate::reedcms::api::routes::configure_api_routes;
//!
//! let app = App::new()
//!     .configure(configure_api_routes);
//! ```

use actix_web::web;

use crate::reedcms::api::get_handlers;
use crate::reedcms::api::set_handlers;
use crate::reedcms::api::batch_handlers;
use crate::reedcms::api::list_handlers;
use crate::reedcms::auth::middleware::AuthMiddleware;
use crate::reedcms::api::security::SecurityMiddleware;

/// Configures all API routes under /api/v1 scope.
///
/// ## Arguments
/// - `cfg`: Actix-web service configuration
///
/// ## Routes Configured
/// - Text operations: GET/POST /api/v1/text/get, /api/v1/text/set
/// - Route operations: GET/POST /api/v1/route/get, /api/v1/route/set
/// - Meta operations: GET/POST /api/v1/meta/get, /api/v1/meta/set
/// - Config operations: GET/POST /api/v1/config/get, /api/v1/config/set
/// - Batch operations: POST /api/v1/batch/get, /api/v1/batch/set
/// - List operations: GET /api/v1/list/text, /api/v1/list/routes, /api/v1/list/layouts
///
/// ## Authentication
/// All routes require authentication via AuthMiddleware.
///
/// ## Performance
/// - Route configuration: < 1ms during server initialisation
/// - Zero runtime overhead for route lookup
///
/// ## Example
/// ```rust
/// use actix_web::{App, web};
/// use crate::reedcms::api::routes::configure_api_routes;
///
/// let app = App::new()
///     .configure(configure_api_routes);
/// ```
pub fn configure_api_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .wrap(AuthMiddleware::authenticated())
            .wrap(SecurityMiddleware::new())
            // Text operations
            .service(
                web::scope("/text")
                    .route("/get", web::get().to(get_handlers::get_text))
                    .route("/set", web::post().to(set_handlers::set_text))
            )
            // Route operations
            .service(
                web::scope("/route")
                    .route("/get", web::get().to(get_handlers::get_route))
                    .route("/set", web::post().to(set_handlers::set_route))
            )
            // Meta operations
            .service(
                web::scope("/meta")
                    .route("/get", web::get().to(get_handlers::get_meta))
                    .route("/set", web::post().to(set_handlers::set_meta))
            )
            // Config operations
            .service(
                web::scope("/config")
                    .route("/get", web::get().to(get_handlers::get_config))
                    .route("/set", web::post().to(set_handlers::set_config))
            )
            // Batch operations
            .service(
                web::scope("/batch")
                    .route("/get", web::post().to(batch_handlers::batch_get))
                    .route("/set", web::post().to(batch_handlers::batch_set))
            )
            // List operations
            .service(
                web::scope("/list")
                    .route("/text", web::get().to(list_handlers::list_text))
                    .route("/routes", web::get().to(list_handlers::list_routes))
                    .route("/layouts", web::get().to(list_handlers::list_layouts))
            )
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_api_routes_configured() {
        // Create test app with API routes
        let app = test::init_service(
            App::new().configure(configure_api_routes)
        ).await;

        // Verify routes are configured (will return 401 Unauthorised without auth, not 404)
        let req = test::TestRequest::get()
            .uri("/api/v1/text/get?key=test.key")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should not be 404 (route exists), will be 401 (not authenticated)
        assert_ne!(resp.status().as_u16(), 404);
    }

    #[actix_web::test]
    async fn test_invalid_route_returns_404() {
        let app = test::init_service(
            App::new().configure(configure_api_routes)
        ).await;

        let req = test::TestRequest::get()
            .uri("/api/v1/invalid/endpoint")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Invalid route should return 404
        assert_eq!(resp.status().as_u16(), 404);
    }
}
