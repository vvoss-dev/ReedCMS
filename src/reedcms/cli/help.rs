// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! CLI help system.
//!
//! Provides help text generation for CLI commands.

use crate::reedcms::reedstream::{current_timestamp, ReedResponse, ReedResult};

/// Command information for help generation.
#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub namespace: String,
    pub action: String,
    pub description: String,
    pub usage: String,
    pub examples: Vec<String>,
    pub required_args: usize,
    pub required_flags: Vec<String>,
    pub optional_flags: Vec<String>,
}

/// Prints general help overview.
///
/// ## Output
/// - Overview of all available commands
/// - Usage examples
/// - Getting started guide
///
/// ## Performance
/// - Generation time: < 5ms
pub fn print_general_help() -> ReedResult<ReedResponse<String>> {
    let help_text = r#"
ReedCMS Command-Line Interface

Usage: reed <command:action> [args] [flags]

Available Commands:

  data:
    data:get <key>                     Retrieve a value from ReedBase
    data:set <key> <value>             Store a value in ReedBase
    data:list [pattern]                List keys matching pattern
    data:delete <key>                  Delete a key from ReedBase

  layout:
    layout:create <name> <variant>     Create a new layout
    layout:list                        List all layouts
    layout:delete <name>               Delete a layout

  user:
    user:create <username>             Create a new user
    user:list                          List all users
    user:show <username>               Show user details
    user:update <username>             Update user information
    user:delete <username>             Delete a user
    user:passwd <username>             Change user password

  role:
    role:create <rolename>             Create a new role
    role:list                          List all roles
    role:show <rolename>               Show role details
    role:update <rolename>             Update role permissions
    role:delete <rolename>             Delete a role

  taxonomy:
    taxonomy:create <term>             Create a new taxonomy term
    taxonomy:list                      List all taxonomy terms
    taxonomy:assign <entity> <terms>   Assign terms to entity
    taxonomy:search <query>            Search taxonomy

  server:
    server:start                       Start the ReedCMS server
    server:stop                        Stop the server
    server:status                      Show server status

  build:
    build:kernel                       Build reed binary
    build:complete                     Complete build with assets
    build:watch                        Watch mode for development

  monitor:
    monitor:status                     Show system status
    monitor:logs                       Show system logs

Global Flags:
  --help, -h                           Show help information
  --version, -v                        Show version information
  --verbose                            Enable verbose output
  --json                               Output in JSON format
  --dry-run                            Show what would be done without doing it

Examples:
  reed data:get knowledge.title@en
  reed data:set knowledge.title@en "Knowledge Base"
  reed user:create admin --email admin@example.com
  reed server:start --port 8333

For command-specific help:
  reed <command:action> --help
"#;

    Ok(ReedResponse {
        data: help_text.trim().to_string(),
        source: "cli_help".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Prints command-specific help.
///
/// ## Input
/// - namespace: Command namespace
/// - action: Command action
///
/// ## Output
/// - Detailed help for specific command
pub fn print_command_help(namespace: &str, action: &str) -> ReedResult<ReedResponse<String>> {
    let help_text = match (namespace, action) {
        ("data", "get") => format!(
            r#"{}:{}

Retrieve a value from ReedBase

Usage:
  reed data:get <key>

Arguments:
  <key>    The key to retrieve (e.g., knowledge.title@en)

Flags:
  --help   Show this help message

Examples:
  reed data:get knowledge.title@en
  reed data:get page-header.logo.url@de"#,
            namespace, action
        ),
        ("data", "set") => format!(
            r#"{}:{}

Store a value in ReedBase

Usage:
  reed data:set <key> <value>

Arguments:
  <key>      The key to store
  <value>    The value to store

Flags:
  --help     Show this help message
  --desc     Optional description

Examples:
  reed data:set knowledge.title@en "Knowledge Base"
  reed data:set page-header.logo.url@de "/assets/logo.svg""#,
            namespace, action
        ),
        ("layout", "create") => format!(
            r#"{}:{}

Create a new layout

Usage:
  reed layout:create <name> <variant>

Arguments:
  <name>      Layout name
  <variant>   Variant (mouse, touch, or reader)

Flags:
  --help      Show this help message

Examples:
  reed layout:create homepage mouse
  reed layout:create blog touch"#,
            namespace, action
        ),
        ("user", "create") => format!(
            r#"{}:{}

Create a new user

Usage:
  reed user:create <username>

Arguments:
  <username>   Username for the new user

Flags:
  --help       Show this help message
  --email      User email address
  --role       User role (default: user)

Examples:
  reed user:create alice --email alice@example.com
  reed user:create admin --role admin"#,
            namespace, action
        ),
        ("role", "create") => format!(
            r#"{}:{}

Create a new role

Usage:
  reed role:create <rolename>

Arguments:
  <rolename>      Name of the role

Flags:
  --help          Show this help message
  --permissions   Comma-separated permissions
  --desc          Role description

Examples:
  reed role:create editor --permissions "text[rwx],route[rw-]"
  reed role:create viewer --permissions "text[r--]""#,
            namespace, action
        ),
        ("taxonomy", "create") => format!(
            r#"{}:{}

Create a new taxonomy term

Usage:
  reed taxonomy:create <term>

Arguments:
  <term>     Term name

Flags:
  --help     Show this help message
  --parent   Parent term ID
  --category Category name

Examples:
  reed taxonomy:create "Rust" --category Programming
  reed taxonomy:create "Advanced" --parent Programming:Rust"#,
            namespace, action
        ),
        ("server", "start") => format!(
            r#"{}:{}

Start the ReedCMS server

Usage:
  reed server:start

Flags:
  --help     Show this help message
  --port     Server port (default: 8333)
  --host     Server host (default: 127.0.0.1)

Examples:
  reed server:start
  reed server:start --port 8080"#,
            namespace, action
        ),
        _ => format!(
            r#"{}:{}

No detailed help available for this command.

Use 'reed --help' to see all available commands."#,
            namespace, action
        ),
    };

    Ok(ReedResponse {
        data: help_text,
        source: "cli_help".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Prints version information.
pub fn print_version() -> ReedResult<ReedResponse<String>> {
    let version_text = format!(
        r#"ReedCMS
Version: {}
Licensed under the Apache License, Version 2.0"#,
        env!("CARGO_PKG_VERSION")
    );

    Ok(ReedResponse {
        data: version_text,
        source: "cli_help".to_string(),
        cached: false,
        timestamp: current_timestamp(),
        metrics: None,
    })
}

/// Generates usage string from command info.
pub fn generate_usage(info: &CommandInfo) -> String {
    let mut usage = format!("reed {}:{}", info.namespace, info.action);

    // Add required args
    for i in 0..info.required_args {
        usage.push_str(&format!(" <arg{}>", i + 1));
    }

    // Add required flags
    for flag in &info.required_flags {
        usage.push_str(&format!(" --{} <value>", flag));
    }

    // Add optional flags hint
    if !info.optional_flags.is_empty() {
        usage.push_str(" [--flags]");
    }

    usage
}

/// Lists all commands with descriptions.
pub fn list_commands() -> Vec<CommandInfo> {
    // TODO: Build this from registered commands
    vec![]
}
