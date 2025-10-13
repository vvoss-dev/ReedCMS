# REED-18-04: Tool Integration

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
- **ID**: REED-18-04
- **Title**: Tool Integration
- **Layer**: CLI Layer (REED-18)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-18-01 (Parser), REED-18-02 (Registry), REED-18-06 (Formatter)
- **Estimated Time**: 3 days

## Objective

Dynamically invoke tool handlers based on registry metadata, marshal arguments, and format responses. This is the final integration point that connects all CLI components.

## Requirements

### Command Flow

```
User Command → Parser → Registry Lookup → Tool Handler → Response Formatter → Output
```

**Example:**
```bash
$ reed query "SELECT * FROM users" --format json

1. Parser extracts: Command { tool: "reedbase", command: "query", args: [...], flags: {...} }
2. Registry finds handler: "execute_query" for reedbase.query
3. Invoke: reedbase::execute_query(&args)
4. Receive: ReedResponse<QueryResult>
5. Format: formatter::format_output(response, OutputFormat::Json)
6. Print: stdout
7. Exit: code 0 (success)
```

### Exit Codes

| Exit Code | Meaning | Example |
|-----------|---------|---------|
| 0 | Success | Command executed successfully |
| 1 | User error | Bad arguments, command not found, invalid syntax |
| 2 | System error | Database failure, I/O error, permission denied |

## Implementation Files

### Primary Implementation

**`reedcli/src/integration.rs`**

One file = Tool invocation only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Tool integration for ReedCLI.
//!
//! Dynamically invokes tool handlers based on registry metadata.

use crate::types::{Command, Registry, CommandOutput, OutputFormat, CliResult, CliError};
use crate::formatter;

/// Execute command by routing to appropriate tool handler.
///
/// ## Input
/// - `command`: Parsed command
/// - `registry`: Loaded registry
///
/// ## Output
/// - `CliResult<String>`: Formatted output string
///
/// ## Performance
/// - Depends on tool handler execution time
/// - Registry lookup: < 10μs (O(1) HashMap)
/// - Formatting: < 5ms for typical outputs
///
/// ## Error Conditions
/// - ToolNotFound: Tool does not exist in registry
/// - CommandNotFound: Command not found for tool
/// - HandlerNotFound: Handler function not implemented
/// - ToolError: Tool-specific execution error
///
/// ## Example Usage
/// ```rust
/// let command = parser::parse_args(env::args())?;
/// let output = execute_command(&command, &registry)?;
/// println!("{}", output);
/// ```
pub fn execute_command(command: &Command, registry: &Registry) -> CliResult<String> {
    // Lookup command spec in registry
    let cmd_spec = registry.find_command(&command.tool, &command.command)?;
    
    // Route to appropriate tool
    let output = match command.tool.as_str() {
        "reedbase" => execute_reedbase_command(command, cmd_spec.handler.as_str())?,
        "reedcms" => execute_reedcms_command(command, cmd_spec.handler.as_str())?,
        _ => return Err(CliError::ToolNotFound {
            tool: command.tool.clone(),
        }),
    };
    
    // Format output
    let use_colour = formatter::supports_colour();
    let formatted = formatter::format_output(&output, use_colour)?;
    
    Ok(formatted)
}

/// Execute ReedBase command.
///
/// ## Input
/// - `command`: Parsed command
/// - `handler`: Handler function name from registry
///
/// ## Output
/// - `CliResult<CommandOutput>`: Command output with data
///
/// ## Performance
/// - Depends on specific ReedBase operation
/// - Query: < 100ms typical
/// - List tables: < 10ms typical
///
/// ## Error Conditions
/// - HandlerNotFound: Handler not implemented
/// - ToolError: ReedBase operation failed
///
/// ## Example Usage
/// ```rust
/// let output = execute_reedbase_command(&command, "execute_query")?;
/// ```
fn execute_reedbase_command(command: &Command, handler: &str) -> CliResult<CommandOutput> {
    match handler {
        "execute_query" => {
            // TODO: Integrate with ReedBase REED-19
            // For now, return stub data
            Ok(CommandOutput {
                data: serde_json::json!([
                    {"id": 1, "name": "Alice"},
                    {"id": 2, "name": "Bob"}
                ]),
                format: determine_output_format(&command.flags),
                exit_code: 0,
            })
        }
        "list_tables" => {
            // TODO: Integrate with ReedBase REED-19
            Ok(CommandOutput {
                data: serde_json::json!(["text", "routes", "meta", "users"]),
                format: determine_output_format(&command.flags),
                exit_code: 0,
            })
        }
        "list_versions" => {
            // TODO: Integrate with ReedBase REED-19
            Err(CliError::HandlerNotFound {
                handler: handler.to_string(),
            })
        }
        "rollback_table" => {
            // TODO: Integrate with ReedBase REED-19
            Err(CliError::HandlerNotFound {
                handler: handler.to_string(),
            })
        }
        _ => Err(CliError::HandlerNotFound {
            handler: handler.to_string(),
        }),
    }
}

