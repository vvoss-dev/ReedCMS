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

## Project Structure Overview

### Core Directories

```
ReedCMS/
├── .reed/                      # ReedBase Key:Value++ Database
│   ├── text.csv                # All content text (pipe-delimited)
│   ├── routes.csv              # URL routing definitions
│   ├── meta.csv                # SEO and technical metadata
│   ├── server.csv              # Server configuration
│   ├── project.csv             # Project settings
│   ├── registry.csv            # Layout registry
│   └── flow/                   # Dispatcher working data (persistent state)
│
├── templates/                  # MiniJinja templates + components
│   ├── components/
│   │   ├── atoms/              # Atomic components (icons, buttons)
│   │   ├── molecules/          # Component groups
│   │   └── organisms/          # Complex components (page-header, etc.)
│   │       └── {name}/
│   │           ├── {name}.{variant}.jinja    # mouse/touch/reader
│   │           ├── {name}.{variant}.css
│   │           └── {name}.text.csv           # Component-local text
│   └── layouts/                # Page layouts
│       └── {layout}/
│           ├── {layout}.{variant}.jinja
│           ├── {layout}.{variant}.css
│           └── {layout}.text.csv             # Layout-local text
│
├── src/reedcms/                # Rust implementation
│   ├── reedstream.rs           # Universal communication types
│   ├── reed/                   # Dispatchers (intelligent coordinators)
│   │   ├── reedbase.rs         # Data dispatcher
│   │   ├── reedcli.rs          # CLI dispatcher
│   │   ├── reedserver.rs       # Server dispatcher
│   │   └── reeddebug.rs        # Debug dispatcher
│   ├── reedbase/               # ReedBase services (get, set, init)
│   ├── cli/                    # CLI command services
│   ├── server/                 # Server services
│   ├── filters/                # MiniJinja filters (text, route, meta)
│   └── csv/                    # Universal CSV handler
│
└── _workbench/                 # Development resources
    ├── Archive/                # Project archives
    │   ├── Legacy/
    │   │   └── libs/           # Legacy system backups (archived source)
    │   └── ReedCMS/
    │       └── Planning/       # Archived planning documents from ReedCMS development
    └── Tickets/                # Implementation tickets (REED-XX-YY)
```

### Key Concepts
- **`.reed/`**: Central CSV database (pipe `|` delimited), single source of truth
- **`templates/`**: Atomic Design structure with variants (mouse/touch/reader)
- **`src/reedcms/`**: Rust implementation following KISS principle
- **Key format**: `lowercase.with.dots@lang` (e.g., `page.header.logo.title@de`)
- **Dispatchers**: Intelligent coordinators in `reed/` with persistence rights
- **Services**: Pure implementation in domain folders, no persistence

## Critical Safety Rules

### 0. Code Reuse - NEVER Duplicate Existing Functions
**⚠️ MANDATORY: ALWAYS CHECK FOR EXISTING FUNCTIONS BEFORE WRITING NEW CODE**

**Function Registry**: See `_workbench/Tickets/project_functions.csv` for complete list of all 984+ functions in the system.

**Rules:**
1. **BEFORE writing ANY function**, search the function registry:
   ```bash
   grep "function_name" _workbench/Tickets/project_functions.csv
   ```

2. **IF a function exists that does what you need**:
   - ✅ **USE the existing function** - import and call it
   - ❌ **DO NOT write a duplicate** - even with slight modifications
   - ❌ **DO NOT copy-paste and modify** - that creates unmaintainable code

3. **IF existing function is ALMOST what you need**:
   - ✅ **EXTEND the existing function** with additional parameters/logic
   - ✅ **ASK the user** if you should refactor the existing function
   - ❌ **DO NOT create a variant** - maintain single source of truth

4. **Common existing functions you MUST use**:
   - **CSV Operations**: `crate::reedcms::csv::{read_csv, write_csv}` - NEVER parse CSV manually
   - **Backup**: `crate::reedcms::backup::create_backup()` - NEVER write own backup logic
   - **Error Handling**: Use `ReedError` variants from `reedstream.rs` - NEVER create custom error types
   - **ReedBase Access**: Use `reedbase::get::{text, route, meta, server}` - NEVER read CSV directly for data access

