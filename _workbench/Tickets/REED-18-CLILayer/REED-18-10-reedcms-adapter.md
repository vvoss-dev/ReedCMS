# REED-18-10: ReedCMS Adapter

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
- **ID**: REED-18-10
- **Title**: ReedCMS Adapter
- **Layer**: CLI Layer (REED-18)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-18-08 (Command Provider Trait), existing CLI commands
- **Estimated Time**: 4 days

## Objective

Create the **ReedCMS Adapter** that implements CommandProvider trait and registers all 37 ReedCMS CLI commands (user, role, taxonomy, layout, build, server, agent, debug).

## Requirements

### Commands to Register (37 total)

**User Commands (7)**:
- `user:create`, `user:list`, `user:show`, `user:update`, `user:delete`, `user:passwd`, `user:roles`

**Role Commands (6)**:
- `role:create`, `role:list`, `role:show`, `role:update`, `role:delete`, `role:permissions`

**Taxonomy Commands (10)**:
- `taxonomy:create`, `taxonomy:list`, `taxonomy:show`, `taxonomy:search`
- `taxonomy:update`, `taxonomy:delete`, `taxonomy:assign`, `taxonomy:unassign`
- `taxonomy:entities`, `taxonomy:usage`

**Layout Commands (1)**:
- `init:layout`

**Build Commands (4)**:
- `build:kernel`, `build:public`, `build:complete`, `build:watch`

**Server Commands (6)**:
- `server:io`, `server:start`, `server:stop`, `server:restart`, `server:status`, `server:logs`

**Agent Commands (8)**:
- `agent:add`, `agent:list`, `agent:show`, `agent:test`
- `agent:update`, `agent:remove`, `agent:generate`, `agent:translate`

**Debug Commands (2)**:
- `debug:request`, `debug:route`

### Architecture

```
src/reedcms/
├── lib.rs                      # Public exports
├── adapter.rs                  # CommandProvider impl (NEW)
└── cli/                        # Existing commands (KEEP HERE)
    ├── mod.rs              
    ├── user_commands.rs
    ├── role_commands.rs
    ├── taxonomy_commands.rs
    ├── layout_commands.rs
    ├── build_commands.rs
    ├── server_commands.rs
    ├── agent_commands.rs
    └── debug_commands.rs
```

## Implementation Files

### Primary Implementation

**`src/reedcms/adapter.rs`** (NEW)

One file = Adapter implementation only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedCMS adapter for CLI command registration.
//!
//! Registers all ReedCMS commands (user, role, taxonomy, layout, build, server, agent, debug)
//! with the CLI router.

use reedcli::{CommandProvider, Router};
use crate::cli;

/// ReedCMS adapter.
///
/// Implements CommandProvider to register all 37 ReedCMS CLI commands.
pub struct ReedCMSAdapter;

impl ReedCMSAdapter {
    /// Create new ReedCMS adapter.
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
    /// let adapter = ReedCMSAdapter::new();
    /// ```
    pub fn new() -> Self {
        Self
    }
}

