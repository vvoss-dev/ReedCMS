// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Adapter registry loader for ReedCLI.
//!
//! Loads adapter definitions from Reed.toml and provides command lookup.

use crate::types::{
    Adapter, AdapterCliConfig, AdapterRegistry, CliError, CliResult, CommandIndex, ValidationResult,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(test)]
#[path = "registry_test.rs"]
mod tests;

/// Load adapter registry from Reed.toml.
///
/// ## Input
/// - `config_path`: Path to Reed.toml file
///
/// ## Output
/// - `CliResult<AdapterRegistry>`: Loaded adapter registry
///
/// ## Performance
/// - O(n) where n = number of adapters
/// - < 10ms for typical configuration
///
/// ## Error Conditions
/// - RegistryNotFound: Reed.toml does not exist
/// - InvalidRegistry: TOML parsing error
/// - RequiredAdapterMissing: Required adapter binary missing
///
/// ## Example Usage
/// ```ignore
/// let registry = load_adapter_registry(Path::new("Reed.toml"))?;
/// println!("Loaded {} adapters", registry.adapters.len());
/// ```
pub fn load_adapter_registry(config_path: &Path) -> CliResult<AdapterRegistry> {
    let content = std::fs::read_to_string(config_path).map_err(|e| CliError::RegistryNotFound {
        path: config_path.to_string_lossy().to_string(),
        source: e,
    })?;

    let toml_value: toml::Value =
        toml::from_str(&content).map_err(|e| CliError::InvalidRegistry {
            reason: format!("TOML parse error: {}", e),
        })?;

    parse_adapter_registry(&toml_value)
}

/// Parse adapter registry from TOML value.
///
/// ## Input
/// - `toml_value`: Parsed TOML value
///
/// ## Output
/// - `CliResult<AdapterRegistry>`: Parsed adapter registry
///
/// ## Performance
/// - O(n) where n = number of adapters
/// - < 10ms typical
///
/// ## Error Conditions
/// - InvalidRegistry: Missing required fields or malformed structure
fn parse_adapter_registry(toml_value: &toml::Value) -> CliResult<AdapterRegistry> {
    // Parse CLI config
    let cli_config = parse_adapter_cli_config(toml_value)?;

    // Parse adapters
    let adapters = parse_adapters(toml_value)?;

    // Build command index
    let command_index = build_command_index(&adapters)?;

    Ok(AdapterRegistry {
        adapters,
        cli_config,
        command_index,
    })
}

/// Parse CLI configuration for adapters.
///
/// ## Input
/// - `toml_value`: Parsed TOML value
///
/// ## Output
/// - `CliResult<AdapterCliConfig>`: CLI configuration
///
/// ## Performance
/// - O(1) operation
/// - < 1ms typical
///
/// ## Error Conditions
/// - InvalidRegistry: Missing [cli] section or adapters list
fn parse_adapter_cli_config(toml_value: &toml::Value) -> CliResult<AdapterCliConfig> {
    let cli_table = toml_value
        .get("cli")
        .ok_or_else(|| CliError::InvalidRegistry {
            reason: "Missing [cli] section".to_string(),
        })?;

    let adapters = cli_table
        .get("adapters")
        .and_then(|v| v.as_array())
        .ok_or_else(|| CliError::InvalidRegistry {
            reason: "Missing cli.adapters list".to_string(),
        })?
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect();

    let namespace_omission = cli_table
        .get("namespace_omission")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    Ok(AdapterCliConfig {
        adapters,
        namespace_omission,
    })
}

/// Parse all adapters from TOML.
///
/// ## Input
/// - `toml_value`: Parsed TOML value
///
/// ## Output
/// - `CliResult<HashMap<String, Adapter>>`: Adapters mapped by name
///
/// ## Performance
/// - O(n) where n = number of adapters
/// - < 10ms typical
///
/// ## Error Conditions
/// - InvalidRegistry: Missing [adapters] section or malformed adapter definitions
fn parse_adapters(toml_value: &toml::Value) -> CliResult<HashMap<String, Adapter>> {
    let adapters_table = toml_value
        .get("adapters")
        .and_then(|v| v.as_table())
        .ok_or_else(|| CliError::InvalidRegistry {
            reason: "Missing [adapters] section".to_string(),
        })?;

    let mut adapters = HashMap::new();

    for (adapter_name, adapter_value) in adapters_table {
        let adapter = parse_adapter(adapter_name, adapter_value)?;
        adapters.insert(adapter_name.clone(), adapter);
    }

    Ok(adapters)
}

