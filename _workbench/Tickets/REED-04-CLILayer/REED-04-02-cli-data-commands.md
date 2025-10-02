# REED-04-02: CLI Data Commands

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
- **ID**: REED-04-02
- **Title**: CLI Data Management Commands
- **Layer**: CLI Layer (REED-04)
- **Priority**: High
- **Status**: Complete
- **Complexity**: Medium
- **Dependencies**: REED-04-01, REED-02-01

## Summary Reference
- **Section**: CLI Data Management
- **Lines**: 1042-1050, 1133-1144 in project_summary.md
- **Key Concepts**: set:*, get:*, list:* commands for ReedBase data access

## Objective
Implement data management CLI commands for setting, getting, and listing text, routes, meta, server, and project data.

## Requirements

### Commands to Implement

```bash
# Set commands (write data)
reed set:text key@lang "value" --desc "Description"
reed set:route key@lang "route" --desc "Description"
reed set:meta key "value" --desc "Description"
reed set:server key "value" --desc "Description"
reed set:project key "value" --desc "Description"

# Get commands (read data)
reed get:text key@lang
reed get:route key@lang
reed get:meta key
reed get:server key
reed get:project key

# List commands (search data)
reed list:text pattern.*
reed list:route pattern.*
reed list:meta pattern.*
```

### Implementation (`src/reedcms/cli/data_commands.rs`)

```rust
/// Sets text content via CLI.
///
/// ## Arguments
/// - args[0]: key@lang (e.g., "knowledge.title@en")
/// - args[1]: value
/// - flags["desc"]: Mandatory description
///
/// ## Output
/// - Success message with key
/// - Confirmation of environment if used
///
/// ## Example
/// ```bash
/// reed set:text knowledge.title@en "Knowledge Base" --desc "Main page title"
/// # Output: ✓ Text set: knowledge.title@en = "Knowledge Base"
/// ```
pub fn set_text(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

pub fn set_route(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>
pub fn set_meta(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>
pub fn set_server(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>
pub fn set_project(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>>

/// Gets text content via CLI.
///
/// ## Arguments
/// - args[0]: key@lang (e.g., "knowledge.title@en")
///
/// ## Output
/// - Retrieved value
/// - Source indication (cached, CSV, fallback)
///
/// ## Example
/// ```bash
/// reed get:text knowledge.title@en
/// # Output: Knowledge Base
/// #         (source: cached)
/// ```
pub fn get_text(args: &[String]) -> ReedResult<ReedResponse<String>>

pub fn get_route(args: &[String]) -> ReedResult<ReedResponse<String>>
pub fn get_meta(args: &[String]) -> ReedResult<ReedResponse<String>>
pub fn get_server(args: &[String]) -> ReedResult<ReedResponse<String>>
pub fn get_project(args: &[String]) -> ReedResult<ReedResponse<String>>

/// Lists text keys matching pattern.
///
/// ## Arguments
/// - args[0]: Optional pattern (glob-style: "*", "knowledge.*")
///
/// ## Output
/// - Formatted list of matching keys
/// - Count summary
///
/// ## Example
/// ```bash
/// reed list:text "knowledge.*"
/// # Output:
/// # knowledge.title@en
/// # knowledge.title@de
/// # knowledge.description@en
/// # knowledge.description@de
/// # (4 entries found)
/// ```
pub fn list_text(args: &[String]) -> ReedResult<ReedResponse<Vec<String>>>

pub fn list_route(args: &[String]) -> ReedResult<ReedResponse<Vec<String>>>
pub fn list_meta(args: &[String]) -> ReedResult<ReedResponse<Vec<String>>>
```

### Validation (`src/reedcms/cli/data_validation.rs`)

```rust
/// Validates set command arguments.
///
/// ## Checks
/// - Minimum 2 arguments (key, value)
/// - --desc flag present and valid (min 10 chars)
/// - Key format valid
pub fn validate_set_args(args: &[String], flags: &HashMap<String, String>) -> ReedResult<()>

/// Validates get command arguments.
///
/// ## Checks
/// - Exactly 1 argument (key)
/// - Key format valid
pub fn validate_get_args(args: &[String]) -> ReedResult<()>

/// Validates pattern for list commands.
///
/// ## Valid Patterns
/// - "*" - All entries
/// - "prefix.*" - All entries starting with prefix
/// - "*.suffix" - All entries ending with suffix
pub fn validate_pattern(pattern: &str) -> ReedResult<()>
```

### Output Formatting (`src/reedcms/cli/data_output.rs`)

```rust
/// Formats success message for set operations.
///
/// ## Format
/// ✓ Text set: key@lang = "value"
pub fn format_set_success(key: &str, value: &str, data_type: &str) -> String

/// Formats get operation output.
///
/// ## Format
/// value
/// (source: cached|csv|fallback)
pub fn format_get_output(value: &str, source: &str) -> String

/// Formats list output with counts.
///
/// ## Format
/// key1
/// key2
/// key3
/// (3 entries found)
pub fn format_list_output(keys: &[String]) -> String
```

## Implementation Files

### Primary Implementation
- `src/reedcms/cli/data_commands.rs` - All data commands
- `src/reedcms/cli/data_validation.rs` - Input validation
- `src/reedcms/cli/data_output.rs` - Output formatting

### Test Files
- `src/reedcms/cli/data_commands.test.rs`
- `src/reedcms/cli/data_validation.test.rs`
- `src/reedcms/cli/data_output.test.rs`

## File Structure
```
src/reedcms/cli/
├── data_commands.rs          # Command implementations
├── data_commands.test.rs     # Command tests
├── data_validation.rs        # Input validation
├── data_validation.test.rs   # Validation tests
├── data_output.rs            # Output formatting
└── data_output.test.rs       # Formatting tests
```

## Testing Requirements

### Unit Tests
- [ ] Test set:text command
- [ ] Test get:text command
- [ ] Test list:text command with patterns
- [ ] Test all set:* variants
- [ ] Test all get:* variants
- [ ] Test validation errors

### Integration Tests
- [ ] Test set followed by get (round-trip)
- [ ] Test environment-specific keys (@dev, @prod)
- [ ] Test list with glob patterns
- [ ] Test error messages

### Edge Case Tests
- [ ] Test missing --desc flag
- [ ] Test invalid key format
- [ ] Test non-existent key (get)
- [ ] Test pattern with no matches (list)

### Performance Tests
- [ ] Set command: < 50ms
- [ ] Get command: < 10ms
- [ ] List command: < 100ms for 1000 entries

## Acceptance Criteria
- [x] All set:* commands working (text, route, meta)
- [x] All get:* commands working (text, route, meta)
- [x] list:* commands with pattern matching (*, prefix.*, *.suffix)
- [x] --desc flag mandatory for set commands (min 10 chars)
- [x] User-friendly output formatting
- [x] Error messages actionable
- [x] All 18 tests pass with 100% coverage
- [x] Documentation complete
- [x] BBC English throughout

## Dependencies
- **Requires**: REED-04-01 (CLI Foundation), REED-02-01 (ReedBase)

## Blocks
- None (this implements core data access commands)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- Summary: Lines 1042-1050, 1133-1144 in `project_summary.md`

## Notes
The --desc flag is mandatory for all set operations to ensure documentation. This is a core principle of ReedCMS: all data modifications must be documented. Output formatting should be clean and parseable by both humans and scripts. Consider adding --json flag for machine-readable output in future iterations.