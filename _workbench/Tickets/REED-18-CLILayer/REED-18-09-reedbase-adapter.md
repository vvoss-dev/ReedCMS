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
- **Status**: **BLOCKED** - Waiting for REED-19
- **Complexity**: Medium
- **Dependencies**: 
  - REED-18-08 (Command Provider Trait)
  - **REED-19-01 through REED-19-13 (Standalone ReedBase - MUST be implemented first)**
- **Estimated Time**: 2 days (AFTER REED-19 complete)

## ⚠️ IMPORTANT: This Ticket is BLOCKED

**This ticket CANNOT be implemented until REED-19 (Standalone ReedBase) is complete.**

The ReedBase Adapter registers CLI commands for the **new standalone ReedBase database** with:
- Binary delta versioning (REED-19-03)
- Concurrent writes (REED-19-05)
- ReedQL query language (REED-19-10)
- Schema validation (REED-19-08)

**The old REED-02 ReedBase is NOT relevant for this ticket.** This adapter is for the new revolutionary database system.

## Objective

Create the **ReedBase Adapter** that implements CommandProvider trait and registers all CLI commands provided by the new standalone ReedBase tool (REED-19).

**Note**: The exact command list will be determined by REED-19 implementation. This ticket provides a template structure that must be updated once REED-19 defines its CLI interface.

## Requirements

### Commands to Register (TBD - defined by REED-19)

**The command list depends on REED-19 implementation. Expected categories:**

**Table Operations**:
- `table:create`, `table:list`, `table:show`, `table:delete`
- `table:schema`, `table:validate`

**Data Operations**:
- `query` (ReedQL - REED-19-10)
- `insert`, `update`, `delete`
- `import`, `export`

**Version Operations**:
- `version:list`, `version:show`, `version:restore`
- `version:diff`, `version:log`

**Schema Operations**:
- `schema:apply`, `schema:validate`, `schema:show`

**Backup Operations**:
- `backup:create`, `backup:list`, `backup:restore`
- `backup:verify`, `backup:prune`

**Debug Operations**:
- `debug:cache`, `debug:stats`, `debug:vacuum`

**Performance Operations**:
- `benchmark`, `analyze`, `optimize`

**⚠️ This list is PROVISIONAL and WILL change based on REED-19 implementation!**

## Architecture

```
reedbase/                       # Standalone ReedBase tool (REED-19)
├── src/
│   ├── cli/                    # CLI commands (defined by REED-19)
│   │   ├── table_commands.rs
│   │   ├── query_commands.rs
│   │   ├── version_commands.rs
│   │   ├── schema_commands.rs
│   │   └── ...
│   ├── adapter.rs              # CommandProvider impl (THIS TICKET)
│   ├── lib.rs
│   └── main.rs                 # ReedBase binary
└── Cargo.toml
```

## Implementation Files

### Primary Implementation

**`reedbase/src/adapter.rs`**

This file will be created AFTER REED-19 defines the CLI interface.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! ReedBase adapter for CLI command registration.
//!
//! Registers all ReedBase commands with the CLI router.

use reedcli::{CommandProvider, Router};
use crate::cli;

/// ReedBase adapter.
///
/// Implements CommandProvider to register all ReedBase CLI commands.
pub struct ReedBaseAdapter;

impl ReedBaseAdapter {
    /// Create new ReedBase adapter.
    pub fn new() -> Self {
        Self
    }
}

impl CommandProvider for ReedBaseAdapter {
    fn register_commands(&self, router: &mut Router) {
        // TODO: Update this list based on REED-19 implementation
        
        // Example structure (WILL CHANGE):
        // Table operations
        router.register("table", "create", cli::table_commands::create);
        router.register("table", "list", cli::table_commands::list);
        
        // Query operations
        router.register("query", "", cli::query_commands::execute);
        
        // Version operations
        router.register("version", "list", cli::version_commands::list);
        router.register("version", "restore", cli::version_commands::restore);
        
        // ... (complete list depends on REED-19)
    }
    
