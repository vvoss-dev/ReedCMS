// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Adapter validator for ReedCLI.
//!
//! Validates adapter configuration and availability.

use crate::adapters::registry::validate_adapter;
use crate::types::{Adapter, AdapterRegistry, CliError, CliResult, RegistryValidation};
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(test)]
#[path = "validator_test.rs"]
mod tests;

/// Validate all adapters in registry.
///
/// ## Input
/// - `registry`: AdapterRegistry to validate
///
/// ## Output
/// - `CliResult<RegistryValidation>`: Validation result
///
/// ## Performance
/// - O(n) where n = number of adapters
/// - ~50ms per adapter (binary checks)
///
/// ## Error Conditions
/// - RequiredAdapterMissing: Required adapter not found
///
/// ## Example Usage
/// ```ignore
/// let validation = validate_all_adapters(&registry)?;
/// for (adapter, result) in &validation.results {
///     if !result.valid {
///         eprintln!("Adapter '{}' invalid: {}", adapter, result.error.unwrap());
///     }
/// }
/// ```
pub fn validate_all_adapters(registry: &AdapterRegistry) -> CliResult<RegistryValidation> {
    let mut results = HashMap::new();
    let mut all_valid = true;

    for (adapter_name, adapter) in &registry.adapters {
        let validation_result = validate_adapter(adapter)?;

        if !validation_result.valid && adapter.required {
            return Err(CliError::RequiredAdapterMissing {
                adapter: adapter_name.clone(),
            });
        }

        if !validation_result.valid {
            all_valid = false;
        }

        results.insert(adapter_name.clone(), validation_result);
    }

    Ok(RegistryValidation {
        valid: all_valid,
        results,
    })
}

/// Check if binary exists in PATH.
///
/// ## Input
/// - `binary_name`: Name of binary to find
///
/// ## Output
/// - `CliResult<Option<PathBuf>>`: Full path if found
///
/// ## Performance
/// - O(n) where n = directories in PATH
/// - < 10ms
///
/// ## Error Conditions
/// - None (returns None if not found)
///
/// ## Example Usage
/// ```ignore
/// if let Some(path) = find_binary_in_path("reedbase")? {
///     println!("Found: {}", path.display());
/// } else {
///     eprintln!("Binary not found");
/// }
/// ```
pub fn find_binary_in_path(binary_name: &str) -> CliResult<Option<PathBuf>> {
    Ok(which::which(binary_name).ok())
}

/// Generate helpful error message for missing adapter.
///
/// ## Input
/// - `adapter`: Adapter that is missing
///
/// ## Output
/// - `String`: Formatted error message with installation hints
///
/// ## Performance
/// - O(1) - string formatting
/// - < 1ms
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```ignore
/// let error_msg = format_missing_adapter_error(&adapter);
/// eprintln!("{}", error_msg);
/// ```
pub fn format_missing_adapter_error(adapter: &Adapter) -> String {
    format!(
        "Adapter '{}' not found.\n\n\
         Binary: {}\n\n\
         To install:\n\
         cargo install {}\n\n\
         Or ensure '{}' binary is in your PATH.",
        adapter.name,
        adapter.binary.display(),
        adapter.name,
        adapter.binary.display()
    )
}
