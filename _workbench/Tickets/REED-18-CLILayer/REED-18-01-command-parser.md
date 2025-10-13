# REED-18-01: Command Parser

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
- **ID**: REED-18-01
- **Title**: Command Parser
- **Layer**: CLI Layer (REED-18)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: None
- **Estimated Time**: 2 days

## Objective

Parse user input (command-line arguments or interactive shell input) into structured `Command` type for tool routing.

## Requirements

### Input Formats

**Command-line Arguments:**
```bash
reed query "SELECT * FROM users WHERE age > 30" --format json
reed server:start --port 8080
reed help reedbase
```

**Interactive Shell:**
```
reed> query "SELECT * FROM users"
reed> tables
reed> \help
```

### Output Structure

```rust
Command {
    tool: "reedbase",              // Inferred from command name
    command: "query",
    args: vec!["SELECT * FROM users WHERE age > 30"],
    flags: hashmap!{
        "format" => "json"
    }
}
```

## Implementation Files

### Primary Implementation

**`reedcli/src/parser.rs`**

One file = Command parsing only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Command parser for ReedCLI.
//!
//! Parses command-line arguments or interactive input into structured Command type.

use crate::types::{Command, CliResult, CliError};
use std::collections::HashMap;

/// Parse command-line arguments into Command.
///
/// ## Input
/// - `args`: Iterator of command-line arguments (from `std::env::args()`)
///
/// ## Output
/// - `CliResult<Command>`: Parsed command or error
///
/// ## Performance
/// - O(n) where n = number of arguments
/// - < 1ms typical for reasonable command lengths
///
/// ## Error Conditions
/// - EmptyCommand: No arguments provided
/// - UnmatchedQuote: Quote not closed
/// - InvalidFlag: Flag without value
///
/// ## Example Usage
/// ```rust
/// let args = vec!["reed", "query", "SELECT * FROM users", "--format", "json"];
/// let command = parse_args(args.into_iter().map(|s| s.to_string()))?;
/// assert_eq!(command.command, "query");
/// assert_eq!(command.flags.get("format"), Some(&"json".to_string()));
/// ```
pub fn parse_args<I>(args: I) -> CliResult<Command>
where
    I: Iterator<Item = String>,
{
    let args_vec: Vec<String> = args.collect();
    
    if args_vec.is_empty() || args_vec.len() == 1 {
        return Err(CliError::EmptyCommand);
    }

    // Skip program name (args[0])
    let command_parts: Vec<String> = args_vec.into_iter().skip(1).collect();
    
    parse_command_parts(&command_parts)
}

/// Parse interactive shell input into Command.
///
/// ## Input
/// - `input`: Raw line from shell
///
/// ## Output
/// - `CliResult<Command>`: Parsed command or error
///
/// ## Performance
/// - O(n) where n = input length
/// - < 1ms typical
///
/// ## Error Conditions
/// - EmptyCommand: Empty or whitespace-only input
/// - UnmatchedQuote: Quote not closed
/// - InvalidFlag: Flag without value
///
/// ## Example Usage
/// ```rust
/// let command = parse_shell_input("query \"SELECT * FROM users\"")?;
/// assert_eq!(command.command, "query");
/// assert_eq!(command.args[0], "SELECT * FROM users");
/// ```
pub fn parse_shell_input(input: &str) -> CliResult<Command> {
    let parts = tokenise_input(input)?;
    parse_command_parts(&parts)
}

