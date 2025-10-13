// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Registry loader for ReedCLI.
//!
//! Loads Reed.toml and provides O(1) command lookups via HashMap.

use crate::types::{CliConfig, CliError, CliResult, CommandSpec, Registry, Tool};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[cfg(test)]
#[path = "registry_test.rs"]
mod tests;

/// Load registry from Reed.toml file.
///
/// ## Input
/// - `path`: Path to Reed.toml file
///
/// ## Output
/// - `CliResult<Registry>`: Loaded and validated registry
///
/// ## Performance
/// - O(n) where n = number of commands (for HashMap construction)
/// - < 10ms typical for reasonably-sized registries (< 100 commands)
///
/// ## Error Conditions
/// - RegistryNotFound: File does not exist
/// - InvalidRegistry: TOML syntax error or missing required fields
/// - CircularDependency: Tool dependency cycle detected
/// - MissingDependency: Tool depends on non-existent tool
///
/// ## Example Usage
/// ```ignore
/// use reedcli::registry::load_registry;
///
/// let registry = load_registry("Reed.toml")?;
/// let cmd = registry.find_command("reedbase", "query")?;
/// assert_eq!(cmd.handler, "execute_query");
/// ```
pub fn load_registry<P: AsRef<Path>>(path: P) -> CliResult<Registry> {
    let content = fs::read_to_string(path.as_ref()).map_err(|e| CliError::RegistryNotFound {
        path: path.as_ref().to_string_lossy().to_string(),
        source: e,
    })?;

    parse_registry(&content)
}

/// Parse registry from TOML string.
///
/// ## Input
/// - `content`: TOML content as string
///
/// ## Output
/// - `CliResult<Registry>`: Parsed and validated registry
///
/// ## Performance
/// - O(n) where n = number of commands
/// - < 10ms typical
///
/// ## Error Conditions
/// - InvalidRegistry: TOML parse error or schema validation failure
/// - CircularDependency: Dependency cycle detected
/// - MissingDependency: Missing dependency reference
///
/// ## Example Usage
/// ```ignore
/// use reedcli::registry::parse_registry;
///
/// let toml = r#"
///     [registry]
///     version = "1.0"
///     [cli]
///     name = "reedcli"
///     [tools.reedbase]
///     name = "reedbase"
///     [tools.reedbase.commands]
///     query = { handler = "execute_query", help = "Execute query" }
/// "#;
/// let registry = parse_registry(toml)?;
/// assert_eq!(registry.version, "1.0");
/// ```
pub fn parse_registry(content: &str) -> CliResult<Registry> {
    let toml_value: toml::Value =
        toml::from_str(content).map_err(|e| CliError::InvalidRegistry {
            reason: format!("TOML parse error: {}", e),
        })?;

    // Extract registry version
    let version = extract_registry_version(&toml_value)?;

    // Extract CLI config
    let cli = parse_cli_config(&toml_value)?;

    // Extract tools
    let tools = parse_tools(&toml_value)?;

    // Validate dependencies
    validate_dependencies(&tools)?;

    Ok(Registry {
        version,
        cli,
        tools,
    })
}

/// Extract registry version from TOML.
///
/// ## Input
/// - `toml_value`: Parsed TOML value
///
/// ## Output
/// - `CliResult<String>`: Registry version
///
/// ## Performance
/// - O(1) operation
/// - < 10μs typical
///
/// ## Error Conditions
/// - InvalidRegistry: Missing [registry] section or version field
fn extract_registry_version(toml_value: &toml::Value) -> CliResult<String> {
    toml_value
        .get("registry")
        .and_then(|r| r.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::InvalidRegistry {
            reason: "Missing [registry] version".to_string(),
        })
        .map(|s| s.to_string())
}

/// Parse CLI configuration section.
///
/// ## Input
/// - `toml_value`: Parsed TOML value
///
/// ## Output
/// - `CliResult<CliConfig>`: CLI configuration
///
/// ## Performance
/// - O(1) operation
/// - < 50μs typical
///
/// ## Error Conditions
/// - InvalidRegistry: Missing [cli] section or required fields
fn parse_cli_config(toml_value: &toml::Value) -> CliResult<CliConfig> {
    let cli_table = toml_value
        .get("cli")
        .ok_or_else(|| CliError::InvalidRegistry {
            reason: "Missing [cli] section".to_string(),
        })?;

    let name = cli_table
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::InvalidRegistry {
            reason: "Missing cli.name".to_string(),
        })?
        .to_string();

    let binary = cli_table
        .get("binary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let shell_prompt = cli_table
        .get("shell_prompt")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "reed> ".to_string());

    let history_file = cli_table
        .get("history_file")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| ".reed_history".to_string());

    Ok(CliConfig {
        name,
        binary,
        shell_prompt,
        history_file,
    })
}

