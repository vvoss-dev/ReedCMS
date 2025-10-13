# REED-18-06: Help System

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
- **ID**: REED-18-06
- **Title**: Help System
- **Layer**: CLI Layer (REED-18)
- **Priority**: High
- **Status**: Open
- **Complexity**: Low
- **Dependencies**: REED-18-02 (Registry Loader)
- **Estimated Time**: 2 days

## Objective

Generate help documentation from Reed.toml registry metadata. All help text is auto-generated from registry data (no hardcoded help strings in CLI code).

## Requirements

### Usage Levels

**Level 1: List all tools**
```bash
$ reed help
Available tools:
  reedbase - CSV-based versioned database
  reedcms  - Content management system
```

**Level 2: List tool commands**
```bash
$ reed help reedbase
Commands for reedbase:
  query      Execute SQL query
  tables     List all tables
  versions   List table versions
  rollback   Rollback to version
```

**Level 3: Command details**
```bash
$ reed help reedbase query
Command: reedbase query
Execute SQL query

Handler: execute_query
```

## Implementation Files

### Primary Implementation

**`reedcli/src/help.rs`**

One file = Help generation only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Help system for ReedCLI.
//!
//! Auto-generates help documentation from Reed.toml registry.

use crate::types::{Registry, CliResult, CliError};

/// Show help documentation.
///
/// ## Input
/// - `registry`: Loaded registry
/// - `args`: Help arguments (empty, tool name, or tool + command name)
///
/// ## Output
/// - `CliResult<String>`: Formatted help text
///
/// ## Performance
/// - O(n) where n = number of tools/commands to list
/// - < 20ms typical
///
/// ## Error Conditions
/// - InvalidArgs: Too many arguments (> 2)
/// - ToolNotFound: Tool does not exist
/// - CommandNotFound: Command not found for tool
///
/// ## Example Usage
/// ```rust
/// // List all tools
/// let help = show_help(&registry, &[])?;
///
/// // List tool commands
/// let help = show_help(&registry, &["reedbase"])?;
///
/// // Show command details
/// let help = show_help(&registry, &["reedbase", "query"])?;
/// ```
pub fn show_help(registry: &Registry, args: &[String]) -> CliResult<String> {
    match args.len() {
        0 => show_tools(registry),
        1 => show_tool_commands(registry, &args[0]),
        2 => show_command_help(registry, &args[0], &args[1]),
        _ => Err(CliError::InvalidArgs {
            message: "Too many arguments for help".to_string(),
        }),
    }
}

/// Show all available tools.
///
/// ## Input
/// - `registry`: Loaded registry
///
/// ## Output
/// - `CliResult<String>`: Formatted tool list
///
/// ## Performance
/// - O(n) where n = number of tools
/// - < 1ms for < 10 tools
///
/// ## Example Usage
/// ```rust
/// let help = show_tools(&registry)?;
/// println!("{}", help);
/// ```
fn show_tools(registry: &Registry) -> CliResult<String> {
    let mut output = String::from("Available tools:\n");
    
    let mut tools: Vec<_> = registry.tools.iter().collect();
    tools.sort_by_key(|(name, _)| *name);
    
    for (name, tool) in tools {
        let desc = tool.description.as_deref().unwrap_or("(no description)");
        output.push_str(&format!("  {:<12} {}\n", name, desc));
    }
    
    output.push_str("\nUse 'reed help <tool>' to see commands for a tool.\n");
    
    Ok(output)
}

/// Show all commands for a tool.
///
/// ## Input
/// - `registry`: Loaded registry
/// - `tool_name`: Tool name
///
/// ## Output
/// - `CliResult<String>`: Formatted command list
///
/// ## Performance
/// - O(n) where n = number of commands
/// - < 5ms for < 50 commands
///
/// ## Error Conditions
/// - ToolNotFound: Tool does not exist
///
/// ## Example Usage
/// ```rust
/// let help = show_tool_commands(&registry, "reedbase")?;
/// println!("{}", help);
/// ```
fn show_tool_commands(registry: &Registry, tool_name: &str) -> CliResult<String> {
    let tool = registry.tools.get(tool_name)
        .ok_or_else(|| CliError::ToolNotFound {
            tool: tool_name.to_string(),
        })?;
    
    let mut output = format!("Commands for {}:\n", tool_name);
    
    if let Some(desc) = &tool.description {
        output.push_str(&format!("{}\n\n", desc));
    }
    
    let mut commands: Vec<_> = tool.commands.iter().collect();
    commands.sort_by_key(|(name, _)| *name);
    
    for (cmd_name, cmd_spec) in commands {
        output.push_str(&format!("  {:<15} {}\n", cmd_name, cmd_spec.help));
    }
    
    output.push_str(&format!("\nUse 'reed help {} <command>' for command details.\n", tool_name));
    
    Ok(output)
}

/// Show details for a specific command.
///
/// ## Input
/// - `registry`: Loaded registry
/// - `tool_name`: Tool name
/// - `cmd_name`: Command name
///
/// ## Output
/// - `CliResult<String>`: Formatted command details
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100μs typical
///
/// ## Error Conditions
/// - ToolNotFound: Tool does not exist
/// - CommandNotFound: Command not found for tool
///
/// ## Example Usage
/// ```rust
/// let help = show_command_help(&registry, "reedbase", "query")?;
/// println!("{}", help);
/// ```
fn show_command_help(registry: &Registry, tool_name: &str, cmd_name: &str) -> CliResult<String> {
    let cmd_spec = registry.find_command(tool_name, cmd_name)?;
    
    let output = format!(
        "Command: {} {}\n{}\n\nHandler: {}\n",
        tool_name,
        cmd_name,
        cmd_spec.help,
        cmd_spec.handler
    );
    
    Ok(output)
}
```

**`reedcli/src/types.rs`** (additions)

```rust
/// Additional CLI errors.
#[derive(Error, Debug)]
pub enum CliError {
    // ... (existing errors)
    