5. **Examples of VIOLATIONS**:
   - ❌ Writing custom CSV parsing when `csv::read_csv()` exists
   - ❌ Duplicating `create_backup()` logic in multiple files
   - ❌ Creating `get_text_custom()` when `reedbase::get::text()` exists
   - ❌ Manual file operations when `csv::write_csv()` does atomic writes

6. **Check these locations for common needs**:
   - **CSV handling**: `src/reedcms/csv/` (read, write, parse, create)
   - **Backup/Restore**: `src/reedcms/backup/` (create, list, restore, cleanup)
   - **Data access**: `src/reedcms/reedbase/` (get, set, init)
   - **Error types**: `src/reedcms/reedstream.rs` (ReedError, ReedResult)
   - **CLI commands**: `src/reedcms/cli/` (all command implementations)
   - **API handlers**: `src/reedcms/api/` (GET, SET, batch, list)

**Penalty for violation**: Code review rejection + mandatory refactoring to use existing functions.

**WHY this rule exists**: The API SET handlers incident where 200+ lines of duplicate CSV code was written instead of using the existing 2-line `csv::read_csv()` and `csv::write_csv()` functions. This wasted tokens, created maintenance burden, and violated DRY principle.

---

### 0.5. Tracking File Maintenance - ALWAYS Keep Updated

**⚠️ MANDATORY: Proactively update tracking files during development WITHOUT being asked**

These files track project state and MUST be kept current:

**When creating/modifying tickets:**
1. ✅ **Update** `_workbench/Tickets/ticket-index.csv`
   - Add new tickets with status (Planned/Open/Complete)
   - Update existing ticket status when changed
   - Include: ticket_id, status, title, layer, priority, complexity, depends_on, summary_reference, description

2. ✅ **Update** `_workbench/Tickets/TICKET-STATUS.md`
   - Add/update ticket section in relevant layer
   - Update executive summary (total counts, percentages)
   - Update "Last Updated" date to current date
   - Include commit hashes for completed tickets

**When implementing new public functions:**
1. ✅ **Update** `_workbench/Tickets/project_functions.csv`
   - Add: file_path, line_number, function_name, signature, description, inputs, return_type, status
   - Only for PUBLIC functions (not internal helpers)

**When doing significant work (bugfix, feature, refactor, investigation >30min):**
1. ✅ **Create** `_workbench/Log/YYMMDD-PXX.csv` (e.g., 251013-P05.csv)
   - Document: process_id, step, phase, action, file, description, result, commit_hash, notes
   - Phases: investigation, architecture, decision, implementation, verification, commit, documentation

2. ✅ **Update** `_workbench/Log/INDEX.csv`
   - Add entry: process_id, title, category, tags, related_processes, files_affected, commits, status, duration_steps, summary
   - Use lowercase-with-hyphens tags for searchability

**Default Behavior:**
- ✅ **DO update** these files proactively during relevant work
- ✅ **DO create** process logs for non-trivial work
- ❌ **DO NOT wait** for user to ask for updates
- ❌ **DO NOT skip** tracking to save tokens

**User will tell you IF:**
- They don't want tracking for specific trivial work
- They want to batch updates later

**Benefits:**
- Persistent project knowledge across sessions
- Searchable history of decisions and implementations
- Onboarding resource for new team members
- Audit trail for debugging

---

### 1. File Operation Safety
**⚠️ CRITICAL: ALWAYS ASK USER BEFORE DESTRUCTIVE OPERATIONS**

- **NEVER execute `rm` commands without explicit user approval**
  - `rm -rf` operations
  - `rm` of any file or directory
  - Any destructive file operations
  - Cleaning build directories that may contain important files

- **NEVER use `sed -i` or in-place modifications** across multiple files
- **NEVER use `mv` on project files** without confirmation
- **Test on ONE file first**, validate, THEN proceed to others
- **Work on copies in `/tmp` first** for risky operations
- **Create backups** before any batch operation affecting >3 files

**Example of correct behaviour:**
```
❌ WRONG: Executing `rm -rf target/` without asking
✅ RIGHT: "I need to clean the target/ directory. May I execute `rm -rf target/`?"
```

