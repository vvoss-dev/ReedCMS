// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Adapter command parser for ReedCLI.
//!
//! Parses adapter-namespaced commands and resolves them to adapters.

use crate::types::{Adapter, AdapterRegistry, CliError, CliResult, ParsedCommand, ResolvedCommand};

#[cfg(test)]
#[path = "parser_test.rs"]
mod tests;

/// Parse command with optional adapter namespace.
///
/// ## Input
/// - `input`: Command string (e.g., "reedbase:query" or "query")
///
/// ## Output
/// - `ParsedCommand`: Parsed command with optional adapter
///
/// ## Performance
/// - O(1) - string splitting
/// - < 10μs
///
/// ## Error Conditions
/// - None (returns None for adapter if no namespace)
///
/// ## Example Usage
/// ```ignore
/// let cmd = parse_adapter_command("reedbase:query")?;
/// assert_eq!(cmd.adapter, Some("reedbase".to_string()));
/// assert_eq!(cmd.command, "query");
///
/// let cmd = parse_adapter_command("query")?;
/// assert_eq!(cmd.adapter, None);
/// assert_eq!(cmd.command, "query");
/// ```
pub fn parse_adapter_command(input: &str) -> ParsedCommand {
    if let Some(colon_pos) = input.find(':') {
        let adapter = input[..colon_pos].to_string();
        let command = input[colon_pos + 1..].to_string();
        ParsedCommand {
            adapter: Some(adapter),
            command,
        }
    } else {
        ParsedCommand {
            adapter: None,
            command: input.to_string(),
        }
    }
}

/// Resolve adapter for command using registry.
///
/// ## Input
/// - `parsed`: ParsedCommand from parser
/// - `registry`: AdapterRegistry for lookup
///
/// ## Output
/// - `CliResult<String>`: Resolved adapter name
///
/// ## Performance
/// - O(1) - hash lookup
/// - < 10μs
///
/// ## Error Conditions
/// - AdapterNotFound: Specified adapter does not exist
/// - CommandNotFound: Command not found in any adapter
/// - AmbiguousCommand: Multiple adapters provide command
///
/// ## Example Usage
/// ```ignore
/// let parsed = parse_adapter_command("query");
/// let adapter_name = resolve_adapter(&parsed, &registry)?;
/// println!("Using adapter: {}", adapter_name);
/// ```
pub fn resolve_adapter(parsed: &ParsedCommand, registry: &AdapterRegistry) -> CliResult<String> {
    // If adapter explicitly specified, validate it exists
    if let Some(ref adapter_name) = parsed.adapter {
        if !registry.adapters.contains_key(adapter_name) {
            return Err(CliError::AdapterNotFound {
                adapter: adapter_name.clone(),
            });
        }
        return Ok(adapter_name.clone());
    }

    // No adapter specified - try to infer from command
    if !registry.cli_config.namespace_omission {
        return Err(CliError::InvalidRegistry {
            reason: "Namespace omission disabled - must specify adapter".to_string(),
        });
    }

    // Look up in command index
    match registry.command_index.find(&parsed.command) {
        Some(adapter_name) => Ok(adapter_name.to_string()),
        None => {
            // Check if ambiguous or not found
            let all_adapters = registry.command_index.find_all(&parsed.command);
            if all_adapters.is_empty() {
                Err(CliError::CommandNotFound {
                    tool: "any adapter".to_string(),
                    command: parsed.command.clone(),
                })
            } else {
                Err(CliError::AmbiguousCommand {
                    command: parsed.command.clone(),
                    adapters: all_adapters.iter().map(|s| s.to_string()).collect(),
                })
            }
        }
    }
}

/// Expand command aliases.
///
/// ## Input
/// - `command`: Command name (possibly aliased)
/// - `adapter`: Adapter with alias definitions
///
/// ## Output
/// - `String`: Expanded command name
///
/// ## Performance
/// - O(1) - hash lookup
/// - < 1μs
///
/// ## Error Conditions
/// - None (returns original if no alias)
///
/// ## Example Usage
/// ```ignore
/// let expanded = expand_alias("q", &adapter);
/// assert_eq!(expanded, "query");
/// ```
pub fn expand_alias(command: &str, adapter: &Adapter) -> String {
    adapter
        .aliases
        .get(command)
        .cloned()
        .unwrap_or_else(|| command.to_string())
}

/// Build resolved command with args.
///
/// ## Input
/// - `adapter`: Adapter name
/// - `command`: Command name (after alias expansion)
/// - `args`: Command arguments
///
/// ## Output
/// - `ResolvedCommand`: Complete resolved command
///
/// ## Performance
/// - O(1) - struct construction
/// - < 1μs
///
/// ## Error Conditions
/// - None
pub fn build_resolved_command(
    adapter: String,
    command: String,
    args: Vec<String>,
) -> ResolvedCommand {
    ResolvedCommand {
        adapter,
        command,
        args,
    }
}
