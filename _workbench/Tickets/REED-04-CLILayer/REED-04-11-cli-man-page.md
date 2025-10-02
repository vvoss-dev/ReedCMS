# REED-04-11: CLI Man Page Documentation

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Testing**: Separate test files as `{name}_test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs`

## Ticket Information
- **ID**: REED-04-11
- **Title**: CLI Man Page Documentation
- **Layer**: CLI Layer (REED-04)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: Low
- **Dependencies**: REED-04-01, REED-04-02, REED-04-03, REED-04-04, REED-04-05, REED-04-06, REED-04-07, REED-04-08, REED-04-09, REED-04-10

## Summary Reference
- **Section**: CLI Interface & Commands
- **Lines**: 1032-1228 in project_summary.md
- **Key Concepts**: Unix/Linux system integration, offline documentation

## Objective
Create comprehensive man page documentation for the `reed` CLI tool following Unix/Linux conventions, enabling `man reed` system integration and providing offline documentation access.

## Rationale

### Industry Standard Practice
Professional CLI tools on Unix/Linux systems provide man pages:
- **cargo**: Comprehensive man pages for all subcommands
- **git**: Extensive man page documentation system
- **docker**: Full man page suite
- **rustup**: Complete man page coverage
- **npm**: Man pages for all commands

### Benefits
1. **System Integration**: Indexed by `apropos`, `whatis`, and `man -k`
2. **Offline Availability**: Works without internet connection
3. **Professional Impression**: Demonstrates production-ready tool maturity
4. **IDE/Editor Integration**: Many editors automatically display man pages
5. **Searchability**: System-wide documentation search
6. **Consistency**: Follows established Unix conventions

### User Expectations
Unix/Linux users expect `man <command>` to work for any installed CLI tool. Absence of man pages signals incomplete or unprofessional tooling.

## Implementation Strategy

### File Format Decision
**Decision**: Use `.ronn` (Markdown-based) for source, compile to `.1` (troff/groff)

**Reasoning**:
- **Maintainability**: Markdown easier to write and maintain than raw troff
- **Version Control**: Markdown diffs are human-readable
- **Toolchain**: `ronn` widely available and stable
- **Output Quality**: Produces proper groff output for system integration

### Directory Structure
```
_workbench/man/              # DEVELOPMENT ONLY (not in release)
├── reed.1.ronn              # Main man page source (Markdown)
├── reed-data.1.ronn         # Data commands man page
├── reed-layout.1.ronn       # Layout commands man page
├── reed-user.1.ronn         # User commands man page
├── reed-role.1.ronn         # Role commands man page
├── reed-taxonomy.1.ronn     # Taxonomy commands man page
├── reed-server.1.ronn       # Server commands man page
├── reed-build.1.ronn        # Build commands man page
└── README.md                # Man page build instructions

man/                         # RELEASE DIRECTORY (in final binary/package)
├── reed.1                   # Compiled main man page
├── reed-data.1              # Compiled data commands
├── reed-layout.1            # Compiled layout commands
├── reed-user.1              # Compiled user commands
├── reed-role.1              # Compiled role commands
├── reed-taxonomy.1          # Compiled taxonomy commands
├── reed-server.1            # Compiled server commands
└── reed-build.1             # Compiled build commands
```

**Important**: The `_workbench/` directory is for development only. Compiled `.1` man pages must be:
1. Generated during build process from `.ronn` sources
2. Placed in `man/` directory at project root
3. Included in release packages (binary distributions, deb, rpm, etc.)
4. Installed to system man path during package installation

## Implementation Files

### Main Man Page (`_workbench/man/reed.1.ronn`)

