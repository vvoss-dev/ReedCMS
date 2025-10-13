# REED-18-00: CLI Layer Overview

**Layer**: CLI Layer  
**Status**: Planning  
**Dependencies**: None  
**Estimated Effort**: 2-3 weeks  

---

## Overview

ReedCLI is the presentation layer for the Reed ecosystem. It provides a unified command-line interface that dynamically routes commands to appropriate tools (ReedBase, ReedCMS, etc.) based on the `Reed.toml` registry.

**Key Principles:**
- **Extensibility**: Tools can register commands via `Reed.toml` without modifying CLI code
- **Unified Interface**: Single `reed` binary for all Reed tools
- **Discoverability**: Auto-generated help from registry metadata
- **Consistency**: Standardised output formatting across all commands
- **Interactivity**: Optional shell mode with history and completion

---

## Motivation

### Problem
Previous architecture required modifying CLI code to add new commands, creating tight coupling between presentation layer and business logic.

### Solution
Registry-based command routing via `Reed.toml`:
```toml
[tools.reedbase.commands]
query = { handler = "execute_query", help = "Execute SQL query" }
tables = { handler = "list_tables", help = "List all tables" }

[tools.reedcms.commands]
"server:start" = { handler = "server_start", help = "Start web server" }
```

ReedCLI loads this registry at startup and routes commands dynamically.

---

## Architecture

### 3-Tier Separation

```
┌─────────────────────────────────────────────────────────────┐
│ ReedCLI (Presentation Layer)                                │
│ - Command parsing                                           │
│ - Registry loading (Reed.toml)                              │
│ - Output formatting                                         │
│ - Help generation                                           │
│ - Interactive shell                                         │
├─────────────────────────────────────────────────────────────┤
│ ReedBase / ReedCMS (Business Logic)                         │
│ - Database operations (ReedBase)                            │
│ - Web server (ReedCMS)                                      │
│ - Business rules                                            │
├─────────────────────────────────────────────────────────────┤
│ .reed/ (Data Layer)                                         │
│ - CSV tables                                                │
│ - Binary deltas                                             │
│ - Version logs                                              │
└─────────────────────────────────────────────────────────────┘
```

### Command Flow

```
User Types:
$ reed query "SELECT * FROM users"

1. Parser extracts:
   - Tool: "reedbase" (inferred from command)
   - Command: "query"
   - Args: ["SELECT * FROM users"]

2. Registry Lookup:
   - Load Reed.toml
   - Find tools.reedbase.commands.query
   - Get handler: "execute_query"

3. Tool Invocation:
   - Call reedbase::execute_query(args)
   - Receive ReedResponse<QueryResult>

4. Output Formatting:
   - Format as table/json/csv (based on --format flag)
   - Apply colour coding (if terminal supports)
   - Print to stdout

5. Exit Code:
   - 0 = success
   - 1 = user error (bad args, etc.)
   - 2 = system error (database issue, etc.)
```

---

## File Structure

```
reedcli/
├── Cargo.toml
├── src/
│   ├── main.rs                    # Binary entry point
│   ├── lib.rs                     # Public API
│   ├── parser.rs                  # Command parsing (REED-18-01)
│   ├── parser.test.rs
│   ├── registry.rs                # Reed.toml loader (REED-18-02)
│   ├── registry.test.rs
│   ├── formatter.rs               # Output formatting (REED-18-03)
│   ├── formatter.test.rs
│   ├── shell.rs                   # Interactive shell (REED-18-04)
│   ├── shell.test.rs
│   ├── help.rs                    # Help generation (REED-18-05)
│   ├── help.test.rs
│   ├── integration.rs             # Tool invocation (REED-18-06)
│   ├── integration.test.rs
│   └── types.rs                   # Shared types
└── tests/
    └── integration/               # End-to-end tests
        ├── command_parsing.rs
        ├── registry_loading.rs
        └── output_formatting.rs
```

---

## Key Types

```rust
/// Parsed command from user input
pub struct Command {
    pub tool: String,              // "reedbase", "reedcms", etc.
    pub command: String,           // "query", "server:start", etc.
    pub args: Vec<String>,         // Positional arguments
    pub flags: HashMap<String, String>, // Named flags
}

/// Tool registry loaded from Reed.toml
pub struct Registry {
    pub version: String,
    pub tools: HashMap<String, Tool>,
}

pub struct Tool {
    pub name: String,
    pub binary: Option<String>,    // Override binary path
    pub dependencies: Vec<String>,
    pub commands: HashMap<String, CommandSpec>,
}

pub struct CommandSpec {
    pub handler: String,           // Function name
    pub help: String,
    pub args: Vec<ArgSpec>,
    pub flags: Vec<FlagSpec>,
}

/// Output format options
pub enum OutputFormat {
    Table,      // ASCII table (default for terminal)
    Json,       // JSON output
    Csv,        // CSV output
    Plain,      // Plain text
}

/// Result from tool invocation
pub struct CommandOutput {
    pub data: serde_json::Value,   // Flexible data structure
    pub format: OutputFormat,
    pub exit_code: i32,
}
```

---

## Reed.toml Format

```toml
[registry]
version = "1.0"

[cli]
name = "reedcli"
binary = "reed"
shell_prompt = "reed> "
history_file = ".reed_history"

[tools.reedbase]
name = "reedbase"
description = "CSV-based versioned database"

[tools.reedbase.commands]
query = { handler = "execute_query", help = "Execute SQL query", args = [{ name = "sql", type = "string" }] }
tables = { handler = "list_tables", help = "List all tables" }
versions = { handler = "list_versions", help = "List table versions", args = [{ name = "table", type = "string" }] }
rollback = { handler = "rollback_table", help = "Rollback to version", args = [{ name = "table" }, { name = "version" }] }

[tools.reedcms]
name = "reedcms"
description = "Content management system"
dependencies = ["reedbase"]

[tools.reedcms.commands]
"server:start" = { handler = "server_start", help = "Start web server", flags = [{ name = "port", type = "u16", default = 8333 }] }
"server:stop" = { handler = "server_stop", help = "Stop web server" }
"build:watch" = { handler = "build_watch", help = "Watch and rebuild assets" }
```

