// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Interactive shell for ReedCLI.
//!
//! Provides REPL with command history and persistent history file.

use crate::parser;
use crate::types::{CliError, CliResult, Registry};
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, EditMode, Editor};

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
/// - ShellError: Cannot save history file on exit
///
/// ## Example Usage
/// ```rust
/// let registry = load_registry("Reed.toml")?;
/// run_shell(&registry)?;
/// ```
pub fn run_shell(registry: &Registry) -> CliResult<()> {
    let config = build_editor_config();
    let mut rl =
        Editor::<(), rustyline::history::FileHistory>::with_config(config).map_err(|e| {
            CliError::ShellError {
                reason: format!("Failed to create editor: {}", e),
            }
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

    save_history(&mut rl, &registry.cli.history_file)?;
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
pub fn build_editor_config() -> Config {
    Config::builder()
        .history_ignore_space(true)
        .history_ignore_dups(true)
        .unwrap()
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
fn load_history(rl: &mut Editor<(), rustyline::history::FileHistory>, history_file: &str) {
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
fn save_history(
    rl: &mut Editor<(), rustyline::history::FileHistory>,
    history_file: &str,
) -> CliResult<()> {
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
fn handle_line(
    line: &str,
    rl: &mut Editor<(), rustyline::history::FileHistory>,
) -> CliResult<bool> {
    let trimmed = line.trim();

    // Ignore empty lines
    if trimmed.is_empty() {
        return Ok(true);
    }

    // Add to history
    let _ = rl.add_history_entry(line);

    // Check for exit commands
    if is_exit_command(trimmed) {
        return Ok(false);
    }

    // Execute command (TODO: integrate with REED-18-04)
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
pub fn is_exit_command(line: &str) -> bool {
    matches!(line, "exit" | "quit" | "\\q")
}

/// Execute shell command (stub for REED-18-04 integration).
///
/// ## Input
/// - `line`: Raw command line
///
/// ## Output
/// - `CliResult<()>`: Ok on success
///
/// ## Performance
/// - Depends on command execution (REED-18-04)
///
/// ## Error Conditions
/// - Depends on command execution (REED-18-04)
fn execute_shell_command(line: &str) -> CliResult<()> {
    let command = parser::parse_shell_input(line)?;

    // TODO: Route to tool handler (REED-18-04)
    println!("Parsed command: {:?}", command);

    Ok(())
}

#[cfg(test)]
#[path = "shell_test.rs"]
mod tests;