/// Parse command parts into Command struct.
///
/// ## Input
/// - `parts`: Tokenised command parts
///
/// ## Output
/// - `CliResult<Command>`: Parsed command
///
/// ## Performance
/// - O(n) where n = number of parts
/// - < 500μs typical
///
/// ## Error Conditions
/// - EmptyCommand: No parts provided
/// - InvalidFlag: Flag without value
///
/// ## Example Usage
/// ```rust
/// let parts = vec!["query".to_string(), "SELECT * FROM users".to_string()];
/// let command = parse_command_parts(&parts)?;
/// assert_eq!(command.command, "query");
/// ```
fn parse_command_parts(parts: &[String]) -> CliResult<Command> {
    if parts.is_empty() {
        return Err(CliError::EmptyCommand);
    }

    let command_name = &parts[0];
    let mut args = Vec::new();
    let mut flags = HashMap::new();
    
    let mut i = 1;
    while i < parts.len() {
        let part = &parts[i];
        
        if part.starts_with("--") {
            // Long flag: --format json
            let flag_name = part.trim_start_matches("--");
            
            if i + 1 >= parts.len() {
                return Err(CliError::InvalidFlag {
                    flag: flag_name.to_string(),
                    reason: "Missing value".to_string(),
                });
            }
            
            let flag_value = &parts[i + 1];
            flags.insert(flag_name.to_string(), flag_value.clone());
            i += 2;
        } else if part.starts_with('-') && part.len() == 2 {
            // Short flag: -f json
            let flag_name = part.trim_start_matches('-');
            
            if i + 1 >= parts.len() {
                return Err(CliError::InvalidFlag {
                    flag: flag_name.to_string(),
                    reason: "Missing value".to_string(),
                });
            }
            
            let flag_value = &parts[i + 1];
            flags.insert(flag_name.to_string(), flag_value.clone());
            i += 2;
        } else {
            // Positional argument
            args.push(part.clone());
            i += 1;
        }
    }
    
    // Infer tool from command name
    let tool = infer_tool(command_name);
    
    Ok(Command {
        tool,
        command: command_name.clone(),
        args,
        flags,
    })
}

