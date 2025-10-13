// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tool integration for ReedCLI.
//!
//! Dynamically invokes tool handlers based on registry metadata.

use crate::types::{
    CliError, CliResult, Command, CommandOutput, CommandSpec, OutputFormat, Registry,
};
use std::collections::HashMap;

/// Execute command by routing to appropriate tool handler.
///
/// ## Input
/// - `command`: Parsed command
/// - `registry`: Loaded registry
///
/// ## Output
/// - `CliResult<CommandOutput>`: Command output with data and exit code
///
/// ## Performance
/// - Depends on tool handler execution time
/// - Registry lookup: < 10μs (O(1) HashMap)
///
/// ## Error Conditions
/// - ToolNotFound: Tool does not exist in registry
/// - CommandNotFound: Command not found for tool
/// - HandlerNotFound: Handler function not implemented
/// - ToolError: Tool-specific execution error
///
/// ## Example Usage
/// ```rust
/// let command = parser::parse_args(env::args())?;
/// let output = execute_command(&command, &registry)?;
/// println!("{:?}", output);
/// ```
pub fn execute_command(command: &Command, registry: &Registry) -> CliResult<CommandOutput> {
    // Lookup command spec in registry
    let cmd_spec = registry
        .tools
        .get(&command.tool)
        .and_then(|tool| tool.commands.get(&command.command))
        .ok_or_else(|| CliError::CommandNotFound {
            tool: command.tool.clone(),
            command: command.command.clone(),
        })?;

    // Route to appropriate tool
    let output = match command.tool.as_str() {
        "reedbase" => execute_reedbase_command(command, cmd_spec)?,
        "reedcms" => execute_reedcms_command(command, cmd_spec)?,
        _ => {
            return Err(CliError::ToolNotFound {
                tool: command.tool.clone(),
            })
        }
    };

    Ok(output)
}

/// Execute ReedBase command.
///
/// ## Input
/// - `command`: Parsed command
/// - `cmd_spec`: Command specification from registry
///
/// ## Output
/// - `CliResult<CommandOutput>`: Command output with data
///
/// ## Performance
/// - Depends on specific ReedBase operation
/// - Query: < 100ms typical
/// - List tables: < 10ms typical
///
/// ## Error Conditions
/// - HandlerNotFound: Handler not implemented
/// - ToolError: ReedBase operation failed
///
/// ## Example Usage
/// ```rust
/// let output = execute_reedbase_command(&command, &cmd_spec)?;
/// ```
fn execute_reedbase_command(command: &Command, cmd_spec: &CommandSpec) -> CliResult<CommandOutput> {
    let handler = cmd_spec.handler.as_str();

    match handler {
        "execute_query" => {
            // TODO: Integrate with ReedBase REED-19
            // For now, return stub data
            Ok(CommandOutput {
                data: serde_json::json!([
                    {"id": 1, "name": "Alice"},
                    {"id": 2, "name": "Bob"}
                ]),
                format: determine_output_format(&command.flags),
                exit_code: 0,
            })
        }
        "list_tables" => {
            // TODO: Integrate with ReedBase REED-19
            Ok(CommandOutput {
                data: serde_json::json!(["text", "routes", "meta", "users"]),
                format: determine_output_format(&command.flags),
                exit_code: 0,
            })
        }
        "list_versions" => {
            // TODO: Integrate with ReedBase REED-19
            Err(CliError::HandlerNotFound {
                handler: handler.to_string(),
            })
        }
        "rollback_table" => {
            // TODO: Integrate with ReedBase REED-19
            Err(CliError::HandlerNotFound {
                handler: handler.to_string(),
            })
        }
        _ => Err(CliError::HandlerNotFound {
            handler: handler.to_string(),
        }),
    }
}