    fn name(&self) -> &str {
        "reedbase"
    }
    
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    
    fn description(&self) -> &str {
        "ReedBase standalone database with versioning and concurrent writes"
    }
}

impl Default for ReedBaseAdapter {
    fn default() -> Self {
        Self::new()
    }
}
```

## Implementation Strategy

### Phase 1: Wait for REED-19 (CURRENT)
- **DO NOT implement this ticket yet**
- Wait for REED-19-01 through REED-19-13 to be complete
- Monitor REED-19 for CLI command definitions

### Phase 2: Update Command List
- Once REED-19 defines its CLI interface, update this ticket
- List all actual commands that ReedBase provides
- Update `register_commands()` implementation

### Phase 3: Implement Adapter
- Create `reedbase/src/adapter.rs`
- Implement CommandProvider trait
- Register all ReedBase commands
- Add tests

### Phase 4: Integration
- Ensure reedbase crate exports adapter
- Verify with REED-18-11 (Main Binary Integration)

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Adapter creation | < 1μs |
| Register N commands | < N×10μs |
| Command execution | No overhead from adapter |

## Error Conditions

None - adapter creation and registration are infallible.

## Acceptance Criteria

**⚠️ These criteria CANNOT be verified until REED-19 is complete:**

- [ ] REED-19-01 through REED-19-13 are complete
- [ ] ReedBase CLI commands are defined
- [ ] ReedBase adapter implements CommandProvider trait
- [ ] All ReedBase commands registered correctly
- [ ] Command signatures match CommandHandler type
- [ ] All ReedBase tests still pass
- [ ] Adapter tests pass
- [ ] All code in BBC English
- [ ] All functions have complete documentation
- [ ] Separate test file as `adapter_test.rs`

## Dependencies

**CRITICAL BLOCKERS**: 
- **REED-19-01**: Registry & Dictionary - MUST be complete first
- **REED-19-02**: Universal Table API - MUST be complete first
- **REED-19-03**: Binary Delta Versioning - MUST be complete first
- **REED-19-04**: Encoded Log System - MUST be complete first
- **REED-19-05**: Concurrent Write System - MUST be complete first
- **REED-19-06**: Row-Level CSV Merge - MUST be complete first
- **REED-19-07**: Conflict Resolution - MUST be complete first
- **REED-19-08**: Schema Validation - MUST be complete first
- **REED-19-09**: Function System & Caching - MUST be complete first
- **REED-19-10**: CLI SQL Query Interface (ReedQL) - MUST be complete first
- **REED-19-11**: Migration from REED-02 - MUST be complete first
- **REED-19-12**: Performance Testing - MUST be complete first
- **REED-19-13**: Documentation - MUST be complete first

**Also Requires**:
- REED-18-08 (Command Provider Trait)

**Blocks**: 
- REED-18-11 (Main Binary Integration)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- REED-18-08: Command Provider Trait
- REED-19-00: ReedBase Layer Overview

## Notes

**WHY this ticket is blocked:**

The old REED-02 ReedBase is a simple CSV-based system integrated into ReedCMS with basic commands like:
- `set:text`, `get:text`, `list:text`
- `config:sync`, `backup:list`

The new REED-19 ReedBase is a **revolutionary standalone database** with:
- Binary delta versioning
- Concurrent writes with conflict resolution
- ReedQL query language
- Schema validation
- Function system with caching

**We cannot write an adapter for something that doesn't exist yet!**

The command set will be completely different. For example:
- Old: `set:text page.title "Welcome"`
- New: `query "UPDATE text SET value='Welcome' WHERE key='page.title'"`

**Once REED-19 is implemented**, come back to this ticket and:
1. Review the actual CLI commands ReedBase provides
2. Update the command list in this ticket
3. Implement the adapter
4. Test integration

**DO NOT try to adapt the old REED-02 system** - that defeats the entire purpose of REED-19!

## Timeline

**Estimated Start**: After REED-19-13 complete (weeks/months from now)  
**Estimated Duration**: 2 days (once REED-19 is done)  
**Priority**: Critical (but BLOCKED)
