# REED-19-01: Registry & Dictionary System

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-19-01
- **Title**: Registry & Dictionary System
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: Critical
- **Status**: Open
- **Complexity**: Low
- **Dependencies**: None
- **Estimated Time**: 4 hours

## Objective

Implement global lookup tables (dictionaries) for encoded metadata in version logs. Actions and users are stored as integer codes with human-readable lookup tables for efficiency.

## Requirements

### File Structure

```
.reed/
└── registry/
    ├── actions.dict      # Action code → name mapping
    └── users.dict        # User code → username mapping
```

### actions.dict Format

```csv
code|name|description
0|delete|Delete operation
1|create|Create new entry
2|update|Update existing entry
3|rollback|Rollback to previous version
4|compact|Compact/cleanup old versions
5|init|Initialise table
6|snapshot|Full snapshot (periodic)
7|automerge|Automatic merge of concurrent writes
8|conflict|Conflict detected
9|resolve|Manual conflict resolution
```

### users.dict Format

```csv
code|username|created_at
0|system|{unix_timestamp}
```

## Implementation Files

### Primary Implementation

**`src/reedbase/registry/mod.rs`**
- Module organisation
- Public exports
- Module-level documentation in BBC English

**`src/reedbase/registry/dictionary.rs`**

One file = Dictionary operations only. NO other responsibilities.

```rust
/// Get action name from code.
///
/// ## Input
/// - `code`: Action code (0-255)
///
/// ## Output
/// - `Result<String>`: Action name
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100ns typical
///
/// ## Error Conditions
/// - UnknownActionCode: Code not in dictionary
///
/// ## Example Usage
/// ```rust
/// let name = get_action_name(2)?;  // "update"
/// ```
pub fn get_action_name(code: u8) -> ReedResult<String>

/// Get action code from name.
///
/// Reverse lookup: name → code.
///
/// ## Input
/// - `name`: Action name (e.g., "update")
///
/// ## Output
/// - `Result<u8>`: Action code
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100ns typical
///
/// ## Error Conditions
/// - UnknownAction: Name not found
pub fn get_action_code(name: &str) -> ReedResult<u8>

/// Get username from code.
///
/// ## Input
/// - `code`: User code (0-4294967295)
///
/// ## Output
/// - `Result<String>`: Username
///
/// ## Performance
/// - O(1) HashMap lookup
/// - < 100ns typical
///
/// ## Error Conditions
/// - UnknownUserCode: Code not in dictionary
pub fn get_username(code: u32) -> ReedResult<String>

/// Get or create user code.
///
/// Auto-increments if user doesn't exist. Thread-safe.
///
/// ## Input
/// - `username`: Username to look up or create
///
/// ## Output
/// - `Result<u32>`: User code (existing or new)
///
/// ## Performance
/// - Existing user: < 100ns (cached)
/// - New user: < 10ms (append to CSV + cache update)
///
/// ## Error Conditions
/// - IoError: Cannot write to users.dict
/// - ParseError: CSV corruption
pub fn get_or_create_user_code(username: &str) -> ReedResult<u32>

/// Reload dictionaries from disk.
///
/// Hot-reload for changes made externally.
///
/// ## Performance
/// - < 10ms for typical dictionary sizes
///
/// ## Error Conditions
/// - IoError: Cannot read dictionary files
/// - ParseError: CSV corruption
pub fn reload_dictionaries() -> ReedResult<()>
```

**`src/reedbase/registry/init.rs`**

One file = Initialisation only. NO other responsibilities.

```rust
/// Initialise registry system.
///
/// Creates directory structure and default dictionaries.
///
/// ## Input
/// - `base_path`: Path to ReedBase directory
///
/// ## Output
/// - `Result<()>`: Success or error
///
/// ## Performance
/// - < 20ms first run (creates files)
/// - < 1ms subsequent runs (files exist)
///
/// ## Error Conditions
/// - IoError: Cannot create directories
/// - PermissionDenied: Insufficient permissions
pub fn init_registry(base_path: &Path) -> ReedResult<()>

/// Create default actions dictionary.
///
/// Creates actions.dict with standard action codes.
///
/// ## Performance
/// - < 5ms
///
/// ## Error Conditions
/// - IoError: Cannot write file
fn create_default_action_dict(path: &Path) -> ReedResult<()>

/// Create default users dictionary.
///
/// Creates users.dict with system user (code 0).
///
/// ## Performance
/// - < 5ms
///
/// ## Error Conditions
/// - IoError: Cannot write file
fn create_default_user_dict(path: &Path) -> ReedResult<()>

