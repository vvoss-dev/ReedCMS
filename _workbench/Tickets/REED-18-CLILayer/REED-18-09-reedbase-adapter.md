# REED-18-09: ReedBase Adapter

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
- **ID**: REED-18-09
- **Title**: ReedBase Adapter
- **Layer**: CLI Layer (REED-18)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-18-08 (Command Provider Trait), REED-02-01 (ReedBase Core)
- **Estimated Time**: 4 days

## Objective

Create the **ReedBase Adapter** that implements CommandProvider trait and registers all 25 ReedBase CLI commands (data, config, backup, debug, migration, validation).

## Requirements

### Commands to Register (25 total)

**Data Commands (12)**:
- `set:text`, `set:route`, `set:meta`, `set:server`, `set:project`
- `get:text`, `get:route`, `get:meta`
- `list:text`, `list:route`, `list:meta`
- `text:aggregate`

**Config Commands (5)**:
- `config:sync`, `config:export`, `config:init`, `config:show`, `config:validate`

**Backup Commands (4)**:
- `backup:list`, `backup:restore`, `backup:verify`, `backup:prune`

**Debug Commands (2)**:
- `debug:cache`, `debug:config`

**Migration Commands (2)**:
- `migrate:text`, `migrate:routes`

**Validation Commands (4)**:
- `validate:routes`, `validate:consistency`, `validate:text`, `validate:references`

### Architecture

```
reedbase/
├── src/
│   ├── lib.rs                  # Public exports
│   ├── adapter.rs              # CommandProvider impl (NEW)
│   └── commands/               # Existing commands (MOVE HERE)
│       ├── mod.rs              
│       ├── data.rs             # Data commands
│       ├── config.rs           # Config commands
│       ├── backup.rs           # Backup commands
│       ├── debug.rs            # Debug commands
│       ├── migration.rs        # Migration commands
│       └── validation.rs       # Validation commands
└── Cargo.toml                  # Add reedcli dependency
```

## Implementation Files

### Primary Implementation

**`reedbase/src/adapter.rs`**

One file = Adapter implementation only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase adapter for CLI command registration.
//!
//! Registers all ReedBase commands (data, config, backup, debug, migration, validation)
//! with the CLI router.

use reedcli::{CommandProvider, Router};
use crate::commands;

/// ReedBase adapter.
///
/// Implements CommandProvider to register all 25 ReedBase CLI commands.
pub struct ReedBaseAdapter;

impl ReedBaseAdapter {
    /// Create new ReedBase adapter.
    ///
    /// ## Output
    /// - `Self`: New adapter instance
    ///
    /// ## Performance
    /// - O(1) operation
    /// - < 1μs typical
    ///
    /// ## Example Usage
    /// ```rust
    /// let adapter = ReedBaseAdapter::new();
    /// ```
    pub fn new() -> Self {
        Self
    }
}

impl CommandProvider for ReedBaseAdapter {
    fn register_commands(&self, router: &mut Router) {
        // Data commands (12)
        router.register("set", "text", commands::data::set_text);
        router.register("set", "route", commands::data::set_route);
        router.register("set", "meta", commands::data::set_meta);
        router.register("set", "server", commands::data::set_server);
        router.register("set", "project", commands::data::set_project);
        
        router.register("get", "text", commands::data::get_text);
        router.register("get", "route", commands::data::get_route);
        router.register("get", "meta", commands::data::get_meta);
        
        router.register("list", "text", commands::data::list_text);
        router.register("list", "route", commands::data::list_route);
        router.register("list", "meta", commands::data::list_meta);
        
        router.register("text", "aggregate", commands::data::aggregate_text);
        
        // Config commands (5)
        router.register("config", "sync", commands::config::config_sync);
        router.register("config", "export", commands::config::config_export);
        router.register("config", "init", commands::config::config_init);
        router.register("config", "show", commands::config::config_show);
        router.register("config", "validate", commands::config::config_validate);
        
        // Backup commands (4)
        router.register("backup", "list", commands::backup::backup_list);
        router.register("backup", "restore", commands::backup::backup_restore);
        router.register("backup", "verify", commands::backup::backup_verify);
        router.register("backup", "prune", commands::backup::backup_prune);
        
        // Debug commands (2)
        router.register("debug", "cache", commands::debug::debug_cache);
        router.register("debug", "config", commands::debug::debug_config);
        
        // Migration commands (2)
        router.register("migrate", "text", commands::migration::migrate_text);
        router.register("migrate", "routes", commands::migration::migrate_routes);
        
        // Validation commands (4)
        router.register("validate", "routes", commands::validation::validate_routes);
        router.register("validate", "consistency", commands::validation::validate_consistency);
        router.register("validate", "text", commands::validation::validate_text);
        router.register("validate", "references", commands::validation::validate_references);
    }
    