### 2. Respect Existing Code & Decisions
- **DO NOT "improve" or "optimise" code** unless explicitly asked
- **PRESERVE existing code style, patterns, and paradigms** - they exist for reasons
- **DO NOT change formatting, whitespace, or structure** when asked to modify content only
- **Keep original comments and documentation** unless specifically told to update them
- **Ask before introducing new patterns** or paradigms not present in the codebase
- **Respect verbose/explicit code** - it's often intentional, not a mistake to "fix"

**⚠️ ABSOLUTELY FORBIDDEN WITHOUT EXPLICIT PERMISSION:**
- **NEVER delete code, functions, or visual elements** (icons, images, styling, HTML elements)
- **NEVER remove template content** (text, structure, components)
- **NEVER simplify existing implementations** to "fix" perceived complexity
- **NEVER change working code** when encountering errors - ASK first
- **NEVER replace existing patterns** with "better" solutions from your training

**If something doesn't work:**
1. ✅ STOP immediately
2. ✅ ASK the user what to do
3. ✅ WAIT for instructions
4. ❌ DO NOT invent your own solution
5. ❌ DO NOT delete "problematic" code
6. ❌ DO NOT simplify to "make it work"

### 3. Instruction Adherence
- **ONLY change what was requested** - no bonus "improvements"
- **Confirm scope** before starting batch operations
- **If instructions seem ambiguous**, ASK before proceeding
- **Stop and report** if you realise you're deviating from instructions
- **Re-read ticket requirements** before marking task complete

### 4. Context & Session Management
**Re-read these rules every 20 minutes** to prevent information rot:
- Project uses: Rust, MiniJinja, pipe-delimited CSVs
- Key format: `lowercase.with.dots@lang`
- CSV files: `.reed/text.csv`, `.reed/routes.csv`, `.reed/meta.csv`
- Safety: Ask before `rm`/`sed -i`/`mv`
- Style: Conservative > Clever
- Commits: `[REED-XX-YY] – type: description`

**Before large refactorings**: Explicitly confirm project constraints.

### 5. Batch Operations Protocol
**For operations affecting >10 files**:
1. Process 2-3 files as proof-of-concept
2. Show results for review
3. Wait for confirmation
4. Continue with remaining files in batches of 20 max

- **Create commits every 10-20 files** for rollback capability
- **If error occurs**: STOP immediately, report, don't continue batch
- **NEVER process 50+ files** without intermediate check-ins

### 6. Quality Checklist - Before Marking Task Complete
Ask yourself:
- [ ] Did I ONLY change what was requested?
- [ ] Did I preserve the project's code style?
- [ ] Did I test risky operations first?
- [ ] Did I maintain all original comments/docs?
- [ ] Did I follow project patterns, not generic best practices?
- [ ] Can every change be easily reviewed and understood?

**If ANY answer is "No" or "Unsure"**: STOP and report before proceeding.

**Remember**: You are an assistant to an experienced developer.
- **Their decisions > Your training**
- **Their patterns > "Best practices"**
- **Safety > Speed**
- **Conservative > Clever**
- **When in doubt: ASK. Never assume.**

### 7. Risk Assessment Matrix

Before ANY operation, evaluate:

| Action | Risk | Required Safety Measure |
|--------|------|-------------------------|
| Read files | Low | None |
| Modify 1-3 files | Low | Review diffs |
| Modify 4-10 files | Medium | Batch review, test first |
| Modify >10 files | High | `/tmp` test, 3-file proof-of-concept |
| `rm` any file | CRITICAL | Explicit user approval REQUIRED |
| `sed -i` >1 file | CRITICAL | Forbidden - use editor APIs |
| Change architecture | CRITICAL | Discuss first, document reasoning |

### 8. Training vs Project Patterns

**⚠️ YOUR TRAINING IS NOT ALWAYS RIGHT FOR THIS PROJECT**

**ReedCMS-Specific Patterns** (from `service-template.md` and `project_summary.md`):