impl CommandProvider for ReedCMSAdapter {
    fn register_commands(&self, router: &mut Router) {
        // User commands (7)
        router.register("user", "create", cli::user_commands::create_user);
        router.register("user", "list", cli::user_commands::list_users);
        router.register("user", "show", cli::user_commands::show_user);
        router.register("user", "update", cli::user_commands::update_user);
        router.register("user", "delete", cli::user_commands::delete_user);
        router.register("user", "passwd", cli::user_commands::change_password);
        router.register("user", "roles", cli::user_commands::manage_roles);
        
        // Role commands (6)
        router.register("role", "create", cli::role_commands::create_role);
        router.register("role", "list", cli::role_commands::list_roles);
        router.register("role", "show", cli::role_commands::show_role);
        router.register("role", "update", cli::role_commands::update_role);
        router.register("role", "delete", cli::role_commands::delete_role);
        router.register("role", "permissions", cli::role_commands::manage_permissions);
        
        // Taxonomy commands (10)
        router.register("taxonomy", "create", cli::taxonomy_commands::create);
        router.register("taxonomy", "list", cli::taxonomy_commands::list);
        router.register("taxonomy", "show", cli::taxonomy_commands::show);
        router.register("taxonomy", "search", cli::taxonomy_commands::search);
        router.register("taxonomy", "update", cli::taxonomy_commands::update);
        router.register("taxonomy", "delete", cli::taxonomy_commands::delete);
        router.register("taxonomy", "assign", cli::taxonomy_commands::assign);
        router.register("taxonomy", "unassign", cli::taxonomy_commands::unassign);
        router.register("taxonomy", "entities", cli::taxonomy_commands::entities);
        router.register("taxonomy", "usage", cli::taxonomy_commands::usage);
        
        // Layout commands (1)
        router.register("init", "layout", cli::layout_commands::init_layout);
        
        // Build commands (4)
        router.register("build", "kernel", cli::build_commands::build_kernel);
        router.register("build", "public", cli::build_commands::build_public);
        router.register("build", "complete", cli::build_commands::build_complete);
        router.register("build", "watch", cli::build_commands::build_watch);
        
        // Server commands (6)
        router.register("server", "io", cli::server_commands::server_io);
        router.register("server", "start", cli::server_commands::server_start);
        router.register("server", "stop", cli::server_commands::server_stop);
        router.register("server", "restart", cli::server_commands::server_restart);
        router.register("server", "status", cli::server_commands::server_status);
        router.register("server", "logs", cli::server_commands::server_logs);
        
        // Agent commands (8)
        router.register("agent", "add", cli::agent_commands::add);
        router.register("agent", "list", cli::agent_commands::list);
        router.register("agent", "show", cli::agent_commands::show);
        router.register("agent", "test", cli::agent_commands::test);
        router.register("agent", "update", cli::agent_commands::update);
        router.register("agent", "remove", cli::agent_commands::remove);
        router.register("agent", "generate", cli::agent_commands::generate);
        router.register("agent", "translate", cli::agent_commands::translate);
        
        // Debug commands (2)
        router.register("debug", "request", cli::debug_commands::debug_request_handler);
        router.register("debug", "route", cli::debug_commands::debug_route_handler);
    }
    
    fn name(&self) -> &str {
        "reedcms"
    }
    
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    
    fn description(&self) -> &str {
        "ReedCMS content management operations"
    }
}

impl Default for ReedCMSAdapter {
    fn default() -> Self {
        Self::new()
    }
}
```

**`src/reedcms/lib.rs`** (modifications)

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedCMS library.

pub mod adapter;  // NEW
pub mod cli;
// ... existing modules ...

// Re-export adapter for convenience
pub use adapter::ReedCMSAdapter;
```

**`Cargo.toml`** (additions)

```toml
[dependencies]
# Existing dependencies
# ... (actix-web, minijinja, etc.)

# NEW: Add reedcli for CommandProvider trait
reedcli = { path = "../reedcli" }
```

### Test Files

