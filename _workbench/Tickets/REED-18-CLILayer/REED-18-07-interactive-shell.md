# REED-18-04: Interactive Shell

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
- **Title**: Interactive Shell
- **Layer**: CLI Layer (REED-18)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-18-01 (Command Parser), REED-18-03 (Output Formatter)
- **Estimated Time**: 3 days

## Objective

Provide interactive shell mode (`reed shell`) with command history, persistent history file, and exit commands.

## Requirements

### Features

- **Interactive prompt** with custom prompt from Reed.toml
- **Command history** with ↑/↓ navigation
- **Persistent history file** (`.reed_history` by default)
- **Exit commands**: `exit`, `quit`, `\q`
- **Ctrl-C** returns to prompt (does not exit)
- **Ctrl-D** exits shell
- **Empty lines** ignored (no error)

### Example Session

```
$ reed shell
reed> query "SELECT * FROM users"
┌────┬───────┐
│ id │ name  │
├────┼───────┤
│ 1  │ Alice │
└────┴───────┘

reed> tables
text
routes
meta
users

reed> exit
$
```

## Implementation Files

### Primary Implementation

**`reedcli/src/shell.rs`**

One file = Interactive shell only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Interactive shell for ReedCLI.
//!
//! Provides REPL with command history and persistent history file.

use rustyline::{Editor, Config, CompletionType, EditMode, error::ReadlineError};
use crate::types::{Registry, CliResult, CliError};
use crate::parser;