| Your Training Says | ReedCMS Project Uses | Reason |
|-------------------|---------------------|--------|
| "Use `?` for error propagation" | `.map_err()` with context | Rich error messages: `.map_err(\|e\| ReedError::SpecificError { context })` |
| "Use inline `#[cfg(test)]` modules" | Separate `_test.rs` files | Clear separation, better organisation |
| "Generic error types are fine" | Specific `ReedError` variants | Detailed error context for debugging |
| "Clone when needed" | Explicit borrowing with `&` | Performance: zero allocations in hot paths |
| "Use `impl Trait` for simplicity" | Explicit type signatures | Clarity and documentation |
| "Simplify verbose validation" | Explicit validation steps | One function = one job (KISS principle) |
| "Generic file names are fine" | Specific responsibility names | `get.rs`, `set.rs` NOT `handler.rs`, `utils.rs` |
| "One test file is enough" | One `_test.rs` per source file | `get.rs` → `get_test.rs` (mirrored structure) |
| "Return types can be simple" | Always `ReedResult<ReedResponse<T>>` | Standardised interface with metrics/caching |

**Additional Project Patterns:**
- **Error handling**: Always `ReedResult<T>` (alias for `Result<T, ReedError>`) with specific variants, never generic `anyhow::Error`
- **Response format**: Always `ReedResponse<T>` with `source`, `cached`, `timestamp`, optional `metrics`
- **Documentation**: Mandatory sections: `## Input`, `## Output`, `## Performance`, `## Error Conditions`, `## Example Usage`
- **CSV operations**: Atomic writes via temp file + rename, never in-place modification
- **File headers**: Mandatory copyright + AI guidelines + file purpose in every `.rs` file
- **Module trait**: All modules implement `ReedModule` trait with `module_name()`, `health_check()`, `version()`
- **Backup implementation**: CSV services must call `create_backup()` before writes (XZ-compressed, keep 32)

**Rule**: When project pattern differs from your training → **PROJECT WINS**.

**Red flag**: If you think "I should improve this" → STOP and ask "Did they request improvements?"

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

### System Setup and Installation

**Setup Script** (single `scripts/setup.sh`):

ReedCMS uses **one setup script** controlled by the `ENVIRONMENT` variable in `.env`:

**Development Mode** (`ENVIRONMENT=dev` in `.env`):
- Creates symlinks: `/usr/local/bin/reed` → `target/release/reed`
- Man pages: symlinks to `src/man/*.1`
- Auto-updates when you run `cargo build --release`
- Requires: sudo (for `/usr/local/bin` access)
- **Best for**: Active development

**Production Mode** (`ENVIRONMENT=prod` in `.env`):
- Copies binary to `/usr/local/bin/reed` (755 permissions)
- Copies man pages to `/usr/local/share/man/man1/` (644 permissions)
- Stable installation, no auto-updates
- Requires: sudo
- **Best for**: Production deployment

**Quick Setup**:
```bash
# 1. Build binary
cargo build --release

# 2. Check .env (should be ENVIRONMENT=dev for development)
cat .env

# 3. Run setup (installs reed command system-wide)
./scripts/setup.sh
# → Will ask for sudo password

# 4. Verify installation
reed --version
man reed

# 5. Start development server
reed server:io --port 8333
# → Server runs on http://localhost:8333
```

**IMPORTANT**: Always use `reed` command, never `./target/release/reed` directly. The setup script ensures `reed` is always available system-wide.

**Uninstall**:
```bash
sudo ./scripts/uninstall.sh
```

**Switch modes**: Edit `.env`, run `sudo ./scripts/uninstall.sh`, then `./scripts/setup.sh`

See `scripts/README.md` for detailed documentation.

---

### MANDATORY Code Standards
Every implementation MUST follow these standards:

