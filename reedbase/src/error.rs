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
