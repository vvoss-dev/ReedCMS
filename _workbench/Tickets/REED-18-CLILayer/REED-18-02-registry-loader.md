# REED-18-02: Registry Loader

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-18-02
- **Title**: Registry Loader
- **Layer**: CLI Layer (REED-18)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-18-01 (Command Parser - for Command type)
- **Estimated Time**: 3 days

## Objective

Load and validate `Reed.toml` registry file, providing O(1) lookups for tool commands and metadata.

## Requirements

### Reed.toml Format

```toml
[registry]
version = "1.0"

[cli]
name = "reedcli"
binary = "reed"
shell_prompt = "reed> "
history_file = ".reed_history"

[tools.reedbase]
name = "reedbase"
description = "CSV-based versioned database"

[tools.reedbase.commands]
query = { handler = "execute_query", help = "Execute SQL query" }
tables = { handler = "list_tables", help = "List all tables" }
versions = { handler = "list_versions", help = "List table versions" }
rollback = { handler = "rollback_table", help = "Rollback to version" }

[tools.reedcms]
name = "reedcms"
description = "Content management system"
dependencies = ["reedbase"]

[tools.reedcms.commands]
"server:start" = { handler = "server_start", help = "Start web server" }
"server:stop" = { handler = "server_stop", help = "Stop web server" }
"build:watch" = { handler = "build_watch", help = "Watch and rebuild assets" }
```

### Output Structure

```rust
Registry {
    version: "1.0",
    cli: CliConfig {
        name: "reedcli",
        binary: Some("reed"),
        shell_prompt: "reed> ",
        history_file: ".reed_history",
    },
    tools: HashMap {
        "reedbase": Tool {
            name: "reedbase",
            description: Some("CSV-based versioned database"),
            binary: None,
            dependencies: vec![],
            commands: HashMap {
                "query": CommandSpec { handler: "execute_query", help: "Execute SQL query" },
                "tables": CommandSpec { handler: "list_tables", help: "List all tables" },
                ...
            },
        },
        "reedcms": Tool { ... },
    },
}
```

## Implementation Files

### Primary Implementation

**`reedcli/src/registry.rs`**

One file = Registry loading and validation only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Registry loader for ReedCLI.
//!
//! Loads Reed.toml and provides O(1) command lookups via HashMap.

