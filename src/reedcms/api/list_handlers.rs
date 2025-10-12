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

use crate::reedcms::api::responses::{ApiError, ApiResponse};

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
pub async fn list_text(_req: HttpRequest, query: web::Query<ListQuery>) -> HttpResponse {
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
pub async fn list_routes(_req: HttpRequest, query: web::Query<ListQuery>) -> HttpResponse {
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
pub async fn list_layouts(_req: HttpRequest, query: web::Query<ListQuery>) -> HttpResponse {
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
    cache_type: &str,
    query: &ListQuery,
) -> Result<ApiResponse<ListResponse>, ApiError> {
    // Determine CSV file path based on cache type
    let csv_path = match cache_type {
        "text" => ".reed/text.csv",
        "route" => ".reed/routes.csv",
        "meta" => ".reed/meta.csv",
        _ => {
            return Err(ApiError::new(
                "INVALID_CACHE_TYPE".to_string(),
                format!("Invalid cache type: {}", cache_type),
            ));
        }
    };

    // Read all keys from CSV file
    let all_keys = match read_keys_from_csv(csv_path).await {
        Ok(keys) => keys,
        Err(e) => {
            return Err(ApiError::new(
                "CSV_READ_FAILED".to_string(),
                format!("Failed to read keys from {}: {}", csv_path, e),
            ));
        }
    };

    // Apply filters
    let filtered_keys = apply_filters(&all_keys, query);

    // Apply pagination
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = query.offset.unwrap_or(0);
    let total = filtered_keys.len();

    let paginated_keys: Vec<String> = filtered_keys.into_iter().skip(offset).take(limit).collect();

    let count = paginated_keys.len();

    // Build response
    let list_response = ListResponse {
        keys: paginated_keys,
        total,
        count,
        offset,
        limit,
    };

    Ok(ApiResponse::new(list_response))
}

/// Reads all keys from a CSV file.
///
/// ## Arguments
/// - `csv_path`: Path to CSV file
///
/// ## Returns
/// - `Result<Vec<String>, String>`: List of keys or error
///
/// ## Performance
/// - O(n) where n is number of lines in CSV
/// - < 10ms for < 10,000 lines
async fn read_keys_from_csv(csv_path: &str) -> Result<Vec<String>, String> {
    use crate::reedcms::csv::read_csv;

    let records =
        read_csv(csv_path).map_err(|e| format!("Failed to read CSV {}: {}", csv_path, e))?;

    let keys: Vec<String> = records.into_iter().map(|r| r.key).collect();

    Ok(keys)
}

/// Applies filters to key list.
///
/// ## Arguments
/// - `keys`: List of all keys
/// - `query`: Filter parameters
///
/// ## Returns
/// - Filtered list of keys
///
/// ## Performance
/// - O(n) where n is number of keys
/// - Early termination where possible
fn apply_filters(keys: &[String], query: &ListQuery) -> Vec<String> {
    keys.iter()
        .filter(|key| {
            // Prefix filter
            if let Some(prefix) = &query.prefix {
                if !key.starts_with(prefix) {
                    return false;
                }
            }

            // Language filter
            if let Some(language) = &query.language {
                if !key.ends_with(&format!("@{}", language)) {
                    return false;
                }
            }

            // Environment filter (checking for @env or @lang@env patterns)
            if let Some(environment) = &query.environment {
                if !key.contains(&format!("@{}", environment)) {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect()
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
async fn fetch_layout_list(query: &ListQuery) -> Result<ApiResponse<ListResponse>, ApiError> {
    // Read layout names from templates/layouts directory
    use std::fs;

    let layouts_dir = "templates/layouts";

    let entries = match fs::read_dir(layouts_dir) {
        Ok(entries) => entries,
        Err(e) => {
            return Err(ApiError::new(
                "DIRECTORY_READ_FAILED".to_string(),
                format!("Failed to read layouts directory: {}", e),
            ));
        }
    };

    // Extract layout directory names
    let mut layouts: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    layouts.push(name.to_string());
                }
            }
        }
    }

    layouts.sort();

    // Apply prefix filter if specified
    let filtered_layouts = if let Some(prefix) = &query.prefix {
        layouts
            .into_iter()
            .filter(|layout| layout.starts_with(prefix))
            .collect()
    } else {
        layouts
    };

    // Apply pagination
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = query.offset.unwrap_or(0);
    let total = filtered_layouts.len();

    let paginated_layouts: Vec<String> = filtered_layouts
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    let count = paginated_layouts.len();

    // Build response
    let list_response = ListResponse {
        keys: paginated_layouts,
        total,
        count,
        offset,
        limit,
    };

    Ok(ApiResponse::new(list_response))
}
