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

    /// Adapter not found
    AdapterNotFound { adapter: String },

    /// Ambiguous command (multiple adapters provide it)
    AmbiguousCommand {
        command: String,
        adapters: Vec<String>,
    },

    /// Adapter execution error
    AdapterError { adapter: String, message: String },

    /// Version mismatch
    VersionMismatch {
        adapter: String,
        required: String,
        actual: String,
    },

    /// Required adapter missing
    RequiredAdapterMissing { adapter: String },

    /// Invalid version format
    InvalidVersion { version: String },
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
            CliError::AdapterNotFound { adapter } => {
                write!(f, "Adapter '{}' not found", adapter)
            }
            CliError::AmbiguousCommand { command, adapters } => {
                write!(
                    f,
                    "Ambiguous command '{}'. Available: {}",
                    command,
                    adapters.join(", ")
                )
            }
            CliError::AdapterError { adapter, message } => {
                write!(f, "Adapter '{}' error: {}", adapter, message)
            }
            CliError::VersionMismatch {
                adapter,
                required,
                actual,
            } => {
                write!(
                    f,
                    "Adapter '{}' version mismatch: required {}, found {}",
                    adapter, required, actual
                )
            }
            CliError::RequiredAdapterMissing { adapter } => {
                write!(f, "Required adapter '{}' is missing", adapter)
            }
            CliError::InvalidVersion { version } => {
                write!(f, "Invalid version format: '{}'", version)
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

// ============================================================================
// Adapter System Types (REED-18-03)
// ============================================================================

/// Adapter registry with all configured adapters.
#[derive(Debug, Clone)]
pub struct AdapterRegistry {
    /// Adapters mapped by name
    pub adapters: HashMap<String, Adapter>,

    /// CLI configuration from Reed.toml
    pub cli_config: AdapterCliConfig,

    /// Command index for fast lookup
    pub command_index: CommandIndex,
}

/// Adapter definition.
#[derive(Debug, Clone)]
pub struct Adapter {
    /// Adapter name
    pub name: String,

    /// Path to binary
    pub binary: std::path::PathBuf,

    /// Adapter description
    pub description: String,

    /// Version requirement (e.g., ">=0.1.0")
    pub version_requirement: Option<String>,

    /// Is this adapter required?
    pub required: bool,

    /// Command aliases (short → full)
    pub aliases: HashMap<String, String>,

    /// Available commands
    pub commands: Vec<String>,

    /// Has been validated?
    pub validated: bool,
}

/// CLI configuration for adapters.
#[derive(Debug, Clone)]
pub struct AdapterCliConfig {
    /// Ordered list of adapters
    pub adapters: Vec<String>,

    /// Allow namespace omission for unambiguous commands
    pub namespace_omission: bool,
}

/// Command index for fast lookup.
#[derive(Debug, Clone)]
pub struct CommandIndex {
    /// Maps command name to list of adapters that provide it
    pub commands: HashMap<String, Vec<String>>,
}

/// Parsed command with optional adapter namespace.
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    /// Adapter name (None if no namespace)
    pub adapter: Option<String>,

    /// Command name
    pub command: String,
}

/// Resolved command with confirmed adapter.
#[derive(Debug, Clone)]
pub struct ResolvedCommand {
    /// Adapter name
    pub adapter: String,

    /// Command name
    pub command: String,

    /// Command arguments
    pub args: Vec<String>,
}

/// Result from adapter execution.
#[derive(Debug, Clone)]
pub struct AdapterResult {
    /// Exit code
    pub exit_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

/// Adapter validation result.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Is adapter valid?
    pub valid: bool,

    /// Error message if invalid
    pub error: Option<String>,

    /// Adapter version if valid
    pub version: Option<String>,
}

/// Registry validation result.
#[derive(Debug, Clone)]
pub struct RegistryValidation {
    /// Are all required adapters valid?
    pub valid: bool,

    /// Validation results per adapter
    pub results: HashMap<String, ValidationResult>,
}

impl CommandIndex {
    /// Create new empty command index.
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Find adapter for command (returns None if ambiguous or not found).
    pub fn find(&self, command: &str) -> Option<&str> {
        match self.commands.get(command) {
            Some(adapters) if adapters.len() == 1 => Some(&adapters[0]),
            _ => None,
        }
    }

    /// Get all adapters that provide this command.
    pub fn find_all(&self, command: &str) -> Vec<&str> {
        self.commands
            .get(command)
            .map(|adapters| adapters.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Check if command is ambiguous (multiple adapters provide it).
    pub fn is_ambiguous(&self, command: &str) -> bool {
        self.commands
            .get(command)
            .map(|adapters| adapters.len() > 1)
            .unwrap_or(false)
    }

    /// Add command to index.
    pub fn add_command(&mut self, command: String, adapter: String) {
        self.commands
            .entry(command)
            .or_insert_with(Vec::new)
            .push(adapter);
    }
}