/// Parse tools section.
///
/// ## Input
/// - `toml_value`: Parsed TOML value
///
/// ## Output
/// - `CliResult<HashMap<String, Tool>>`: Tools mapped by name
///
/// ## Performance
/// - O(n) where n = number of tools
/// - < 5ms typical for < 10 tools
///
/// ## Error Conditions
/// - InvalidRegistry: Missing [tools] section or malformed tool definitions
fn parse_tools(toml_value: &toml::Value) -> CliResult<HashMap<String, Tool>> {
    let tools_table = toml_value
        .get("tools")
        .and_then(|v| v.as_table())
        .ok_or_else(|| CliError::InvalidRegistry {
            reason: "Missing [tools] section".to_string(),
        })?;

    let mut tools = HashMap::new();

    for (tool_name, tool_value) in tools_table {
        let tool = parse_tool(tool_name, tool_value)?;
        tools.insert(tool_name.clone(), tool);
    }

    Ok(tools)
}

/// Parse individual tool.
///
/// ## Input
/// - `name`: Tool name
/// - `value`: TOML value for tool
///
/// ## Output
/// - `CliResult<Tool>`: Parsed tool
///
/// ## Performance
/// - O(n) where n = number of commands for tool
/// - < 1ms typical
///
/// ## Error Conditions
/// - InvalidRegistry: Tool is not a table or missing required fields
fn parse_tool(name: &str, value: &toml::Value) -> CliResult<Tool> {
    let table = value.as_table().ok_or_else(|| CliError::InvalidRegistry {
        reason: format!("Tool '{}' must be a table", name),
    })?;

    let display_name = table
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(name)
        .to_string();

    let description = table
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let binary = table
        .get("binary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let dependencies = parse_dependencies(table)?;
    let commands = parse_commands(table)?;

    Ok(Tool {
        name: display_name,
        description,
        binary,
        dependencies,
        commands,
    })
}

/// Parse tool dependencies.
///
/// ## Input
/// - `tool_table`: TOML table for tool
///
/// ## Output
/// - `CliResult<Vec<String>>`: List of dependency tool names
///
/// ## Performance
/// - O(n) where n = number of dependencies
/// - < 100μs typical
///
/// ## Error Conditions
/// - None (missing dependencies field returns empty vector)
fn parse_dependencies(tool_table: &toml::map::Map<String, toml::Value>) -> CliResult<Vec<String>> {
    Ok(tool_table
        .get("dependencies")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default())
}

/// Parse commands for a tool.
///
/// ## Input
/// - `tool_table`: TOML table for tool
///
/// ## Output
/// - `CliResult<HashMap<String, CommandSpec>>`: Commands mapped by name
///
/// ## Performance
/// - O(n) where n = number of commands
/// - < 500μs typical for < 20 commands
///
/// ## Error Conditions
/// - InvalidRegistry: Missing commands section or malformed command definitions
fn parse_commands(
    tool_table: &toml::map::Map<String, toml::Value>,
) -> CliResult<HashMap<String, CommandSpec>> {
    let commands_table = tool_table
        .get("commands")
        .and_then(|v| v.as_table())
        .ok_or_else(|| CliError::InvalidRegistry {
            reason: "Missing commands section for tool".to_string(),
        })?;

    let mut commands = HashMap::new();

    for (cmd_name, cmd_value) in commands_table {
        let cmd_spec = parse_command_spec(cmd_value)?;
        commands.insert(cmd_name.clone(), cmd_spec);
    }

    Ok(commands)
}

/// Parse command specification.
///
/// ## Input
/// - `value`: TOML value for command
///
/// ## Output
/// - `CliResult<CommandSpec>`: Parsed command spec
///
/// ## Performance
/// - O(1) operation
/// - < 50μs typical
///
/// ## Error Conditions
/// - InvalidRegistry: Command is not a table or missing handler field
fn parse_command_spec(value: &toml::Value) -> CliResult<CommandSpec> {
    let table = value.as_table().ok_or_else(|| CliError::InvalidRegistry {
        reason: "Command must be a table".to_string(),
    })?;

    let handler = table
        .get("handler")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::InvalidRegistry {
            reason: "Command missing 'handler' field".to_string(),
        })?
        .to_string();

    let help = table
        .get("help")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(CommandSpec { handler, help })
}

