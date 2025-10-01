# Claude Code Guidelines for ReedCMS

This document contains project-specific guidelines for Claude Code when working on ReedCMS.

## Language Rules

### Documentation and Code Comments
- **ALL documentation and code comments MUST be written in BBC English**
- **NO exceptions** - this applies to:
  - All Markdown files
  - All inline code comments
  - All docstrings and documentation blocks
  - All commit messages
  - All issue descriptions
  - All pull request descriptions

### Communication with User
- Communication with the user (Vivian Voss) can be in German or English as appropriate
- The user may communicate in German - respond naturally
- Internal project documentation stays in BBC English

## Critical Safety Rules

### File Deletion (rm commands)
**⚠️ CRITICAL: ALWAYS ASK USER BEFORE DELETING FILES**

- **NEVER execute `rm` commands without explicit user approval**
- **ALWAYS ask the user before:**
  - `rm -rf` operations
  - `rm` of any file or directory
  - Any destructive file operations
  - Cleaning build directories that may contain important files

**Example of correct behaviour:**
```
❌ WRONG: Executing `rm -rf target/` without asking
✅ RIGHT: "I need to clean the target/ directory. May I execute `rm -rf target/`?"
```

## Git Commit Message Format

### Commit Message Structure
All commit messages MUST follow this format:

```
[TICKET-ID] – type: short description

Optional longer description with details.
Can span multiple lines.

- Bullet points for changes
- More details if needed
```

### Ticket ID Format
- **REQUIRED**: Every commit MUST have a ticket reference
- Format: `[REED-XX-YY]` where XX is the layer and YY is the ticket number
- Examples:
  - `[REED-01-01]` - Foundation layer, ticket 1
  - `[REED-06-02]` - Server layer, ticket 2
  - `[REED-10-04]` - Monitor layer, ticket 4

### Commit Types
Use conventional commit types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Build process or auxiliary tool changes
- `ci`: CI/CD configuration changes

### Examples

**Good commit messages:**
```
[REED-02-01] – feat: implement ReedBase core HashMap cache

Added O(1) lookup HashMap for text, route, and meta data.
Includes environment fallback logic and RwLock for thread safety.

- HashMap-based cache with RwLock
- Environment suffix resolution (@dev, @prod)
- Fallback chain implementation
```

```
[REED-06-03] – fix: correct Argon2 password verification timing

Fixed constant-time comparison issue in auth middleware.
```

```
[REED-05-02] – docs: update MiniJinja setup documentation

Clarified hot-reload configuration for DEV mode.
```

### Attribution Rules

**🚫 NEVER mention Claude, Claude Code, or AI assistance in commit messages**

- ❌ WRONG: "Generated with Claude Code"
- ❌ WRONG: "Co-Authored-By: Claude <noreply@anthropic.com>"
- ❌ WRONG: "AI-assisted implementation"
- ✅ RIGHT: Focus on the actual changes and their purpose

**Author field should always be:**
```
Author: Vivian Voss <ask@vvoss.dev>
```

## Development Standards

### MANDATORY Code Standards
Every implementation MUST follow these standards:

1. **Language**: All code comments and docs in BBC English
2. **Principle**: KISS (Keep It Simple, Stupid)
3. **File Naming**: One file = one clear responsibility
4. **Functions**: One function = one distinctive job
5. **Testing**: Separate `.test.rs` files, never inline `#[cfg(test)]`
6. **Avoid**: Swiss Army knife functions
7. **Avoid**: Generic names like `handler.rs`, `middleware.rs`, `utils.rs`

### License Header in Code Files

**MANDATORY**: Every code file MUST start with license and copyright information in the first two lines:

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

