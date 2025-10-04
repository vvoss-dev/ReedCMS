// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0
//
// == AI CODING GUIDELINES ==
// MANDATORY: Follow KISS principle - One function = One job
// MANDATORY: BBC English for all documentation and comments
// MANDATORY: Type-safe HashMap lookups, O(1) performance priority
// MANDATORY: Environment-aware with @suffix support (key@dev, key@prod)
// MANDATORY: CSV format: key|value|description (pipe-delimited, quoted when needed)
// MANDATORY: Error handling with ReedResult<T> pattern
//
// == FILE PURPOSE ==
// This file: Universal communication interface for all ReedCMS modules
// Architecture: Foundation layer - no dependencies on other ReedCMS modules
// Performance: Zero-allocation type system, <1μs creation times
// Dependencies: serde for serialisation, thiserror for error handling
// Data Flow: All modules communicate through ReedRequest → ReedResponse pattern

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Standard Result type for all ReedCMS operations.
pub type ReedResult<T> = Result<T, ReedError>;

/// Standard Error types across all modules.
///
/// These are the base error variants defined in REED-01-01.
/// Additional specific variants will be added in REED-01-02.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum ReedError {
    /// Resource not found (e.g., key not in CSV, template file missing).
    #[error("Resource not found: {resource}, context: {context:?}")]
    NotFound {
        resource: String,
        context: Option<String>,
    },

    /// Data parsing or validation error (e.g., invalid key format, malformed CSV).
    #[error("Validation error in field '{field}': value '{value}' does not meet constraint '{constraint}'")]
    ValidationError {
        field: String,
        value: String,
        constraint: String,
    },

    /// File system or I/O operation error.
    #[error("I/O error during operation '{operation}' on path '{path}': {reason}")]
    IoError {
        operation: String,
        path: String,
        reason: String,
    },

    /// CSV file operation error.
    #[error("CSV error in file '{file_type}' during operation '{operation}': {reason}")]
    CsvError {
        file_type: String,
        operation: String,
        reason: String,
    },

    /// Authentication or authorisation failure.
    #[error("Authentication failed for user {user:?} during action '{action}': {reason}")]
    AuthError {
        user: Option<String>,
        action: String,
        reason: String,
    },

    /// Configuration or setup error.
    #[error("Configuration error in component '{component}': {reason}")]
    ConfigError { component: String, reason: String },

    /// Template rendering error.
    #[error("Template rendering failed for '{template}': {reason}")]
    TemplateError { template: String, reason: String },

    /// Server or network operation error.
    #[error("Server error in component '{component}': {reason}")]
    ServerError { component: String, reason: String },

    /// Invalid CLI command or parameters.
    #[error("Invalid command '{command}': {reason}")]
    InvalidCommand { command: String, reason: String },

    /// Data parsing error (distinct from validation).
    #[error("Parse error for input '{input}': {reason}")]
    ParseError { input: String, reason: String },

    /// File not found error.
    #[error("File not found: {path}, reason: {reason}")]
    FileNotFound { path: String, reason: String },

    /// Directory not found error.
    #[error("Directory not found: {path}, reason: {reason}")]
    DirectoryNotFound { path: String, reason: String },

    /// Write operation error.
    #[error("Write error to path '{path}': {reason}")]
    WriteError { path: String, reason: String },

    /// Compression operation error.
    #[error("Compression failed: {reason}")]
    CompressionFailed { reason: String },

    /// Security violation (e.g., path traversal attempt).
    #[error("Security violation: {reason}")]
    SecurityViolation { reason: String },

    /// Invalid metadata error.
    #[error("Invalid metadata: {reason}")]
    InvalidMetadata { reason: String },

    /// Build operation error.
    #[error("Build error in component '{component}': {reason}")]
    BuildError { component: String, reason: String },
}

/// Standard Request structure for all module communications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedRequest {
    pub key: String,
    pub language: Option<String>,
    pub environment: Option<String>,
    pub context: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
}

/// Cache information for responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub hit: bool,
    pub ttl_remaining_s: Option<u64>,
    pub cache_key: String,
    pub cache_layer: String,
}

/// Performance metrics for responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetrics {
    pub processing_time_us: u64,
    pub memory_allocated: Option<u64>,
    pub csv_files_accessed: u8,
    pub cache_info: Option<CacheInfo>,
}

/// Standard Response structure with performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedResponse<T> {
    pub data: T,
    pub source: String,
    pub cached: bool,
    pub timestamp: u64,
    pub metrics: Option<ResponseMetrics>,
}

impl<T> ReedResponse<T> {
    /// Creates a new ReedResponse with minimal fields.
    ///
    /// ## Input
    /// - `data`: The response payload
    /// - `source`: Module name that generated this response
    ///
    /// ## Output
    /// - `ReedResponse<T>`: New response with current timestamp and cached=false
    ///
    /// ## Performance
    /// - **Execution time**: < 1μs (single timestamp call)
    /// - **Memory usage**: Single allocation for source string
    ///
    /// ## Example Usage
    /// ```rust
    /// let response = ReedResponse::new("value", "reedbase::get");
    /// ```
    pub fn new(data: T, source: &str) -> Self {
        Self {
            data,
            source: source.to_string(),
            cached: false,
            timestamp: current_timestamp(),
            metrics: None,
        }
    }