/// Execute ReedCMS command.
///
/// ## Input
/// - `command`: Parsed command
/// - `handler`: Handler function name from registry
///
/// ## Output
/// - `CliResult<CommandOutput>`: Command output with data
///
/// ## Performance
/// - Depends on specific ReedCMS operation
///
/// ## Error Conditions
/// - HandlerNotFound: Handler not implemented
/// - ToolError: ReedCMS operation failed
///
/// ## Example Usage
/// ```rust
/// let output = execute_reedcms_command(&command, "server_start")?;
/// ```
fn execute_reedcms_command(command: &Command, handler: &str) -> CliResult<CommandOutput> {
    match handler {
        "server_start" => {
            // TODO: Integrate with existing ReedCMS server code
            Err(CliError::HandlerNotFound {
                handler: handler.to_string(),
            })
        }
        "server_stop" => {
            // TODO: Integrate with existing ReedCMS server code
            Err(CliError::HandlerNotFound {
                handler: handler.to_string(),
            })
        }
        "build_watch" => {
            // TODO: Integrate with existing ReedCMS build code
            Err(CliError::HandlerNotFound {
                handler: handler.to_string(),
            })
        }
        _ => Err(CliError::HandlerNotFound {
            handler: handler.to_string(),
        }),
    }
}

/// Determine output format from command flags.
///
/// ## Input
/// - `flags`: Command flags
///
/// ## Output
/// - `OutputFormat`: Determined format (default: Table)
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 1μs typical
///
/// ## Logic
/// - Check --format flag
/// - Valid values: "table", "json", "csv", "plain"
/// - Default: "table" if not specified or invalid
///
/// ## Example Usage
/// ```rust
/// let mut flags = HashMap::new();
/// flags.insert("format".to_string(), "json".to_string());
/// assert_eq!(determine_output_format(&flags), OutputFormat::Json);
/// ```
fn determine_output_format(flags: &std::collections::HashMap<String, String>) -> OutputFormat {
    flags.get("format")
        .and_then(|f| match f.as_str() {
            "json" => Some(OutputFormat::Json),
            "csv" => Some(OutputFormat::Csv),
            "plain" => Some(OutputFormat::Plain),
            "table" => Some(OutputFormat::Table),
            _ => None,
        })
        .unwrap_or(OutputFormat::Table)
}

/// Get exit code from command output.
///
/// ## Input
/// - `output`: Command output
///
/// ## Output
/// - `i32`: Exit code (0 = success, 1 = user error, 2 = system error)
///
/// ## Performance
/// - O(1) operation
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// let code = get_exit_code(&output);
/// std::process::exit(code);
/// ```
pub fn get_exit_code(output: &CommandOutput) -> i32 {
    output.exit_code
}

/// Convert CLI error to exit code.
///
/// ## Input
/// - `error`: CLI error
///
/// ## Output
/// - `i32`: Exit code (1 = user error, 2 = system error)
///
/// ## Performance
/// - O(1) operation
/// - < 1μs typical
///
/// ## Logic
/// - User errors (bad input, not found): exit code 1
/// - System errors (I/O, database): exit code 2
///
/// ## Example Usage
/// ```rust
/// match execute_command(&cmd, &registry) {
///     Ok(output) => println!("{}", output),
///     Err(e) => {
///         eprintln!("Error: {}", e);
///         std::process::exit(error_to_exit_code(&e));
///     }
/// }
/// ```
pub fn error_to_exit_code(error: &CliError) -> i32 {
    match error {
        // User errors (exit code 1)
        CliError::EmptyCommand |
        CliError::UnmatchedQuote |
        CliError::InvalidFlag { .. } |
        CliError::ToolNotFound { .. } |
        CliError::CommandNotFound { .. } |
        CliError::HandlerNotFound { .. } |
        CliError::InvalidArgs { .. } => 1,
        
        // System errors (exit code 2)
        CliError::RegistryNotFound { .. } |
        CliError::InvalidRegistry { .. } |
        CliError::CircularDependency { .. } |
        CliError::MissingDependency { .. } |
        CliError::FormatError { .. } |
        CliError::ShellError { .. } |
        CliError::ToolError { .. } => 2,
    }
}
```

**`reedcli/src/types.rs`** (additions)

```rust
/// Additional CLI errors.
#[derive(Error, Debug)]
pub enum CliError {
    // ... (existing errors)
    
    #[error("Handler '{handler}' not found")]
    HandlerNotFound {
        handler: String,
    },
    