1. **Language**: All code comments and docs in BBC English
2. **Principle**: KISS (Keep It Simple, Stupid)
3. **File Naming**: One file = one clear responsibility
4. **Functions**: One function = one distinctive job
5. **Testing**: Separate test files as `{name}_test.rs` - never inline `#[cfg(test)]` modules
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
│   │   ├── specific_feature_test.rs
│   │   └── another_feature.rs
```

### Service Implementation
- See `_workbench/Tickets/templates/service-template.md` for complete guide
- See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Process Logging System

**⚠️ MANDATORY: ALL significant development activities MUST be logged**

### When to Create a Process Log

Create a new process log (PXXX) for:
- ✅ Bug fixes (non-trivial)
- ✅ New features or functionality
- ✅ Architectural changes
- ✅ Refactoring affecting multiple files
- ✅ Performance optimizations
- ✅ Security fixes
- ✅ Complex investigations or debugging sessions

**Skip logging for:**
- ❌ Trivial typo fixes
- ❌ Simple documentation updates (unless part of larger process)
- ❌ Single-line code changes

### Process Log Structure

**Location:** `_workbench/Log/`

**Two-level system:**
1. **INDEX.csv** - Quick overview, one line per process
2. **PXXX-YYMMDD-HHMM.csv** - Detailed step-by-step log

### Workflow: Creating a Process Log

#### Step 1: Start New Process
```bash
# 1. Determine next process ID
grep "^P" _workbench/Log/INDEX.csv | tail -1  # Get last ID, increment

# 2. Create detailed log file
# Format: PXXX-YYMMDD-HHMM.csv
# Example: P002-251013-0830.csv
```

#### Step 2: Research Existing Processes (MANDATORY)

**⚠️ BEFORE starting, ALWAYS check for related work:**

```bash
# 1. Search by tags (fastest, most reliable)
grep -E "your-topic|your-component" _workbench/Log/INDEX.csv

# 2. Search by files you'll modify
grep "your_file.rs" _workbench/Log/INDEX.csv

# 3. Check for in-progress work
grep "in-progress" _workbench/Log/INDEX.csv

# 4. Read relevant detailed logs to understand decisions
cat _workbench/Log/PXXX-YYMMDD-HHMM.csv | grep "architecture\|decision"
```

**Why:** Avoid conflicts, understand existing decisions, maintain consistency.

#### Step 3: Add to INDEX
```csv
# _workbench/Log/INDEX.csv
process_id|date|title|category|tags|related_processes|files_affected|commits|status|duration_steps|summary
P002|2025-10-13|Brief descriptive title|bugfix|tag1,tag2,tag3|P001|file1.rs, file2.rs|n/a|in-progress|0|What you're about to do
```

**Categories:** bugfix, feature, refactor, architecture, documentation, performance, security

**Tags:** 3-7 lowercase-with-hyphens keywords (e.g., `api`, `language`, `set-handlers`, `dead-code`)
- Technical: `api`, `reedbase`, `cache`, `csv`
- Functional: `language`, `environment`, `routing`
- Component: `set-handlers`, `get-handlers`, `batch-operations`
- Issue: `dead-code`, `performance`, `bug`, `security-fix`

**Related Processes:** Comma-separated process IDs or "n/a"
- Link to processes you're continuing/extending
- Link to processes you're modifying/fixing
- Link to processes with conflicting decisions

#### Step 4: Document Each Step
```csv
# _workbench/Log/251013-P02.csv
process_id|step|phase|action|file|description|result|commit_hash|notes
251013-P02|01|investigation|analyze_problem|src/file.rs|What I'm doing|What I found|n/a|Additional context
251013-P02|02|implementation|create_function|src/file.rs|Added function X|SUCCESS|abc1234|Technical details
```

**Phases:**
- `investigation` - Problem analysis, root cause finding
- `architecture` - System understanding, design decisions
- `decision` - Choosing approach, documenting rationale
- `implementation` - Writing code, making changes
- `verification` - Testing, validation, checks
- `commit` - Git operations
- `documentation` - Creating/updating docs

#### Step 5: Update INDEX on Completion
```csv
251013-P02|Title|bugfix|api,language,fix|251013-P01|file1.rs, file2.rs|abc1234,def5678|completed|12|Final summary of what was achieved
```

### Format Specifications

**Process ID Format:** `YYMMDD-PXX`
- `YYMMDD` - Date (251013 = 2025-10-13)
- `PXX` - Daily counter (P01, P02, P03, etc.)
- Example: `251013-P01` = October 13, 2025, first process
- Example: `251014-P01` = October 14, 2025, first process

**Filename:** `YYMMDD-PXX.csv` (e.g., `251013-P01.csv`, `251013-P02.csv`)

**INDEX.csv columns (10 total):**
1. `process_id` - YYMMDD-PXX format (date encoded in ID)
2. `title` - Brief title (< 80 chars)
3. `category` - bugfix/feature/refactor/architecture/documentation/performance/security
4. `tags` - 3-7 searchable keywords (comma-separated, lowercase-with-hyphens)
5. `related_processes` - Related process IDs (comma-separated, e.g., "251013-P01,251014-P01") or "n/a"
6. `files_affected` - Main files changed (comma-separated)
7. `commits` - Git commit hashes (comma-separated)
8. `status` - completed/in-progress/paused/abandoned
9. `duration_steps` - Total number of steps in detailed log
10. `summary` - One-line summary of what was done

**Detailed log columns:**
1. `process_id` - YYMMDD-PXX (matches INDEX)
2. `step` - Sequential number (01, 02, 03, ...)
3. `phase` - Process phase (see list above)
4. `action` - Specific action (analyze_problem, create_function, etc.)
5. `file` - File(s) affected or "all"
6. `description` - What you did (detailed)
7. `result` - Outcome (SUCCESS, FAILED, description)
8. `commit_hash` - Git hash or "n/a"
9. `notes` - Additional context, technical details

### Benefits of Process Logging

✅ **Audit Trail** - Complete history of why changes were made
✅ **Maintenance Reference** - Understand past decisions during future work
✅ **Bug Investigation** - Trace back reasoning for implementations
✅ **Knowledge Transfer** - Onboarding, team collaboration
✅ **Compliance** - Document decision-making process
✅ **Learning Resource** - Reference for similar problems

### Research Examples

**Before modifying SET handlers:**
```bash
# 1. Find all SET handler work
grep "set-handler" _workbench/Log/INDEX.csv
# Result: 251013-P01 - Symmetrical language/environment

