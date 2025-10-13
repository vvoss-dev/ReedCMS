// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Help system for ReedCLI.
//!
//! Auto-generates help documentation from Reed.toml registry.

use crate::types::{CliError, CliResult, Registry};

/// Show help documentation.
///
/// ## Input
/// - `registry`: Loaded registry
/// - `args`: Help arguments (empty, tool name, or tool + command name)
///
/// ## Output
/// - `CliResult<String>`: Formatted help text
///
/// ## Performance
/// - O(n) where n = number of tools/commands to list
/// - < 20ms typical
///
/// ## Error Conditions
/// - InvalidArgs: Too many arguments (> 2)
/// - ToolNotFound: Tool does not exist
/// - CommandNotFound: Command not found for tool
///
/// ## Example Usage
/// ```rust
/// // List all tools
/// let help = show_help(&registry, &[])?;
///
/// // List tool commands
/// let help = show_help(&registry, &["reedbase"])?;
///
/// // Show command details
/// let help = show_help(&registry, &["reedbase", "query"])?;
/// ```
pub fn show_help(registry: &Registry, args: &[String]) -> CliResult<String> {
    match args.len() {
        0 => show_tools(registry),
        1 => show_tool_commands(registry, &args[0]),
        2 => show_command_help(registry, &args[0], &args[1]),
        _ => Err(CliError::InvalidArgs {
            reason: "Too many arguments for help".to_string(),
        }),
    }
}

/// Show all available tools.
///
/// ## Input
/// - `registry`: Loaded registry
///
/// ## Output
/// - `CliResult<String>`: Formatted tool list
///
/// ## Performance
/// - O(n) where n = number of tools
/// - < 1ms for < 10 tools
///
/// ## Example Usage
/// ```rust
/// let help = show_tools(&registry)?;
/// println!("{}", help);
/// ```
fn show_tools(registry: &Registry) -> CliResult<String> {
    let mut output = String::from("Available tools:\n");

    let mut tools: Vec<_> = registry.tools.iter().collect();
    tools.sort_by_key(|(name, _)| *name);

    for (name, tool) in tools {
        let desc = tool.description.as_deref().unwrap_or("(no description)");
        output.push_str(&format!("  {:<12} {}\n", name, desc));
    }

    output.push_str("\nUse 'reed help <tool>' to see commands for a tool.\n");

    Ok(output)
}

/// Show all commands for a tool.
///
/// ## Input
/// - `registry`: Loaded registry
/// - `tool_name`: Tool name
///
/// ## Output
/// - `CliResult<String>`: Formatted command list
///
/// ## Performance
/// - O(n) where n = number of commands
/// - < 5ms for < 50 commands
///
/// ## Error Conditions
/// - ToolNotFound: Tool does not exist
///
/// ## Example Usage
/// ```rust
/// let help = show_tool_commands(&registry, "reedbase")?;
/// println!("{}", help);
/// ```
fn show_tool_commands(registry: &Registry, tool_name: &str) -> CliResult<String> {
    let tool = registry
        .tools
        .get(tool_name)
        .ok_or_else(|| CliError::ToolNotFound {
            tool: tool_name.to_string(),
        })?;

    let mut output = format!("Commands for {}:\n", tool_name);

    if let Some(desc) = &tool.description {
        output.push_str(&format!("{}\n\n", desc));
    }

    let mut commands: Vec<_> = tool.commands.iter().collect();
    commands.sort_by_key(|(name, _)| *name);

    for (cmd_name, cmd_spec) in commands {
        output.push_str(&format!("  {:<15} {}\n", cmd_name, cmd_spec.help));
    }

    output.push_str(&format!(
        "\nUse 'reed help {} <command>' for command details.\n",
        tool_name
    ));

    Ok(output)
}

/// Show details for a specific command.
///
/// ## Input
/// - `registry`: Loaded registry
/// - `tool_name`: Tool name
/// - `cmd_name`: Command name
///
/// ## Output
/// - `CliResult<String>`: Formatted command details
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100μs typical
///
/// ## Error Conditions
/// - ToolNotFound: Tool does not exist
/// - CommandNotFound: Command not found for tool
///
/// ## Example Usage
/// ```rust
/// let help = show_command_help(&registry, "reedbase", "query")?;
/// println!("{}", help);
/// ```
fn show_command_help(registry: &Registry, tool_name: &str, cmd_name: &str) -> CliResult<String> {
    let tool = registry
        .tools
        .get(tool_name)
        .ok_or_else(|| CliError::ToolNotFound {
            tool: tool_name.to_string(),
        })?;

    let cmd_spec = tool
        .commands
        .get(cmd_name)
        .ok_or_else(|| CliError::CommandNotFound {
            tool: tool_name.to_string(),
            command: cmd_name.to_string(),
        })?;

    let output = format!(
        "Command: {} {}\n{}\n\nHandler: {}\n",
        tool_name, cmd_name, cmd_spec.help, cmd_spec.handler
    );

    Ok(output)
}

#[cfg(test)]
#[path = "help_test.rs"]
mod tests;
