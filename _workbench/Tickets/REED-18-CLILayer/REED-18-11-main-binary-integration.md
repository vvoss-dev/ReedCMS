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
- **Status**: **BLOCKED** - Waiting for REED-19
- **Complexity**: Medium
- **Dependencies**: 
  - REED-18-08 (Command Provider Trait)
  - **REED-19-** (Standalone ReedBase - MUST exist first)
  - **REED-18-09** (ReedBase Adapter - MUST be complete)
  - **REED-18-10** (ReedCMS Adapter - MUST be complete)
- **Estimated Time**: 2 days (AFTER all adapters complete)

## ⚠️ IMPORTANT: This Ticket is BLOCKED

**This ticket CANNOT be implemented until:**
1. **REED-19 (Standalone ReedBase) is complete**
2. **REED-18-09 (ReedBase Adapter) is complete**
3. **REED-18-10 (ReedCMS Adapter) is complete**

This is the **final integration step** that brings everything together.

## Objective

Integrate all adapters (ReedBase, ReedCMS) into the main `reed` binary using the new adapter-based system. The result: **clean modular architecture** with three separate tools working together via CLI.

## Requirements

### Core Functionality

- **Auto-discover adapters** via Cargo features
- **Register all commands** from both adapters
- **Feature flags** to enable/disable adapters
- **Help system** shows all registered commands
- **Error handling** with proper exit codes

### Architecture (After REED-19)

```
CLI > ReedBase > ReedCMS

reed binary (src/main.rs)
  ↓
reedcli::discover_adapters()
  ├─→ ReedBaseAdapter [REED-18-09]
  │     ├─ Standalone database (REED-19)
  │     └─ Commands: table:*, query, version:*, etc.
  │
  └─→ ReedCMSAdapter [REED-18-10]
        ├─ Uses ReedBase as backend
        └─ Commands: content:*, user:*, role:*, server:*, etc.
  ↓
User sees: reed <command:action>
```

## Implementation Files

### Primary Implementation

**`src/main.rs`** (REPLACE when ready)

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
    if args.len() == 1 || args.get(1).map(|s| s.as_str()) == Some("--help") {
        print_general_help(&router);
        process::exit(0);
    }
    
    if args.get(1).map(|s| s.as_str()) == Some("--version") {
        print_version(&router);
        process::exit(0);
    }
    
    // Parse and run command
    let command_args = args.into_iter().skip(1).collect();
    
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

// ... (helper functions as defined in original ticket)
```

**`Cargo.toml`** (workspace structure)

```toml
[workspace]
members = ["reedcli", "reedbase", "reedcms"]

[package]
name = "reed"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "reed"
path = "src/main.rs"

[dependencies]
reedcli = { path = "reedcli" }

# Adapters (optional via features)
reedbase = { path = "reedbase", optional = true }
reedcms = { path = "reedcms", optional = true }

[features]
default = ["reedbase-adapter", "reedcms-adapter"]

# Adapter features
reedbase-adapter = ["dep:reedbase", "reedcli/reedbase"]
reedcms-adapter = ["dep:reedcms", "reedcli/reedcms"]
```

## Testing Strategy

### Integration Tests (AFTER adapters ready)

```rust
// tests/cli_integration_test.rs

#[test]
fn test_reedbase_commands_available() {
    let mut cmd = Command::cargo_bin("reed").unwrap();
    
    // ReedBase commands from REED-19
    cmd.arg("query")
        .arg("SELECT * FROM text")
        .assert()
        .success();
}

#[test]
fn test_reedcms_commands_available() {
    let mut cmd = Command::cargo_bin("reed").unwrap();
    
    // ReedCMS commands
    cmd.arg("content:list")
        .assert()
        .success();
}

#[test]
fn test_adapter_isolation() {
    // Build with only ReedBase
    // Verify ReedCMS commands are NOT available
    
    // Build with only ReedCMS  
    // Verify it uses ReedBase correctly
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Binary startup | < 50ms |
| Adapter discovery | < 5ms |
| Command routing | < 1ms |

## Error Conditions

- **Exit code 0**: Success
- **Exit code 1**: User error (bad args, unknown command)
- **Exit code 2**: System error (adapter failure, internal error)

## Acceptance Criteria

**⚠️ These criteria CANNOT be verified until REED-19, REED-18-09, REED-18-10 are complete:**

- [ ] REED-19 (Standalone ReedBase) is complete
- [ ] REED-18-09 (ReedBase Adapter) is complete
- [ ] REED-18-10 (ReedCMS Adapter) is complete
- [ ] Main binary discovers both adapters automatically
- [ ] All commands work correctly
- [ ] `reed --help` shows both adapters
- [ ] `reed --version` shows adapter versions
- [ ] Feature flags allow disabling adapters
- [ ] Error handling with correct exit codes
- [ ] Integration tests pass
- [ ] Performance targets met
- [ ] All code in BBC English

## Dependencies

**CRITICAL BLOCKERS**: 
- **REED-19-01 through REED-19-13** - Standalone ReedBase MUST exist
- **REED-18-09** - ReedBase Adapter MUST be complete
- **REED-18-10** - ReedCMS Adapter MUST be complete

**Also Requires**:
- REED-18-08 (Command Provider Trait)

**Blocks**: Nothing (this is the final step)

## References
- REED-18-08: Command Provider Trait
- REED-18-09: ReedBase Adapter
- REED-18-10: ReedCMS Adapter
- REED-19-00: ReedBase Layer Overview

## Notes

**The Three-Tool Architecture:**

After this ticket is complete, we will have:

1. **ReedCLI** (reedcli/)
   - Presentation layer
   - Parser, formatter, router
   - CommandProvider trait
   - NO business logic

2. **ReedBase** (reedbase/)
   - Standalone database
   - Binary deltas, versioning
   - ReedQL query language
   - Schema validation
   - Can be used independently

3. **ReedCMS** (src/reedcms/)
   - Content management
   - Uses ReedBase as backend
   - Templates, server, assets
   - User/role management

**Dependency Chain**: `CLI → ReedBase → ReedCMS`

Each tool can be:
- Developed independently
- Tested independently
- Released independently
- Used independently (ReedBase can be used without ReedCMS)

**Example Usage After Integration:**

```bash
# ReedBase commands (direct database access)
reed query "SELECT * FROM text WHERE key LIKE 'page.%'"
reed table:list
reed version:restore abc123

# ReedCMS commands (use ReedBase backend)
reed content:create --title "My Page"
reed user:create admin
reed server:start

# Both adapters registered
reed --help
# Shows:
#   reedbase v1.0.0 - Standalone database (25 commands)
#   reedcms v1.0.0 - Content management (30 commands)
```

**Why This is the Last Step:**

Until REED-19 exists, we cannot:
- Test the adapter pattern with real tools
- Verify the three-tool architecture works
- Demonstrate the separation of concerns
- Measure performance impact

**DO NOT implement this until all dependencies are ready!**

## Timeline

**Estimated Start**: After REED-19, REED-18-09, REED-18-10 complete (months from now)  
**Estimated Duration**: 2 days  
**Priority**: Critical (but BLOCKED)

## Implementation Checklist (When Ready)

- [ ] Verify REED-19 has CLI interface
- [ ] Verify REED-18-09 adapter works
- [ ] Verify REED-18-10 adapter works
- [ ] Update Cargo.toml with workspace
- [ ] Create new main.rs with adapter discovery
- [ ] Add integration tests
- [ ] Test all three tools together
- [ ] Verify feature flags work
- [ ] Performance benchmarks
- [ ] Documentation update
