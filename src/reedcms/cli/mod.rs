// Copyright 2025 Vivian Voss. Licensed under the Apache Licence, Version 2.0.
// SPDX-Licence-Identifier: Apache-2.0

//! CLI Layer module organisation.

pub mod agent_commands;
pub mod build_commands;
pub mod data_commands;
pub mod help;
pub mod layout_commands;
pub mod migration_commands;
pub mod parser;
pub mod role_commands;
pub mod router;
pub mod server_commands;
pub mod taxonomy_commands;
pub mod user_commands;
pub mod validation_commands;

#[cfg(test)]
mod data_commands_test;
#[cfg(test)]
mod help_test;
#[cfg(test)]
mod layout_commands_test;
#[cfg(test)]
mod migration_commands_test;
#[cfg(test)]
mod parser_test;
#[cfg(test)]
mod role_commands_test;
#[cfg(test)]
mod router_test;
#[cfg(test)]
mod taxonomy_commands_test;
#[cfg(test)]
mod user_commands_test;
#[cfg(test)]
mod validation_commands_test;

pub use help::{print_command_help, print_general_help, print_version, CommandInfo};
pub use parser::{parse_command, Command};
pub use router::{create_router, CommandHandler, Router};

use crate::reedcms::reedstream::ReedResult;

/// Main CLI entry point.
///
/// ## Input
/// - args: CLI arguments (excluding binary name)
///
/// ## Output
/// - String output to print to stdout
///
/// ## Error Conditions
/// - Parse errors
/// - Unknown commands
/// - Command execution errors
pub fn run(args: Vec<String>) -> ReedResult<String> {
    // Handle special cases first
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        let response = print_general_help()?;
        return Ok(response.data);
    }

    if args[0] == "--version" || args[0] == "-v" {
        let response = print_version()?;
        return Ok(response.data);
    }

    // Parse command
    let cmd = parse_command(args)?;

    // Route and execute
    let router = create_router();
    let response = router.route(cmd)?;

    Ok(response.data)
}