---

## Implementation Tickets

### REED-18-01: Command Parser
Parse user input into structured `Command` type.
- Handle quoted arguments
- Parse flags (`--format json`, `-f json`)
- Infer tool from command name
- Error messages for malformed input

### REED-18-02: Registry Loader
Load and validate `Reed.toml`.
- TOML parsing
- Schema validation
- Command lookups (O(1) HashMap)
- Dependency resolution

### REED-18-03: Output Formatter
Format command output for display.
- Table formatting (ASCII borders, alignment)
- JSON/CSV/Plain formatters
- Colour coding for terminals
- Pagination for large outputs

### REED-18-04: Interactive Shell
Provide shell mode (`reed shell`).
- Command history (↑/↓ keys)
- Tab completion
- Multi-line input
- Persistent history file

### REED-18-05: Help System
Generate help from registry metadata.
- `reed help` - List all tools
- `reed help <tool>` - List tool commands
- `reed help <tool> <command>` - Command details
- Auto-generated from Reed.toml

### REED-18-06: Tool Integration
Invoke tool handlers dynamically.
- Function lookup by name
- Argument marshalling
- Error propagation
- Response formatting

---

## Performance Requirements

| Operation | Target |
|-----------|--------|
| CLI startup | < 50ms |
| Command parsing | < 1ms |
| Registry loading | < 10ms |
| Help generation | < 20ms |
| Shell prompt response | < 5ms |

---

## Testing Strategy

### Unit Tests
- Each module has `.test.rs` file
- Test all error paths
- Mock tool handlers for integration tests

### Integration Tests
- End-to-end command execution
- Registry loading with various Reed.toml files
- Output formatting for different data types

### Performance Tests
- Startup time benchmarks
- Large output formatting tests

---

## Dependencies

```toml
[dependencies]
clap = "4.5"              # Argument parsing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"              # Reed.toml parsing
rustyline = "15.0"        # Interactive shell
prettytable-rs = "0.10"   # Table formatting
anyhow = "1.0"
thiserror = "2.0"
```

---

## MANDATORY Development Standards

### Language and Documentation
- **ALL code comments, documentation, and commit messages MUST be written in BBC English**
- No exceptions for inline comments, docstrings, or markdown files

### Code Principles
- **KISS Principle**: Keep implementations simple and straightforward
- **Single Responsibility**: One file = one clear responsibility, one function = one distinctive job
- **No Swiss Army Knives**: Avoid multi-purpose functions or generic utility files

### File Naming and Structure
- Use specific, descriptive names: `parser.rs`, `registry.rs`, NOT `utils.rs`, `helpers.rs`
- Place files in `reedcli/src/` (CLI-specific) or `reedcli/src/types.rs` (shared types)

### Testing
- **Separate test files MANDATORY**: `{name}.test.rs` alongside source files
- NEVER use inline `#[cfg(test)]` modules
- Example: `parser.rs` → `parser.test.rs`

### Code Templates
- Follow `_workbench/Tickets/templates/service-template.md` for service structure
- Follow `_workbench/Tickets/templates/service-template.test.md` for test structure

### Error Handling
- Use specific error types, never generic errors
- Define `CliError` enum with variants for each error category
- Always use `.map_err()` with context for rich error messages

### Documentation Format
```rust
/// Brief one-line description.
///
/// ## Arguments
/// - `arg`: Description
///
/// ## Returns
/// - Description of return value
///
/// ## Performance
/// - O(1) operation, < 1ms typical
///
/// ## Example
/// ```rust
/// let result = function(arg)?;
/// ```
pub fn function(arg: Type) -> CliResult<Type> {
    // Implementation
}
```

### Git Commits
- Format: `[REED-18-XX] – type: description`
- Types: feat, fix, docs, style, refactor, perf, test, chore
- Example: `[REED-18-01] – feat: implement command parser with quoted arguments`

---

## Timeline Estimate

| Ticket | Effort | Priority |
|--------|--------|----------|
| REED-18-01 | 2 days | Critical |
| REED-18-02 | 3 days | Critical |
| REED-18-03 | 2 days | High |
| REED-18-04 | 3 days | Medium |
| REED-18-05 | 2 days | High |
| REED-18-06 | 3 days | Critical |
| **Total** | **15 days** | |

**Note**: Assumes full-time work. Can be parallelised after REED-18-01 and REED-18-02 are complete.

---

## Success Criteria

- [ ] Single `reed` binary accepts all commands
- [ ] New tools can register commands via Reed.toml without CLI modifications
- [ ] Help is auto-generated from registry metadata
- [ ] Output formats (table/json/csv) work correctly
- [ ] Interactive shell provides history and completion
- [ ] All unit tests pass (100% coverage target)
- [ ] Integration tests cover major workflows
- [ ] Performance targets met (< 50ms startup)
- [ ] Documentation complete (inline docs + user manual)

---

## Notes

- CLI is **read-only** for tool code - tools cannot modify CLI behaviour at runtime
- Registry is loaded once at startup (no hot-reload)
- Shell mode is optional - all commands work in single-shot mode
- Exit codes follow Unix conventions (0=success, 1=user error, 2=system error)