```markdown
reed(1) -- High-Performance Headless Rust CMS
==============================================

## SYNOPSIS

`reed` <command>:<action> [<args>...] [<flags>...]

## DESCRIPTION

ReedCMS is a high-performance headless CMS written in Rust, designed for 
speed, reliability, and developer experience. The `reed` command-line 
interface provides comprehensive access to all CMS operations.

## COMMANDS

  * `data`:
    Manage content data in ReedBase key-value store
    See reed-data(1) for details

  * `layout`:
    Create and manage page layouts
    See reed-layout(1) for details

  * `user`:
    User account management
    See reed-user(1) for details

  * `role`:
    Role and permission management
    See reed-role(1) for details

  * `taxonomy`:
    Taxonomy and content organisation
    See reed-taxonomy(1) for details

  * `server`:
    HTTP server operations
    See reed-server(1) for details

  * `build`:
    Build and asset compilation
    See reed-build(1) for details

## GLOBAL OPTIONS

  * `-h`, `--help`:
    Display help information

  * `-v`, `--version`:
    Display version information

  * `--verbose`:
    Enable verbose output

  * `--json`:
    Output in JSON format

  * `--dry-run`:
    Show what would be done without executing

## EXAMPLES

Get text content:

    $ reed data:get knowledge.title@en

Set text content with description:

    $ reed data:set knowledge.title@en "Knowledge Base" --desc "Page title"

Create new user:

    $ reed user:create admin --email admin@example.com --role admin

Start HTTP server:

    $ reed server:start --port 8333

## FILES

  * `.reed/`:
    ReedBase CSV database directory

  * `.reed/text.csv`:
    Text content storage

  * `.reed/routes.csv`:
    URL routing definitions

  * `.reed/meta.csv`:
    Metadata storage

  * `templates/`:
    MiniJinja template directory

## ENVIRONMENT

  * `REED_ENV`:
    Environment suffix (@dev, @prod, @christmas)
    Default: @dev

  * `REED_PORT`:
    Default server port
    Default: 8333

## EXIT STATUS

  * `0`:
    Successful execution

  * `1`:
    Error occurred (details written to stderr)

## SEE ALSO

reed-data(1), reed-layout(1), reed-user(1), reed-role(1), 
reed-taxonomy(1), reed-server(1), reed-build(1)

## AUTHOR

Vivian Voss <ask@vvoss.dev>

## COPYRIGHT

Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
SPDX-License-Identifier: Apache-2.0

## BUGS

Report bugs at: https://github.com/vvoss-dev/reedcms/issues

## WEBSITE

https://reedcms.dev
```

### Data Commands Man Page (`_workbench/man/reed-data.1.ronn`)

```markdown
reed-data(1) -- ReedCMS data management commands
================================================

## SYNOPSIS

`reed` `data:get` <key>
`reed` `data:set` <key> <value> [--desc <description>]
`reed` `data:list` [<pattern>]
`reed` `data:delete` <key>

## DESCRIPTION

Manage content data in ReedBase, the ReedCMS key-value store. All text 
content, metadata, and routing information is stored using dot-notation 
keys with language suffixes.

## COMMANDS

  * `data:get` <key>:
    Retrieve value for specified key
    Supports environment suffixes (@dev, @prod)
    Falls back to base key if environment-specific key not found

  * `data:set` <key> <value> [--desc <description>]:
    Store value for specified key
    Optionally add description for documentation

  * `data:list` [<pattern>]:
    List all keys matching optional pattern
    Pattern supports wildcards (* and ?)

  * `data:delete` <key>:
    Remove key from ReedBase
    Requires confirmation unless --force flag used

## KEY FORMAT

Keys use dot-notation with optional language suffix:

    namespace.component.property@language

Examples:

    page-header.logo.title@en
    page-header.logo.title@de
    knowledge.hero.subtitle@en

## EXAMPLES

Retrieve English page title:

    $ reed data:get knowledge.title@en

Set German page title with description:

    $ reed data:set knowledge.title@de "Wissensdatenbank" \
        --desc "German translation of page title"

List all English keys:

    $ reed data:list "*@en"

Delete development key:

    $ reed data:delete test.key@dev --force

## SEE ALSO

reed(1)

## AUTHOR

Vivian Voss <ask@vvoss.dev>
```

### Build Script Integration (`scripts/build-man-pages.sh`)

```bash
#!/usr/bin/env bash
# Build man pages from .ronn sources
# Output goes to man/ directory (included in release)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SOURCE_DIR="${PROJECT_ROOT}/_workbench/man"
TARGET_DIR="${PROJECT_ROOT}/man"  # Changed from target/man to man/

# Check for ronn
if ! command -v ronn &> /dev/null; then
    echo "Error: ronn not found. Install with: gem install ronn-ng"
    exit 1
fi

# Create target directory
mkdir -p "${TARGET_DIR}"

# Build all .ronn files
for ronn_file in "${SOURCE_DIR}"/*.ronn; do
    if [ -f "${ronn_file}" ]; then
        basename=$(basename "${ronn_file}" .ronn)
        echo "Building ${basename}..."
        ronn --roff --pipe "${ronn_file}" > "${TARGET_DIR}/${basename}"
    fi
done

echo "Man pages built successfully in ${TARGET_DIR}"
echo "These files are included in the release package"
```

### Cargo.toml Integration

