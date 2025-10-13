# REED-18-03: CLI Adapter System

**Status**: Not Started  
**Priority**: High  
**Estimated Effort**: 3-4 days  
**Layer**: CLI (Presentation Layer)  
**Dependencies**: REED-18-01, REED-18-02  

---

## Overview

This ticket implements a CLI adapter system that enables ReedCLI to discover and route commands to external Reed tools (ReedBase, ReedCMS) through explicit adapter configuration in Reed.toml.

**Purpose**: Provide a clean, extensible architecture for integrating multiple Reed tools into a unified CLI experience whilst maintaining clear separation of concerns.

**Scope**:
- Adapter registration and discovery via Reed.toml
- Command routing with namespace support (`reed adapter:command`)
- Automatic command discovery from adapters
- Namespace omission for unambiguous commands
- Adapter versioning and validation
- CLI-only (no library or plugin support in this ticket)

---

## MANDATORY Development Standards

1. **Language**: All code comments and documentation in BBC English
2. **Principle**: KISS (Keep It Simple, Stupid)
3. **File Naming**: Each file has unique theme and clear responsibility
4. **Files**: One file = One responsibility (no multi-purpose files)
5. **Functions**: One function = One distinctive job (no Swiss Army knives)
6. **Testing**: Separate test files as `{name}.test.rs` (never inline `#[cfg(test)]`)
7. **Avoid**: Generic names like `handler.rs`, `middleware.rs`, `utils.rs`
8. **Templates**: Reference `service-template.md` and `service-template.test.md`

---

## Architecture Overview

### Command Flow with Adapters

```
User Input: "reed reedbase:query users 'SELECT *'"
    │
    ▼
┌─────────────────────────────────────────┐
│ ReedCLI Parser (REED-18-01)             │
│ Parses: adapter="reedbase"              │
│         command="query"                 │
│         args=["users", "SELECT *"]      │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ Adapter Registry (REED-18-03)           │
│ Loads Reed.toml adapter definitions     │
│ Finds: reedbase → /usr/local/bin/reedbase│
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ Adapter Executor (REED-18-03)           │
│ Executes: reedbase query users "SELECT *"│
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ External Binary: ReedBase CLI           │
│ Processes query and returns results     │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ Output Formatter (REED-18-06)           │
│ Formats output for display              │
└─────────────────────────────────────────┘
```

### Namespace Handling

**Explicit namespace** (always works):
```bash
reed reedbase:query users "SELECT *"
reed reedcms:page:list
```

**Namespace omission** (when unambiguous):
```bash
# If only reedbase has "query" command:
reed query users "SELECT *"  # → reed reedbase:query

# If both reedbase and reedcms have "list" command:
reed list  # → Error: Ambiguous command "list"
           # → Available: reedbase:list, reedcms:list
```

---

## Reed.toml Configuration Format

### Adapter Configuration

```toml
# Reed.toml

# CLI Configuration
[cli]
# List of enabled adapters (order matters for namespace resolution)
adapters = ["reedbase", "reedcms", "reedstream"]

# Allow commands without namespace if unambiguous
namespace_omission = true

# Adapter Definitions
[adapters.reedbase]
binary = "reedbase"
description = "ReedBase database operations"
version = ">=0.1.0"  # Optional: minimum version requirement
required = false     # Optional: fail if not available (default: false)

# Command aliases (optional)
[adapters.reedbase.aliases]
q = "query"
t = "table"
db = "database"

[adapters.reedcms]
binary = "reedcms"
description = "ReedCMS content management"
version = ">=2.0.0"
required = true  # Project requires ReedCMS

[adapters.reedcms.aliases]
p = "page"
c = "component"

[adapters.reedstream]
binary = "reedstream"
description = "ReedStream monitoring"
required = false
```

### Command Discovery

Adapters must support `--list-commands` flag for auto-discovery:

```bash
$ reedbase --list-commands
query
table:create
table:list
table:info
set
get
delete
version:list
version:rollback

$ reedcms --list-commands
page:list
page:create
component:build
server:start
```

**Format**: One command per line, no additional output.

---

## Implementation Files

### 1. `src/reedcli/adapters/registry.rs`

**Purpose**: Load and manage adapter definitions from Reed.toml.

