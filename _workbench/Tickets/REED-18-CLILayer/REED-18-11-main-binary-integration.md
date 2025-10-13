# REED-18-11: Main Binary Integration

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}_test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-18-11
- **Title**: Main Binary Integration
- **Layer**: CLI Layer (REED-18)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-18-08 (CommandProvider), REED-18-09 (ReedBase), REED-18-10 (ReedCMS)
- **Estimated Time**: 2 days

## Objective

Integrate all adapters (ReedBase, ReedCMS) into the main `reed` binary, replacing the old hardcoded router with the new adapter-based system. The result: **identical CLI behaviour** with clean architecture.

## Requirements

### Core Functionality

- **Auto-discover adapters** via Cargo features
- **Register all 62 commands** from both adapters
- **Maintain backwards compatibility** - all existing commands work identically
- **Feature flags** to enable/disable adapters
- **Help system** shows all registered commands
- **Error handling** with proper exit codes

### Architecture

```
src/main.rs
  ↓
reedcli::Router::new()
  ↓
discover_adapters() [REED-18-08]
  ├─→ ReedBaseAdapter::new() [REED-18-09]
  │     └─→ registers 25 commands
  └─→ ReedCMSAdapter::new() [REED-18-10]
        └─→ registers 37 commands
  ↓
run_with_router() [reedcli]
  ↓
User sees: reed <command:action> (works identically to before)
```

## Implementation Files

### Primary Implementation

**`src/main.rs`** (REPLACE existing)

One file = Binary entry point only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedCMS CLI binary.
//!
//! Main entry point for the reed command-line interface.
//! Discovers and registers adapters, then runs the CLI.

use reedcli::{Router, discover_adapters};
use std::process;

fn main() {
    // Create router
    let mut router = Router::new();
    
    // Discover and register all adapters
    match discover_adapters(&mut router) {
        Ok(adapters) => {
            if adapters.is_empty() {
                eprintln!("Warning: No adapters registered. CLI will have no commands.");
            } else {
                eprintln!("Registered adapters: {}", adapters.join(", "));
            }
        }
        Err(e) => {
            eprintln!("Fatal: Failed to discover adapters: {}", e);
            process::exit(2);
        }
    }
    
    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    
    // Handle special cases
    if args.len() == 1 || args.get(1).map(|s| s.as_str()) == Some("--help") || args.get(1).map(|s| s.as_str()) == Some("-h") {
        print_general_help(&router);
        process::exit(0);
    }
    
    if args.get(1).map(|s| s.as_str()) == Some("--version") || args.get(1).map(|s| s.as_str()) == Some("-v") {
        print_version(&router);
        process::exit(0);
    }
    
    // Parse command
    let command_args = args.into_iter().skip(1).collect();
    
    // Run CLI
    match run_cli(&router, command_args) {
        Ok(output) => {
            println!("{}", output);
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            let exit_code = error_to_exit_code(&e);
            process::exit(exit_code);
        }
    }
}

/// Run CLI with router and arguments.
///
/// ## Input
/// - `router`: Router with registered commands
/// - `args`: CLI arguments (excluding binary name)
///
/// ## Output
/// - `CliResult<String>`: Command output or error
///
/// ## Performance
/// - Depends on command execution
/// - Routing overhead: < 1ms
///
/// ## Error Conditions
/// - Parse errors
/// - Unknown commands
/// - Command execution errors
fn run_cli(router: &Router, args: Vec<String>) -> CliResult<String> {
    use reedcli::parser;
    
    // Parse command
    let command = parser::parse_args(args)?;
    
    // Route and execute
    let response = router.route(command)?;
    
    Ok(response.data)
}

/// Print general help.
///
/// ## Input
/// - `router`: Router with registered commands
fn print_general_help(router: &Router) {
    println!("ReedCMS Command-Line Interface\n");
    println!("Usage: reed <command:action> [args] [flags]\n");
    println!("Available Adapters:");
    
    for adapter in router.adapters() {
        println!("  {} v{} - {} ({} commands)", 
            adapter.name, 
            adapter.version, 
            adapter.description,
            adapter.command_count
        );
    }
    
    println!("\nGlobal Flags:");
    println!("  --help, -h       Show this help");
    println!("  --version, -v    Show version");
    
    println!("\nFor command-specific help:");
    println!("  reed <command:action> --help");
    
    println!("\nExamples:");
    println!("  reed set:text page.title \"Welcome\"");
    println!("  reed user:create admin");
    println!("  reed server:start");
}

/// Print version information.
///
/// ## Input
/// - `router`: Router with registered commands
fn print_version(router: &Router) {
    println!("reed {}", env!("CARGO_PKG_VERSION"));
    println!("\nAdapters:");
    
    for adapter in router.adapters() {
        println!("  {} v{}", adapter.name, adapter.version);
    }
}

/// Convert error to Unix exit code.
///
/// ## Input
/// - `error`: CLI error
///
/// ## Output
/// - `i32`: Exit code (0 = success, 1 = user error, 2 = system error)
fn error_to_exit_code(error: &CliError) -> i32 {
    use reedcli::types::CliError;
    
    match error {
        // User errors (exit code 1)
        CliError::EmptyCommand 
        | CliError::UnmatchedQuote 
        | CliError::InvalidFlag { .. }
        | CliError::InvalidArgs { .. }
        | CliError::UnknownCommand { .. } => 1,
        
        // System errors (exit code 2)
        | CliError::RegistryNotFound { .. }
        | CliError::InvalidRegistry { .. }
        | CliError::ToolNotFound { .. }
        | CliError::HandlerNotFound { .. }
        | CliError::AdapterNotFound { .. }
        | CliError::ShellError { .. } => 2,
    }
}
```

**`Cargo.toml`** (modifications)

```toml
[package]
name = "reedcms"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "reed"
path = "src/main.rs"

