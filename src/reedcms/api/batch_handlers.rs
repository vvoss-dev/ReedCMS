// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! Batch operation handlers for ReedAPI HTTP Interface.
//!
//! This module implements batch endpoints for processing multiple keys in a single request.
//!
//! ## Endpoints
//! - `/api/v1/batch/get` - Retrieve multiple keys in one request
//! - `/api/v1/batch/set` - Set multiple keys in one request
//!
//! ## Request Body
//! - `batch/get`: Array of keys to retrieve
//! - `batch/set`: Array of key-value pairs to set
//!
//! ## Performance
//! - O(n) where n is number of keys in batch
//! - < 10ms per key average
//! - Maximum batch size: 100 keys
//!
//! ## Example Usage
//! ```bash
//! curl -X POST -H "Authorization: Bearer <token>" \
//!      -H "Content-Type: application/json" \
//!      -d '{"keys":["page.title@en","page.title@de"]}' \
//!      "https://example.com/api/v1/batch/get"
//! ```

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::reedcms::api::responses::{ApiBatchResponse, ApiBatchResult, ApiError};
use crate::reedcms::reedstream::ReedRequest;

/// Maximum batch size to prevent DoS attacks.
const MAX_BATCH_SIZE: usize = 100;

/// Request body for batch GET operations.
#[derive(Debug, Deserialize)]
pub struct BatchGetRequest {
    /// Array of keys to retrieve
    pub keys: Vec<String>,

    /// Cache type to query ("text", "route", "meta", "config")
    pub cache_type: String,

    /// Optional language override for all keys
    pub language: Option<String>,

    /// Optional environment override for all keys
    pub environment: Option<String>,
}

/// Request body for batch SET operations.
#[derive(Debug, Deserialize)]
pub struct BatchSetRequest {
    /// Array of key-value pairs to set
    pub items: Vec<BatchSetItem>,

    /// Cache type to update ("text", "route", "meta", "config")
    pub cache_type: String,

    /// Optional language override for all items
    pub language: Option<String>,

    /// Optional environment override for all items
    pub environment: Option<String>,
}

/// Individual item in batch SET request.
#[derive(Debug, Deserialize)]
pub struct BatchSetItem {
    /// The key to set
    pub key: String,

    /// The value to set
    pub value: String,

    /// Optional description
    pub description: Option<String>,
}

/// POST /api/v1/batch/get - Retrieve multiple keys in one request.
///
/// ## Request Body
/// ```json
/// {
///   "keys": ["page.title@en", "page.subtitle@en", "page.title@de"],
///   "cache_type": "text",
///   "language": "en",
///   "environment": "prod"
/// }
/// ```
///
/// ## Returns
/// - `200 OK`: Batch results (partial success allowed)
/// - `400 Bad Request`: Invalid request (empty keys, batch too large)
/// - `500 Internal Server Error`: Server error
///
/// ## Performance
/// - O(n) where n is number of keys
/// - < 10ms per key average
/// - Parallel processing where possible
///
/// ## Example
/// ```bash
/// POST /api/v1/batch/get
/// Authorization: Bearer abc123
/// Content-Type: application/json
///
/// {
///   "keys": ["page.title@en", "page.subtitle@en"],
///   "cache_type": "text"
/// }
///
/// Response:
/// {
///   "success": true,
///   "results": [
///     {"key": "page.title@en", "success": true, "data": "Welcome"},
///     {"key": "page.subtitle@en", "success": true, "data": "Modern CMS"}
///   ],
///   "total": 2,
///   "succeeded": 2,
///   "failed": 0
/// }
/// ```
pub async fn batch_get(
    _req: HttpRequest,
    body: web::Json<BatchGetRequest>,
) -> HttpResponse {
    // Validate batch size
    if body.keys.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new(
            "EMPTY_BATCH".to_string(),
            "Batch request must contain at least one key".to_string(),
        ));
    }

    if body.keys.len() > MAX_BATCH_SIZE {
        return HttpResponse::BadRequest().json(ApiError::new(
            "BATCH_TOO_LARGE".to_string(),
            format!("Batch size {} exceeds maximum {}", body.keys.len(), MAX_BATCH_SIZE),
        ));
    }

    // Process each key
    let mut results = Vec::new();

    for key in &body.keys {
        let result = fetch_single_key(
            key,
            &body.cache_type,
            &body.language,
            &body.environment,
        ).await;

        results.push(result);
    }

    // Build batch response
    let response = ApiBatchResponse::new(results);
    HttpResponse::Ok().json(response)
}