**Functions**:

```rust
/// Load adapter registry from Reed.toml.
///
/// ## Arguments
/// - config_path: Path to Reed.toml file
///
/// ## Returns
/// - AdapterRegistry with all configured adapters
///
/// ## Performance
/// - O(n) where n = number of adapters
/// - < 10ms for typical configuration
///
/// ## Error Conditions
/// - ReedError::FileNotFound: Reed.toml does not exist
/// - ReedError::InvalidFormat: TOML parsing error
/// - ReedError::AdapterNotFound: Required adapter binary missing
///
/// ## Example Usage
/// ```rust
/// let registry = load_adapter_registry(Path::new("Reed.toml"))?;
/// println!("Loaded {} adapters", registry.adapters.len());
/// ```
pub fn load_adapter_registry(config_path: &Path) -> ReedResult<AdapterRegistry>

/// Discover commands from adapter binary.
///
/// ## Arguments
/// - adapter: Adapter to query
///
/// ## Returns
/// - Vec of command names
///
/// ## Performance
/// - O(1) - subprocess call
/// - < 100ms (depends on adapter responsiveness)
///
/// ## Error Conditions
/// - ReedError::AdapterNotFound: Binary not in PATH
/// - ReedError::AdapterError: Binary does not support --list-commands
///
/// ## Example Usage
/// ```rust
/// let commands = discover_adapter_commands(&adapter)?;
/// for cmd in commands {
///     println!("Available: {}", cmd);
/// }
/// ```
pub fn discover_adapter_commands(adapter: &Adapter) -> ReedResult<Vec<String>>

/// Validate adapter binary exists and meets version requirements.
///
/// ## Arguments
/// - adapter: Adapter to validate
///
/// ## Returns
/// - ValidationResult with status
///
/// ## Performance
/// - O(1) - binary check + version query
/// - < 50ms
///
/// ## Error Conditions
/// - ReedError::AdapterNotFound: Binary not found
/// - ReedError::VersionMismatch: Version requirement not met
///
/// ## Example Usage
/// ```rust
/// let result = validate_adapter(&adapter)?;
/// if !result.valid {
///     eprintln!("Adapter validation failed: {}", result.error);
/// }
/// ```
pub fn validate_adapter(adapter: &Adapter) -> ReedResult<ValidationResult>

/// Check adapter version.
///
/// ## Arguments
/// - adapter: Adapter to check
///
/// ## Returns
/// - Version string (e.g., "0.1.0")
///
/// ## Performance
/// - O(1) - subprocess call
/// - < 50ms
///
/// ## Error Conditions
/// - ReedError::AdapterNotFound: Binary not found
/// - ReedError::AdapterError: Binary does not support --version
///
/// ## Example Usage
/// ```rust
/// let version = get_adapter_version(&adapter)?;
/// println!("ReedBase version: {}", version);
/// ```
pub fn get_adapter_version(adapter: &Adapter) -> ReedResult<String>

/// Build command index for fast lookup.
///
/// ## Arguments
/// - registry: AdapterRegistry with discovered commands
///
/// ## Returns
/// - CommandIndex mapping command names to adapters
///
/// ## Performance
/// - O(n × m) where n = adapters, m = commands per adapter
/// - < 10ms
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```rust
/// let index = build_command_index(&registry)?;
/// if let Some(adapter) = index.find("query") {
///     println!("'query' belongs to adapter: {}", adapter.name);
/// }
/// ```
pub fn build_command_index(registry: &AdapterRegistry) -> ReedResult<CommandIndex>
```

**Key Types**:

```rust
pub struct AdapterRegistry {
    pub adapters: HashMap<String, Adapter>,
    pub cli_config: CliConfig,
    pub command_index: CommandIndex,
}

pub struct Adapter {
    pub name: String,
    pub binary: PathBuf,
    pub description: String,
    pub version_requirement: Option<String>,
    pub required: bool,
    pub aliases: HashMap<String, String>,
    pub commands: Vec<String>,
    pub validated: bool,
}

pub struct CliConfig {
    pub adapters: Vec<String>,  // Ordered list from Reed.toml
    pub namespace_omission: bool,
}

pub struct CommandIndex {
    // Maps command name to list of adapters that provide it
    commands: HashMap<String, Vec<String>>,
}

