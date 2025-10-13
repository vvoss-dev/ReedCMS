// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Adapter command executor for ReedCLI.
//!
//! Executes commands via adapter binaries as subprocesses.

use crate::types::{AdapterRegistry, AdapterResult, CliError, CliResult, ResolvedCommand};
use std::process::Command;
use std::time::Instant;

#[cfg(test)]
#[path = "executor_test.rs"]
mod tests;

/// Execute command via adapter binary.
///
/// ## Input
/// - `resolved`: ResolvedCommand with adapter and command
/// - `registry`: AdapterRegistry for binary path lookup
///
/// ## Output
/// - `CliResult<AdapterResult>`: Execution result with exit code and output
///
/// ## Performance
/// - Depends on adapter binary
/// - Subprocess overhead: ~10ms
///
/// ## Error Conditions
/// - AdapterNotFound: Adapter not in registry
/// - AdapterError: Binary execution failed
///
/// ## Example Usage
/// ```ignore
/// let result = execute_adapter_command(&resolved, &registry)?;
/// if result.exit_code == 0 {
///     println!("{}", result.stdout);
/// } else {
///     eprintln!("Error: {}", result.stderr);
/// }
/// ```
pub fn execute_adapter_command(
    resolved: &ResolvedCommand,
    registry: &AdapterRegistry,
) -> CliResult<AdapterResult> {
    let adapter =
        registry
            .adapters
            .get(&resolved.adapter)
            .ok_or_else(|| CliError::AdapterNotFound {
                adapter: resolved.adapter.clone(),
            })?;

    let args = build_adapter_args(resolved);

    let start = Instant::now();

    let output = Command::new(&adapter.binary)
        .args(&args)
        .output()
        .map_err(|e| CliError::AdapterError {
            adapter: resolved.adapter.clone(),
            message: format!("Failed to execute: {}", e),
        })?;

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(AdapterResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        duration_ms,
    })
}

/// Build command line arguments for adapter.
///
/// ## Input
/// - `resolved`: ResolvedCommand with command and args
///
/// ## Output
/// - `Vec<String>`: Arguments to pass to binary
///
/// ## Performance
/// - O(n) where n = number of arguments
/// - < 10μs
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```ignore
/// let args = build_adapter_args(&resolved);
/// // For: reed reedbase:query users "SELECT *"
/// // Returns: ["query", "users", "SELECT *"]
/// ```
pub fn build_adapter_args(resolved: &ResolvedCommand) -> Vec<String> {
    let mut args = vec![resolved.command.clone()];
    args.extend(resolved.args.clone());
    args
}

/// Handle adapter exit code and output.
///
/// ## Input
/// - `result`: AdapterResult from execution
///
/// ## Output
/// - `CliResult<String>`: Formatted output string
///
/// ## Performance
/// - O(n) where n = output length
/// - < 1ms
///
/// ## Error Conditions
/// - AdapterError: Non-zero exit code
///
/// ## Example Usage
/// ```ignore
/// let output = handle_adapter_result(result)?;
/// println!("{}", output);
/// ```
pub fn handle_adapter_result(result: AdapterResult, adapter_name: &str) -> CliResult<String> {
    if result.exit_code == 0 {
        Ok(result.stdout)
    } else {
        Err(CliError::AdapterError {
            adapter: adapter_name.to_string(),
            message: format!("Exit code {}: {}", result.exit_code, result.stderr.trim()),
        })
    }
}