[dependencies]
# NEW: ReedCLI presentation layer
reedcli = { path = "reedcli" }

# NEW: Adapters (optional, controlled by features)
reedbase = { path = "reedbase", optional = true }
# reedcms adapter is built-in (this crate)

# Existing dependencies
actix-web = "4"
minijinja = "2"
# ... etc

[features]
default = ["reedbase-adapter", "reedcms-adapter"]

# Adapter features
reedbase-adapter = ["dep:reedbase", "reedcli/reedbase"]
reedcms-adapter = ["reedcli/reedcms"]

[workspace]
members = ["reedcli", "reedbase"]
```

**`reedcli/Cargo.toml`** (additions for feature flags)

```toml
[dependencies]
# ... existing dependencies

# Optional adapter dependencies
reedbase = { path = "../reedbase", optional = true }
reedcms = { path = "../", optional = true }

[features]
# Adapter features
reedbase = ["dep:reedbase"]
reedcms = ["dep:reedcms"]
```

## Testing Strategy

### Integration Tests

**`tests/cli_integration_test.rs`** (NEW)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for reed binary.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_shows_adapters() {
    let mut cmd = Command::cargo_bin("reed").unwrap();
    
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("reedbase"))
        .stdout(predicate::str::contains("reedcms"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("reed").unwrap();
    
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("reed"));
}

#[test]
fn test_reedbase_command_available() {
    let mut cmd = Command::cargo_bin("reed").unwrap();
    
    // This should work if ReedBase adapter is registered
    cmd.arg("list:text")
        .assert()
        .success(); // or specific output check
}

#[test]
fn test_reedcms_command_available() {
    let mut cmd = Command::cargo_bin("reed").unwrap();
    
    // This should work if ReedCMS adapter is registered
    cmd.arg("user:list")
        .assert()
        .success(); // or specific output check
}

#[test]
fn test_unknown_command_fails() {
    let mut cmd = Command::cargo_bin("reed").unwrap();
    
    cmd.arg("unknown:command")
        .assert()
        .failure()
        .code(1); // User error
}

#[test]
fn test_adapter_count() {
    // Verify both adapters are registered
    let mut cmd = Command::cargo_bin("reed").unwrap();
    
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("25 commands")) // ReedBase
        .stdout(predicate::str::contains("37 commands")); // ReedCMS
}
```

### Manual Testing Checklist

```bash
# Test all 62 commands work identically
reed set:text page.title "Test"
reed get:text page.title
reed list:text

reed user:create testuser
reed user:list
reed role:create testrole

reed server:status
reed build:complete
reed backup:list

# Test help system
reed --help
reed set:text --help

# Test error handling
reed unknown:command  # Should exit 1
reed set:text  # Missing args, should exit 1
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Binary startup | < 50ms |
| Adapter discovery | < 5ms |
| Command routing | < 1ms |
| Help generation | < 10ms |

## Error Conditions

- **Exit code 0**: Success
- **Exit code 1**: User error (bad args, unknown command)
- **Exit code 2**: System error (adapter failure, internal error)

## Acceptance Criteria

- [ ] Main binary discovers both adapters automatically
- [ ] All 62 commands work identically to before
- [ ] `reed --help` shows both adapters
- [ ] `reed --version` shows adapter versions
- [ ] Feature flags allow disabling adapters
- [ ] Error handling with correct exit codes
- [ ] Integration tests pass
- [ ] Manual testing checklist complete
- [ ] Performance targets met
- [ ] All code in BBC English
- [ ] Clean architecture maintained

## Migration Path

### Phase 1: Backup
```bash
# Backup old main.rs
cp src/main.rs src/main.rs.backup
```

### Phase 2: Implementation
1. Replace `src/main.rs` with new adapter-based version
2. Update `Cargo.toml` with workspace and features
3. Add integration tests

### Phase 3: Testing
```bash
# Build with all adapters
cargo build --release --all-features

# Test binary
./target/release/reed --help
./target/release/reed list:text
./target/release/reed user:list

# Run integration tests
cargo test --test cli_integration_test
```

### Phase 4: Validation
- Run ALL existing CLI tests
- Verify backwards compatibility
- Check performance benchmarks

## Dependencies

**Requires**: 
- REED-18-08 (Command Provider Trait)
- REED-18-09 (ReedBase Adapter - 25 commands)
- REED-18-10 (ReedCMS Adapter - 37 commands)

**Blocks**: None (this completes the adapter system)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- REED-18-08: Command Provider Trait
- REED-18-09: ReedBase Adapter
- REED-18-10: ReedCMS Adapter

## Notes

**Design Goals**:
- **Zero behaviour change** - users see no difference
- **Clean architecture** - adapters instead of hardcoded router
- **Easy extension** - new adapters can be added via features
- **Performance** - no overhead from adapter system

**Feature Flag Benefits**:
```toml
# Build with only ReedBase
cargo build --no-default-features --features reedbase-adapter

# Build with only ReedCMS
cargo build --no-default-features --features reedcms-adapter

# Build with everything (default)
cargo build
```

**Backwards Compatibility**:
- Old: `reed set:text page.title "Test"` ✅ Still works
- Old: `reed user:create admin` ✅ Still works
- Old: `reed --help` ✅ Still works
- All 62 commands: ✅ Identical behaviour

**What Changed**:
- Implementation: Adapter-based instead of hardcoded
- Architecture: Clean separation of concerns
- Extensibility: Easy to add new adapters

**What Stayed Same**:
- CLI syntax: Identical
- Commands: All 62 work exactly as before
- Behaviour: No user-visible changes
- Performance: Same speed (< 1ms routing overhead)
