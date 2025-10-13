# Command Migration Status: Old CLI → New ReedCLI

**Last Updated**: 2025-10-13  
**Migration Status**: 0/62 commands migrated to adapter system

---

## Overview

This document tracks the migration of CLI commands from the **old monolithic CLI** (`src/reedcms/cli/`) to the **new adapter-based ReedCLI** system.

### Architecture Comparison

**Old System** (Current - Fully Functional):
```
reed command:action
  ↓
src/main.rs → cli::run()
  ↓
router.rs (hardcoded 62 commands)
  ↓
*_commands.rs modules
  ↓
Direct function calls
```

**New System** (Planned - Adapter-Based):
```
reed command:action
  ↓
reedcli binary
  ↓
Reed.toml registry (dynamic)
  ↓
Adapter binaries (reedbase, reedcms)
  ↓
Subprocess execution with JSON I/O
```

---

## Migration Status Summary

| Category | Old Commands | Migrated | Status |
|----------|--------------|----------|--------|
| Data Commands | 12 | 0 | ❌ Not started |
| User Commands | 7 | 0 | ❌ Not started |
| Role Commands | 6 | 0 | ❌ Not started |
| Taxonomy Commands | 10 | 0 | ❌ Not started |
| Layout Commands | 1 | 0 | ❌ Not started |
| Migration Commands | 2 | 0 | ❌ Not started |
| Validation Commands | 4 | 0 | ❌ Not started |
| Build Commands | 4 | 0 | ❌ Not started |
| Server Commands | 6 | 0 | ❌ Not started |
| Agent Commands | 6 | 0 | ❌ Not started |
| Config Commands | 5 | 0 | ❌ Not started |
| Backup Commands | 4 | 0 | ❌ Not started |
| Debug Commands | 4 | 0 | ❌ Not started |
| **TOTAL** | **62** | **0** | **0%** |

---

## Detailed Command List

### Data Commands (12 commands) - REED-04-02

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `set:text` | data_commands.rs | ❌ Not migrated | reedbase | Set text value with lang |
| `set:route` | data_commands.rs | ❌ Not migrated | reedbase | Set URL routing |
| `set:meta` | data_commands.rs | ❌ Not migrated | reedbase | Set metadata value |
| `set:server` | data_commands.rs | ❌ Not migrated | reedbase | Set server config |
| `set:project` | data_commands.rs | ❌ Not migrated | reedbase | Set project config |
| `get:text` | data_commands.rs | ❌ Not migrated | reedbase | Get text with fallback |
| `get:route` | data_commands.rs | ❌ Not migrated | reedbase | Get route layout |
| `get:meta` | data_commands.rs | ❌ Not migrated | reedbase | Get metadata |
| `list:text` | data_commands.rs | ❌ Not migrated | reedbase | List all text keys |
| `list:route` | data_commands.rs | ❌ Not migrated | reedbase | List all routes |
| `list:meta` | data_commands.rs | ❌ Not migrated | reedbase | List all metadata |
| `text:aggregate` | data_commands.rs | ❌ Not migrated | reedbase | Aggregate component text |

---

### User Commands (7 commands) - REED-04-04

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `user:create` | user_commands.rs | ❌ Not migrated | reedcms | Create user with Argon2 hash |
| `user:list` | user_commands.rs | ❌ Not migrated | reedcms | List all users |
| `user:show` | user_commands.rs | ❌ Not migrated | reedcms | Show user details |
| `user:update` | user_commands.rs | ❌ Not migrated | reedcms | Update user profile |
| `user:delete` | user_commands.rs | ❌ Not migrated | reedcms | Delete user account |
| `user:passwd` | user_commands.rs | ❌ Not migrated | reedcms | Change user password |
| `user:roles` | user_commands.rs | ❌ Not migrated | reedcms | Manage user roles |

---

### Role Commands (6 commands) - REED-04-05

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `role:create` | role_commands.rs | ❌ Not migrated | reedcms | Create role |
| `role:list` | role_commands.rs | ❌ Not migrated | reedcms | List all roles |
| `role:show` | role_commands.rs | ❌ Not migrated | reedcms | Show role details |
| `role:update` | role_commands.rs | ❌ Not migrated | reedcms | Update role |
| `role:delete` | role_commands.rs | ❌ Not migrated | reedcms | Delete role |
| `role:permissions` | role_commands.rs | ❌ Not migrated | reedcms | Manage role permissions |

---

### Taxonomy Commands (10 commands) - REED-04-06

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `taxonomy:create` | taxonomy_commands.rs | ❌ Not migrated | reedcms | Create taxonomy term |
| `taxonomy:list` | taxonomy_commands.rs | ❌ Not migrated | reedcms | List all terms |
| `taxonomy:show` | taxonomy_commands.rs | ❌ Not migrated | reedcms | Show term details |
| `taxonomy:search` | taxonomy_commands.rs | ❌ Not migrated | reedcms | Search terms |
| `taxonomy:update` | taxonomy_commands.rs | ❌ Not migrated | reedcms | Update term |
| `taxonomy:delete` | taxonomy_commands.rs | ❌ Not migrated | reedcms | Delete term |
| `taxonomy:assign` | taxonomy_commands.rs | ❌ Not migrated | reedcms | Assign term to entity |
| `taxonomy:unassign` | taxonomy_commands.rs | ❌ Not migrated | reedcms | Unassign term |
| `taxonomy:entities` | taxonomy_commands.rs | ❌ Not migrated | reedcms | List entities with term |
| `taxonomy:usage` | taxonomy_commands.rs | ❌ Not migrated | reedcms | Show term usage stats |

