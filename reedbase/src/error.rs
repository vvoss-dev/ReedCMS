// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Error types for ReedBase operations.
//!
//! Provides structured error handling with detailed context for debugging.

use std::fmt;

/// Standard Result type for all ReedBase operations.
pub type ReedResult<T> = Result<T, ReedError>;

/// Error types for ReedBase operations.
#[derive(Debug, Clone)]
pub enum ReedError {
    /// Unknown action code in dictionary.
    UnknownActionCode { code: u8 },

    /// Unknown user code in dictionary.
    UnknownUserCode { code: u32 },

    /// Unknown action name (reverse lookup failed).
    UnknownAction { name: String },

    /// Dictionary file corrupted (CSV parse error).
    DictionaryCorrupted {
        file: String,
        reason: String,
        line: usize,
    },

    /// Duplicate code detected (data integrity issue).
    DuplicateCode { code: String, file: String },

    /// I/O error during file operations.
    IoError { operation: String, reason: String },

    /// Permission denied for file operation.
    PermissionDenied { path: String },

    /// CSV parsing error.
    CsvError {
        file: String,
        operation: String,
        reason: String,
    },

    /// Table not found.
    TableNotFound { name: String },

    /// Table already exists.
    TableAlreadyExists { name: String },

    /// Version not found.
    VersionNotFound { timestamp: u64 },

    /// Invalid CSV format.
    InvalidCsv { reason: String, line: usize },

    /// Version log corrupted.
    LogCorrupted { reason: String },

    /// Delta corrupted or invalid.
    DeltaCorrupted { timestamp: u64, reason: String },

    /// Confirmation required but not provided.
    NotConfirmed { operation: String },

    /// Delta generation failed.
    DeltaGenerationFailed { reason: String },

    /// Delta application failed.
    DeltaApplicationFailed { reason: String },

    /// Compression failed.
    CompressionFailed { reason: String },

    /// Decompression failed.
    DecompressionFailed { reason: String },

    /// Parse error (invalid format).
    ParseError { reason: String },

    /// Corrupted log entry (CRC32 mismatch or invalid magic bytes).
    CorruptedLogEntry { line: usize, reason: String },

    /// Command execution failed.
    CommandFailed { command: String, error: String },

    /// No tables found for operation.
    NoTablesFound,

    /// Table restore failed.
    TableRestoreFailed { table: String, reason: String },

    /// Lock timeout waiting for exclusive access.
    LockTimeout { table: String, timeout_secs: u64 },

    /// Write queue is full.
    QueueFull { table: String, size: usize },

    /// Invalid queue file format.
    InvalidQueueFile { path: String },

    /// Serialisation error.
    SerializationError { reason: String },

    /// Deserialisation error.
    DeserializationError { reason: String },

    /// Schema not found.
    SchemaNotFound { table: String },

    /// Invalid schema format.
    InvalidSchema { reason: String },

    /// Schema validation error.
    ValidationError {
        column: String,
        reason: String,
        value: Option<String>,
    },

    /// Invalid B+-Tree order.
    InvalidOrder { order: u16, min: u16 },

    /// Corrupted index page.
    CorruptedIndex { page_id: u32, reason: String },

    /// WAL recovery failed.
    WalRecoveryFailed { reason: String },

    /// Invalid page format.
    InvalidPageFormat { page_id: u32, reason: String },

    /// Index operation not supported by backend.
    IndexOperationUnsupported {
        operation: String,
        backend: String,
        reason: String,
    },

    /// Index not found during query execution.
    IndexNotFound { name: String },

    /// Index already exists.
    IndexAlreadyExists { table: String, column: String },

    /// Query optimization failed.
    QueryOptimizationFailed { query: String, reason: String },

    /// Version log read failed.
    VersionLogRead {
        path: std::path::PathBuf,
        reason: String,
    },

    /// Version index corrupted.
    VersionIndexCorrupted {
        index_type: String, // "timestamp" | "frame"
        path: std::path::PathBuf,
    },
}