use crate::types::{Registry, Tool, CommandSpec, CliConfig, CliResult, CliError};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
/// ```rust
/// let registry = load_registry("Reed.toml")?;
/// let cmd = registry.find_command("reedbase", "query")?;
/// assert_eq!(cmd.handler, "execute_query");
/// ```
pub fn load_registry<P: AsRef<Path>>(path: P) -> CliResult<Registry> {
    let content = fs::read_to_string(path.as_ref())
        .map_err(|e| CliError::RegistryNotFound {
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
/// ```rust
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
    let toml_value: toml::Value = toml::from_str(content)
        .map_err(|e| CliError::InvalidRegistry {
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
    let table = value
        .as_table()
        .ok_or_else(|| CliError::InvalidRegistry {
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
fn parse_commands(tool_table: &toml::map::Map<String, toml::Value>) -> CliResult<HashMap<String, CommandSpec>> {
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
    let table = value
        .as_table()
        .ok_or_else(|| CliError::InvalidRegistry {
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
    
    Ok(CommandSpec {
        handler,
        help,
    })
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
        let mut visited = std::collections::HashSet::new();
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
    visited: &mut std::collections::HashSet<String>,
) -> CliResult<()> {
    if visited.contains(tool_name) {
        return Err(CliError::CircularDependency {
            tool: tool_name.to_string(),
        });
    }
    
    visited.insert(tool_name.to_string());
    
    for dep_name in &tool.dependencies {
        let dep_tool = all_tools.get(dep_name)
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
    /// ```rust
    /// let cmd = registry.find_command("reedbase", "query")?;
    /// assert_eq!(cmd.handler, "execute_query");
    /// ```
    pub fn find_command(&self, tool: &str, command: &str) -> CliResult<&CommandSpec> {
        let tool_entry = self.tools.get(tool)
            .ok_or_else(|| CliError::ToolNotFound {
                tool: tool.to_string(),
            })?;
        
        tool_entry.commands.get(command)
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
    /// ```rust
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
    /// ```rust
    /// let commands = registry.list_commands("reedbase")?;
    /// assert!(commands.contains(&"query"));
    /// ```
    pub fn list_commands(&self, tool: &str) -> CliResult<Vec<&str>> {
        let tool_entry = self.tools.get(tool)
            .ok_or_else(|| CliError::ToolNotFound {
                tool: tool.to_string(),
            })?;
        
        Ok(tool_entry.commands.keys().map(|s| s.as_str()).collect())
    }
}
```

**`reedcli/src/types.rs`** (additions)

```rust
use std::collections::HashMap;

/// Registry loaded from Reed.toml.
#[derive(Debug, Clone)]
pub struct Registry {
    pub version: String,
    pub cli: CliConfig,
    pub tools: HashMap<String, Tool>,
}

/// CLI configuration.
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub name: String,
    pub binary: Option<String>,
    pub shell_prompt: String,
    pub history_file: String,
}

/// Tool specification.
#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub binary: Option<String>,
    pub dependencies: Vec<String>,
    pub commands: HashMap<String, CommandSpec>,
}

/// Command specification.
#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub handler: String,
    pub help: String,
}

/// Additional CLI errors.
#[derive(Error, Debug)]
pub enum CliError {
    // ... (existing errors from REED-18-01)
    
    #[error("Registry not found at '{path}': {source}")]
    RegistryNotFound {
        path: String,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Invalid registry: {reason}")]
    InvalidRegistry {
        reason: String,
    },
    
    #[error("Circular dependency detected for tool '{tool}'")]
    CircularDependency {
        tool: String,
    },
    
    #[error("Tool '{tool}' depends on missing tool '{dependency}'")]
    MissingDependency {
        tool: String,
        dependency: String,
    },
}
```

### Test Files

**`reedcli/src/registry.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_minimal_registry() {
        let toml = r#"
            [registry]
            version = "1.0"
            
            [cli]
            name = "reedcli"
            
            [tools.reedbase]
            name = "reedbase"
            
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "Execute query" }
        "#;
        
        let registry = parse_registry(toml).unwrap();
        assert_eq!(registry.version, "1.0");
        assert_eq!(registry.cli.name, "reedcli");
        assert!(registry.tools.contains_key("reedbase"));
    }
    
    #[test]
    fn test_find_command() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "Execute query" }
        "#;
        
        let registry = parse_registry(toml).unwrap();
        let cmd = registry.find_command("reedbase", "query").unwrap();
        assert_eq!(cmd.handler, "execute_query");
        assert_eq!(cmd.help, "Execute query");
    }
    
    #[test]
    fn test_command_not_found() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;
        
        let registry = parse_registry(toml).unwrap();
        let result = registry.find_command("reedbase", "nonexistent");
        assert!(matches!(result, Err(CliError::CommandNotFound { .. })));
    }
    
    #[test]
    fn test_tool_not_found() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;
        
        let registry = parse_registry(toml).unwrap();
        let result = registry.find_command("nonexistent", "query");
        assert!(matches!(result, Err(CliError::ToolNotFound { .. })));
    }
    
    #[test]
    fn test_parse_dependencies() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
            [tools.reedcms]
            name = "reedcms"
            dependencies = ["reedbase"]
            [tools.reedcms.commands]
            "server:start" = { handler = "server_start", help = "" }
        "#;
        
        let registry = parse_registry(toml).unwrap();
        let tool = registry.tools.get("reedcms").unwrap();
        assert_eq!(tool.dependencies, vec!["reedbase"]);
    }
    
    #[test]
    fn test_circular_dependency_detection() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.a]
            name = "a"
            dependencies = ["b"]
            [tools.a.commands]
            cmd = { handler = "handler", help = "" }
            [tools.b]
            name = "b"
            dependencies = ["a"]
            [tools.b.commands]
            cmd = { handler = "handler", help = "" }
        "#;
        
        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::CircularDependency { .. })));
    }
    
    #[test]
    fn test_missing_dependency_detection() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedcms]
            name = "reedcms"
            dependencies = ["nonexistent"]
            [tools.reedcms.commands]
            cmd = { handler = "handler", help = "" }
        "#;
        
        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::MissingDependency { .. })));
    }
    
    #[test]
    fn test_list_tools() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
            [tools.reedcms]
            name = "reedcms"
            [tools.reedcms.commands]
            cmd = { handler = "handler", help = "" }
        "#;
        
        let registry = parse_registry(toml).unwrap();
        let mut tools = registry.list_tools();
        tools.sort();
        assert_eq!(tools, vec!["reedbase", "reedcms"]);
    }
    
    #[test]
    fn test_list_commands() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
            tables = { handler = "list_tables", help = "" }
        "#;
        
        let registry = parse_registry(toml).unwrap();
        let mut commands = registry.list_commands("reedbase").unwrap();
        commands.sort();
        assert_eq!(commands, vec!["query", "tables"]);
    }
    
    #[test]
    fn test_missing_registry_version() {
        let toml = r#"
            [cli]
            name = "reedcli"
        "#;
        
        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::InvalidRegistry { .. })));
    }
    
    #[test]
    fn test_missing_cli_section() {
        let toml = r#"
            [registry]
            version = "1.0"
        "#;
        
        let result = parse_registry(toml);
        assert!(matches!(result, Err(CliError::InvalidRegistry { .. })));
    }
    
    #[test]
    fn test_default_shell_prompt() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;
        
        let registry = parse_registry(toml).unwrap();
        assert_eq!(registry.cli.shell_prompt, "reed> ");
    }
    
    #[test]
    fn test_custom_shell_prompt() {
        let toml = r#"
            [registry]
            version = "1.0"
            [cli]
            name = "reedcli"
            shell_prompt = "custom> "
            [tools.reedbase]
            name = "reedbase"
            [tools.reedbase.commands]
            query = { handler = "execute_query", help = "" }
        "#;
        
        let registry = parse_registry(toml).unwrap();
        assert_eq!(registry.cli.shell_prompt, "custom> ");
    }
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Load Reed.toml (100 commands) | < 10ms |
| Find command (O(1) lookup) | < 10μs |
| List tools (n=10) | < 100μs |
| List commands (n=50) | < 200μs |
| Validate dependencies (n=10, d=3) | < 1ms |