# 2. Read 251013-P01's architecture decisions
cat _workbench/Log/251013-P01.csv | grep "architecture\|decision"

# 3. Check if anyone modified 251013-P01 after
grep "related_processes.*251013-P01" _workbench/Log/INDEX.csv
```

**Token-efficient search:**
```bash
# DON'T: Read all logs blindly (wastes tokens)
# DO: Use tags to find relevant processes first
grep "authentication" _workbench/Log/INDEX.csv

# Then read ONLY those specific detailed logs
```

**Multi-tag search (AND logic):**
```bash
# Find processes touching BOTH api AND language
grep "api" _workbench/Log/INDEX.csv | grep "language"
```

### Complete Documentation

See `_workbench/Log/README.md` for complete process log system documentation.

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
- Separate test files (`_test.rs`)
- Unit tests, integration tests, performance tests

### Test Structure
```rust
// my_feature_test.rs
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

## Configuration Management (REED-04-12)

### Configuration Architecture

**CRITICAL: CSV files are the single source of truth at runtime!**

```
┌──────────────────────────────────────────────────────────────┐
│ Configuration Priority & Data Flow                           │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Reed.toml.example     Reed.toml          .reed/*.csv        │
│  (documentation)       (bootstrap)        (runtime truth)    │
│        │                   │                     │           │
│        │                   │                     │           │
│        └──[read]──────┐    │                     │           │
│                       ▼    │                     │           │
│                   Developer ────[config:sync]───►│           │
│                       │    └─────[--force]──────►│           │
│                       │                          │           │
│                       │◄─────[config:export]─────┤           │
│                       │                          │           │
│                       └─────[set:project]───────►│           │
│                       └─────[set:server]────────►│           │
│                                                  │           │
│                       Server Start ───[reads]───►│           │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Configuration Files

1. **`Reed.toml.example`** (Documentation)
   - Complete reference with ALL available options
   - Includes defaults, descriptions, usage examples
   - NEVER edited directly - for reference only
   - Versioned in git

2. **`Reed.toml`** (Bootstrap Configuration)
   - Minimal file with only project-specific overrides
   - Used ONLY during `reed config:sync` command
   - Can be versioned in git for team sharing
   - NOT read during server startup

3. **`.reed/project.csv`** and **`.reed/server.csv`** (Runtime Truth)
   - Single source of truth during runtime
   - Read by server on startup
   - Modified by CLI commands (`set:project`, `set:server`)
   - NOT versioned in git (local configuration)
   - Backed up automatically before modifications

4. **`.env`** (Environment Control)
   - Controls server binding mode
   - `ENVIRONMENT=dev` → localhost:8333 (HTTP)
   - `ENVIRONMENT=prod` → /tmp/reed.sock (Unix socket)

### Configuration Workflow

#### Initial Project Setup
```bash
# 1. Copy settings you need from Reed.toml.example to Reed.toml
cp Reed.toml.example Reed.toml
vim Reed.toml  # Edit your settings

