// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! CLI command parser for colon notation.
//!
//! Parses `reed command:action [args] [--flags]` syntax into structured Command.

use crate::reedcms::reedstream::{ReedError, ReedResult};
use std::collections::HashMap;

/// Parsed command structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    /// Command namespace (e.g., "set", "get", "user").
    pub namespace: String,
    /// Command action (e.g., "text", "create", "list").
    pub action: String,
    /// Positional arguments after command.
    pub args: Vec<String>,
    /// Named flags (--flag value or --flag for boolean).
    pub flags: HashMap<String, String>,
}

/// Known boolean flags that don't require values.
const BOOLEAN_FLAGS: &[&str] = &[
    "help",
    "h",
    "verbose",
    "v",
    "dry-run",
    "confirm",
    "recursive",
    "minify",
    "follow",
    "fuzzy",
    "show-permissions",
    "tree",
    "json",
    "force",
    "quiet",
    "watch",
];

/// Parses CLI arguments into Command structure.
///
/// ## Input
/// - args: Raw CLI arguments (excluding binary name)
///
/// ## Output
/// - Command structure with namespace, action, args, flags
///
/// ## Performance
/// - O(n) where n = number of arguments
/// - Target: < 1ms for typical commands
///
/// ## Error Conditions
/// - Empty arguments
/// - Invalid command format (no colon)
/// - Invalid flag format
///
/// ## Example Usage
/// ```rust
/// let args = vec!["set:text", "key@de", "value", "--desc", "Description"];
/// let cmd = parse_command(args)?;
/// assert_eq!(cmd.namespace, "set");
/// assert_eq!(cmd.action, "text");
/// ```
pub fn parse_command(args: Vec<String>) -> ReedResult<Command> {
    if args.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: String::new(),
            reason: "No command provided".to_string(),
        });
    }

    // First arg is command:action
    let command_str = &args[0];
    let (namespace, action) = parse_command_format(command_str)?;

    // Parse remaining args into positional and flags
    let (positional_args, flags) = parse_args_and_flags(&args[1..])?;

    Ok(Command {
        namespace,
        action,
        args: positional_args,
        flags,
    })
}

/// Parses command format (namespace:action).
///
/// ## Input
/// - cmd: Command string in format "namespace:action"
///
/// ## Output
/// - Tuple of (namespace, action)
///
/// ## Performance
/// - O(1) string operations
/// - < 0.1ms typical
///
/// ## Error Conditions
/// - No colon found
/// - Empty namespace or action
/// - Invalid characters
fn parse_command_format(cmd: &str) -> ReedResult<(String, String)> {
    if !cmd.contains(':') {
        return Err(ReedError::InvalidCommand {
            command: cmd.to_string(),
            reason: "Command must be in format 'namespace:action' (e.g., 'set:text')".to_string(),
        });
    }

    let parts: Vec<&str> = cmd.split(':').collect();
    if parts.len() != 2 {
        return Err(ReedError::InvalidCommand {
            command: cmd.to_string(),
            reason: format!(
                "Command must contain exactly one colon, found {}",
                parts.len() - 1
            ),
        });
    }

    let namespace = parts[0].trim();
    let action = parts[1].trim();

    if namespace.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: cmd.to_string(),
            reason: "Namespace cannot be empty".to_string(),
        });
    }

    if action.is_empty() {
        return Err(ReedError::InvalidCommand {
            command: cmd.to_string(),
            reason: "Action cannot be empty".to_string(),
        });
    }

    // Validate characters (alphanumeric + underscore + hyphen)
    validate_identifier(namespace, "namespace")?;
    validate_identifier(action, "action")?;

    Ok((namespace.to_string(), action.to_string()))
}

/// Validates identifier contains only allowed characters.
fn validate_identifier(id: &str, context: &str) -> ReedResult<()> {
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ReedError::ValidationError {
            field: context.to_string(),
            value: id.to_string(),
            constraint: "alphanumeric characters, underscore, or hyphen only".to_string(),
        });
    }
    Ok(())
}

/// Parses arguments and flags from remaining CLI args.
///
/// ## Parsing Rules
/// - Arguments starting with '--' are flags
/// - Boolean flags don't require values
/// - Other flags consume next argument as value
/// - Quoted values preserve spaces
/// - Everything else is positional argument
fn parse_args_and_flags(args: &[String]) -> ReedResult<(Vec<String>, HashMap<String, String>)> {
    let mut positional = Vec::new();
    let mut flags = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if let Some(flag_name) = arg.strip_prefix("--") {
            // Parse long flag (--flag)
            if flag_name.is_empty() {
                return Err(ReedError::ParseError {
                    input: arg.clone(),
                    reason: "Flag name cannot be empty after '--'".to_string(),
                });
            }

            // Check if this is a known boolean flag
            if BOOLEAN_FLAGS.contains(&flag_name) {
                flags.insert(flag_name.to_string(), "true".to_string());
                i += 1;
            } else {
                // Value flag - consume next argument
                if i + 1 >= args.len() {
                    return Err(ReedError::InvalidCommand {
                        command: arg.clone(),
                        reason: format!("Flag '--{}' requires a value", flag_name),
                    });
                }

                i += 1;
                let value = &args[i];

                // Validate value doesn't start with '--' or '-' (likely missing value)
                if value.starts_with('-') {
                    return Err(ReedError::InvalidCommand {
                        command: arg.clone(),
                        reason: format!(
                            "Flag '--{}' requires a value, got another flag instead",
                            flag_name
                        ),
                    });
                }

                flags.insert(flag_name.to_string(), value.clone());
                i += 1;
            }
        } else if arg.starts_with('-') && arg.len() == 2 {
            // Parse short flag (-h, -v)
            let flag_name = &arg[1..]; // Remove "-" prefix

            // Short flags are always treated as boolean
            flags.insert(flag_name.to_string(), "true".to_string());
            i += 1;
        } else {
            // Positional argument
            positional.push(arg.clone());
            i += 1;
        }
    }

    Ok((positional, flags))
}

/// Checks if command has a specific flag.
impl Command {
    /// Returns true if flag exists (regardless of value).
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains_key(flag)
    }

    /// Gets flag value, returns None if not present.
    pub fn get_flag(&self, flag: &str) -> Option<&String> {
        self.flags.get(flag)
    }

    /// Gets flag value as boolean.
    pub fn get_flag_bool(&self, flag: &str) -> bool {
        self.flags.get(flag).map(|v| v == "true").unwrap_or(false)
    }
}
