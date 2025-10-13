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
#[derive(Debug)]
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

    /// Registry file not found
    RegistryNotFound {
        path: String,
        source: std::io::Error,
    },

    /// Invalid registry format or content
    InvalidRegistry { reason: String },

    /// Circular dependency detected
    CircularDependency { tool: String },

    /// Missing dependency
    MissingDependency { tool: String, dependency: String },

    /// Command not found for tool
    CommandNotFound { tool: String, command: String },
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
            CliError::RegistryNotFound { path, source } => {
                write!(f, "Registry not found at '{}': {}", path, source)
            }
            CliError::InvalidRegistry { reason } => {
                write!(f, "Invalid registry: {}", reason)
            }
            CliError::CircularDependency { tool } => {
                write!(f, "Circular dependency detected for tool '{}'", tool)
            }
            CliError::MissingDependency { tool, dependency } => {
                write!(
                    f,
                    "Tool '{}' depends on missing tool '{}'",
                    tool, dependency
                )
            }
            CliError::CommandNotFound { tool, command } => {
                write!(f, "Command '{}' not found for tool '{}'", command, tool)
            }
        }
    }
}

impl std::error::Error for CliError {}

/// Registry loaded from Reed.toml.
#[derive(Debug, Clone)]
pub struct Registry {
    /// Registry format version
    pub version: String,

    /// CLI configuration
    pub cli: CliConfig,

    /// Tools mapped by name
    pub tools: HashMap<String, Tool>,
}

/// CLI configuration.
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// CLI name
    pub name: String,

    /// Binary name (optional, defaults to CLI name)
    pub binary: Option<String>,

    /// Shell prompt (default: "reed> ")
    pub shell_prompt: String,

    /// History file path (default: ".reed_history")
    pub history_file: String,
}

/// Tool specification.
#[derive(Debug, Clone)]
pub struct Tool {
    /// Tool name
    pub name: String,

    /// Tool description (optional)
    pub description: Option<String>,

    /// Binary path (optional)
    pub binary: Option<String>,

    /// Dependencies on other tools
    pub dependencies: Vec<String>,

    /// Commands mapped by name
    pub commands: HashMap<String, CommandSpec>,
}

/// Command specification from registry.
#[derive(Debug, Clone)]
pub struct CommandSpec {
    /// Handler function name
    pub handler: String,

    /// Help text for command
    pub help: String,
}