    fn name(&self) -> &str {
        "reedbase"
    }
    
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    
    fn description(&self) -> &str {
        "ReedBase database operations"
    }
}

impl Default for ReedBaseAdapter {
    fn default() -> Self {
        Self::new()
    }
}
```

**`reedbase/src/lib.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase library.
//!
//! Provides data access layer and CLI adapter.

pub mod adapter;
pub mod commands;

// Re-export adapter for convenience
pub use adapter::ReedBaseAdapter;
```

**`reedbase/src/commands/mod.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase CLI commands.

pub mod data;
pub mod config;
pub mod backup;
pub mod debug;
pub mod migration;
pub mod validation;
```

**`reedbase/src/commands/data.rs`**

Move existing commands from `src/reedcms/cli/data_commands.rs`:

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Data commands for ReedBase.
//!
//! Handles set, get, list, and aggregate operations.

use reedcli::types::{CliResult, ReedResponse};
use std::collections::HashMap;

/// Set text value.
///
/// ## Input
/// - `args`: [key, value, optional_lang]
/// - `flags`: Optional --lang flag
///
/// ## Output
/// - `ReedResponse<String>`: Success message
///
/// ## Example Usage
/// ```bash
/// reed set:text page.title "Welcome" --lang en
/// ```
pub fn set_text(args: &[String], flags: &HashMap<String, String>) -> CliResult<ReedResponse<String>> {
    // Move existing implementation from src/reedcms/cli/data_commands.rs
    todo!("Move existing set_text implementation")
}

// ... repeat for all 12 data commands
```

**`reedbase/Cargo.toml`** (additions)

```toml
[dependencies]
# Existing dependencies
# ... (csv, serde, etc.)

