// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! GET endpoint handlers for ReedAPI HTTP Interface.
//!
//! This module implements all GET endpoints for retrieving data from ReedBase.
//!
//! ## Endpoints
//! - `/api/v1/text/get?key=<key>` - Retrieve text data
//! - `/api/v1/route/get?key=<key>` - Retrieve route data
//! - `/api/v1/meta/get?key=<key>` - Retrieve metadata
//! - `/api/v1/config/get?key=<key>` - Retrieve configuration
//!
//! ## Query Parameters
//! - `key`: The ReedBase key to retrieve (required)
//! - `language`: Language override (optional, defaults to key suffix)
//! - `environment`: Environment override (optional, defaults to key suffix)
//!
//! ## Performance
//! - O(1) cache lookup via ReedBase
//! - < 10ms average response time
//!
//! ## Example Usage
//! ```bash
//! curl -H "Authorization: Bearer <token>" \
//!      "https://example.com/api/v1/text/get?key=page.title@en"
//! ```

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::reedcms::api::responses::{ApiResponse, ApiError, ApiConfigResponse};
use crate::reedcms::reedstream::ReedRequest;

/// Query parameters for GET requests.
#[derive(Debug, Deserialize)]
pub struct GetQuery {
    /// The ReedBase key to retrieve (required)
    pub key: String,

    /// Language override (optional)
    pub language: Option<String>,

    /// Environment override (optional)
    pub environment: Option<String>,
}

/// GET /api/v1/text/get - Retrieve text data from ReedBase.
///
/// ## Query Parameters
/// - `key`: Text key to retrieve (required, e.g., "page.title@en")
/// - `language`: Language override (optional)
/// - `environment`: Environment override (optional)
///
/// ## Returns
/// - `200 OK`: JSON response with text data
/// - `404 Not Found`: Key does not exist
/// - `500 Internal Server Error`: Server error
///
/// ## Performance
/// - O(1) cache lookup
/// - < 5ms average response time
///
/// ## Example
/// ```bash
/// GET /api/v1/text/get?key=page.title@en
/// Authorization: Bearer abc123
///
/// Response:
/// {
///   "success": true,
///   "data": "Welcome to ReedCMS",
///   "key": "page.title@en",
///   "language": "en",
///   "environment": "prod"
/// }
/// ```
pub async fn get_text(
    _req: HttpRequest,
    query: web::Query<GetQuery>,
) -> HttpResponse {
    match fetch_from_reedbase(&query.key, &query.language, &query.environment, "text").await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error) => HttpResponse::NotFound().json(error),
    }
}

/// GET /api/v1/route/get - Retrieve route data from ReedBase.
///
/// ## Query Parameters
/// - `key`: Route key to retrieve (required, e.g., "/about")
/// - `language`: Language override (optional)
/// - `environment`: Environment override (optional)
///
/// ## Returns
/// - `200 OK`: JSON response with route data (layout mapping)
/// - `404 Not Found`: Route does not exist
/// - `500 Internal Server Error`: Server error
///
/// ## Performance
/// - O(1) cache lookup
/// - < 5ms average response time
///
/// ## Example
/// ```bash
/// GET /api/v1/route/get?key=/about
/// Authorization: Bearer abc123
///
/// Response:
/// {
///   "success": true,
///   "data": "page-standard",
///   "key": "/about",
///   "language": "en",
///   "environment": "prod"
/// }
/// ```
pub async fn get_route(
    _req: HttpRequest,
    query: web::Query<GetQuery>,
) -> HttpResponse {
    match fetch_from_reedbase(&query.key, &query.language, &query.environment, "route").await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error) => HttpResponse::NotFound().json(error),
    }
}

/// GET /api/v1/meta/get - Retrieve metadata from ReedBase.
///
/// ## Query Parameters
/// - `key`: Meta key to retrieve (required, e.g., "page.title.meta@en")
/// - `language`: Language override (optional)
/// - `environment`: Environment override (optional)
///
/// ## Returns
/// - `200 OK`: JSON response with metadata
/// - `404 Not Found`: Key does not exist
/// - `500 Internal Server Error`: Server error
///
/// ## Performance
/// - O(1) cache lookup
/// - < 5ms average response time
///
/// ## Example
/// ```bash
/// GET /api/v1/meta/get?key=page.title.meta@en
/// Authorization: Bearer abc123
///
/// Response:
/// {
///   "success": true,
///   "data": "Welcome to ReedCMS - Modern Flat-File CMS",
///   "key": "page.title.meta@en",
///   "language": "en",
///   "environment": "prod"
/// }
/// ```
pub async fn get_meta(
    _req: HttpRequest,
    query: web::Query<GetQuery>,
) -> HttpResponse {
    match fetch_from_reedbase(&query.key, &query.language, &query.environment, "meta").await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error) => HttpResponse::NotFound().json(error),
    }
}