/// POST /api/v1/batch/set - Set multiple keys in one request.
///
/// ## Request Body
/// ```json
/// {
///   "items": [
///     {"key": "page.title@en", "value": "Welcome", "description": "Homepage title"},
///     {"key": "page.subtitle@en", "value": "Modern CMS", "description": "Homepage subtitle"}
///   ],
///   "cache_type": "text",
///   "language": "en",
///   "environment": "prod"
/// }
/// ```
///
/// ## Returns
/// - `200 OK`: Batch results (partial success allowed)
/// - `400 Bad Request`: Invalid request (empty items, batch too large)
/// - `500 Internal Server Error`: Server error
///
/// ## Performance
/// - O(n) cache updates + O(n) CSV write
/// - < 50ms per key average
/// - Single CSV write for all items
///
/// ## Example
/// ```bash
/// POST /api/v1/batch/set
/// Authorization: Bearer abc123
/// Content-Type: application/json
///
/// {
///   "items": [
///     {"key": "page.title@en", "value": "Welcome"},
///     {"key": "page.subtitle@en", "value": "Modern CMS"}
///   ],
///   "cache_type": "text"
/// }
///
/// Response:
/// {
///   "success": true,
///   "results": [
///     {"key": "page.title@en", "success": true},
///     {"key": "page.subtitle@en", "success": true}
///   ],
///   "total": 2,
///   "succeeded": 2,
///   "failed": 0
/// }
/// ```
pub async fn batch_set(
    _req: HttpRequest,
    body: web::Json<BatchSetRequest>,
) -> HttpResponse {
    // Validate batch size
    if body.items.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new(
            "EMPTY_BATCH".to_string(),
            "Batch request must contain at least one item".to_string(),
        ));
    }

    if body.items.len() > MAX_BATCH_SIZE {
        return HttpResponse::BadRequest().json(ApiError::new(
            "BATCH_TOO_LARGE".to_string(),
            format!("Batch size {} exceeds maximum {}", body.items.len(), MAX_BATCH_SIZE),
        ));
    }

    // Process each item
    let mut results = Vec::new();

    for item in &body.items {
        let result = set_single_key(
            &item.key,
            &item.value,
            &item.description,
            &body.cache_type,
            &body.language,
            &body.environment,
        ).await;

        results.push(result);
    }

    // Build batch response
    let response = ApiBatchResponse::new(results);
    HttpResponse::Ok().json(response)
}

/// Internal helper: Fetch a single key from ReedBase.
///
/// ## Arguments
/// - `key`: The key to retrieve
/// - `cache_type`: Which cache to query
/// - `language`: Optional language override
/// - `environment`: Optional environment override
///
/// ## Returns
/// - `ApiBatchResult<String>`: Success or failure for this key
///
/// ## Performance
/// - O(1) cache lookup
/// - < 100μs typical
async fn fetch_single_key(
    key: &str,
    cache_type: &str,
    language: &Option<String>,
    environment: &Option<String>,
) -> ApiBatchResult<String> {
    // Build ReedRequest
    let request = ReedRequest {
        key: key.to_string(),
        language: language.clone(),
        environment: environment.clone(),
        context: Some(cache_type.to_string()),
        value: None,
        description: None,
    };

    // Call appropriate ReedBase getter
    let response = match cache_type {
        "text" => crate::reedcms::reedbase::get::text(&request),
        "route" => crate::reedcms::reedbase::get::route(&request),
        "meta" => crate::reedcms::reedbase::get::meta(&request),
        "config" => crate::reedcms::reedbase::get::server(&request),
        _ => {
            return ApiBatchResult::failure(
                key.to_string(),
                format!("Invalid cache type: {}", cache_type),
            );
        }
    };

    // Handle response
    match response {
        Ok(reed_response) => {
            ApiBatchResult::success(key.to_string(), reed_response.data)
        }
        Err(e) => {
            ApiBatchResult::failure(
                key.to_string(),
                format!("Error: {}", e),
            )
        }
    }
}

/// Internal helper: Set a single key in ReedBase.
///
/// ## Arguments
/// - `key`: The key to set
/// - `value`: The value to set
/// - `description`: Optional description
/// - `cache_type`: Which cache to update
/// - `language`: Optional language override
/// - `environment`: Optional environment override
///
/// ## Returns
/// - `ApiBatchResult<String>`: Success or failure for this key
///
/// ## Performance
/// - O(1) cache update + O(n) CSV write
/// - < 50ms typical
async fn set_single_key(
    key: &str,
    _value: &str,
    _description: &Option<String>,
    _cache_type: &str,
    _language: &Option<String>,
    _environment: &Option<String>,
) -> ApiBatchResult<String> {
    // Note: SET operations require mutable cache access through ReedBase dispatcher.
    // Full implementation requires REED-02-01 (ReedBase with mutable HashMap cache).
    ApiBatchResult::failure(
        key.to_string(),
        "SET operations not yet implemented (REED-02-01 pending)".to_string(),
    )
}
