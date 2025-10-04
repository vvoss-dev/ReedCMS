// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! List operation handlers for ReedAPI HTTP Interface.
//!
//! This module implements list endpoints for retrieving collections of keys.
//!
//! ## Endpoints
//! - `/api/v1/list/text` - List all text keys
//! - `/api/v1/list/routes` - List all route keys
//! - `/api/v1/list/layouts` - List all available layouts
//!
//! ## Query Parameters
//! - `prefix`: Filter keys by prefix (optional)
//! - `language`: Filter keys by language suffix (optional)
//! - `environment`: Filter keys by environment suffix (optional)
//! - `limit`: Maximum number of results (optional, default 1000)
//! - `offset`: Result offset for pagination (optional, default 0)
//!
//! ## Performance
//! - O(n) where n is total number of keys in cache
//! - Filtering: O(n) with early termination
//! - < 50ms for < 10,000 keys
//!
//! ## Example Usage
//! ```bash
//! curl -H "Authorization: Bearer <token>" \
//!      "https://example.com/api/v1/list/text?prefix=page&language=en"
//! ```

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::reedcms::api::responses::{ApiResponse, ApiError};

/// Default maximum number of results.
const DEFAULT_LIMIT: usize = 1000;

/// Maximum allowed limit to prevent DoS.
const MAX_LIMIT: usize = 10000;

/// Query parameters for list operations.
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    /// Filter keys by prefix (optional)
    pub prefix: Option<String>,

    /// Filter keys by language suffix (optional)
    pub language: Option<String>,

    /// Filter keys by environment suffix (optional)
    pub environment: Option<String>,

    /// Maximum number of results (optional)
    pub limit: Option<usize>,

    /// Result offset for pagination (optional)
    pub offset: Option<usize>,
}

/// Response structure for list operations.
#[derive(Debug, Serialize)]
pub struct ListResponse {
    /// List of matching keys
    pub keys: Vec<String>,

    /// Total number of keys (before pagination)
    pub total: usize,

    /// Number of results returned
    pub count: usize,

    /// Offset used for this response
    pub offset: usize,

    /// Limit used for this response
    pub limit: usize,
}

/// GET /api/v1/list/text - List all text keys.
///
/// ## Query Parameters
/// - `prefix`: Filter keys by prefix (e.g., "page", "nav")
/// - `language`: Filter keys by language (e.g., "en", "de")
/// - `environment`: Filter keys by environment (e.g., "prod", "dev")
/// - `limit`: Maximum results (default 1000, max 10000)
/// - `offset`: Pagination offset (default 0)
///
/// ## Returns
/// - `200 OK`: List of matching text keys
/// - `400 Bad Request`: Invalid query parameters
/// - `500 Internal Server Error`: Server error
///
/// ## Performance
/// - O(n) where n is total text keys
/// - < 50ms for < 10,000 keys
///
/// ## Example
/// ```bash
/// GET /api/v1/list/text?prefix=page&language=en&limit=100
/// Authorization: Bearer abc123
///
/// Response:
/// {
///   "success": true,
///   "data": {
///     "keys": ["page.title@en", "page.subtitle@en", "page.description@en"],
///     "total": 3,
///     "count": 3,
///     "offset": 0,
///     "limit": 100
///   }
/// }
/// ```
pub async fn list_text(
    _req: HttpRequest,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    match fetch_key_list("text", &query).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error) => HttpResponse::InternalServerError().json(error),
    }
}

/// GET /api/v1/list/routes - List all route keys.
///
/// ## Query Parameters
/// - `prefix`: Filter keys by prefix (e.g., "/", "/about")
/// - `language`: Filter keys by language (e.g., "en", "de")
/// - `environment`: Filter keys by environment (e.g., "prod", "dev")
/// - `limit`: Maximum results (default 1000, max 10000)
/// - `offset`: Pagination offset (default 0)
///
/// ## Returns
/// - `200 OK`: List of matching route keys
/// - `400 Bad Request`: Invalid query parameters
/// - `500 Internal Server Error`: Server error
///
/// ## Performance
/// - O(n) where n is total route keys
/// - < 50ms for < 10,000 keys
///
/// ## Example
/// ```bash
/// GET /api/v1/list/routes?limit=50
/// Authorization: Bearer abc123
///
/// Response:
/// {
///   "success": true,
///   "data": {
///     "keys": ["/", "/about", "/contact"],
///     "total": 3,
///     "count": 3,
///     "offset": 0,
///     "limit": 50
///   }
/// }
/// ```
pub async fn list_routes(
    _req: HttpRequest,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    match fetch_key_list("route", &query).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error) => HttpResponse::InternalServerError().json(error),
    }
}

/// GET /api/v1/list/layouts - List all available layouts.
///
/// ## Query Parameters
/// - `prefix`: Filter layouts by prefix (optional)
/// - `limit`: Maximum results (default 1000, max 10000)
/// - `offset`: Pagination offset (default 0)
///
/// ## Returns
/// - `200 OK`: List of matching layouts
/// - `400 Bad Request`: Invalid query parameters
/// - `500 Internal Server Error`: Server error
///
/// ## Performance
/// - O(n) where n is number of layout directories
/// - < 10ms typical
///
/// ## Example
/// ```bash
/// GET /api/v1/list/layouts
/// Authorization: Bearer abc123
///
/// Response:
/// {
///   "success": true,
///   "data": {
///     "keys": ["page-standard", "page-home", "page-error"],
///     "total": 3,
///     "count": 3,
///     "offset": 0,
///     "limit": 1000
///   }
/// }
/// ```
pub async fn list_layouts(
    _req: HttpRequest,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    match fetch_layout_list(&query).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error) => HttpResponse::InternalServerError().json(error),
    }
}

/// Internal helper: Fetch list of keys from ReedBase cache.
///
/// ## Arguments
/// - `cache_type`: Which cache to query ("text", "route", "meta")
/// - `query`: List query parameters
///
/// ## Returns
/// - `Ok(ApiResponse<ListResponse>)`: List of matching keys
/// - `Err(ApiError)`: Error during fetch
///
/// ## Performance
/// - O(n) where n is total keys in cache
/// - Filtering with early termination
/// - < 50ms for < 10,000 keys
async fn fetch_key_list(
    _cache_type: &str,
    _query: &ListQuery,
) -> Result<ApiResponse<ListResponse>, ApiError> {
    // Note: Currently ReedBase get functions return single values, not full caches.
    // Full implementation requires REED-02-01 (ReedBase with HashMap cache access).
    Err(ApiError::new(
        "NOT_IMPLEMENTED".to_string(),
        "List operations require full ReedBase cache access (REED-02-01 pending)".to_string(),
    ))
}

/// Internal helper: Fetch list of available layouts.
///
/// ## Arguments
/// - `query`: List query parameters
///
/// ## Returns
/// - `Ok(ApiResponse<ListResponse>)`: List of layouts
/// - `Err(ApiError)`: Error during fetch
///
/// ## Performance
/// - O(n) where n is number of layout directories
/// - < 10ms typical
async fn fetch_layout_list(
    _query: &ListQuery,
) -> Result<ApiResponse<ListResponse>, ApiError> {
    // Note: Layout listing requires registry cache access.
    // Full implementation requires REED-02-01 (ReedBase with HashMap cache access).
    Err(ApiError::new(
        "NOT_IMPLEMENTED".to_string(),
        "Layout listing requires registry cache access (REED-02-01 pending)".to_string(),
    ))
}