/// Run interactive shell.
///
/// ## Input
/// - `registry`: Loaded registry for command routing
///
/// ## Output
/// - `CliResult<()>`: Ok on normal exit, error on failure
///
/// ## Performance
/// - Prompt response: < 5ms
/// - History load: < 10ms for typical history files (< 1000 entries)
///
/// ## Error Conditions
/// - IoError: Cannot save history file on exit
///
/// ## Example Usage
/// ```rust
/// let registry = load_registry("Reed.toml")?;
/// run_shell(&registry)?;
/// ```
pub fn run_shell(registry: &Registry) -> CliResult<()> {
    let config = build_editor_config();
    let mut rl = Editor::<()>::with_config(config)
        .map_err(|e| CliError::ShellError {
            reason: format!("Failed to create editor: {}", e),
        })?;
    
    load_history(&mut rl, &registry.cli.history_file);
    
    loop {
        let readline = rl.readline(&registry.cli.shell_prompt);
        
        match readline {
            Ok(line) => {
                if !handle_line(&line, &mut rl)? {
                    break; // Exit requested
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: continue (don't exit)
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D: exit
                break;
            }
            Err(e) => {
                return Err(CliError::ShellError {
                    reason: format!("Readline error: {}", e),
                });
            }
        }
    }
    
    save_history(&rl, &registry.cli.history_file)?;
    Ok(())
}

/// Build editor configuration.
///
/// ## Output
/// - `Config`: rustyline configuration
///
/// ## Performance
/// - O(1) operation
/// - < 1ms typical
fn build_editor_config() -> Config {
    Config::builder()
        .history_ignore_space(true)
        .history_ignore_dups(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build()
}

/// Load history from file.
///
/// ## Input
/// - `rl`: Editor instance
/// - `history_file`: Path to history file
///
/// ## Performance
/// - O(n) where n = number of history entries
/// - < 10ms for 1000 entries
///
/// ## Error Conditions
/// - None (prints warning if file doesn't exist, continues)
fn load_history(rl: &mut Editor<()>, history_file: &str) {
    if let Err(e) = rl.load_history(history_file) {
        eprintln!("No previous history: {}", e);
    }
}

/// Save history to file.
///
/// ## Input
/// - `rl`: Editor instance
/// - `history_file`: Path to history file
///
/// ## Output
/// - `CliResult<()>`: Ok on success
///
/// ## Performance
/// - O(n) where n = number of history entries
/// - < 20ms for 1000 entries
///
/// ## Error Conditions
/// - ShellError: Cannot write history file
fn save_history(rl: &Editor<()>, history_file: &str) -> CliResult<()> {
    rl.save_history(history_file)
        .map_err(|e| CliError::ShellError {
            reason: format!("Failed to save history: {}", e),
        })
}

/// Handle a single line of input.
///
/// ## Input
/// - `line`: Raw input line
/// - `rl`: Editor instance (for adding to history)
///
/// ## Output
/// - `CliResult<bool>`: True = continue, False = exit shell
///
/// ## Performance
/// - < 1ms for typical lines
///
/// ## Error Conditions
/// - None (errors are printed, shell continues)
fn handle_line(line: &str, rl: &mut Editor<()>) -> CliResult<bool> {
    let trimmed = line.trim();
    
    // Ignore empty lines
    if trimmed.is_empty() {
        return Ok(true);
    }
    
    // Add to history
    rl.add_history_entry(line);
    
    // Check for exit commands
    if is_exit_command(trimmed) {
        return Ok(false);
    }
    
    // Execute command (TODO: integrate with REED-18-06)
    if let Err(e) = execute_shell_command(trimmed) {
        eprintln!("Error: {}", e);
    }
    
    Ok(true)
}

/// Check if line is an exit command.
///
/// ## Input
/// - `line`: Trimmed input line
///
/// ## Output
/// - `bool`: True if exit command
///
/// ## Performance
/// - O(1) operation
/// - < 1μs typical
///
/// ## Example Usage
/// ```rust
/// assert!(is_exit_command("exit"));
/// assert!(is_exit_command("quit"));
/// assert!(is_exit_command("\\q"));
/// assert!(!is_exit_command("query"));
/// ```
fn is_exit_command(line: &str) -> bool {
    matches!(line, "exit" | "quit" | "\\q")
}

/// Execute shell command (stub for REED-18-06 integration).
///
/// ## Input
/// - `line`: Raw command line
///
/// ## Output
/// - `CliResult<()>`: Ok on success
///
/// ## Performance
/// - Depends on command execution (REED-18-06)
///
/// ## Error Conditions
/// - Depends on command execution (REED-18-06)
fn execute_shell_command(line: &str) -> CliResult<()> {
    let command = parser::parse_shell_input(line)?;
    
    // TODO: Route to tool handler (REED-18-06)
    println!("Parsed command: {:?}", command);
    
    Ok(())
}
```

**`reedcli/src/types.rs`** (additions)

```rust
/// Additional CLI errors.
#[derive(Error, Debug)]
pub enum CliError {
    // ... (existing errors from REED-18-01, REED-18-02, REED-18-03)
    
    #[error("Shell error: {reason}")]
    ShellError {
        reason: String,
    },
}
```

### Test Files

**`reedcli/src/shell.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_exit_command_exit() {
        assert!(is_exit_command("exit"));
    }
    
    #[test]
    fn test_is_exit_command_quit() {
        assert!(is_exit_command("quit"));
    }
    
    #[test]
    fn test_is_exit_command_backslash_q() {
        assert!(is_exit_command("\\q"));
    }
    
    #[test]
    fn test_is_exit_command_not_exit() {
        assert!(!is_exit_command("query"));
        assert!(!is_exit_command("tables"));
        assert!(!is_exit_command("exiting"));
    }
    
    #[test]
    fn test_build_editor_config() {
        let config = build_editor_config();
        // Config is opaque, just verify it builds
        assert!(true);
    }
    
    // Note: Full integration tests for run_shell() would require
    // mocking stdin/stdout, which is complex. Focus on unit tests
    // for individual functions. Integration testing can be done
    // manually or with expect-style tests.
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Prompt response | < 5ms |
| Load history (1000 entries) | < 10ms |
| Save history (1000 entries) | < 20ms |
| Check exit command | < 1μs |

## Error Conditions

- **ShellError**: Editor creation failure, history save failure, or readline error

## CLI Commands

```bash
# Start interactive shell
reed shell

# Example session
$ reed shell
reed> query "SELECT * FROM users"
[output]
reed> exit
$
```

## Acceptance Criteria

- [ ] Interactive prompt with custom prompt from Reed.toml
- [ ] Command history with ↑/↓ navigation
- [ ] Persistent history file (loaded on start, saved on exit)
- [ ] Exit commands work (`exit`, `quit`, `\q`)
- [ ] Ctrl-C returns to prompt (does not exit)
- [ ] Ctrl-D exits shell
- [ ] Empty lines ignored (no error)
- [ ] History ignores duplicate commands
- [ ] History ignores commands starting with space
- [ ] Errors displayed inline (shell continues)
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] All functions have complete documentation (Input/Output/Performance/Error Conditions/Example Usage)
- [ ] No Swiss Army knife functions (each function = one job)
- [ ] Separate test file as `shell.test.rs`

## Dependencies

**Requires**: 
- REED-18-01 (Command Parser - for parsing shell input)
- REED-18-03 (Output Formatter - for displaying results)

**Blocks**: None (optional feature, doesn't block other tickets)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-18-00: Layer Overview

## Notes

The shell does NOT:
- Execute commands (that's REED-18-06's job - integration point is `execute_shell_command()`)
- Provide tab completion (future enhancement)
- Support multi-line input (future enhancement)

The shell ONLY:
- Provides interactive prompt
- Manages command history
- Handles exit/interrupt signals

Integration with REED-18-06 (Tool Integration) happens in `execute_shell_command()`. The stub prints the parsed command; final implementation will route to tool handlers.
