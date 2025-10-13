// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Adapter system for ReedCLI.
//!
//! Provides adapter discovery, command routing, and execution.

pub mod executor;
pub mod parser;
pub mod registry;
pub mod validator;

use crate::types::{AdapterRegistry, CliError, CliResult};
use std::path::Path;

/// Initialise adapter system from Reed.toml.
///
/// ## Input
/// - `config_path`: Path to Reed.toml
///
/// ## Output
/// - `CliResult<AdapterRegistry>`: Loaded and validated adapter registry
///
/// ## Performance
/// - O(n) where n = number of adapters
/// - ~100ms with validation
///
/// ## Error Conditions
/// - RegistryNotFound: Reed.toml missing
/// - InvalidRegistry: TOML parsing error
/// - RequiredAdapterMissing: Required adapter not found
///
/// ## Example Usage
/// ```ignore
/// let registry = initialise_adapters(Path::new("Reed.toml"))?;
/// // Ready to execute commands
/// ```
pub fn initialise_adapters(config_path: &Path) -> CliResult<AdapterRegistry> {
    // Load adapter registry
    let mut registry = registry::load_adapter_registry(config_path)?;

    // Discover commands for each adapter
    for (adapter_name, adapter) in &mut registry.adapters {
        match registry::discover_adapter_commands(adapter) {
            Ok(commands) => {
                adapter.commands = commands;
            }
            Err(e) => {
                // If adapter is required, fail immediately
                if adapter.required {
                    return Err(e);
                }
                // Otherwise, log warning and continue
                eprintln!(
                    "Warning: Could not discover commands for adapter '{}': {}",
                    adapter_name, e
                );
            }
        }
    }

    // Rebuild command index with discovered commands
    registry.command_index = registry::build_command_index(&registry.adapters)?;

    // Validate all adapters
    let validation = validator::validate_all_adapters(&registry)?;

    // Mark validated adapters
    for (adapter_name, validation_result) in &validation.results {
        if let Some(adapter) = registry.adapters.get_mut(adapter_name) {
            adapter.validated = validation_result.valid;
        }
    }

    Ok(registry)
}

/// Execute command with automatic adapter resolution.
///
/// ## Input
/// - `command_str`: Command string (e.g., "reedbase:query" or "query")
/// - `args`: Command arguments
/// - `registry`: AdapterRegistry
///
/// ## Output
/// - `CliResult<String>`: Command output
///
/// ## Performance
/// - Depends on adapter
/// - Overhead: ~10ms (resolution + subprocess)
///
/// ## Error Conditions
/// - CommandNotFound: Command not found
/// - AmbiguousCommand: Multiple adapters provide command
/// - AdapterError: Adapter execution failed
///
/// ## Example Usage
/// ```ignore
/// let output = execute_command(
///     "query",
///     vec!["users".to_string(), "SELECT *".to_string()],
///     &registry
/// )?;
/// println!("{}", output);
/// ```
pub fn execute_command(
    command_str: &str,
    args: Vec<String>,
    registry: &AdapterRegistry,
) -> CliResult<String> {
    // Parse command (with optional adapter namespace)
    let parsed = parser::parse_adapter_command(command_str);

    // Resolve adapter
    let adapter_name = parser::resolve_adapter(&parsed, registry)?;

    // Get adapter
    let adapter =
        registry
            .adapters
            .get(&adapter_name)
            .ok_or_else(|| CliError::AdapterNotFound {
                adapter: adapter_name.clone(),
            })?;

    // Expand alias if present
    let expanded_command = parser::expand_alias(&parsed.command, adapter);

    // Build resolved command
    let resolved = parser::build_resolved_command(adapter_name.clone(), expanded_command, args);

    // Execute via adapter
    let result = executor::execute_adapter_command(&resolved, registry)?;

    // Handle result
    executor::handle_adapter_result(result, &adapter_name)
}