/// Validate tool dependencies (no circular deps, all deps exist).
///
/// ## Input
/// - `tools`: All tools
///
/// ## Output
/// - `CliResult<()>`: Ok if valid
///
/// ## Performance
/// - O(n*d) where n = tools, d = max dependency depth
/// - < 1ms typical
///
/// ## Error Conditions
/// - CircularDependency: Dependency cycle detected
/// - MissingDependency: Tool depends on non-existent tool
fn validate_dependencies(tools: &HashMap<String, Tool>) -> CliResult<()> {
    for (tool_name, tool) in tools {
        let mut visited = HashSet::new();
        check_circular_deps(tool_name, tool, tools, &mut visited)?;
    }

    Ok(())
}

/// Recursively check for circular dependencies.
///
/// ## Input
/// - `tool_name`: Current tool name
/// - `tool`: Current tool
/// - `all_tools`: All tools
/// - `visited`: Set of visited tool names (for cycle detection)
///
/// ## Output
/// - `CliResult<()>`: Ok if no cycles
///
/// ## Performance
/// - O(d) where d = dependency depth
/// - < 100μs typical
///
/// ## Error Conditions
/// - CircularDependency: Tool already in visited set
/// - MissingDependency: Dependency not found in all_tools
fn check_circular_deps(
    tool_name: &str,
    tool: &Tool,
    all_tools: &HashMap<String, Tool>,
    visited: &mut HashSet<String>,
) -> CliResult<()> {
    if visited.contains(tool_name) {
        return Err(CliError::CircularDependency {
            tool: tool_name.to_string(),
        });
    }

    visited.insert(tool_name.to_string());

    for dep_name in &tool.dependencies {
        let dep_tool = all_tools
            .get(dep_name)
            .ok_or_else(|| CliError::MissingDependency {
                tool: tool_name.to_string(),
                dependency: dep_name.clone(),
            })?;

        check_circular_deps(dep_name, dep_tool, all_tools, visited)?;
    }

    visited.remove(tool_name);
    Ok(())
}

impl Registry {
    /// Find command specification for a tool/command pair.
    ///
    /// ## Input
    /// - `tool`: Tool name (e.g., "reedbase")
    /// - `command`: Command name (e.g., "query")
    ///
    /// ## Output
    /// - `CliResult<&CommandSpec>`: Command specification reference
    ///
    /// ## Performance
    /// - O(1) HashMap lookup
    /// - < 10μs typical
    ///
    /// ## Error Conditions
    /// - ToolNotFound: Tool does not exist in registry
    /// - CommandNotFound: Command not found for tool
    ///
    /// ## Example Usage
    /// ```ignore
    /// let cmd = registry.find_command("reedbase", "query")?;
    /// assert_eq!(cmd.handler, "execute_query");
    /// ```
    pub fn find_command(&self, tool: &str, command: &str) -> CliResult<&CommandSpec> {
        let tool_entry = self.tools.get(tool).ok_or_else(|| CliError::ToolNotFound {
            tool: tool.to_string(),
        })?;

        tool_entry
            .commands
            .get(command)
            .ok_or_else(|| CliError::CommandNotFound {
                tool: tool.to_string(),
                command: command.to_string(),
            })
    }

    /// List all available tools.
    ///
    /// ## Output
    /// - `Vec<&str>`: Tool names
    ///
    /// ## Performance
    /// - O(n) where n = number of tools
    /// - < 100μs for < 10 tools
    ///
    /// ## Example Usage
    /// ```ignore
    /// let tools = registry.list_tools();
    /// assert!(tools.contains(&"reedbase"));
    /// ```
    pub fn list_tools(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// List all commands for a tool.
    ///
    /// ## Input
    /// - `tool`: Tool name
    ///
    /// ## Output
    /// - `CliResult<Vec<&str>>`: Command names
    ///
    /// ## Performance
    /// - O(n) where n = number of commands for tool
    /// - < 200μs for < 20 commands
    ///
    /// ## Error Conditions
    /// - ToolNotFound: Tool does not exist
    ///
    /// ## Example Usage
    /// ```ignore
    /// let commands = registry.list_commands("reedbase")?;
    /// assert!(commands.contains(&"query"));
    /// ```
    pub fn list_commands(&self, tool: &str) -> CliResult<Vec<&str>> {
        let tool_entry = self.tools.get(tool).ok_or_else(|| CliError::ToolNotFound {
            tool: tool.to_string(),
        })?;

        Ok(tool_entry.commands.keys().map(|s| s.as_str()).collect())
    }
}