impl CommandIndex {
    /// Find adapter for command (returns None if ambiguous or not found).
    pub fn find(&self, command: &str) -> Option<&str>
    
    /// Get all adapters that provide this command.
    pub fn find_all(&self, command: &str) -> Vec<&str>
    
    /// Check if command is ambiguous (multiple adapters provide it).
    pub fn is_ambiguous(&self, command: &str) -> bool
}

pub struct ValidationResult {
    pub valid: bool,
    pub error: Option<String>,
    pub version: Option<String>,
}
```

---

### 2. `src/reedcli/adapters/parser.rs`

**Purpose**: Parse adapter-namespaced commands.

**Functions**:

```rust
/// Parse command with optional adapter namespace.
///
/// ## Arguments
/// - input: Command string (e.g., "reedbase:query" or "query")
///
/// ## Returns
/// - ParsedCommand with optional adapter and command
///
/// ## Performance
/// - O(1) - string splitting
/// - < 10μs
///
/// ## Error Conditions
/// - None (returns None for adapter if no namespace)
///
/// ## Example Usage
/// ```rust
/// let cmd = parse_adapter_command("reedbase:query")?;
/// assert_eq!(cmd.adapter, Some("reedbase"));
/// assert_eq!(cmd.command, "query");
/// 
/// let cmd = parse_adapter_command("query")?;
/// assert_eq!(cmd.adapter, None);
/// assert_eq!(cmd.command, "query");
/// ```
pub fn parse_adapter_command(input: &str) -> ReedResult<ParsedCommand>

/// Resolve adapter for command using registry.
///
/// ## Arguments
/// - parsed: ParsedCommand from parser
/// - registry: AdapterRegistry for lookup
///
/// ## Returns
/// - ResolvedCommand with adapter name
///
/// ## Performance
/// - O(1) - hash lookup
/// - < 10μs
///
/// ## Error Conditions
/// - ReedError::CommandNotFound: Command not found in any adapter
/// - ReedError::AmbiguousCommand: Multiple adapters provide command
/// - ReedError::AdapterNotFound: Specified adapter does not exist
///
/// ## Example Usage
/// ```rust
/// let parsed = parse_adapter_command("query")?;
/// let resolved = resolve_adapter(&parsed, &registry)?;
/// println!("Using adapter: {}", resolved.adapter);
/// ```
pub fn resolve_adapter(
    parsed: &ParsedCommand,
    registry: &AdapterRegistry,
) -> ReedResult<ResolvedCommand>

/// Expand command aliases.
///
/// ## Arguments
/// - command: Command name (possibly aliased)
/// - adapter: Adapter with alias definitions
///
/// ## Returns
/// - Expanded command name
///
/// ## Performance
/// - O(1) - hash lookup
/// - < 1μs
///
/// ## Error Conditions
/// - None (returns original if no alias)
///
/// ## Example Usage
/// ```rust
/// let expanded = expand_alias("q", &adapter)?;
/// assert_eq!(expanded, "query");
/// ```
pub fn expand_alias(command: &str, adapter: &Adapter) -> String
```

**Key Types**:

```rust
pub struct ParsedCommand {
    pub adapter: Option<String>,  // None if no namespace
    pub command: String,
}

pub struct ResolvedCommand {
    pub adapter: String,
    pub command: String,
    pub args: Vec<String>,
}
```

---

### 3. `src/reedcli/adapters/executor.rs`

**Purpose**: Execute commands via adapter binaries.

**Functions**:

```rust
/// Execute command via adapter binary.
///
/// ## Arguments
/// - resolved: ResolvedCommand with adapter and command
/// - registry: AdapterRegistry for binary path lookup
///
/// ## Returns
/// - AdapterResult with exit code and output
///
/// ## Performance
/// - Depends on adapter binary
/// - Subprocess overhead: ~10ms
///
/// ## Error Conditions
/// - ReedError::AdapterNotFound: Binary not found
/// - ReedError::AdapterError: Binary execution failed
///
/// ## Example Usage
/// ```rust
/// let result = execute_adapter_command(&resolved, &registry)?;
/// if result.exit_code == 0 {
///     println!("{}", result.stdout);
/// } else {
///     eprintln!("Error: {}", result.stderr);
/// }
/// ```
pub fn execute_adapter_command(
    resolved: &ResolvedCommand,
    registry: &AdapterRegistry,
) -> ReedResult<AdapterResult>