---

### Layout Commands (1 command) - REED-04-03

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `init:layout` | layout_commands.rs | ❌ Not migrated | reedcms | Initialize layout structure |

---

### Migration Commands (2 commands) - REED-04-07

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `migrate:text` | migration_commands.rs | ❌ Not migrated | reedbase | Migrate text to new namespace |
| `migrate:routes` | migration_commands.rs | ❌ Not migrated | reedbase | Migrate route format |

---

### Validation Commands (4 commands) - REED-04-07

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `validate:routes` | validation_commands.rs | ❌ Not migrated | reedbase | Validate routes.csv |
| `validate:consistency` | validation_commands.rs | ❌ Not migrated | reedbase | Check data consistency |
| `validate:text` | validation_commands.rs | ❌ Not migrated | reedbase | Validate text.csv |
| `validate:references` | validation_commands.rs | ❌ Not migrated | reedbase | Check broken references |

---

### Build Commands (4 commands) - REED-04-08

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `build:kernel` | build_commands.rs | ❌ Not migrated | reedcms | Build core binary |
| `build:public` | build_commands.rs | ❌ Not migrated | reedcms | Build public assets |
| `build:complete` | build_commands.rs | ❌ Not migrated | reedcms | Full system build |
| `build:watch` | build_commands.rs | ❌ Not migrated | reedcms | Watch and auto-rebuild |

---

### Server Commands (6 commands) - REED-04-09

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `server:io` | server_commands.rs | ❌ Not migrated | reedcms | Start I/O server |
| `server:start` | server_commands.rs | ❌ Not migrated | reedcms | Start HTTP server |
| `server:stop` | server_commands.rs | ❌ Not migrated | reedcms | Stop server |
| `server:restart` | server_commands.rs | ❌ Not migrated | reedcms | Restart server |
| `server:status` | server_commands.rs | ❌ Not migrated | reedcms | Server status |
| `server:logs` | server_commands.rs | ❌ Not migrated | reedcms | Show server logs |

---

### Agent Commands (6 commands) - REED-04-10

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `agent:add` | agent_commands.rs | ❌ Not migrated | reedcms | Register AI agent |
| `agent:list` | agent_commands.rs | ❌ Not migrated | reedcms | List agents |
| `agent:show` | agent_commands.rs | ❌ Not migrated | reedcms | Show agent details |
| `agent:test` | agent_commands.rs | ❌ Not migrated | reedcms | Test agent connection |
| `agent:update` | agent_commands.rs | ❌ Not migrated | reedcms | Update agent config |
| `agent:remove` | agent_commands.rs | ❌ Not migrated | reedcms | Remove agent |
| `agent:generate` | agent_commands.rs | ❌ Not migrated | reedcms | Generate content with agent |
| `agent:translate` | agent_commands.rs | ❌ Not migrated | reedcms | Translate content with agent |

**Note**: Agent commands include 2 extra (generate, translate) = 8 total

---

### Config Commands (5 commands) - REED-04-12

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `config:sync` | config_commands.rs | ❌ Not migrated | reedbase | Sync Reed.toml → CSV |
| `config:export` | config_commands.rs | ❌ Not migrated | reedbase | Export CSV → Reed.toml |
| `config:init` | config_commands.rs | ❌ Not migrated | reedbase | Initialize config files |
| `config:show` | config_commands.rs | ❌ Not migrated | reedbase | Show current config |
| `config:validate` | config_commands.rs | ❌ Not migrated | reedbase | Validate Reed.toml |

---

### Backup Commands (4 commands) - REED-10-04

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `backup:list` | backup_commands.rs | ❌ Not migrated | reedbase | List XZ backups |
| `backup:restore` | backup_commands.rs | ❌ Not migrated | reedbase | Restore from backup |
| `backup:verify` | backup_commands.rs | ❌ Not migrated | reedbase | Verify backup integrity |
| `backup:prune` | backup_commands.rs | ❌ Not migrated | reedbase | Clean old backups |

---

### Debug Commands (4 commands) - REED-10-03

| Command | Old Location | New Status | Target Adapter | Notes |
|---------|--------------|------------|----------------|-------|
| `debug:request` | debug_commands.rs | ❌ Not migrated | reedcms | Inspect HTTP request |
| `debug:cache` | debug_commands.rs | ❌ Not migrated | reedbase | Show cache statistics |
| `debug:route` | debug_commands.rs | ❌ Not migrated | reedcms | Test route resolution |
| `debug:config` | debug_commands.rs | ❌ Not migrated | reedbase | Show config with env |

---

## Migration Strategy

