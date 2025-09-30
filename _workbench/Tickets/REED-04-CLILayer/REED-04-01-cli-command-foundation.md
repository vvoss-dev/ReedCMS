# REED-04-01: CLI Command Foundation

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function (CONFIG and SYSTEM use identical logic)
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs` - every topic (auth, login,...) has its own scoped rs service file
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-04-01
- **Title**: CLI Command Foundation
- **Layer**: CLI Layer (REED-04)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-01-01

## Summary Reference
- **Section**: CLI Interface & Commands
- **Lines**: 1032-1228 in project_summary.md
- **Key Concepts**: Colon notation parsing, command routing, unified reed binary

## Objective
Implement CLI command parser with colon notation (`reed command:action`) and command routing system that serves as the foundation for all CLI operations.

## Requirements

### Command Syntax
```bash
reed command:action [arguments] [--flags]

Examples:
reed set:text key@lang "value" --desc "Description"
reed get:text key@lang
reed init:layout knowledge --preset docs
reed user:create username --roles "editor,author"
reed server:io --port 8333
```

### Implementation Files

#### Main Binary Entry Point (`src/main.rs`)

```rust
/// ReedCMS unified CLI binary entry point.
///
/// ## Architecture
/// - Single binary for all operations
/// - Command routing to appropriate modules
/// - Consistent error handling
/// - Automatic help generation
fn main() {
    let args: Vec<String> = std::env::args().collect();

    match cli::run(args) {
        Ok(output) => {
            println!("{}", output);
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

#### Command Parser (`src/reedcms/cli/parser.rs`)

```rust
/// Parses CLI arguments into Command structure.
///
/// ## Parsing Rules
/// - First arg: command:action format
/// - Remaining args: positional arguments
/// - --flag value: named flags
/// - --flag: boolean flags
///
/// ## Performance
/// - Parsing time: < 1ms
pub fn parse_command(args: Vec<String>) -> ReedResult<Command>

/// Validates command format.
///
/// ## Format Rules
/// - Must contain exactly one colon
/// - Namespace and action alphanumeric + underscore
/// - No spaces or special characters
pub fn validate_command_format(cmd: &str) -> ReedResult<()>

/// Parses flags from arguments.
///
/// ## Examples
/// - --desc "My description" → ("desc", "My description")
/// - --verbose → ("verbose", "true")
pub fn parse_flags(args: &[String]) -> ReedResult<HashMap<String, String>>

/// Command structure
#[derive(Debug, Clone)]
pub struct Command {
    pub namespace: String,      // e.g., "set", "get", "user"
    pub action: String,          // e.g., "text", "create", "list"
    pub args: Vec<String>,       // Positional arguments
    pub flags: HashMap<String, String>,  // Named flags
}
```

#### Command Router (`src/reedcms/cli/router.rs`)

```rust
/// Routes command to appropriate handler.
///
/// ## Routing Table
/// - set:* → data_commands module
/// - get:* → data_commands module
/// - list:* → data_commands module
/// - init:* → layout_commands module
/// - user:* → user_commands module
/// - role:* → role_commands module
/// - taxonomy:* → taxonomy_commands module
/// - server:* → server_commands module
/// - build:* → build_commands module
/// - debug:* → debug_commands module
///
/// ## Performance
/// - Routing time: < 0.1ms (HashMap lookup)
pub fn route_command(cmd: Command) -> ReedResult<ReedResponse<String>>

/// Registers command handler.
pub fn register_handler(namespace: &str, handler: CommandHandler)

/// Command handler function type
pub type CommandHandler = fn(Command) -> ReedResult<ReedResponse<String>>;
```

#### Help System (`src/reedcms/cli/help.rs`)

```rust
/// Prints help for command or general help.
///
/// ## Help Levels
/// - reed --help: General overview
/// - reed set:text --help: Command-specific help
/// - reed --version: Version information
pub fn print_help(command: Option<&str>)

/// Generates command usage string.
///
/// ## Format
/// Usage: reed set:text <key@lang> <value> --desc <description>
pub fn generate_usage(command: &Command) -> String

/// Lists all available commands.
pub fn list_commands() -> Vec<CommandInfo>

/// Command information structure
#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub namespace: String,
    pub action: String,
    pub description: String,
    pub usage: String,
    pub examples: Vec<String>,
}
```

## Implementation Files

### Primary Implementation
- `src/main.rs` - Binary entry point
- `src/reedcms/cli/parser.rs` - Command parsing
- `src/reedcms/cli/router.rs` - Command routing
- `src/reedcms/cli/help.rs` - Help system

### Test Files
- `src/reedcms/cli/parser.test.rs`
- `src/reedcms/cli/router.test.rs`
- `src/reedcms/cli/help.test.rs`

## File Structure
```
src/
├── main.rs                    # Binary entry point
└── reedcms/
    └── cli/
        ├── parser.rs          # Command parsing
        ├── parser.test.rs     # Parser tests
        ├── router.rs          # Command routing
        ├── router.test.rs     # Router tests
        ├── help.rs            # Help system
        └── help.test.rs       # Help tests
```

## Testing Requirements

### Unit Tests
- [ ] Test command parsing (colon notation)
- [ ] Test flag parsing (--flag value)
- [ ] Test boolean flags (--verbose)
- [ ] Test command validation
- [ ] Test routing to correct handlers
- [ ] Test help generation

### Integration Tests
- [ ] Test complete CLI workflow (parse → route → execute)
- [ ] Test error messages user-friendly
- [ ] Test --help for all commands
- [ ] Test --version flag

### Edge Case Tests
- [ ] Test invalid command format (no colon)
- [ ] Test unknown command
- [ ] Test missing required arguments
- [ ] Test conflicting flags

### Performance Tests
- [ ] Command parsing: < 1ms
- [ ] Routing: < 0.1ms
- [ ] Help generation: < 5ms

## Acceptance Criteria
- [ ] Colon notation parsing works correctly
- [ ] Command routing to correct services functional
- [ ] Help system implemented for all commands
- [ ] Flag parsing (--flag value) working
- [ ] Boolean flags supported
- [ ] Error messages user-friendly and actionable
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete
- [ ] BBC English throughout

## Dependencies
- **Requires**: REED-01-01 (ReedStream)

## Blocks
All CLI command tickets depend on this:
- REED-04-02 through REED-04-09

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1032-1228 in `project_summary.md`

## Notes
This is the foundation for the entire CLI system. The colon notation (command:action) provides a clear and consistent interface. The routing system must be extensible to allow new commands to be added easily. Error messages must be actionable and guide users to correct syntax.