# NEW: Add reedcli for CommandProvider trait
reedcli = { path = "../reedcli" }
```

### Test Files

**`reedbase/src/adapter_test.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use reedcli::Router;
    
    #[test]
    fn test_adapter_creation() {
        let adapter = ReedBaseAdapter::new();
        assert_eq!(adapter.name(), "reedbase");
        assert!(!adapter.version().is_empty());
        assert_eq!(adapter.description(), "ReedBase database operations");
    }
    
    #[test]
    fn test_adapter_registers_25_commands() {
        let mut router = Router::new();
        let adapter = ReedBaseAdapter::new();
        
        let initial_count = router.command_count();
        adapter.register_commands(&mut router);
        let final_count = router.command_count();
        
        assert_eq!(final_count - initial_count, 25, "Should register exactly 25 commands");
    }
    
    #[test]
    fn test_data_commands_registered() {
        let mut router = Router::new();
        let adapter = ReedBaseAdapter::new();
        adapter.register_commands(&mut router);
        
        // Verify data commands
        assert!(router.has_command("set", "text"));
        assert!(router.has_command("get", "text"));
        assert!(router.has_command("list", "text"));
        assert!(router.has_command("text", "aggregate"));
    }
    
    #[test]
    fn test_config_commands_registered() {
        let mut router = Router::new();
        let adapter = ReedBaseAdapter::new();
        adapter.register_commands(&mut router);
        
        // Verify config commands
        assert!(router.has_command("config", "sync"));
        assert!(router.has_command("config", "export"));
        assert!(router.has_command("config", "init"));
        assert!(router.has_command("config", "show"));
        assert!(router.has_command("config", "validate"));
    }
    
    #[test]
    fn test_backup_commands_registered() {
        let mut router = Router::new();
        let adapter = ReedBaseAdapter::new();
        adapter.register_commands(&mut router);
        
        // Verify backup commands
        assert!(router.has_command("backup", "list"));
        assert!(router.has_command("backup", "restore"));
        assert!(router.has_command("backup", "verify"));
        assert!(router.has_command("backup", "prune"));
    }
    
    #[test]
    fn test_migration_commands_registered() {
        let mut router = Router::new();
        let adapter = ReedBaseAdapter::new();
        adapter.register_commands(&mut router);
        
        // Verify migration commands
        assert!(router.has_command("migrate", "text"));
        assert!(router.has_command("migrate", "routes"));
    }
    
    #[test]
    fn test_validation_commands_registered() {
        let mut router = Router::new();
        let adapter = ReedBaseAdapter::new();
        adapter.register_commands(&mut router);
        
        // Verify validation commands
        assert!(router.has_command("validate", "routes"));
        assert!(router.has_command("validate", "consistency"));
        assert!(router.has_command("validate", "text"));
        assert!(router.has_command("validate", "references"));
    }
}
```

## Migration Strategy

### Phase 1: Setup Structure
1. Create `reedbase/` directory as separate crate
2. Create `src/adapter.rs`
3. Create `src/commands/` directory structure
4. Add reedcli dependency to Cargo.toml

### Phase 2: Move Commands
1. Copy `src/reedcms/cli/data_commands.rs` → `reedbase/src/commands/data.rs`
2. Copy `src/reedcms/cli/config_commands.rs` → `reedbase/src/commands/config.rs`
3. Copy `src/reedcms/cli/backup_commands.rs` → `reedbase/src/commands/backup.rs`
4. Copy `src/reedcms/cli/debug_commands.rs` (cache, config only) → `reedbase/src/commands/debug.rs`
5. Copy `src/reedcms/cli/migration_commands.rs` → `reedbase/src/commands/migration.rs`
6. Copy `src/reedcms/cli/validation_commands.rs` → `reedbase/src/commands/validation.rs`

### Phase 3: Adapt Signatures
- Change return types to match `CommandHandler` signature
- Ensure all functions accept `&[String]` and `&HashMap<String, String>`
- Update imports to use `reedcli::types::{CliResult, ReedResponse}`

### Phase 4: Testing
- Run all existing command tests
- Add adapter registration tests
- Verify all 25 commands callable

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Adapter creation | < 1μs |
| Register 25 commands | < 1ms |
| Command execution | Same as original (no overhead) |

## Error Conditions

None - adapter creation and registration are infallible.

## Acceptance Criteria

- [ ] ReedBase adapter implements CommandProvider trait
- [ ] All 25 commands registered correctly
- [ ] Commands moved to `reedbase/src/commands/` structure
- [ ] Command signatures match CommandHandler type
- [ ] All existing command tests still pass
- [ ] 7 new adapter tests pass
- [ ] No duplicate commands with ReedCMS adapter
- [ ] All code in BBC English
- [ ] All functions have complete documentation
- [ ] Separate test file as `adapter_test.rs`
- [ ] Cargo.toml has reedcli dependency

## Dependencies

**Requires**: 
- REED-18-08 (Command Provider Trait)
- REED-02-01 (ReedBase Core - existing commands)

**Blocks**: 
- REED-18-11 (Main Binary Integration)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- REED-18-08: Command Provider Trait
- REED-02-01: ReedBase Core Services

## Notes

**Design Decisions**:
- Separate `reedbase/` crate for clean separation
- Keep existing command implementations (just move, don't rewrite)
- All 25 commands in one adapter (not split by category)

**File Organization**:
- `adapter.rs`: CommandProvider implementation ONLY
- `commands/`: Command implementations (moved from cli/)
- NO business logic in adapter (just registration)

**Command Mapping**:
```
Old Location                    → New Location
src/reedcms/cli/data_commands   → reedbase/commands/data
src/reedcms/cli/config_commands → reedbase/commands/config
src/reedcms/cli/backup_commands → reedbase/commands/backup
```

**NOT in scope**:
- Rewriting command implementations
- Changing command behaviour
- New commands (only existing 25)