### Phase 1: Create Command Registry (Not Started)
- [ ] Create `Reed.toml` with command registry
- [ ] Define `[tools.reedbase]` section
- [ ] Define `[tools.reedcms]` section
- [ ] Register all 62 commands with handlers

### Phase 2: Create Adapter Binaries (Not Started)
- [ ] Extract ReedBase to standalone binary
- [ ] Extract ReedCMS to standalone binary
- [ ] Implement `--list-commands` protocol
- [ ] Implement JSON I/O for subprocess communication

### Phase 3: Migrate Commands (Not Started)
- [ ] Migrate data commands (12) to reedbase adapter
- [ ] Migrate config/backup/debug (13) to reedbase adapter
- [ ] Migrate user/role/taxonomy (23) to reedcms adapter
- [ ] Migrate build/server/agent (14) to reedcms adapter

### Phase 4: Testing (Not Started)
- [ ] Integration tests with real adapters
- [ ] End-to-end command tests
- [ ] Performance benchmarks

### Phase 5: Cutover (Not Started)
- [ ] Switch main binary to reedcli
- [ ] Deprecate old CLI
- [ ] Update documentation

---

## Current System Status

### Old CLI (Fully Functional) ✅
- **Location**: `src/reedcms/cli/`
- **Binary**: `target/release/reed`
- **Commands**: 62 commands fully implemented
- **Status**: Production-ready, all tests passing
- **Registration**: Hardcoded in `router.rs::create_router()`

### New ReedCLI (Presentation Layer Only) ⚠️
- **Location**: `reedcli/`
- **Binary**: Not built yet
- **Commands**: 0 commands available (stub handlers only)
- **Status**: Presentation layer 100% complete, no adapters
- **Registration**: Test-only (no production Reed.toml)

---

## Why Migration Not Started

### Blocker 1: No Command Registry
The current `Reed.toml` only contains project configuration, not command registry:
```toml
# Current Reed.toml (project config only)
[project]
name = "ReedCMS"
url = "https://vvoss.dev"

[server]
workers = 4
```

**Needed**:
```toml
# Required for adapter system
[registry]
version = "1.0.0"

[cli]
name = "reed"
shell_prompt = "reed> "

[tools.reedbase]
binary = "reedbase"
description = "ReedBase database operations"

[tools.reedbase.commands.set-text]
handler = "set_text"
description = "Set text value"
help = "Usage: reed set:text <key> <value> [--lang <lang>]"

[tools.reedcms]
binary = "reedcms"
description = "ReedCMS content management"
# ... commands ...
```

### Blocker 2: No Standalone Adapters
ReedBase and ReedCMS are currently modules, not binaries:
- ReedBase: `src/reedcms/reedbase/` (library)
- ReedCMS: `src/reedcms/` (monolith)

**Needed**:
- Standalone `reedbase` binary with CLI
- Standalone `reedcms` binary with CLI
- JSON I/O protocol implementation
- `--list-commands` support

### Blocker 3: Integration Not Tested
The new ReedCLI has stub handlers:
```rust
// integration.rs (current)
pub fn execute_reedbase_command(...) -> CliResult<CommandOutput> {
    // STUB: Return placeholder JSON
    Ok(CommandOutput {
        data: json!({"result": "stub"}),
        format: OutputFormat::Json,
        exit_code: 0,
    })
}
```

**Needed**: Replace stubs with real subprocess execution

---

## Decision Point

### Option A: Continue with Old CLI (Recommended for Now)
**Pros**:
- ✅ Fully functional (62 commands working)
- ✅ All tests passing
- ✅ Production-ready
- ✅ No migration risk

**Cons**:
- ❌ Monolithic architecture
- ❌ Tight coupling
- ❌ No adapter extensibility

### Option B: Complete Migration to Adapter System
**Pros**:
- ✅ Clean separation of concerns
- ✅ Extensible via Reed.toml
- ✅ Dynamic command registration
- ✅ Better testing (subprocess mocking)

**Cons**:
- ❌ Significant work (create adapters, registry, integration)
- ❌ Migration risk
- ❌ Requires REED-19 (standalone ReedBase)

---

## Recommendation

**Keep old CLI operational** while building adapter system in parallel:

1. **Phase 1** (Current): Old CLI in production, new ReedCLI in development
2. **Phase 2**: Create standalone ReedBase/ReedCMS binaries
3. **Phase 3**: Build command registry
4. **Phase 4**: Migrate commands one category at a time
5. **Phase 5**: Cutover when all 62 commands migrated and tested

**Timeline**: 
- Old CLI: Production-ready NOW
- New adapter system: Estimated 2-3 months of development

---

## Summary

**Current Status**: Old CLI is fully functional with 62 commands. New ReedCLI presentation layer is complete (47 functions, 190 tests) but has **zero commands migrated** because:
1. No command registry exists in Reed.toml
2. No standalone adapter binaries (reedbase, reedcms)
3. Stub handlers not replaced with real integration

**Action Required**: Decide whether to:
- **A)** Keep old CLI and defer adapter system to REED-19
- **B)** Invest 2-3 months in complete migration now

**Recommendation**: Option A - Keep old CLI, migrate during REED-19 when ReedBase becomes standalone.