    /// Adds performance metrics to an existing response.
    ///
    /// ## Input
    /// - `self`: Response to enhance
    /// - `metrics`: Performance metrics to attach
    ///
    /// ## Output
    /// - `Self`: Modified response with metrics attached
    ///
    /// ## Performance
    /// - **Execution time**: < 0.1μs (simple field assignment)
    /// - **Memory usage**: No additional allocation
    ///
    /// ## Example Usage
    /// ```rust
    /// let response = ReedResponse::new("value", "reedbase::get")
    ///     .with_metrics(ResponseMetrics { ... });
    /// ```
    pub fn with_metrics(mut self, metrics: ResponseMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }
}

/// Standard Module trait - ALL modules must implement.
pub trait ReedModule {
    /// Returns the module name for identification.
    fn module_name() -> &'static str;

    /// Performs a health check on the module.
    fn health_check() -> ReedResult<ReedResponse<String>>;

    /// Returns the module version.
    fn version() -> &'static str {
        "1.0.0"
    }

    /// Returns the list of module dependencies.
    fn dependencies() -> Vec<&'static str> {
        Vec::new()
    }
}

// == HELPER FUNCTIONS ==

/// Returns current Unix timestamp in seconds.
///
/// ## Output
/// - `u64`: Seconds since UNIX epoch
///
/// ## Performance
/// - **Execution time**: < 1μs (system call)
/// - **Memory usage**: Zero allocations
///
/// ## Example Usage
/// ```rust
/// let now = current_timestamp();
/// ```
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_secs()
}

// == CONVENIENCE FUNCTIONS ==

/// Creates a NotFound error.
pub fn not_found(resource: impl Into<String>) -> ReedError {
    ReedError::NotFound {
        resource: resource.into(),
        context: None,
    }
}

/// Creates a ValidationError.
pub fn validation_error(
    field: impl Into<String>,
    value: impl Into<String>,
    constraint: impl Into<String>,
) -> ReedError {
    ReedError::ValidationError {
        field: field.into(),
        value: value.into(),
        constraint: constraint.into(),
    }
}

/// Creates a CsvError.
pub fn csv_error(
    file_type: impl Into<String>,
    operation: impl Into<String>,
    reason: impl Into<String>,
) -> ReedError {
    ReedError::CsvError {
        file_type: file_type.into(),
        operation: operation.into(),
        reason: reason.into(),
    }
}

/// Creates an IoError.
pub fn io_error(
    operation: impl Into<String>,
    path: impl Into<String>,
    reason: impl Into<String>,
) -> ReedError {
    ReedError::IoError {
        operation: operation.into(),
        path: path.into(),
        reason: reason.into(),
    }
}

/// Creates an AuthError.
pub fn auth_error(
    user: Option<String>,
    action: impl Into<String>,
    reason: impl Into<String>,
) -> ReedError {
    ReedError::AuthError {
        user,
        action: action.into(),
        reason: reason.into(),
    }
}

/// Creates a ConfigError.
pub fn config_error(component: impl Into<String>, reason: impl Into<String>) -> ReedError {
    ReedError::ConfigError {
        component: component.into(),
        reason: reason.into(),
    }
}

/// Creates a TemplateError.
pub fn template_error(template: impl Into<String>, reason: impl Into<String>) -> ReedError {
    ReedError::TemplateError {
        template: template.into(),
        reason: reason.into(),
    }
}

/// Creates a ServerError.
pub fn server_error(component: impl Into<String>, reason: impl Into<String>) -> ReedError {
    ReedError::ServerError {
        component: component.into(),
        reason: reason.into(),
    }
}

/// Creates an InvalidCommand error.
pub fn invalid_command(command: impl Into<String>, reason: impl Into<String>) -> ReedError {
    ReedError::InvalidCommand {
        command: command.into(),
        reason: reason.into(),
    }
}

/// Creates a ParseError.
pub fn parse_error(input: impl Into<String>, reason: impl Into<String>) -> ReedError {
    ReedError::ParseError {
        input: input.into(),
        reason: reason.into(),
    }
}

// == ERROR TRAIT IMPLEMENTATIONS ==

impl ReedError {
    /// Adds context to NotFound errors.
    ///
    /// ## Input
    /// - `self`: The error to enhance
    /// - `context`: Additional context information
    ///
    /// ## Output
    /// - `Self`: Error with context added (only affects NotFound variant)
    ///
    /// ## Example Usage
    /// ```rust
    /// let err = not_found("key").with_context("CSV lookup");
    /// ```
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        if let ReedError::NotFound {
            context: ref mut ctx,
            ..
        } = self
        {
            *ctx = Some(context.into());
        }
        self
    }
}

/// Automatic conversion from std::io::Error to ReedError.
impl From<std::io::Error> for ReedError {
    fn from(err: std::io::Error) -> Self {
        ReedError::IoError {
            operation: "io".to_string(),
            path: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

/// Automatic conversion from csv::Error to ReedError.
impl From<csv::Error> for ReedError {
    fn from(err: csv::Error) -> Self {
        ReedError::CsvError {
            file_type: "unknown".to_string(),
            operation: "csv".to_string(),
            reason: err.to_string(),
        }
    }
}