impl fmt::Display for ReedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownActionCode { code } => {
                write!(f, "Unknown action code: {}", code)
            }
            Self::UnknownUserCode { code } => {
                write!(f, "Unknown user code: {}", code)
            }
            Self::UnknownAction { name } => {
                write!(f, "Unknown action name: '{}'", name)
            }
            Self::DictionaryCorrupted { file, reason, line } => {
                write!(
                    f,
                    "Dictionary '{}' corrupted at line {}: {}",
                    file, line, reason
                )
            }
            Self::DuplicateCode { code, file } => {
                write!(f, "Duplicate code '{}' in dictionary '{}'", code, file)
            }
            Self::IoError { operation, reason } => {
                write!(f, "I/O error during '{}': {}", operation, reason)
            }
            Self::PermissionDenied { path } => {
                write!(f, "Permission denied: {}", path)
            }
            Self::CsvError {
                file,
                operation,
                reason,
            } => {
                write!(
                    f,
                    "CSV error in '{}' during '{}': {}",
                    file, operation, reason
                )
            }
            Self::TableNotFound { name } => {
                write!(f, "Table '{}' not found", name)
            }
            Self::TableAlreadyExists { name } => {
                write!(f, "Table '{}' already exists", name)
            }
            Self::VersionNotFound { timestamp } => {
                write!(f, "Version {} not found", timestamp)
            }
            Self::InvalidCsv { reason, line } => {
                write!(f, "Invalid CSV at line {}: {}", line, reason)
            }
            Self::LogCorrupted { reason } => {
                write!(f, "Version log corrupted: {}", reason)
            }
            Self::DeltaCorrupted { timestamp, reason } => {
                write!(f, "Delta {} corrupted: {}", timestamp, reason)
            }
            Self::NotConfirmed { operation } => {
                write!(f, "Operation '{}' requires confirmation", operation)
            }
            Self::DeltaGenerationFailed { reason } => {
                write!(f, "Delta generation failed: {}", reason)
            }
            Self::DeltaApplicationFailed { reason } => {
                write!(f, "Delta application failed: {}", reason)
            }
            Self::CompressionFailed { reason } => {
                write!(f, "Compression failed: {}", reason)
            }
            Self::DecompressionFailed { reason } => {
                write!(f, "Decompression failed: {}", reason)
            }
            Self::ParseError { reason } => {
                write!(f, "Parse error: {}", reason)
            }
            Self::CorruptedLogEntry { line, reason } => {
                write!(f, "Corrupted log entry at line {}: {}", line, reason)
            }
            Self::CommandFailed { command, error } => {
                write!(f, "Command '{}' failed: {}", command, error)
            }
            Self::NoTablesFound => {
                write!(f, "No tables found")
            }
            Self::TableRestoreFailed { table, reason } => {
                write!(f, "Table '{}' restore failed: {}", table, reason)
            }
            Self::LockTimeout {
                table,
                timeout_secs,
            } => {
                write!(
                    f,
                    "Lock timeout for table '{}' after {}s",
                    table, timeout_secs
                )
            }
            Self::QueueFull { table, size } => {
                write!(f, "Queue full for table '{}' ({} pending)", table, size)
            }
            Self::InvalidQueueFile { path } => {
                write!(f, "Invalid queue file: {}", path)
            }
            Self::SerializationError { reason } => {
                write!(f, "Serialisation error: {}", reason)
            }
            Self::DeserializationError { reason } => {
                write!(f, "Deserialisation error: {}", reason)
            }
            Self::SchemaNotFound { table } => {
                write!(f, "Schema not found for table '{}'", table)
            }
            Self::InvalidSchema { reason } => {
                write!(f, "Invalid schema: {}", reason)
            }
            Self::ValidationError {
                column,
                reason,
                value,
            } => {
                if let Some(val) = value {
                    write!(
                        f,
                        "Validation error in column '{}': {} (value: '{}')",
                        column, reason, val
                    )
                } else {
                    write!(f, "Validation error in column '{}': {}", column, reason)
                }
            }
            Self::InvalidOrder { order, min } => {
                write!(f, "Invalid B+-Tree order: {} (minimum: {})", order, min)
            }
            Self::CorruptedIndex { page_id, reason } => {
                write!(f, "Corrupted index page {}: {}", page_id, reason)
            }
            Self::WalRecoveryFailed { reason } => {
                write!(f, "WAL recovery failed: {}", reason)
            }
            Self::InvalidPageFormat { page_id, reason } => {
                write!(f, "Invalid page format at page {}: {}", page_id, reason)
            }
            Self::IndexOperationUnsupported {
                operation,
                backend,
                reason,
            } => {
                write!(
                    f,
                    "Operation '{}' not supported by '{}' backend: {}",
                    operation, backend, reason
                )
            }
            Self::IndexNotFound { name } => {
                write!(f, "Index '{}' not found", name)
            }
            Self::IndexAlreadyExists { table, column } => {
                write!(f, "Index already exists on {}.{}", table, column)
            }
            Self::QueryOptimizationFailed { query, reason } => {
                write!(f, "Query optimization failed for '{}': {}", query, reason)
            }
            Self::VersionLogRead { path, reason } => {
                write!(
                    f,
                    "Version log read failed for '{}': {}",
                    path.display(),
                    reason
                )
            }
            Self::VersionIndexCorrupted { index_type, path } => {
                write!(
                    f,
                    "Version index '{}' corrupted at '{}'",
                    index_type,
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for ReedError {}

// Convenience conversion from std::io::Error
impl From<std::io::Error> for ReedError {
    fn from(err: std::io::Error) -> Self {
        ReedError::IoError {
            operation: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}
