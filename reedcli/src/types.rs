// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Type definitions for ReedCLI.
//!
//! Contains Command, CliResult, and CliError types.

use std::collections::HashMap;
use std::fmt;

/// Parsed command structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    /// Tool to route command to (e.g., "reedbase", "reedcms")
    pub tool: String,

    /// Command name (e.g., "query", "server:start")
    pub command: String,

    /// Positional arguments
    pub args: Vec<String>,

    /// Flags (e.g., --format json → {"format": "json"})
    pub flags: HashMap<String, String>,
}

/// CLI operation result type.
pub type CliResult<T> = Result<T, CliError>;

/// CLI error types.
#[derive(Debug, Clone, PartialEq)]
pub enum CliError {
    /// No command provided
    EmptyCommand,

    /// Unmatched quote in input
    UnmatchedQuote,

    /// Invalid flag usage
    InvalidFlag { flag: String, reason: String },

    /// Unknown command
    UnknownCommand { command: String },

    /// Tool not found
    ToolNotFound { tool: String },
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::EmptyCommand => {
                write!(f, "No command provided")
            }
            CliError::UnmatchedQuote => {
                write!(f, "Unmatched quote in input")
            }
            CliError::InvalidFlag { flag, reason } => {
                write!(f, "Invalid flag '--{}': {}", flag, reason)
            }
            CliError::UnknownCommand { command } => {
                write!(f, "Unknown command: {}", command)
            }
            CliError::ToolNotFound { tool } => {
                write!(f, "Tool not found: {}", tool)
            }
        }
    }
}

impl std::error::Error for CliError {}