    #[error("Invalid arguments: {message}")]
    InvalidArgs {
        message: String,
    },
}
```

### Test Files

**`reedcli/src/help.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::types::{Registry, CliConfig, Tool, CommandSpec};
    
    fn create_test_registry() -> Registry {
        let mut tools = HashMap::new();
        
        let mut reedbase_commands = HashMap::new();
        reedbase_commands.insert("query".to_string(), CommandSpec {
            handler: "execute_query".to_string(),
            help: "Execute SQL query".to_string(),
        });
        reedbase_commands.insert("tables".to_string(), CommandSpec {
            handler: "list_tables".to_string(),
            help: "List all tables".to_string(),
        });
        
        tools.insert("reedbase".to_string(), Tool {
            name: "reedbase".to_string(),
            description: Some("CSV-based versioned database".to_string()),
            binary: None,
            dependencies: vec![],
            commands: reedbase_commands,
        });
        
        Registry {
            version: "1.0".to_string(),
            cli: CliConfig {
                name: "reedcli".to_string(),
                binary: Some("reed".to_string()),
                shell_prompt: "reed> ".to_string(),
                history_file: ".reed_history".to_string(),
            },
            tools,
        }
    }
    
    #[test]
    fn test_show_tools() {
        let registry = create_test_registry();
        let help = show_tools(&registry).unwrap();
        
        assert!(help.contains("Available tools:"));
        assert!(help.contains("reedbase"));
        assert!(help.contains("CSV-based versioned database"));
    }
    
    #[test]
    fn test_show_tool_commands() {
        let registry = create_test_registry();
        let help = show_tool_commands(&registry, "reedbase").unwrap();
        
        assert!(help.contains("Commands for reedbase:"));
        assert!(help.contains("query"));
        assert!(help.contains("Execute SQL query"));
        assert!(help.contains("tables"));
        assert!(help.contains("List all tables"));
    }
    
    #[test]
    fn test_show_tool_commands_not_found() {
        let registry = create_test_registry();
        let result = show_tool_commands(&registry, "nonexistent");
        
        assert!(matches!(result, Err(CliError::ToolNotFound { .. })));
    }
    
    #[test]
    fn test_show_command_help() {
        let registry = create_test_registry();
        let help = show_command_help(&registry, "reedbase", "query").unwrap();
        
        assert!(help.contains("Command: reedbase query"));
        assert!(help.contains("Execute SQL query"));
        assert!(help.contains("Handler: execute_query"));
    }
    
    #[test]
    fn test_show_command_help_command_not_found() {
        let registry = create_test_registry();
        let result = show_command_help(&registry, "reedbase", "nonexistent");
        
        assert!(matches!(result, Err(CliError::CommandNotFound { .. })));
    }
    
    #[test]
    fn test_show_help_no_args() {
        let registry = create_test_registry();
        let help = show_help(&registry, &[]).unwrap();
        
        assert!(help.contains("Available tools:"));
        assert!(help.contains("reedbase"));
    }
    
    #[test]
    fn test_show_help_tool_arg() {
        let registry = create_test_registry();
        let args = vec!["reedbase".to_string()];
        let help = show_help(&registry, &args).unwrap();
        
        assert!(help.contains("Commands for reedbase:"));
    }
    
    #[test]
    fn test_show_help_tool_and_command_arg() {
        let registry = create_test_registry();
        let args = vec!["reedbase".to_string(), "query".to_string()];
        let help = show_help(&registry, &args).unwrap();
        
        assert!(help.contains("Command: reedbase query"));
    }
    
    #[test]
    fn test_show_help_too_many_args() {
        let registry = create_test_registry();
        let args = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = show_help(&registry, &args);
        
        assert!(matches!(result, Err(CliError::InvalidArgs { .. })));
    }
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| List tools (n=10) | < 1ms |
| List commands (n=50) | < 5ms |
| Show command details | < 100μs |

## Error Conditions

- **InvalidArgs**: Too many arguments (> 2)
- **ToolNotFound**: Tool does not exist in registry
- **CommandNotFound**: Command not found for tool

## CLI Commands

```bash
# List all tools
reed help

# List tool commands
reed help reedbase

# Show command details
reed help reedbase query
```

## Acceptance Criteria

- [ ] `reed help` lists all tools with descriptions
- [ ] `reed help <tool>` lists tool commands with help text
- [ ] `reed help <tool> <command>` shows command details
- [ ] Auto-generated from Reed.toml (no hardcoded help)
- [ ] Sorted output (tools and commands alphabetically sorted)
- [ ] Error for unknown tools
- [ ] Error for unknown commands
- [ ] Error for too many arguments
- [ ] Formatted output with alignment
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation (Input/Output/Performance/Error Conditions/Example Usage)
- [ ] No Swiss Army knife functions (each function = one job)
- [ ] Separate test file as `help.test.rs`

## Dependencies

**Requires**: 
- REED-18-02 (Registry Loader - for registry data)

**Blocks**: None

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-18-00: Layer Overview

## Notes

The help system does NOT:
- Store help text in CLI code (all from registry)
- Provide man pages (future enhancement)
- Support detailed argument documentation (future enhancement - would extend CommandSpec in Reed.toml)

The help system ONLY:
- Reads registry metadata
- Formats help text
- Pure data formatting, no I/O (except for reading registry, which is already in memory)

This design ensures help is always synchronized with Reed.toml. When tools are added/removed/changed in Reed.toml, help automatically reflects those changes without CLI code modifications.