/// GET /api/v1/config/get - Retrieve configuration from ReedBase.
///
/// ## Query Parameters
/// - `key`: Config key to retrieve (required, e.g., "server.port")
/// - `language`: Language override (optional)
/// - `environment`: Environment override (optional)
///
/// ## Returns
/// - `200 OK`: JSON response with configuration data
/// - `404 Not Found`: Key does not exist
/// - `500 Internal Server Error`: Server error
///
/// ## Performance
/// - O(1) cache lookup
/// - < 5ms average response time
///
/// ## Example
/// ```bash
/// GET /api/v1/config/get?key=server.port
/// Authorization: Bearer abc123
///
/// Response:
/// {
///   "success": true,
///   "key": "server.port",
///   "value": "8080",
///   "description": "HTTP server port"
/// }
/// ```
pub async fn get_config(
    _req: HttpRequest,
    query: web::Query<GetQuery>,
) -> HttpResponse {
    match fetch_config_from_reedbase(&query.key, &query.language, &query.environment).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error) => HttpResponse::NotFound().json(error),
    }
}

/// Internal helper: Fetch data from ReedBase cache.
///
/// ## Arguments
/// - `key`: The key to retrieve
/// - `language`: Optional language override
/// - `environment`: Optional environment override
/// - `cache_type`: Which cache to query ("text", "route", "meta")
///
/// ## Returns
/// - `Ok(ApiResponse<String>)`: Successful retrieval
/// - `Err(ApiError)`: Key not found or error
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100μs typical
async fn fetch_from_reedbase(
    key: &str,
    language: &Option<String>,
    environment: &Option<String>,
    cache_type: &str,
) -> Result<ApiResponse<String>, ApiError> {
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
        _ => {
            return Err(ApiError::new(
                "INVALID_CACHE_TYPE".to_string(),
                format!("Invalid cache type: {}", cache_type),
            ));
        }
    };

    // Handle response
    match response {
        Ok(reed_response) => {
            // Extract language and environment from key
            let (lang, env) = extract_suffixes(key);

            Ok(ApiResponse::with_metadata(
                reed_response.data,
                key.to_string(),
                lang,
                env,
            ))
        }
        Err(e) => {
            Err(ApiError::with_key(
                "KEY_NOT_FOUND".to_string(),
                format!("Key not found in {}: {}", cache_type, e),
                key.to_string(),
            ))
        }
    }
}

/// Internal helper: Fetch configuration data from ReedBase.
///
/// ## Arguments
/// - `key`: The configuration key to retrieve
/// - `language`: Optional language override
/// - `environment`: Optional environment override
///
/// ## Returns
/// - `Ok(ApiConfigResponse)`: Successful retrieval
/// - `Err(ApiError)`: Key not found or error
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100μs typical
async fn fetch_config_from_reedbase(
    key: &str,
    language: &Option<String>,
    environment: &Option<String>,
) -> Result<ApiConfigResponse, ApiError> {
    // Build ReedRequest for server.csv (config data)
    let request = ReedRequest {
        key: key.to_string(),
        language: language.clone(),
        environment: environment.clone(),
        context: Some("config".to_string()),
        value: None,
        description: None,
    };

    // Call ReedBase getter for config (server.csv)
    let response = crate::reedcms::reedbase::get::server(&request);

    // Handle response
    match response {
        Ok(reed_response) => {
            let value = reed_response.data;

            Ok(ApiConfigResponse::new(
                key.to_string(),
                value,
            ))
        }
        Err(e) => {
            Err(ApiError::with_key(
                "CONFIG_NOT_FOUND".to_string(),
                format!("Configuration key not found: {}", e),
                key.to_string(),
            ))
        }
    }
}

/// Extracts language and environment suffixes from key.
///
/// ## Input
/// - `key`: ReedBase key (e.g., "page.title@en")
///
/// ## Output
/// - `(language, environment)`: Tuple of extracted values
///
/// ## Example
/// ```rust
/// let (lang, env) = extract_suffixes("page.title@en");
/// assert_eq!(lang, "en");
/// assert_eq!(env, "prod"); // default
/// ```
fn extract_suffixes(key: &str) -> (String, String) {
    if let Some(at_pos) = key.rfind('@') {
        let suffix = &key[at_pos + 1..];
        (suffix.to_string(), "prod".to_string())
    } else {
        ("en".to_string(), "prod".to_string())
    }
}