/// Build command line arguments for adapter.
///
/// ## Arguments
/// - resolved: ResolvedCommand with command and args
///
/// ## Returns
/// - Vec of arguments to pass to binary
///
/// ## Performance
/// - O(n) where n = number of arguments
/// - < 10μs
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```rust
/// let args = build_adapter_args(&resolved);
/// // For: reed reedbase:query users "SELECT *"
/// // Returns: ["query", "users", "SELECT *"]
/// ```
pub fn build_adapter_args(resolved: &ResolvedCommand) -> Vec<String>

/// Handle adapter exit code and output.
///
/// ## Arguments
/// - result: AdapterResult from execution
///
/// ## Returns
/// - Formatted output string
///
/// ## Performance
/// - O(n) where n = output length
/// - < 1ms
///
/// ## Error Conditions
/// - ReedError::AdapterError: Non-zero exit code
///
/// ## Example Usage
/// ```rust
/// let output = handle_adapter_result(result)?;
/// println!("{}", output);
/// ```
pub fn handle_adapter_result(result: AdapterResult) -> ReedResult<String>
```

**Key Types**:

```rust
pub struct AdapterResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}
```

---

### 4. `src/reedcli/adapters/validator.rs`

**Purpose**: Validate adapter configuration and availability.

**Functions**:

```rust
/// Validate all adapters in registry.
///
/// ## Arguments
/// - registry: AdapterRegistry to validate
///
/// ## Returns
/// - RegistryValidation with status for each adapter
///
/// ## Performance
/// - O(n) where n = number of adapters
/// - ~50ms per adapter (binary checks)
///
/// ## Error Conditions
/// - ReedError::RequiredAdapterMissing: Required adapter not found
///
/// ## Example Usage
/// ```rust
/// let validation = validate_all_adapters(&registry)?;
/// for (adapter, result) in &validation.results {
///     if !result.valid {
///         eprintln!("Adapter '{}' invalid: {}", adapter, result.error.unwrap());
///     }
/// }
/// ```
pub fn validate_all_adapters(
    registry: &AdapterRegistry,
) -> ReedResult<RegistryValidation>

/// Check if binary exists in PATH.
///
/// ## Arguments
/// - binary_name: Name of binary to find
///
/// ## Returns
/// - Option<PathBuf> with full path if found
///
/// ## Performance
/// - O(n) where n = directories in PATH
/// - < 10ms
///
/// ## Error Conditions
/// - None (returns None if not found)
///
/// ## Example Usage
/// ```rust
/// if let Some(path) = find_binary_in_path("reedbase")? {
///     println!("Found: {}", path.display());
/// } else {
///     eprintln!("Binary not found");
/// }
/// ```
pub fn find_binary_in_path(binary_name: &str) -> ReedResult<Option<PathBuf>>

/// Compare version strings.
///
/// ## Arguments
/// - version: Actual version (e.g., "0.1.2")
/// - requirement: Version requirement (e.g., ">=0.1.0")
///
/// ## Returns
/// - true if version meets requirement
///
/// ## Performance
/// - O(1) - simple comparison
/// - < 10μs
///
/// ## Error Conditions
/// - ReedError::InvalidVersion: Cannot parse version
///
/// ## Example Usage
/// ```rust
/// assert!(version_matches("0.1.2", ">=0.1.0")?);
/// assert!(!version_matches("0.0.9", ">=0.1.0")?);
/// ```
pub fn version_matches(version: &str, requirement: &str) -> ReedResult<bool>