/// Parse individual adapter.
///
/// ## Input
/// - `name`: Adapter name
/// - `value`: TOML value for adapter
///
/// ## Output
/// - `CliResult<Adapter>`: Parsed adapter
///
/// ## Performance
/// - O(1) operation
/// - < 1ms typical
///
/// ## Error Conditions
/// - InvalidRegistry: Adapter is not a table or missing required fields
fn parse_adapter(name: &str, value: &toml::Value) -> CliResult<Adapter> {
    let table = value.as_table().ok_or_else(|| CliError::InvalidRegistry {
        reason: format!("Adapter '{}' must be a table", name),
    })?;

    let binary_str = table
        .get("binary")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::InvalidRegistry {
            reason: format!("Adapter '{}' missing 'binary' field", name),
        })?;

    let binary = PathBuf::from(binary_str);

    let description = table
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let version_requirement = table
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let required = table
        .get("required")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let aliases = parse_adapter_aliases(table)?;

    Ok(Adapter {
        name: name.to_string(),
        binary,
        description,
        version_requirement,
        required,
        aliases,
        commands: Vec::new(), // Filled by discover_adapter_commands
        validated: false,
    })
}

/// Parse adapter aliases.
///
/// ## Input
/// - `adapter_table`: TOML table for adapter
///
/// ## Output
/// - `CliResult<HashMap<String, String>>`: Aliases mapped short → full
///
/// ## Performance
/// - O(n) where n = number of aliases
/// - < 1ms typical
///
/// ## Error Conditions
/// - None (returns empty map if no aliases)
fn parse_adapter_aliases(
    adapter_table: &toml::map::Map<String, toml::Value>,
) -> CliResult<HashMap<String, String>> {
    Ok(adapter_table
        .get("aliases")
        .and_then(|v| v.as_table())
        .map(|aliases_table| {
            aliases_table
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default())
}

/// Discover commands from adapter binary.
///
/// ## Input
/// - `adapter`: Adapter to query
///
/// ## Output
/// - `CliResult<Vec<String>>`: List of command names
///
/// ## Performance
/// - O(1) - subprocess call
/// - < 100ms (depends on adapter responsiveness)
///
/// ## Error Conditions
/// - AdapterNotFound: Binary not in PATH
/// - AdapterError: Binary does not support --list-commands
///
/// ## Example Usage
/// ```ignore
/// let commands = discover_adapter_commands(&adapter)?;
/// for cmd in commands {
///     println!("Available: {}", cmd);
/// }
/// ```
pub fn discover_adapter_commands(adapter: &Adapter) -> CliResult<Vec<String>> {
    let output = Command::new(&adapter.binary)
        .arg("--list-commands")
        .output()
        .map_err(|_e| CliError::AdapterNotFound {
            adapter: adapter.name.clone(),
        })?;

    if !output.status.success() {
        return Err(CliError::AdapterError {
            adapter: adapter.name.clone(),
            message: format!(
                "Failed to list commands: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    let commands = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(commands)
}

/// Validate adapter binary exists and meets version requirements.
///
/// ## Input
/// - `adapter`: Adapter to validate
///
/// ## Output
/// - `CliResult<ValidationResult>`: Validation result
///
/// ## Performance
/// - O(1) - binary check + version query
/// - < 50ms
///
/// ## Error Conditions
/// - None (returns ValidationResult with valid=false on error)
///
/// ## Example Usage
/// ```ignore
/// let result = validate_adapter(&adapter)?;
/// if !result.valid {
///     eprintln!("Adapter validation failed: {}", result.error.unwrap());
/// }
/// ```
pub fn validate_adapter(adapter: &Adapter) -> CliResult<ValidationResult> {
    // Check if binary exists
    let _binary_path = match which::which(&adapter.binary) {
        Ok(path) => path,
        Err(_) => {
            return Ok(ValidationResult {
                valid: false,
                error: Some(format!(
                    "Binary '{}' not found in PATH",
                    adapter.binary.display()
                )),
                version: None,
            });
        }
    };

    // Get version if requirement specified
    if let Some(ref requirement) = adapter.version_requirement {
        match get_adapter_version(adapter) {
            Ok(version) => match version_matches(&version, requirement) {
                Ok(true) => {
                    return Ok(ValidationResult {
                        valid: true,
                        error: None,
                        version: Some(version),
                    });
                }
                Ok(false) => {
                    return Ok(ValidationResult {
                        valid: false,
                        error: Some(format!(
                            "Version mismatch: required {}, found {}",
                            requirement, version
                        )),
                        version: Some(version),
                    });
                }
                Err(e) => {
                    return Ok(ValidationResult {
                        valid: false,
                        error: Some(format!("Version comparison failed: {}", e)),
                        version: Some(version),
                    });
                }
            },
            Err(e) => {
                return Ok(ValidationResult {
                    valid: false,
                    error: Some(format!("Failed to get version: {}", e)),
                    version: None,
                });
            }
        }
    }

    // No version requirement - just check binary exists
    Ok(ValidationResult {
        valid: true,
        error: None,
        version: None,
    })
}

/// Check adapter version.
///
/// ## Input
/// - `adapter`: Adapter to check
///
/// ## Output
/// - `CliResult<String>`: Version string (e.g., "0.1.0")
///
/// ## Performance
/// - O(1) - subprocess call
/// - < 50ms
///
/// ## Error Conditions
/// - AdapterNotFound: Binary not found
/// - AdapterError: Binary does not support --version
///
/// ## Example Usage
/// ```ignore
/// let version = get_adapter_version(&adapter)?;
/// println!("ReedBase version: {}", version);
/// ```
pub fn get_adapter_version(adapter: &Adapter) -> CliResult<String> {
    let output = Command::new(&adapter.binary)
        .arg("--version")
        .output()
        .map_err(|_| CliError::AdapterNotFound {
            adapter: adapter.name.clone(),
        })?;

    if !output.status.success() {
        return Err(CliError::AdapterError {
            adapter: adapter.name.clone(),
            message: "Failed to get version".to_string(),
        });
    }

    let version_output = String::from_utf8_lossy(&output.stdout);

    // Extract version number (e.g., "reedbase 0.1.0" → "0.1.0")
    let version = version_output
        .split_whitespace()
        .last()
        .ok_or_else(|| CliError::AdapterError {
            adapter: adapter.name.clone(),
            message: "Invalid version output".to_string(),
        })?
        .to_string();

    Ok(version)
}

/// Compare version strings.
///
/// ## Input
/// - `version`: Actual version (e.g., "0.1.2")
/// - `requirement`: Version requirement (e.g., ">=0.1.0")
///
/// ## Output
/// - `CliResult<bool>`: true if version meets requirement
///
/// ## Performance
/// - O(1) - simple comparison
/// - < 10μs
///
/// ## Error Conditions
/// - InvalidVersion: Cannot parse version
fn version_matches(version: &str, requirement: &str) -> CliResult<bool> {
    let (operator, required_version) = if requirement.starts_with(">=") {
        (">=", &requirement[2..])
    } else if requirement.starts_with("<=") {
        ("<=", &requirement[2..])
    } else if requirement.starts_with('>') {
        (">", &requirement[1..])
    } else if requirement.starts_with('<') {
        ("<", &requirement[1..])
    } else if requirement.starts_with('=') {
        ("=", &requirement[1..])
    } else {
        ("=", requirement)
    };

    let actual_parts = parse_version(version)?;
    let required_parts = parse_version(required_version)?;

    let cmp = compare_versions(&actual_parts, &required_parts);

    Ok(match operator {
        ">=" => cmp >= 0,
        "<=" => cmp <= 0,
        ">" => cmp > 0,
        "<" => cmp < 0,
        "=" => cmp == 0,
        _ => false,
    })
}

/// Parse version string into (major, minor, patch).
fn parse_version(version: &str) -> CliResult<(u32, u32, u32)> {
    let parts: Vec<&str> = version.trim().split('.').collect();

    if parts.len() != 3 {
        return Err(CliError::InvalidVersion {
            version: version.to_string(),
        });
    }

    let major = parts[0]
        .parse::<u32>()
        .map_err(|_| CliError::InvalidVersion {
            version: version.to_string(),
        })?;

    let minor = parts[1]
        .parse::<u32>()
        .map_err(|_| CliError::InvalidVersion {
            version: version.to_string(),
        })?;

    let patch = parts[2]
        .parse::<u32>()
        .map_err(|_| CliError::InvalidVersion {
            version: version.to_string(),
        })?;

    Ok((major, minor, patch))
}

/// Compare two version tuples (-1, 0, 1).
fn compare_versions(a: &(u32, u32, u32), b: &(u32, u32, u32)) -> i32 {
    if a.0 != b.0 {
        return if a.0 > b.0 { 1 } else { -1 };
    }
    if a.1 != b.1 {
        return if a.1 > b.1 { 1 } else { -1 };
    }
    if a.2 != b.2 {
        return if a.2 > b.2 { 1 } else { -1 };
    }
    0
}

/// Build command index for fast lookup.
///
/// ## Input
/// - `adapters`: All adapters
///
/// ## Output
/// - `CliResult<CommandIndex>`: Command index
///
/// ## Performance
/// - O(n × m) where n = adapters, m = commands per adapter
/// - < 10ms
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```ignore
/// let index = build_command_index(&adapters)?;
/// if let Some(adapter) = index.find("query") {
///     println!("'query' belongs to adapter: {}", adapter);
/// }
/// ```
pub fn build_command_index(adapters: &HashMap<String, Adapter>) -> CliResult<CommandIndex> {
    let mut index = CommandIndex::new();

    for (adapter_name, adapter) in adapters {
        for command in &adapter.commands {
            index.add_command(command.clone(), adapter_name.clone());
        }
    }

    Ok(index)
}
