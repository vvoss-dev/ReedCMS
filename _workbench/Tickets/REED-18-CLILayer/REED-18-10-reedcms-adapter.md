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
- **Status**: **BLOCKED** - Waiting for REED-19
- **Complexity**: Medium
- **Dependencies**: 
  - REED-18-08 (Command Provider Trait)
  - **REED-19-** (ReedBase - ReedCMS depends on the database)
  - REED-18-09 (ReedBase Adapter)
- **Estimated Time**: 2 days (AFTER REED-19 and REED-18-09 complete)

## ⚠️ IMPORTANT: This Ticket is BLOCKED

**This ticket CANNOT be implemented until:**
1. **REED-19 (Standalone ReedBase) is complete** - ReedCMS uses ReedBase as its database
2. **REED-18-09 (ReedBase Adapter) is complete** - We need to see the pattern

**Dependency Chain**: `ReedCLI → ReedBase → ReedCMS`

ReedCMS commands will interact with the new ReedBase (REED-19), not the old REED-02 system. Until ReedBase is standalone, we cannot properly separate ReedCMS.

## Objective

Create the **ReedCMS Adapter** that implements CommandProvider trait and registers all ReedCMS CLI commands that handle content management operations (using the new ReedBase as backend).

## Requirements

### Commands to Register (Provisional)

**Content Management Commands**:
- `content:create`, `content:list`, `content:update`, `content:delete`
- `content:publish`, `content:unpublish`

**User Commands**:
- `user:create`, `user:list`, `user:show`, `user:update`, `user:delete`
- `user:passwd`, `user:roles`

**Role Commands**:
- `role:create`, `role:list`, `role:show`, `role:update`, `role:delete`
- `role:permissions`

**Taxonomy Commands**:
- `taxonomy:create`, `taxonomy:list`, `taxonomy:show`, `taxonomy:search`
- `taxonomy:update`, `taxonomy:delete`, `taxonomy:assign`, `taxonomy:unassign`

**Layout Commands**:
- `layout:create`, `layout:list`, `layout:update`, `layout:delete`

**Build Commands**:
- `build:assets`, `build:templates`, `build:complete`
- `build:watch`

**Server Commands**:
- `server:start`, `server:stop`, `server:restart`
- `server:status`, `server:logs`

**⚠️ This list is PROVISIONAL and WILL change based on final architecture!**

## Architecture (After REED-19)

```
src/reedcms/
├── lib.rs                      # Public exports
├── adapter.rs                  # CommandProvider impl (THIS TICKET)
├── cli/                        # CLI commands
│   ├── content_commands.rs     # Content management
│   ├── user_commands.rs
│   ├── role_commands.rs
│   ├── taxonomy_commands.rs
│   ├── layout_commands.rs
│   ├── build_commands.rs
│   └── server_commands.rs
└── ... (existing ReedCMS modules)
```

## Implementation Strategy

### Phase 1: Wait for Dependencies (CURRENT)
- **DO NOT implement this ticket yet**
- Wait for REED-19 to be complete
- Wait for REED-18-09 (ReedBase Adapter) to be complete
- Study how ReedCMS will interact with new ReedBase

### Phase 2: Design ReedCMS Commands
- Determine which commands belong to ReedCMS vs ReedBase
- Design command interfaces that use ReedBase as backend
- Update this ticket with actual command list

### Phase 3: Implement Adapter
- Create `src/reedcms/adapter.rs`
- Implement CommandProvider trait
- Register all ReedCMS commands
- Ensure no overlap with ReedBase commands

### Phase 4: Integration
- Test with REED-18-11 (Main Binary Integration)
- Verify ReedCMS + ReedBase work together

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Adapter creation | < 1μs |
| Register N commands | < N×10μs |
| Command execution | No overhead from adapter |

## Acceptance Criteria

**⚠️ These criteria CANNOT be verified until REED-19 is complete:**

- [ ] REED-19 (Standalone ReedBase) is complete
- [ ] REED-18-09 (ReedBase Adapter) is complete
- [ ] ReedCMS commands are defined and separated from ReedBase
- [ ] ReedCMS adapter implements CommandProvider trait
- [ ] All ReedCMS commands registered correctly
- [ ] No command overlap with ReedBase
- [ ] ReedCMS commands use ReedBase as backend
- [ ] All ReedCMS tests pass
- [ ] Adapter tests pass
- [ ] All code in BBC English
- [ ] Separate test file as `adapter_test.rs`

## Dependencies

**CRITICAL BLOCKERS**: 
- **REED-19-** (Standalone ReedBase) - MUST be complete first
- **REED-18-09** (ReedBase Adapter) - MUST be complete first

**Also Requires**:
- REED-18-08 (Command Provider Trait)

**Blocks**: 
- REED-18-11 (Main Binary Integration)

## References
- REED-18-08: Command Provider Trait
- REED-18-09: ReedBase Adapter (similar pattern)
- REED-19-00: ReedBase Layer Overview

## Notes

**WHY this ticket is blocked:**

Currently, ReedCMS contains:
- ReedBase implementation (in `src/reedcms/reedbase/`)
- ReedCMS features (templates, server, etc.)
- All CLI commands mixed together

The goal is:
- **ReedBase**: Standalone database tool
- **ReedCMS**: Content management using ReedBase

**We cannot separate ReedCMS until ReedBase is standalone!**

Example of the dependency:
```rust
// ReedCMS command will use ReedBase:
pub fn content_create(args: &[String]) -> CliResult<...> {
    // This needs the NEW ReedBase, not old REED-02
    reedbase::query("INSERT INTO content ...")?;
    // ...
}
```

**Once REED-19 is done**, come back and:
1. Review which commands belong to ReedCMS
2. Ensure they use ReedBase backend correctly
3. Implement the adapter
4. Test integration

## Timeline

**Estimated Start**: After REED-19 and REED-18-09 complete  
**Estimated Duration**: 2 days  
**Priority**: Critical (but BLOCKED)