/// Generate helpful error message for missing adapter.
///
/// ## Arguments
/// - adapter: Adapter that is missing
///
/// ## Returns
/// - Formatted error message with installation hints
///
/// ## Performance
/// - O(1) - string formatting
/// - < 1ms
///
/// ## Error Conditions
/// - None
///
/// ## Example Usage
/// ```rust
/// let error_msg = format_missing_adapter_error(&adapter);
/// eprintln!("{}", error_msg);
/// // Output:
/// // Adapter 'reedbase' not found.
/// //
/// // To install:
/// //   cargo install reedbase
/// //
/// // Or ensure 'reedbase' binary is in your PATH.
/// ```
pub fn format_missing_adapter_error(adapter: &Adapter) -> String
```

**Key Types**:

```rust
pub struct RegistryValidation {
    pub valid: bool,
    pub results: HashMap<String, ValidationResult>,
}
```

---

### 5. `src/reedcli/adapters/mod.rs`

**Purpose**: Public API for adapter system.

**Functions**:

```rust
/// Initialise adapter system from Reed.toml.
///
/// ## Arguments
/// - config_path: Path to Reed.toml
///
/// ## Returns
/// - AdapterRegistry ready for use
///
/// ## Performance
/// - O(n) where n = number of adapters
/// - ~100ms with validation
///
/// ## Error Conditions
/// - ReedError::FileNotFound: Reed.toml missing
/// - ReedError::InvalidFormat: TOML parsing error
/// - ReedError::RequiredAdapterMissing: Required adapter not found
///
/// ## Example Usage
/// ```rust
/// let registry = initialise_adapters(Path::new("Reed.toml"))?;
/// // Ready to execute commands
/// ```
pub fn initialise_adapters(config_path: &Path) -> ReedResult<AdapterRegistry>

/// Execute command with automatic adapter resolution.
///
/// ## Arguments
/// - command_str: Command string (e.g., "reedbase:query" or "query")
/// - args: Command arguments
/// - registry: AdapterRegistry
///
/// ## Returns
/// - Command output as string
///
/// ## Performance
/// - Depends on adapter
/// - Overhead: ~10ms (resolution + subprocess)
///
/// ## Error Conditions
/// - ReedError::CommandNotFound: Command not found
/// - ReedError::AmbiguousCommand: Multiple adapters provide command
/// - ReedError::AdapterError: Adapter execution failed
///
/// ## Example Usage
/// ```rust
/// let output = execute_command(
///     "query",
///     vec!["users".to_string(), "SELECT *".to_string()],
///     &registry
/// )?;
/// println!("{}", output);
/// ```
pub fn execute_command(
    command_str: &str,
    args: Vec<String>,
    registry: &AdapterRegistry,
) -> ReedResult<String>
```

---

## CLI Integration

### Updated `src/reedcli/main.rs`

```rust
use reedcli::adapters;

fn main() -> ReedResult<()> {
    // Load adapter registry
    let registry = adapters::initialise_adapters(Path::new("Reed.toml"))?;
    
    // Parse command line
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        show_help(&registry)?;
        return Ok(());
    }
    
    // Execute command via adapter
    let command = &args[0];
    let command_args = args[1..].to_vec();
    
    let output = adapters::execute_command(command, command_args, &registry)?;
    println!("{}", output);
    
    Ok(())
}
```

---

## CLI Commands

### `reed adapters:list`
**Purpose**: List all configured adapters.

```bash
reed adapters:list

# Output:
# Configured Adapters
# ===================
# reedbase (✓)
#   Binary: /usr/local/bin/reedbase
#   Version: 0.1.0
#   Commands: 15
#   Description: ReedBase database operations
#
# reedcms (✓)
#   Binary: /usr/local/bin/reedcms
#   Version: 2.0.1
#   Commands: 23
#   Description: ReedCMS content management
#
# reedstream (✗)
#   Error: Binary not found in PATH
#   Required: No
```

### `reed adapters:validate`
**Purpose**: Validate all adapter configurations.

```bash
reed adapters:validate

# Output:
# Validating Adapters
# ===================
# ✓ reedbase: OK (version 0.1.0)
# ✓ reedcms: OK (version 2.0.1)
# ✗ reedstream: Binary not found
#
# 2/3 adapters valid
```

### `reed adapters:commands`
**Purpose**: List all available commands across all adapters.

```bash
reed adapters:commands

# Output:
# Available Commands
# ==================
# reedbase:
#   query, table:create, table:list, set, get, delete, ...
#
# reedcms:
#   page:list, page:create, component:build, server:start, ...
#
# Ambiguous (require namespace):
#   list (reedbase, reedcms)
#
# Unambiguous (can omit namespace):
#   query (reedbase)
#   server:start (reedcms)
```

### Command Execution Examples

```bash
# Explicit namespace (always works)
reed reedbase:query users "SELECT *"
reed reedcms:page:list