# 2. Sync to CSV files (creates .reed/*.csv)
reed config:sync --force

# 3. CSV files are now active - server will read from them
reed server:start
```

#### Runtime Configuration Changes (Recommended)
```bash
# Change a setting via CLI (writes directly to CSV)
reed set:project name "My New Project Name"

# Change server setting
reed set:server workers 8

# Changes are IMMEDIATE - restart server to apply
reed server:restart
```

#### Backup Current Configuration
```bash
# Export current CSV values to Reed.toml
reed config:export --force

# Result: Reed.toml now contains current CSV values
# Can be committed to git for backup/versioning
```

#### Reset to Defaults
```bash
# WARNING: This overwrites CSV with Reed.toml values!
reed config:export           # Backup first!
reed config:sync --force     # Reset from Reed.toml
```

### Configuration Commands

| Command | Direction | Purpose | Safety |
|---------|-----------|---------|--------|
| `reed config:sync` | TOML → CSV | Bootstrap from Reed.toml | ⚠️ Overwrites CSV! Shows warning |
| `reed config:export` | CSV → TOML | Backup CSV to Reed.toml | ✅ Safe (only writes TOML) |
| `reed set:project` | Direct → CSV | Runtime config change | ✅ Safe (immediate, no TOML) |
| `reed set:server` | Direct → CSV | Runtime config change | ✅ Safe (immediate, no TOML) |
| `reed config:show` | Read CSV | Display current config | ✅ Read-only |
| `reed config:validate` | Read TOML | Validate Reed.toml syntax | ✅ Read-only |

### Important Rules

1. **Server startup NEVER reads Reed.toml** - only reads `.reed/*.csv`
2. **CLI commands NEVER write to Reed.toml** - only write to `.reed/*.csv`
3. **`config:sync` is DESTRUCTIVE** - shows warning, requires `--force`
4. **CSV backups are automatic** - 32 XZ-compressed backups kept
5. **Always use `config:export` before `config:sync`** to avoid data loss

### Example Scenarios

**Scenario 1: Team Configuration Sharing**
```bash
# Developer A: Share configuration with team
reed config:export --force     # CSV → Reed.toml
git add Reed.toml
git commit -m "Update project configuration"
git push

# Developer B: Apply team configuration
git pull
reed config:sync --force       # Reed.toml → CSV
reed server:restart
```

**Scenario 2: Production Deployment**
```bash
# Local development
reed set:project url "https://example.com"
reed config:export --force     # Save to Reed.toml

# Deploy to production
scp Reed.toml server:/app/
ssh server
cd /app
reed config:sync --force       # Apply production config
ENVIRONMENT=prod reed server:start
```

**Scenario 3: Lost Configuration Recovery**
```bash
# If .reed/*.csv files are lost/corrupted
reed config:sync --force       # Restore from Reed.toml
# All values from Reed.toml written to CSV
```

## Project-Specific Notes

### CSV Format
- **Delimiter**: pipe (`|`)
- **Format**: `key|value|description`
- **All files in `.reed/` directory**
- **Automatic backups before modifications**

### Key Nomenclature (MANDATORY)
- **Dot-notation EVERYWHERE**: `lowercase.with.dots` (NO hyphens in keys!)
- **Directory names**: Use hyphens (e.g., `page-header/`, `landing-hero/`)
- **CSV keys**: Use dots only (e.g., `page.header.logo.title`, `landing.hero.headline`)
- **Sub-layouts**: Flat structure (`agility.title`, NOT `knowledge.agility.title`)
- **Component namespacing**: `page.header.logo.title` (NOT `page-header.logo.title`)
- **Nesting depth**: Optimal 4, maximum 8 levels
- **Language suffix**: Lowercase after key (`@de`, `@en`, NOT `@DE`, `@EN`)
- **Example mapping**: Directory `page-header/` → Keys `page.header.*`

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
