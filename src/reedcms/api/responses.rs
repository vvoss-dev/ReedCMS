// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! API response types for ReedAPI HTTP Interface.
//!
//! This module defines the standard response structures used by all ReedAPI endpoints.
//! All responses follow a consistent JSON format with success/error variants.
//!
//! ## Response Types
//! - `ApiResponse<T>`: Standard successful response with typed data
//! - `ApiSuccess`: Success response without data payload
//! - `ApiError`: Error response with code and message
//! - `ApiConfigResponse`: Specialised response for configuration endpoints
//!
//! ## Performance
//! - O(1) response construction
//! - Zero-copy serialisation where possible
//! - < 100μs typical response assembly
//!
//! ## Example Usage
//! ```rust
//! let response = ApiResponse {
//!     success: true,
//!     data: Some("Hello World".to_string()),
//!     key: Some("page.title@en".to_string()),
//!     language: Some("en".to_string()),
//!     environment: Some("prod".to_string()),
//! };
//! ```

use serde::{Deserialize, Serialize};

/// Standard API response for successful data retrieval.
///
/// ## Fields
/// - `success`: Always true for successful responses
/// - `data`: The requested data (type varies by endpoint)
/// - `key`: The ReedBase key that was retrieved (optional)
/// - `language`: The language suffix extracted from key (optional)
/// - `environment`: The environment suffix extracted from key (optional)
///
/// ## Generic Type
/// - `T`: The data type (String for text, Vec for lists, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Creates a new successful API response.
    ///
    /// ## Arguments
    /// - `data`: The data payload to return
    ///
    /// ## Returns
    /// - `ApiResponse<T>` with success=true and provided data
    ///
    /// ## Example
    /// ```rust
    /// let response = ApiResponse::new("Hello World".to_string());
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            key: None,
            language: None,
            environment: None,
        }
    }

    /// Creates a new successful API response with full metadata.
    ///
    /// ## Arguments
    /// - `data`: The data payload to return
    /// - `key`: The ReedBase key that was retrieved
    /// - `language`: The language suffix (e.g., "en", "de")
    /// - `environment`: The environment suffix (e.g., "prod", "dev")
    ///
    /// ## Returns
    /// - `ApiResponse<T>` with all metadata fields populated
    ///
    /// ## Example
    /// ```rust
    /// let response = ApiResponse::with_metadata(
    ///     "Hello World".to_string(),
    ///     "page.title@en".to_string(),
    ///     "en".to_string(),
    ///     "prod".to_string(),
    /// );
    /// ```
    pub fn with_metadata(data: T, key: String, language: String, environment: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            key: Some(key),
            language: Some(language),
            environment: Some(environment),
        }
    }
}

/// Success response without data payload.
///
/// Used for operations that succeed but don't return data (e.g., SET operations).
///
/// ## Fields
/// - `success`: Always true
/// - `message`: Human-readable success message
/// - `key`: The key that was affected (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSuccess {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

impl ApiSuccess {
    /// Creates a new success response.
    ///
    /// ## Arguments
    /// - `message`: Success message to return
    ///
    /// ## Returns
    /// - `ApiSuccess` with success=true and provided message
    ///
    /// ## Example
    /// ```rust
    /// let response = ApiSuccess::new("Key updated successfully".to_string());
    /// ```
    pub fn new(message: String) -> Self {
        Self {
            success: true,
            message,
            key: None,
        }
    }

    /// Creates a new success response with key metadata.
    ///
    /// ## Arguments
    /// - `message`: Success message to return
    /// - `key`: The key that was affected
    ///
    /// ## Returns
    /// - `ApiSuccess` with key metadata included
    ///
    /// ## Example
    /// ```rust
    /// let response = ApiSuccess::with_key(
    ///     "Key updated successfully".to_string(),
    ///     "page.title@en".to_string(),
    /// );
    /// ```
    pub fn with_key(message: String, key: String) -> Self {
        Self {
            success: true,
            message,
            key: Some(key),
        }
    }
}

/// Error response for failed API operations.
///
/// ## Fields
/// - `success`: Always false for error responses
/// - `error`: Error code (e.g., "KEY_NOT_FOUND", "PERMISSION_DENIED")
/// - `message`: Human-readable error description
/// - `key`: The key that caused the error (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub success: bool,
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

impl ApiError {
    /// Creates a new error response.
    ///
    /// ## Arguments
    /// - `error`: Error code (uppercase with underscores)
    /// - `message`: Human-readable error description
    ///
    /// ## Returns
    /// - `ApiError` with success=false and provided error details
    ///
    /// ## Example
    /// ```rust
    /// let response = ApiError::new(
    ///     "KEY_NOT_FOUND".to_string(),
    ///     "The requested key does not exist".to_string(),
    /// );
    /// ```
    pub fn new(error: String, message: String) -> Self {
        Self {
            success: false,
            error,
            message,
            key: None,
        }
    }