```toml
[package.metadata]
# Man pages are in project root man/ directory
# These will be included in binary releases

[package.metadata.deb]
# Debian package configuration
assets = [
    ["target/release/reed", "usr/bin/", "755"],
    ["man/reed.1", "usr/share/man/man1/", "644"],
    ["man/reed-data.1", "usr/share/man/man1/", "644"],
    ["man/reed-layout.1", "usr/share/man/man1/", "644"],
    ["man/reed-user.1", "usr/share/man/man1/", "644"],
    ["man/reed-role.1", "usr/share/man/man1/", "644"],
    ["man/reed-taxonomy.1", "usr/share/man/man1/", "644"],
    ["man/reed-server.1", "usr/share/man/man1/", "644"],
    ["man/reed-build.1", "usr/share/man/man1/", "644"],
]

# Cargo install will need custom handling or documentation
# for man page installation to ~/.local/share/man/man1/
```

## Testing Requirements

### Manual Verification
```bash
# Build man pages (compiles to man/ directory)
./scripts/build-man-pages.sh

# View main man page
man ./man/reed.1

# View data commands man page
man ./man/reed-data.1

# Test man page search (after installation)
man -k reed

# Test apropos search (after installation)
apropos reed

# Verify all pages compiled
ls -lh man/*.1
```

### Quality Checklist
- [ ] All man pages compile without errors
- [ ] Formatting displays correctly in `man` viewer
- [ ] Cross-references work (SEE ALSO sections)
- [ ] Examples are accurate and tested
- [ ] Consistent terminology across all pages
- [ ] No spelling or grammar errors
- [ ] All commands documented
- [ ] All flags documented

## Installation Integration

### Debian/Ubuntu Package
```bash
# Install to /usr/share/man/man1/
install -D -m 644 target/man/reed.1 /usr/share/man/man1/reed.1
install -D -m 644 target/man/reed-data.1 /usr/share/man/man1/reed-data.1
# ... (other man pages)

# Update man database
mandb
```

### Homebrew Formula
```ruby
def install
  # ... (binary installation)
  
  # Install man pages
  man1.install Dir["target/man/*.1"]
end
```

### Cargo Install Hook
```toml
# Consider using cargo-install hook or post-install script
# to copy man pages to appropriate location
```

## Acceptance Criteria
- [ ] Main `reed.1.ronn` man page created with complete overview
- [ ] Separate man pages for each command namespace (data, layout, user, role, taxonomy, server, build)
- [ ] Build script `scripts/build-man-pages.sh` successfully compiles all pages
- [ ] All man pages display correctly with `man` command
- [ ] Cross-references between man pages work
- [ ] All examples tested and verified accurate
- [ ] Installation instructions documented in `_workbench/man/README.md`
- [ ] Man pages include proper headers (NAME, SYNOPSIS, DESCRIPTION, etc.)
- [ ] SEE ALSO sections cross-reference related commands
- [ ] Man pages follow standard Unix man page conventions

## Performance Targets
- Build time: < 2 seconds for all man pages
- Man page display: Instant (system-cached)

## Dependencies
- **ronn-ng**: Ruby gem for converting Markdown to groff
  - Install: `gem install ronn-ng`
  - Alternative: `ronn` (original version)

## Notes
- Man pages should be updated whenever command syntax changes
- Consider adding man page generation to CI/CD pipeline
- Man pages become part of official documentation strategy
- Future: Consider automated generation from CLI help text
- Future: Multi-language man pages (German translation)

## Release Integration

**Build Process**:
1. Developer edits `.ronn` source in `_workbench/man/`
2. Run `./scripts/build-man-pages.sh` to compile to `man/*.1`
3. Commit compiled `.1` files to git (they're part of the release)
4. Binary releases include `man/` directory

**Package Installation**:
- Debian/Ubuntu: Man pages → `/usr/share/man/man1/`
- Homebrew: Man pages → `$(brew --prefix)/share/man/man1/`
- Manual install: User copies `man/*.1` to `~/.local/share/man/man1/`

**.gitignore Considerations**:
```
# DO NOT ignore man/*.1 files!
# They must be in git for releases

# DO ignore _workbench/ for releases
_workbench/
```

## Implementation Order
1. Create `_workbench/man/README.md` with build instructions
2. Write main `reed.1.ronn` man page
3. Write `reed-data.1.ronn` as reference template
4. Create remaining command-specific man pages
5. Implement `scripts/build-man-pages.sh` build script
6. Test manual viewing with `man` command
7. Document installation integration
8. Update project documentation with man page references
