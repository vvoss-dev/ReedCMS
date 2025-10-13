// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Command parser for ReedCLI.
//!
//! Parses command-line arguments or interactive input into structured Command type.

use crate::types::{CliError, CliResult, Command};
use std::collections::HashMap;

#[cfg(test)]
#[path = "parser_test.rs"]
mod tests;

/// Parse command-line arguments into Command.
///
/// ## Input
/// - `args`: Iterator of command-line arguments (from `std::env::args()`)
///
/// ## Output
/// - `CliResult<Command>`: Parsed command or error
///
/// ## Performance
/// - O(n) where n = number of arguments
/// - < 1ms typical for reasonable command lengths
///
/// ## Error Conditions
/// - EmptyCommand: No arguments provided
/// - UnmatchedQuote: Quote not closed
/// - InvalidFlag: Flag without value
///
/// ## Example Usage
/// ```rust
/// use reedcli::parser::parse_args;
///
/// let args = vec!["reed", "query", "SELECT * FROM users", "--format", "json"];
/// let command = parse_args(args.into_iter().map(|s| s.to_string())).unwrap();
/// assert_eq!(command.command, "query");
/// assert_eq!(command.flags.get("format"), Some(&"json".to_string()));
/// ```
pub fn parse_args<I>(args: I) -> CliResult<Command>
where
    I: Iterator<Item = String>,
{
    let args_vec: Vec<String> = args.collect();

    if args_vec.is_empty() || args_vec.len() == 1 {
        return Err(CliError::EmptyCommand);
    }

    // Skip program name (args[0])
    let command_parts: Vec<String> = args_vec.into_iter().skip(1).collect();

    parse_command_parts(&command_parts)
}

/// Parse interactive shell input into Command.
///
/// ## Input
/// - `input`: Raw line from shell
///
/// ## Output
/// - `CliResult<Command>`: Parsed command or error
///
/// ## Performance
/// - O(n) where n = input length
/// - < 1ms typical
///
/// ## Error Conditions
/// - EmptyCommand: Empty or whitespace-only input
/// - UnmatchedQuote: Quote not closed
/// - InvalidFlag: Flag without value
///
/// ## Example Usage
/// ```rust
/// use reedcli::parser::parse_shell_input;
///
/// let command = parse_shell_input("query \"SELECT * FROM users\"").unwrap();
/// assert_eq!(command.command, "query");
/// assert_eq!(command.args[0], "SELECT * FROM users");
/// ```
pub fn parse_shell_input(input: &str) -> CliResult<Command> {
    let parts = tokenise_input(input)?;
    parse_command_parts(&parts)
}

/// Parse command parts into Command struct.
///
/// ## Input
/// - `parts`: Tokenised command parts
///
/// ## Output
/// - `CliResult<Command>`: Parsed command
///
/// ## Performance
/// - O(n) where n = number of parts
/// - < 500μs typical
///
/// ## Error Conditions
/// - EmptyCommand: No parts provided
/// - InvalidFlag: Flag without value
///
/// ## Example Usage
/// ```rust
/// use reedcli::parser::parse_command_parts;
///
/// let parts = vec!["query".to_string(), "SELECT * FROM users".to_string()];
/// let command = parse_command_parts(&parts).unwrap();
/// assert_eq!(command.command, "query");
/// ```
pub fn parse_command_parts(parts: &[String]) -> CliResult<Command> {
    if parts.is_empty() {
        return Err(CliError::EmptyCommand);
    }

    let command_name = &parts[0];
    let mut args = Vec::new();
    let mut flags = HashMap::new();

    let mut i = 1;
    while i < parts.len() {
        let part = &parts[i];

        if part.starts_with("--") {
            // Long flag: --format json
            let flag_name = part.trim_start_matches("--");

            if i + 1 >= parts.len() {
                return Err(CliError::InvalidFlag {
                    flag: flag_name.to_string(),
                    reason: "Missing value".to_string(),
                });
            }

            let flag_value = &parts[i + 1];
            flags.insert(flag_name.to_string(), flag_value.clone());
            i += 2;
        } else if part.starts_with('-') && part.len() == 2 {
            // Short flag: -f json
            let flag_name = part.trim_start_matches('-');

            if i + 1 >= parts.len() {
                return Err(CliError::InvalidFlag {
                    flag: flag_name.to_string(),
                    reason: "Missing value".to_string(),
                });
            }

            let flag_value = &parts[i + 1];
            flags.insert(flag_name.to_string(), flag_value.clone());
            i += 2;
        } else {
            // Positional argument
            args.push(part.clone());
            i += 1;
        }
    }

    // Infer tool from command name
    let tool = infer_tool(command_name);

    Ok(Command {
        tool,
        command: command_name.clone(),
        args,
        flags,
    })
}

/// Tokenise input string, respecting quoted strings.
///
/// ## Input
/// - `input`: Raw input string
///
/// ## Output
/// - `CliResult<Vec<String>>`: Tokenised parts
///
/// ## Performance
/// - O(n) where n = input length
/// - < 200μs for 100-char input
///
/// ## Error Conditions
/// - UnmatchedQuote: Quote not closed
///
/// ## Example Usage
/// ```rust
/// use reedcli::parser::tokenise_input;
///
/// let tokens = tokenise_input(r#"query "SELECT * FROM users" --format json"#).unwrap();
/// assert_eq!(tokens[0], "query");
/// assert_eq!(tokens[1], "SELECT * FROM users");
/// assert_eq!(tokens[2], "--format");
/// assert_eq!(tokens[3], "json");
/// ```
pub fn tokenise_input(input: &str) -> CliResult<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            current_token.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                escape_next = true;
            }
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' if !in_quotes => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    if in_quotes {
        return Err(CliError::UnmatchedQuote);
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    Ok(tokens)
}

/// Infer tool name from command name.
///
/// ## Input
/// - `command`: Command name (e.g., "query", "server:start")
///
/// ## Output
/// - Tool name (e.g., "reedbase", "reedcms")
///
/// ## Performance
/// - O(1) operation
/// - < 10μs typical
///
/// ## Logic
/// - Commands with `:` → Tool is prefix (e.g., "server:start" → "server")
/// - Known ReedBase commands → "reedbase"
/// - Default → "reedcms"
///
/// ## Example Usage
/// ```rust
/// use reedcli::parser::infer_tool;
///
/// assert_eq!(infer_tool("query"), "reedbase");
/// assert_eq!(infer_tool("server:start"), "server");
/// assert_eq!(infer_tool("unknown"), "reedcms");
/// ```
pub fn infer_tool(command: &str) -> String {
    // If command contains ':', tool is the prefix
    if let Some(pos) = command.find(':') {
        return command[..pos].to_string();
    }

    // Known ReedBase commands
    const REEDBASE_COMMANDS: &[&str] = &["query", "tables", "versions", "rollback"];

    if REEDBASE_COMMANDS.contains(&command) {
        return "reedbase".to_string();
    }

    // Default to reedcms
    "reedcms".to_string()
}