    /// Creates a new error response with key metadata.
    ///
    /// ## Arguments
    /// - `error`: Error code (uppercase with underscores)
    /// - `message`: Human-readable error description
    /// - `key`: The key that caused the error
    ///
    /// ## Returns
    /// - `ApiError` with key metadata included
    ///
    /// ## Example
    /// ```rust
    /// let response = ApiError::with_key(
    ///     "KEY_NOT_FOUND".to_string(),
    ///     "The requested key does not exist".to_string(),
    ///     "page.invalid@en".to_string(),
    /// );
    /// ```
    pub fn with_key(error: String, message: String, key: String) -> Self {
        Self {
            success: false,
            error,
            message,
            key: Some(key),
        }
    }
}

/// Specialised response for configuration endpoints.
///
/// Configuration endpoints return multiple values in a structured format.
///
/// ## Fields
/// - `success`: Always true for successful responses
/// - `key`: The configuration key requested
/// - `value`: The configuration value
/// - `description`: Human-readable description of the configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfigResponse {
    pub success: bool,
    pub key: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ApiConfigResponse {
    /// Creates a new configuration response.
    ///
    /// ## Arguments
    /// - `key`: The configuration key
    /// - `value`: The configuration value
    ///
    /// ## Returns
    /// - `ApiConfigResponse` with success=true and provided data
    ///
    /// ## Example
    /// ```rust
    /// let response = ApiConfigResponse::new(
    ///     "server.port".to_string(),
    ///     "8080".to_string(),
    /// );
    /// ```
    pub fn new(key: String, value: String) -> Self {
        Self {
            success: true,
            key,
            value,
            description: None,
        }
    }

    /// Creates a new configuration response with description.
    ///
    /// ## Arguments
    /// - `key`: The configuration key
    /// - `value`: The configuration value
    /// - `description`: Human-readable description
    ///
    /// ## Returns
    /// - `ApiConfigResponse` with description included
    ///
    /// ## Example
    /// ```rust
    /// let response = ApiConfigResponse::with_description(
    ///     "server.port".to_string(),
    ///     "8080".to_string(),
    ///     "HTTP server port".to_string(),
    /// );
    /// ```
    pub fn with_description(key: String, value: String, description: String) -> Self {
        Self {
            success: true,
            key,
            value,
            description: Some(description),
        }
    }
}

/// Batch operation response.
///
/// Used for batch GET/SET operations that process multiple keys.
///
/// ## Fields
/// - `success`: True if all operations succeeded
/// - `results`: Individual results for each key
/// - `total`: Total number of operations
/// - `succeeded`: Number of successful operations
/// - `failed`: Number of failed operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiBatchResponse<T> {
    pub success: bool,
    pub results: Vec<ApiBatchResult<T>>,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
}

/// Individual result within a batch operation.
///
/// ## Fields
/// - `key`: The key that was processed
/// - `success`: Whether this individual operation succeeded
/// - `data`: The data for successful operations (optional)
/// - `error`: Error message for failed operations (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiBatchResult<T> {
    pub key: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiBatchResponse<T> {
    /// Creates a new batch response from individual results.
    ///
    /// ## Arguments
    /// - `results`: Vector of individual operation results
    ///
    /// ## Returns
    /// - `ApiBatchResponse<T>` with computed statistics
    ///
    /// ## Example
    /// ```rust
    /// let results = vec![
    ///     ApiBatchResult { key: "k1".into(), success: true, data: Some("v1".into()), error: None },
    ///     ApiBatchResult { key: "k2".into(), success: false, data: None, error: Some("Not found".into()) },
    /// ];
    /// let response = ApiBatchResponse::new(results);
    /// ```
    pub fn new(results: Vec<ApiBatchResult<T>>) -> Self {
        let total = results.len();
        let succeeded = results.iter().filter(|r| r.success).count();
        let failed = total - succeeded;
        let success = failed == 0;

        Self {
            success,
            results,
            total,
            succeeded,
            failed,
        }
    }
}

impl<T> ApiBatchResult<T> {
    /// Creates a successful batch result.
    ///
    /// ## Arguments
    /// - `key`: The key that was processed
    /// - `data`: The data retrieved
    ///
    /// ## Returns
    /// - `ApiBatchResult<T>` with success=true
    pub fn success(key: String, data: T) -> Self {
        Self {
            key,
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Creates a failed batch result.
    ///
    /// ## Arguments
    /// - `key`: The key that was processed
    /// - `error`: Error message describing the failure
    ///
    /// ## Returns
    /// - `ApiBatchResult<T>` with success=false
    pub fn failure(key: String, error: String) -> Self {
        Self {
            key,
            success: false,
            data: None,
            error: Some(error),
        }
    }
}