/// Tokenise input string, respecting quoted strings.
///
/// ## Input
/// - `input`: Raw input string
///
/// ## Output
/// - `CliResult<Vec<String>>`: Tokenised parts
///
/// ## Performance
/// - O(n) where n = input length
/// - < 200μs for 100-char input
///
/// ## Error Conditions
/// - UnmatchedQuote: Quote not closed
///
/// ## Example Usage
/// ```rust
/// let tokens = tokenise_input(r#"query "SELECT * FROM users" --format json"#)?;
/// assert_eq!(tokens[0], "query");
/// assert_eq!(tokens[1], "SELECT * FROM users");
/// assert_eq!(tokens[2], "--format");
/// assert_eq!(tokens[3], "json");
/// ```
fn tokenise_input(input: &str) -> CliResult<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;
    
    for ch in input.chars() {
        if escape_next {
            current_token.push(ch);
            escape_next = false;
            continue;
        }
        
        match ch {
            '\\' => {
                escape_next = true;
            }
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' if !in_quotes => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }
    
    if in_quotes {
        return Err(CliError::UnmatchedQuote);
    }
    
    if !current_token.is_empty() {
        tokens.push(current_token);
    }
    
    Ok(tokens)
}

/// Infer tool name from command name.
///
/// ## Input
/// - `command`: Command name (e.g., "query", "server:start")
///
/// ## Output
/// - Tool name (e.g., "reedbase", "reedcms")
///
/// ## Performance
/// - O(1) operation
/// - < 10μs typical
///
/// ## Logic
/// - Commands with `:` → Tool is prefix (e.g., "server:start" → "server")
/// - Known ReedBase commands → "reedbase"
/// - Default → "reedcms"
///
/// ## Example Usage
/// ```rust
/// assert_eq!(infer_tool("query"), "reedbase");
/// assert_eq!(infer_tool("server:start"), "server");
/// assert_eq!(infer_tool("unknown"), "reedcms");
/// ```
fn infer_tool(command: &str) -> String {
    // If command contains ':', tool is the prefix
    if let Some(pos) = command.find(':') {
        return command[..pos].to_string();
    }
    
    // Known ReedBase commands
    const REEDBASE_COMMANDS: &[&str] = &[
        "query", "tables", "versions", "rollback",
    ];
    
    if REEDBASE_COMMANDS.contains(&command) {
        return "reedbase".to_string();
    }
    
    // Default to reedcms
    "reedcms".to_string()
}
```

**`reedcli/src/types.rs`**

Shared types for CLI. One file = Type definitions only.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use thiserror::Error;

/// Parsed command from user input.
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    /// Tool to route to (e.g., "reedbase", "reedcms")
    pub tool: String,
    
    /// Command name (e.g., "query", "server:start")
    pub command: String,
    
    /// Positional arguments
    pub args: Vec<String>,
    
    /// Named flags (e.g., --format json → {"format": "json"})
    pub flags: HashMap<String, String>,
}

/// CLI error types.
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Empty command")]
    EmptyCommand,
    
    #[error("Unmatched quote in input")]
    UnmatchedQuote,
    
    #[error("Invalid flag '{flag}': {reason}")]
    InvalidFlag {
        flag: String,
        reason: String,
    },
    
    #[error("Tool '{tool}' not found in registry")]
    ToolNotFound {
        tool: String,
    },
    
    #[error("Command '{command}' not found for tool '{tool}'")]
    CommandNotFound {
        tool: String,
        command: String,
    },
}

pub type CliResult<T> = Result<T, CliError>;
```

### Test Files

**`reedcli/src/parser.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_command() {
        let args = vec!["reed", "query", "SELECT * FROM users"];
        let cmd = parse_args(args.into_iter().map(|s| s.to_string())).unwrap();
        
        assert_eq!(cmd.tool, "reedbase");
        assert_eq!(cmd.command, "query");
        assert_eq!(cmd.args, vec!["SELECT * FROM users"]);
        assert!(cmd.flags.is_empty());
    }
    
    #[test]
    fn test_parse_command_with_long_flags() {
        let args = vec!["reed", "query", "SELECT * FROM users", "--format", "json"];
        let cmd = parse_args(args.into_iter().map(|s| s.to_string())).unwrap();
        
        assert_eq!(cmd.flags.get("format"), Some(&"json".to_string()));
    }
    
    #[test]
    fn test_parse_command_with_short_flags() {
        let args = vec!["reed", "query", "SELECT * FROM users", "-f", "json"];
        let cmd = parse_args(args.into_iter().map(|s| s.to_string())).unwrap();
        
        assert_eq!(cmd.flags.get("f"), Some(&"json".to_string()));
    }
    
    #[test]
    fn test_parse_namespaced_command() {
        let args = vec!["reed", "server:start", "--port", "8080"];
        let cmd = parse_args(args.into_iter().map(|s| s.to_string())).unwrap();
        
        assert_eq!(cmd.tool, "server");
        assert_eq!(cmd.command, "server:start");
        assert_eq!(cmd.flags.get("port"), Some(&"8080".to_string()));
    }
    
    #[test]
    fn test_tokenise_quoted_strings() {
        let input = r#"query "SELECT * FROM users WHERE name = 'John'" --format json"#;
        let tokens = tokenise_input(input).unwrap();
        
        assert_eq!(tokens[0], "query");
        assert_eq!(tokens[1], "SELECT * FROM users WHERE name = 'John'");
        assert_eq!(tokens[2], "--format");
        assert_eq!(tokens[3], "json");
    }
    
    #[test]
    fn test_tokenise_escaped_quotes() {
        let input = r#"query "SELECT * FROM users WHERE name = \"John\"" "#;
        let tokens = tokenise_input(input).unwrap();
        
        assert_eq!(tokens[1], r#"SELECT * FROM users WHERE name = \"John\""#);
    }
    
    #[test]
    fn test_unmatched_quote_error() {
        let input = r#"query "SELECT * FROM users"#;
        let result = tokenise_input(input);
        
        assert!(matches!(result, Err(CliError::UnmatchedQuote)));
    }
    
    #[test]
    fn test_empty_command_error() {
        let args: Vec<String> = vec!["reed".to_string()];
        let result = parse_args(args.into_iter());
        
        assert!(matches!(result, Err(CliError::EmptyCommand)));
    }
    
    #[test]
    fn test_missing_flag_value_error() {
        let args = vec!["reed", "query", "SELECT * FROM users", "--format"];
        let result = parse_args(args.into_iter().map(|s| s.to_string()));
        
        assert!(matches!(result, Err(CliError::InvalidFlag { .. })));
    }
    
    #[test]
    fn test_infer_tool_from_known_command() {
        assert_eq!(infer_tool("query"), "reedbase");
        assert_eq!(infer_tool("tables"), "reedbase");
    }
    
    #[test]
    fn test_infer_tool_from_namespaced_command() {
        assert_eq!(infer_tool("server:start"), "server");
        assert_eq!(infer_tool("build:watch"), "build");
    }
    
    #[test]
    fn test_infer_tool_default() {
        assert_eq!(infer_tool("unknown"), "reedcms");
    }
    
    #[test]
    fn test_shell_input_parsing() {
        let input = "query \"SELECT * FROM users\"";
        let cmd = parse_shell_input(input).unwrap();
        
        assert_eq!(cmd.command, "query");
        assert_eq!(cmd.args, vec!["SELECT * FROM users"]);
    }
    
    #[test]
    fn test_multiple_arguments() {
        let args = vec!["reed", "query", "arg1", "arg2", "arg3"];
        let cmd = parse_args(args.into_iter().map(|s| s.to_string())).unwrap();
        
        assert_eq!(cmd.args.len(), 3);
        assert_eq!(cmd.args[0], "arg1");
        assert_eq!(cmd.args[1], "arg2");
        assert_eq!(cmd.args[2], "arg3");
    }
    
    #[test]
    fn test_mixed_flags_and_args() {
        let args = vec!["reed", "query", "arg1", "--flag1", "val1", "arg2", "-f", "val2"];
        let cmd = parse_args(args.into_iter().map(|s| s.to_string())).unwrap();
        
        assert_eq!(cmd.args, vec!["arg1", "arg2"]);
        assert_eq!(cmd.flags.get("flag1"), Some(&"val1".to_string()));
        assert_eq!(cmd.flags.get("f"), Some(&"val2".to_string()));
    }
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Parse simple command (3 args) | < 100μs |
| Parse command with 10 args | < 500μs |
| Tokenise 100-char input | < 200μs |
| Infer tool from command | < 10μs |

## Error Conditions

- **EmptyCommand**: No arguments provided or only program name
- **UnmatchedQuote**: Opening quote without closing quote
- **InvalidFlag**: Flag specified without value (e.g., `--format` with nothing after)

## CLI Commands

Not applicable - this is an internal parsing module, not a CLI command.

## Acceptance Criteria

- [ ] Parse command-line arguments into `Command` struct
- [ ] Parse interactive shell input into `Command` struct
- [ ] Handle quoted strings (preserving spaces inside quotes)
- [ ] Handle escaped quotes (`\"`)
- [ ] Parse long flags (`--format json`)
- [ ] Parse short flags (`-f json`)
- [ ] Infer tool from command name (`:` prefix or known commands)
- [ ] Return EmptyCommand error for no input
- [ ] Return UnmatchedQuote error for unclosed quotes
- [ ] Return InvalidFlag error for flags without values
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation (Input/Output/Performance/Error Conditions/Example Usage)
- [ ] No Swiss Army knife functions (each function = one job)
- [ ] Separate test file as `parser.test.rs`

## Dependencies

**Requires**: None (foundation ticket)

**Blocks**: 
- REED-18-04 (Interactive Shell - needs parser)
- REED-18-06 (Tool Integration - needs Command type)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-18-00: Layer Overview

## Notes

The parser does NOT:
- Validate commands against registry (that's REED-18-02's job)
- Invoke tools (that's REED-18-06's job)
- Format output (that's REED-18-03's job)

The parser ONLY:
- Parses strings into structured `Command` type
- Pure string manipulation, no I/O, no network, no filesystem

This separation ensures the parser can be tested in isolation and reused in different contexts (CLI, shell, tests).