/// Execute ReedCMS command.
///
/// ## Input
/// - `command`: Parsed command
/// - `cmd_spec`: Command specification from registry
///
/// ## Output
/// - `CliResult<CommandOutput>`: Command output with data
///
/// ## Performance
/// - Depends on specific ReedCMS operation
///
/// ## Error Conditions
/// - HandlerNotFound: Handler not implemented
/// - ToolError: ReedCMS operation failed
///
/// ## Example Usage
/// ```rust
/// let output = execute_reedcms_command(&command, &cmd_spec)?;
/// ```
fn execute_reedcms_command(_command: &Command, cmd_spec: &CommandSpec) -> CliResult<CommandOutput> {
    let handler = cmd_spec.handler.as_str();

    match handler {
        "server_start" => {
            // TODO: Integrate with existing ReedCMS server code
            Err(CliError::HandlerNotFound {
                handler: handler.to_string(),
            })
        }
        "server_stop" => {
            // TODO: Integrate with existing ReedCMS server code
            Err(CliError::HandlerNotFound {
                handler: handler.to_string(),
            })
        }
        "build_watch" => {
            // TODO: Integrate with existing ReedCMS build code
            Err(CliError::HandlerNotFound {
                handler: handler.to_string(),
            })
        }
        _ => Err(CliError::HandlerNotFound {
            handler: handler.to_string(),
        }),
    }
}

/// Determine output format from command flags.
///
/// ## Input
/// - `flags`: Command flags
///
/// ## Output
/// - `OutputFormat`: Determined format (default: Table)
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 1μs typical
///
/// ## Logic
/// - Check --format flag
/// - Valid values: "table", "json", "csv", "plain"
/// - Default: "table" if not specified or invalid
///
/// ## Example Usage
/// ```rust
/// let mut flags = HashMap::new();
/// flags.insert("format".to_string(), "json".to_string());
/// assert_eq!(determine_output_format(&flags), OutputFormat::Json);
/// ```
pub fn determine_output_format(flags: &HashMap<String, String>) -> OutputFormat {
    flags
        .get("format")
        .and_then(|f| match f.as_str() {
            "json" => Some(OutputFormat::Json),
            "csv" => Some(OutputFormat::Csv),
            "plain" => Some(OutputFormat::Plain),
            "table" => Some(OutputFormat::Table),
            _ => None,
        })
        .unwrap_or(OutputFormat::Table)
}

/// Get exit code from command output.
///
/// ## Input
/// - `output`: Command output
///
/// ## Output
/// - `i32`: Exit code (0 = success, 1 = user error, 2 = system error)
///
/// ## Performance
/// - O(1) operation
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// let code = get_exit_code(&output);
/// std::process::exit(code);
/// ```
pub fn get_exit_code(output: &CommandOutput) -> i32 {
    output.exit_code
}

/// Convert CLI error to exit code.
///
/// ## Input
/// - `error`: CLI error
///
/// ## Output
/// - `i32`: Exit code (1 = user error, 2 = system error)
///
/// ## Performance
/// - O(1) operation
/// - < 1μs typical
///
/// ## Logic
/// - User errors (bad input, not found): exit code 1
/// - System errors (I/O, database): exit code 2
///
/// ## Example Usage
/// ```rust
/// match execute_command(&cmd, &registry) {
///     Ok(output) => println!("{:?}", output),
///     Err(e) => {
///         eprintln!("Error: {}", e);
///         std::process::exit(error_to_exit_code(&e));
///     }
/// }
/// ```
pub fn error_to_exit_code(error: &CliError) -> i32 {
    match error {
        // User errors (exit code 1)
        CliError::EmptyCommand
        | CliError::UnmatchedQuote
        | CliError::InvalidFlag { .. }
        | CliError::ToolNotFound { .. }
        | CliError::CommandNotFound { .. }
        | CliError::HandlerNotFound { .. }
        | CliError::InvalidArgs { .. }
        | CliError::UnknownCommand { .. }
        | CliError::AdapterNotFound { .. }
        | CliError::AmbiguousCommand { .. } => 1,

        // System errors (exit code 2)
        CliError::RegistryNotFound { .. }
        | CliError::InvalidRegistry { .. }
        | CliError::CircularDependency { .. }
        | CliError::MissingDependency { .. }
        | CliError::FormatError { .. }
        | CliError::ShellError { .. }
        | CliError::ToolError { .. }
        | CliError::AdapterError { .. }
        | CliError::VersionMismatch { .. }
        | CliError::RequiredAdapterMissing { .. }
        | CliError::InvalidVersion { .. } => 2,
    }
}

#[cfg(test)]
#[path = "integration_test.rs"]
mod tests;