# Namespace omission (when unambiguous)
reed query users "SELECT *"  # → reedbase:query

# Alias usage
reed reedbase:q users "SELECT *"  # q → query

# Error: ambiguous command
reed list
# Error: Ambiguous command "list"
# Available:
#   reed reedbase:list
#   reed reedcms:list

# Help for specific adapter
reed reedbase:help
reed reedcms:help
```

---

## Test Files

### `src/reedcli/adapters/registry.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_load_adapter_registry()
// Verify: Registry loads from Reed.toml

#[test]
fn test_discover_adapter_commands()
// Verify: Commands discovered via --list-commands

#[test]
fn test_validate_adapter()
// Verify: Adapter validation checks binary and version

#[test]
fn test_build_command_index()
// Verify: Command index built correctly

#[test]
fn test_command_index_find()
// Verify: find() returns correct adapter

#[test]
fn test_command_index_ambiguous()
// Verify: is_ambiguous() detects conflicts

#[test]
fn test_required_adapter_missing()
// Verify: Error if required adapter not found
```

### `src/reedcli/adapters/parser.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_parse_adapter_command_with_namespace()
// Verify: "reedbase:query" parses correctly

#[test]
fn test_parse_adapter_command_without_namespace()
// Verify: "query" parses without adapter

#[test]
fn test_resolve_adapter_explicit()
// Verify: Explicit namespace resolution

#[test]
fn test_resolve_adapter_implicit()
// Verify: Implicit resolution when unambiguous

#[test]
fn test_resolve_adapter_ambiguous_error()
// Verify: Error when multiple adapters provide command

#[test]
fn test_expand_alias()
// Verify: Aliases expand correctly
```

### `src/reedcli/adapters/executor.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_execute_adapter_command()
// Verify: Command executes via subprocess

#[test]
fn test_build_adapter_args()
// Verify: Arguments passed correctly to binary

#[test]
fn test_handle_adapter_result_success()
// Verify: Exit code 0 handled correctly

#[test]
fn test_handle_adapter_result_error()
// Verify: Non-zero exit code produces error

#[test]
fn test_adapter_not_found()
// Verify: Error when binary missing
```

### `src/reedcli/adapters/validator.test.rs`

**Test Coverage**:

```rust
#[test]
fn test_validate_all_adapters()
// Verify: All adapters validated

#[test]
fn test_find_binary_in_path()
// Verify: Binary found in PATH

#[test]
fn test_version_matches()
// Verify: Version comparison works

#[test]
fn test_version_matches_operators()
// Verify: >=, <=, =, >, < operators

#[test]
fn test_format_missing_adapter_error()
// Verify: Helpful error message generated
```

---

## Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Load adapter registry | < 10ms | Wall time |
| Discover commands (per adapter) | < 100ms | Wall time |
| Validate adapter | < 50ms | Wall time |
| Parse adapter command | < 10μs | Wall time |
| Resolve adapter | < 10μs | Wall time |
| Execute adapter command | ~10ms overhead | Wall time (+ adapter time) |
| Build command index | < 10ms | Wall time |

---

## Error Conditions

### `ReedError::AdapterNotFound`
**When**: Adapter binary not found in PATH.  
**Context**: Adapter name, binary name, installation hints.  
**Recovery**: Install adapter or add to PATH.

### `ReedError::AmbiguousCommand`
**When**: Multiple adapters provide command without namespace.  
**Context**: Command name, list of conflicting adapters.  
**Recovery**: Use explicit namespace (adapter:command).

### `ReedError::CommandNotFound`
**When**: Command not found in any adapter.  
**Context**: Command name, available commands.  
**Recovery**: Check command name or use `reed adapters:commands`.

### `ReedError::VersionMismatch`
**When**: Adapter version does not meet requirement.  
**Context**: Adapter name, required version, actual version.  
**Recovery**: Update adapter to required version.

### `ReedError::RequiredAdapterMissing`
**When**: Required adapter not available.  
**Context**: Adapter name, required flag.  
**Recovery**: Install required adapter.

### `ReedError::AdapterError`
**When**: Adapter binary execution fails.  
**Context**: Exit code, stderr output.  
**Recovery**: Check adapter-specific error message.

---

## Acceptance Criteria

- [ ] `registry.rs` loads adapters from Reed.toml
- [ ] `parser.rs` parses adapter-namespaced commands
- [ ] `executor.rs` executes commands via adapter binaries
- [ ] `validator.rs` validates adapter availability and versions
- [ ] Command discovery via `--list-commands` works
- [ ] Namespace omission works for unambiguous commands
- [ ] Ambiguous commands produce clear error messages
- [ ] Adapter aliases expand correctly
- [ ] Version requirements validated
- [ ] Required adapters enforced
- [ ] CLI commands (`adapters:list`, `adapters:validate`, `adapters:commands`) work
- [ ] Error messages helpful with installation hints
- [ ] Performance targets met for all operations
- [ ] Test coverage 100% for all modules
- [ ] All tests pass
- [ ] Documentation complete with examples
- [ ] Integration with existing REED-18-01 parser seamless

---

## Dependencies

- **REED-18-01**: Command parser (extends to handle adapter namespaces)
- **REED-18-02**: Registry loader (replaces with adapter-based registry)

---

## Migration from REED-18-02

**Old format** (REED-18-02):
```toml
[tools.reedbase]
binary = "reedbase"