**`src/reedcms/adapter_test.rs`**

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use reedcli::Router;
    
    #[test]
    fn test_adapter_creation() {
        let adapter = ReedCMSAdapter::new();
        assert_eq!(adapter.name(), "reedcms");
        assert!(!adapter.version().is_empty());
        assert_eq!(adapter.description(), "ReedCMS content management operations");
    }
    
    #[test]
    fn test_adapter_registers_37_commands() {
        let mut router = Router::new();
        let adapter = ReedCMSAdapter::new();
        
        let initial_count = router.command_count();
        adapter.register_commands(&mut router);
        let final_count = router.command_count();
        
        assert_eq!(final_count - initial_count, 37, "Should register exactly 37 commands");
    }
    
    #[test]
    fn test_user_commands_registered() {
        let mut router = Router::new();
        let adapter = ReedCMSAdapter::new();
        adapter.register_commands(&mut router);
        
        assert!(router.has_command("user", "create"));
        assert!(router.has_command("user", "list"));
        assert!(router.has_command("user", "show"));
        assert!(router.has_command("user", "update"));
        assert!(router.has_command("user", "delete"));
        assert!(router.has_command("user", "passwd"));
        assert!(router.has_command("user", "roles"));
    }
    
    #[test]
    fn test_role_commands_registered() {
        let mut router = Router::new();
        let adapter = ReedCMSAdapter::new();
        adapter.register_commands(&mut router);
        
        assert!(router.has_command("role", "create"));
        assert!(router.has_command("role", "list"));
        assert!(router.has_command("role", "show"));
        assert!(router.has_command("role", "update"));
        assert!(router.has_command("role", "delete"));
        assert!(router.has_command("role", "permissions"));
    }
    
    #[test]
    fn test_taxonomy_commands_registered() {
        let mut router = Router::new();
        let adapter = ReedCMSAdapter::new();
        adapter.register_commands(&mut router);
        
        assert!(router.has_command("taxonomy", "create"));
        assert!(router.has_command("taxonomy", "list"));
        assert!(router.has_command("taxonomy", "assign"));
        assert!(router.has_command("taxonomy", "usage"));
    }
    
    #[test]
    fn test_build_commands_registered() {
        let mut router = Router::new();
        let adapter = ReedCMSAdapter::new();
        adapter.register_commands(&mut router);
        
        assert!(router.has_command("build", "kernel"));
        assert!(router.has_command("build", "public"));
        assert!(router.has_command("build", "complete"));
        assert!(router.has_command("build", "watch"));
    }
    
    #[test]
    fn test_server_commands_registered() {
        let mut router = Router::new();
        let adapter = ReedCMSAdapter::new();
        adapter.register_commands(&mut router);
        
        assert!(router.has_command("server", "io"));
        assert!(router.has_command("server", "start"));
        assert!(router.has_command("server", "stop"));
        assert!(router.has_command("server", "restart"));
        assert!(router.has_command("server", "status"));
        assert!(router.has_command("server", "logs"));
    }
    
    #[test]
    fn test_agent_commands_registered() {
        let mut router = Router::new();
        let adapter = ReedCMSAdapter::new();
        adapter.register_commands(&mut router);
        
        assert!(router.has_command("agent", "add"));
        assert!(router.has_command("agent", "list"));
        assert!(router.has_command("agent", "generate"));
        assert!(router.has_command("agent", "translate"));
    }
    
    #[test]
    fn test_no_overlap_with_reedbase() {
        // Verify ReedCMS doesn't register commands that belong to ReedBase
        let mut router = Router::new();
        let adapter = ReedCMSAdapter::new();
        adapter.register_commands(&mut router);
        
        // These should NOT be registered (they belong to ReedBase)
        assert!(!router.has_command("set", "text"));
        assert!(!router.has_command("get", "text"));
        assert!(!router.has_command("config", "sync"));
        assert!(!router.has_command("backup", "list"));
    }
}
```

## Migration Strategy

### Phase 1: Setup
1. Create `src/reedcms/adapter.rs`
2. Add reedcli dependency to Cargo.toml
3. Export adapter in lib.rs

### Phase 2: Adapt Command Signatures (if needed)
- Verify all commands match `CommandHandler` signature
- Ensure all functions accept `&[String]` and `&HashMap<String, String>`
- Update return types to match `CliResult<ReedResponse<String>>`

### Phase 3: Testing
- Run all existing command tests
- Add adapter registration tests
- Verify all 37 commands callable
- Verify no overlap with ReedBase commands

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Adapter creation | < 1μs |
| Register 37 commands | < 1ms |
| Command execution | Same as original (no overhead) |

## Error Conditions

None - adapter creation and registration are infallible.

## Acceptance Criteria

- [ ] ReedCMS adapter implements CommandProvider trait
- [ ] All 37 commands registered correctly
- [ ] Commands remain in `src/reedcms/cli/` (no move needed)
- [ ] Command signatures match CommandHandler type
- [ ] All existing command tests still pass
- [ ] 9 new adapter tests pass
- [ ] No overlap with ReedBase commands
- [ ] All code in BBC English
- [ ] All functions have complete documentation
- [ ] Separate test file as `adapter_test.rs`
- [ ] Cargo.toml has reedcli dependency

## Dependencies

**Requires**: 
- REED-18-08 (Command Provider Trait)
- Existing CLI commands in `src/reedcms/cli/`

**Blocks**: 
- REED-18-11 (Main Binary Integration)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- REED-18-08: Command Provider Trait
- REED-18-09: ReedBase Adapter (similar pattern)

## Notes

**Design Decisions**:
- Adapter in `src/reedcms/` (not separate crate - ReedCMS is already a crate)
- Keep existing CLI commands in place (no move needed)
- All 37 commands in one adapter (not split by category)

**Command Distribution**:
- ReedBase: 25 commands (data-focused)
- ReedCMS: 37 commands (content/server-focused)
- Total: 62 commands

**NO overlap** - each command belongs to exactly one adapter:
```
ReedBase Commands:
- Data: set:*, get:*, list:*, text:aggregate
- Config: config:*
- Backup: backup:*
- Debug: debug:cache, debug:config
- Migration: migrate:*
- Validation: validate:*

ReedCMS Commands:
- Security: user:*, role:*
- Content: taxonomy:*, init:layout
- System: build:*, server:*, agent:*
- Debug: debug:request, debug:route
```

**NOT in scope**:
- Rewriting command implementations
- Changing command behaviour
- New commands (only existing 37)
- Moving CLI files (they stay in src/reedcms/cli/)