/// Validate dictionary integrity.
///
/// Checks CSV format and code uniqueness.
///
/// ## Performance
/// - < 10ms for typical dictionaries
///
/// ## Error Conditions
/// - DictionaryCorrupted: Invalid CSV format
/// - DuplicateCode: Code collision detected
pub fn validate_dictionaries(base_path: &Path) -> ReedResult<()>
```

### Test Files

**`src/reedbase/registry/dictionary.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_action_name_valid_code() {
        // Test valid code returns correct name
    }
    
    #[test]
    fn test_get_action_name_unknown_code() {
        // Test unknown code returns error
    }
    
    #[test]
    fn test_get_action_code_valid_name() {
        // Test valid name returns correct code
    }
    
    #[test]
    fn test_get_action_code_case_insensitive() {
        // Test "UPDATE" and "update" both work
    }
    
    #[test]
    fn test_get_username_system_user() {
        // Test code 0 returns "system"
    }
    
    #[test]
    fn test_create_new_user_code() {
        // Test new user gets auto-incremented code
    }
    
    #[test]
    fn test_create_user_idempotent() {
        // Test same user twice returns same code
    }
    
    #[test]
    fn test_concurrent_user_creation() {
        // Test thread-safety of user creation
    }
    
    #[test]
    fn test_reload_dictionaries() {
        // Test hot-reload works
    }
    
    #[test]
    fn test_action_code_roundtrip() {
        // Test name → code → name returns original
    }
}
```

**`src/reedbase/registry/init.test.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_init_registry_creates_directories() {
        // Test directory creation
    }
    
    #[test]
    fn test_init_registry_idempotent() {
        // Test multiple calls don't fail
    }
    
    #[test]
    fn test_create_default_action_dict() {
        // Test actions.dict created correctly
    }
    
    #[test]
    fn test_create_default_user_dict() {
        // Test users.dict created correctly
    }
    
    #[test]
    fn test_validate_dictionaries_valid() {
        // Test validation passes for valid dicts
    }
    
    #[test]
    fn test_validate_dictionaries_corrupted() {
        // Test validation fails for corrupted CSV
    }
}
```

## CLI Commands

```bash
# List all action codes
reed dict:actions
# Output:
# 0 - delete
# 1 - create
# 2 - update
# 3 - rollback
# 4 - compact
# 5 - init
# 6 - snapshot
# 7 - automerge
# 8 - conflict
# 9 - resolve

# List all users
reed dict:users
# Output:
# 0 - system
# 1 - admin
# 2 - peter

# Validate dictionaries
reed dict:validate
# Output: ✓ All dictionaries valid

# Reload dictionaries (hot-reload)
reed dict:reload
# Output: ✓ Dictionaries reloaded
```

## Performance Requirements

- Dictionary load: < 5ms
- Lookup (cached): < 100ns (O(1) HashMap)
- User creation: < 10ms (append to CSV)
- Reload: < 10ms
- Memory: < 50 KB for typical dictionaries

## Error Conditions

- **UnknownActionCode**: Action code not in dictionary
- **UnknownUserCode**: User code not in dictionary  
- **UnknownAction**: Action name not found
- **DictionaryCorrupted**: CSV parse error
- **DuplicateUserCode**: Code collision (should never happen with proper locking)
- **IoError**: File system errors
- **PermissionDenied**: Insufficient permissions

## Acceptance Criteria

- [ ] actions.dict created with default 10 actions
- [ ] users.dict created with system user (code 0)
- [ ] get_action_name() works for all default codes
- [ ] get_action_code() reverse lookup works
- [ ] get_username() works for registered users
- [ ] get_or_create_user_code() auto-increments correctly
- [ ] Concurrent user creation is thread-safe
- [ ] Dictionary reload works without restart
- [ ] All tests pass with 100% coverage
- [ ] CLI commands work
- [ ] Performance benchmarks meet targets
- [ ] All code in BBC English
- [ ] All functions have proper documentation
- [ ] No Swiss Army knife functions
- [ ] Each file has one clear responsibility

## Dependencies
- **Requires**: None (foundation ticket)

## Blocks
- REED-19-07 (Encoded Log System - uses registries)

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-19-00: Layer Overview

## Notes

The registry system provides efficient integer encoding for frequently-used values in logs. By storing codes instead of strings:

- **50% smaller log files** (integer vs string)
- **5x faster parsing** (no string comparison)
- **Better compression** (repeating integers compress well with XZ)

The trade-off is an additional lookup table, but with HashMap caching this overhead is negligible (< 100ns per lookup).

**Implementation Note**: Use `OnceLock` for global caches and `RwLock` for thread-safe updates. Never use global mutable state without synchronisation.