[tools.reedbase.commands.query]
handler = "query"
```

**New format** (REED-18-03):
```toml
[cli]
adapters = ["reedbase"]

[adapters.reedbase]
binary = "reedbase"
# Commands auto-discovered via --list-commands
```

**Migration strategy**:
1. Keep REED-18-02 as fallback for backwards compatibility
2. Prefer `[adapters.*]` over `[tools.*]` if both present
3. Deprecate `[tools.*]` format in documentation
4. Automatic migration command: `reed config:migrate-adapters`

---

## Notes

### Why CLI-Only?

**Advantages**:
- Simple subprocess model
- Process isolation
- No dependency hell
- Easy to test
- Clear security boundaries

**Future extensions** (if needed later):
- Library adapters (`.so`/`.dylib`/`.dll`) for performance
- Plugin adapters (WASM) for sandboxing
- Both can be added without breaking CLI adapter model

### Command Discovery Protocol

Adapters must implement:

```bash
# List all commands
adapter --list-commands
# Output: One command per line

# Show version
adapter --version
# Output: Semantic version (e.g., "0.1.0")

# Show help
adapter --help
# Output: General help text
```

**Example ReedBase implementation**:
```rust
// src/reedbase/cli/mod.rs
match args[0].as_str() {
    "--list-commands" => {
        println!("query");
        println!("table:create");
        println!("table:list");
        // ... all commands
    }
    "--version" => {
        println!("{}", env!("CARGO_PKG_VERSION"));
    }
    _ => {
        // Normal command handling
    }
}
```

### Namespace Resolution Priority

1. **Explicit namespace**: Always use specified adapter
   ```bash
   reed reedbase:query  # Always uses reedbase
   ```

2. **Adapter list order**: First adapter in `cli.adapters` list wins
   ```toml
   [cli]
   adapters = ["reedbase", "reedcms"]
   # If both have "list", reedbase wins
   ```

3. **Ambiguity error**: If multiple adapters provide command
   ```bash
   reed list  # Error if both reedbase and reedcms have "list"
   ```

### Version Requirement Syntax

Supported operators:
- `>=0.1.0` - Greater than or equal
- `<=0.2.0` - Less than or equal
- `=0.1.5` - Exact version
- `>0.1.0` - Greater than (not equal)
- `<0.2.0` - Less than (not equal)

Future: Support ranges like `>=0.1.0,<0.2.0`

### Performance Considerations

**Subprocess overhead**:
- ~10ms per command (acceptable for CLI)
- Cached registry avoids repeated TOML parsing
- Command discovery cached after first load

**Optimisations**:
- Lazy validation (only validate when adapter used)
- Command index built once at startup
- Binary path cached after first lookup

---

## References

- Service Template: `_workbench/Tickets/templates/service-template.md`
- Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-18-01: Command Parser
- REED-18-02: Registry Loader (to be replaced/extended)