// Rest of the file...
```

This applies to:
- All `.rs` (Rust) files
- All `.js` (JavaScript) files
- All `.css` files (use `/* */` comment style)
- Any other source code files

**CSS/JS example:**
```css
/* Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0. */
/* SPDX-License-Identifier: Apache-2.0 */
```

### File Organization
```
src/
├── reedcms/
│   ├── module_name/
│   │   ├── specific_feature.rs
│   │   ├── specific_feature.test.rs
│   │   └── another_feature.rs
```

### Service Implementation
- See `_workbench/Tickets/templates/service-template.md` for complete guide
- See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket System

### Ticket Structure
ReedCMS uses a 10-layer architecture with 37 tickets:

1. **REED-01**: Foundation Layer (2 tickets)
2. **REED-02**: Data Layer (4 tickets)
3. **REED-03**: Security Layer (3 tickets)
4. **REED-04**: CLI Layer (9 tickets)
5. **REED-05**: Template Layer (3 tickets)
6. **REED-06**: Server Layer (4 tickets)
7. **REED-07**: API Layer (2 tickets)
8. **REED-08**: Asset Layer (3 tickets)
9. **REED-09**: Build Layer (3 tickets)
10. **REED-10**: Monitor Layer (4 tickets)

### Working with Tickets
- All tickets are in `_workbench/Tickets/REED-XX-YYYLayer/`
- Each ticket contains complete implementation specifications
- Reference ticket number in all related commits
- Mark acceptance criteria as completed in ticket files

## Performance Requirements

### General Guidelines
- O(1) operations preferred for lookups
- Sub-millisecond response times for cached operations
- Minimize allocations in hot paths
- Use appropriate data structures (HashMap, Vec, etc.)

### Specific Targets
- ReedBase lookups: < 100μs
- Template rendering: < 50ms
- Request handling: < 10ms average
- Asset bundling: < 10s for complete build

## Testing Requirements

### Test Coverage
- Target: 100% code coverage for all modules
- Separate test files (`.test.rs`)
- Unit tests, integration tests, performance tests

### Test Structure
```rust
// my_feature.test.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_behaviour() {
        // Arrange
        // Act
        // Assert
    }
}
```

## Documentation Standards

### Code Documentation
```rust
/// Brief one-line description.
///
/// ## Arguments
/// - param: Description of parameter
///
/// ## Returns
/// - Description of return value
///
/// ## Examples
/// ```rust
/// let result = function(param);
/// ```
///
/// ## Performance
/// - O(1) operation
/// - < 100μs typical
pub fn function(param: Type) -> Result<Type> {
    // Implementation
}
```

### Markdown Documentation
- Use BBC English
- Clear headings and structure
- Code examples where helpful
- Performance notes where relevant

## Build and Development

### Development Workflow
```bash
# Start development with file watching
reed build:watch

# Run server
reed server:start

# Run tests
cargo test

# Build release
reed build:release
```

### Before Committing
1. Ensure all tests pass
2. Check code formatting (`cargo fmt`)
3. Check for warnings (`cargo clippy`)
4. Reference correct ticket number
5. Use proper commit message format

## Project-Specific Notes

### CSV Format
- **Delimiter**: pipe (`|`)
- **Format**: `key|value|description`
- **All files in `.reed/` directory**
- **Automatic backups before modifications**

### Key Nomenclature (MANDATORY)
- **Dot-notation**: `lowercase.with.dots` (NOT `UPPERCASE_WITH_UNDERSCORES`)
- **Sub-layouts**: Flat structure (`agility.title`, NOT `knowledge.agility.title`)
- **Global components**: With namespace (`page-header.logo.title`)
- **Nesting depth**: Optimal 4, maximum 8 levels
- **Language suffix**: Lowercase after key (`@de`, `@en`, NOT `@DE`, `@EN`)

### CSV File Separation
- **`.reed/text.csv`**: All content text
- **`.reed/routes.csv`**: All URL routing (central, not in component files)
- **`.reed/meta.csv`**: All SEO metadata (title, description) and technical meta (cache, access)

### Environment Suffixes
- `@dev` - Development environment
- `@prod` - Production environment
- `@christmas`, `@easter` - Seasonal themes
- Fallback: key@env → key if not found

### Variant System
- `mouse` - Desktop browsers
- `touch` - Mobile/tablet devices
- `reader` - Text-only/reader mode

## Contact

**Project Owner**: Vivian Voss
**Email**: ask@vvoss.dev
**GitHub**: @vvoss-dev

## License

Apache License 2.0 - See LICENSE file for details