    #[error("Tool error: {reason}")]
    ToolError {
        reason: String,
    },
}
```

### Test Files

**`reedcli/src/integration.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_determine_output_format_json() {
        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "json".to_string());
        assert_eq!(determine_output_format(&flags), OutputFormat::Json);
    }
    
    #[test]
    fn test_determine_output_format_csv() {
        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "csv".to_string());
        assert_eq!(determine_output_format(&flags), OutputFormat::Csv);
    }
    
    #[test]
    fn test_determine_output_format_plain() {
        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "plain".to_string());
        assert_eq!(determine_output_format(&flags), OutputFormat::Plain);
    }
    
    #[test]
    fn test_determine_output_format_table() {
        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "table".to_string());
        assert_eq!(determine_output_format(&flags), OutputFormat::Table);
    }
    
    #[test]
    fn test_determine_output_format_default() {
        let flags = HashMap::new();
        assert_eq!(determine_output_format(&flags), OutputFormat::Table);
    }
    
    #[test]
    fn test_determine_output_format_invalid() {
        let mut flags = HashMap::new();
        flags.insert("format".to_string(), "invalid".to_string());
        assert_eq!(determine_output_format(&flags), OutputFormat::Table);
    }
    
    #[test]
    fn test_get_exit_code() {
        let output = CommandOutput {
            data: serde_json::json!({}),
            format: OutputFormat::Table,
            exit_code: 42,
        };
        assert_eq!(get_exit_code(&output), 42);
    }
    
    #[test]
    fn test_error_to_exit_code_user_errors() {
        assert_eq!(error_to_exit_code(&CliError::EmptyCommand), 1);
        assert_eq!(error_to_exit_code(&CliError::UnmatchedQuote), 1);
        assert_eq!(error_to_exit_code(&CliError::InvalidFlag {
            flag: "test".to_string(),
            reason: "test".to_string(),
        }), 1);
        assert_eq!(error_to_exit_code(&CliError::ToolNotFound {
            tool: "test".to_string(),
        }), 1);
    }
    
    #[test]
    fn test_error_to_exit_code_system_errors() {
        assert_eq!(error_to_exit_code(&CliError::InvalidRegistry {
            reason: "test".to_string(),
        }), 2);
        assert_eq!(error_to_exit_code(&CliError::FormatError {
            reason: "test".to_string(),
        }), 2);
    }
    
    // Note: Full integration tests for execute_command() would require
    // actual ReedBase/ReedCMS integration, which happens in REED-19.
    // For now, focus on unit tests for helper functions.
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Registry lookup | < 10μs |
| Determine output format | < 1μs |
| Get exit code | < 1μs |
| Error to exit code conversion | < 1μs |

## Error Conditions

- **ToolNotFound**: Tool does not exist in registry
- **CommandNotFound**: Command not found for tool
- **HandlerNotFound**: Handler function not implemented
- **ToolError**: Tool-specific execution error (wrapped from ReedBase/ReedCMS)

## CLI Commands

Not applicable - this is the internal routing layer. All commands go through this module.

## Acceptance Criteria

- [ ] Route commands to appropriate tool (reedbase, reedcms)
- [ ] Lookup handlers via registry
- [ ] Marshal command arguments to handlers
- [ ] Receive responses from handlers
- [ ] Format responses via formatter
- [ ] Determine output format from --format flag
- [ ] Return appropriate exit codes (0, 1, 2)
- [ ] Convert errors to exit codes
- [ ] Stub handlers for ReedBase commands (integration in REED-19)
- [ ] Stub handlers for ReedCMS commands (integration with existing code)
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation (Input/Output/Performance/Error Conditions/Example Usage)
- [ ] No Swiss Army knife functions (each function = one job)
- [ ] Separate test file as `integration.test.rs`

## Dependencies

**Requires**: 
- REED-18-01 (Command Parser - for Command type)
- REED-18-02 (Registry Loader - for handler lookups)
- REED-18-06 (Output Formatter - for response formatting)

**Blocks**: None (but ReedBase/ReedCMS integration happens later)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-18-00: Layer Overview

## Notes

**Integration Points:**

This ticket provides stub implementations for tool handlers. Actual integration happens in:
- **REED-19**: ReedBase handlers (`execute_query`, `list_tables`, etc.)
- **Existing ReedCMS code**: Server handlers (`server_start`, `server_stop`, etc.)

**Current State:**
- `execute_reedbase_command()` returns stub data
- `execute_reedcms_command()` returns HandlerNotFound errors

**Future Work:**
- Replace stub data with actual ReedBase calls (REED-19)
- Wire up existing ReedCMS server/build commands

**Exit Code Philosophy:**
- **0**: Success - command executed without errors
- **1**: User error - bad input, typos, missing files (user can fix)
- **2**: System error - database down, permissions denied (system/admin must fix)

This convention follows Unix standards and allows shell scripts to distinguish between error types.