## Error Conditions

- **RegistryNotFound**: Reed.toml file does not exist
- **InvalidRegistry**: TOML syntax error or schema validation failure
- **CircularDependency**: Tool A depends on B, B depends on A (cycle)
- **MissingDependency**: Tool depends on non-existent tool

## CLI Commands

Not applicable - this is an internal registry loading module, not a CLI command.

## Acceptance Criteria

- [ ] Load Reed.toml from filesystem
- [ ] Parse TOML into Registry struct
- [ ] Validate registry structure (required fields present)
- [ ] Detect circular dependencies
- [ ] Detect missing dependencies
- [ ] O(1) command lookups via HashMap
- [ ] List all tools
- [ ] List all commands for a tool
- [ ] Return specific errors for invalid registries
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation (Input/Output/Performance/Error Conditions/Example Usage)
- [ ] No Swiss Army knife functions (each function = one job)
- [ ] Separate test file as `registry.test.rs`

## Dependencies

**Requires**: 
- REED-18-01 (Command Parser - for Command type)

**Blocks**: 
- REED-18-05 (Help System - needs registry for metadata)
- REED-18-06 (Tool Integration - needs registry for command lookups)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-18-00: Layer Overview

## Notes

The registry is loaded ONCE at CLI startup (no hot-reload). This design:
- **Simplifies implementation** (no file watching, no reload logic)
- **Improves performance** (no repeated file I/O)
- **Ensures consistency** (registry doesn't change mid-execution)

If Reed.toml changes, the user must restart the CLI. This is acceptable because registry changes are rare (adding/removing tools, not day-to-day usage).

The HashMap-based lookups provide O(1) command resolution, which is critical for CLI responsiveness (< 10μs typical).
