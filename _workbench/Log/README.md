# ReedCMS Process Log System

This directory contains detailed process logs for significant development activities, bug fixes, and architectural changes.

## Structure

### `INDEX.csv` - Quick Overview
**Purpose:** Fast lookup and search across all processes with tag-based references
**Columns:**
- `process_id` - Unique identifier (P001, P002, etc.)
- `date` - Date of process (YYYY-MM-DD)
- `title` - Brief descriptive title
- `category` - Type of work (bugfix, feature, refactor, architecture, documentation)
- `tags` - Searchable keywords (comma-separated, lowercase-with-hyphens)
- `related_processes` - Related process IDs (comma-separated, e.g., "P003,P007" or "n/a")
- `files_affected` - Comma-separated list of main files changed
- `commits` - Git commit hash(es)
- `status` - Current status (completed, in-progress, paused, abandoned)
- `duration_steps` - Total number of steps in process
- `summary` - One-line summary of what was done

### `PXXX-YYMMDD-HHMM.csv` - Detailed Process Log
**Purpose:** Step-by-step documentation of a specific process
**Columns:**
- `process_id` - Links back to INDEX
- `step` - Sequential step number (01, 02, 03, etc.)
- `phase` - Process phase (investigation, architecture, decision, implementation, verification, commit, documentation)
- `action` - Specific action taken
- `file` - File(s) affected by this step
- `description` - Detailed description of what was done
- `result` - Outcome of the action
- `commit_hash` - Git commit if applicable, otherwise "n/a"
- `notes` - Additional context, technical details, decisions made

## Process Phases

1. **investigation** - Problem analysis, root cause identification, understanding current state
2. **architecture** - System analysis, understanding relationships, documenting architecture
3. **decision** - Choosing between approaches, user/team decisions, documenting rationale
4. **implementation** - Code changes, function creation, integration work
5. **verification** - Testing, compilation checks, validation
6. **commit** - Git operations (staging, committing, pushing)
7. **documentation** - Creating/updating docs, process logs, comments

## Tag System

### Purpose
Tags enable fast, token-efficient discovery of related processes without reading all logs.

### Tag Guidelines
- **Format:** lowercase-with-hyphens (e.g., `api`, `set-handlers`, `dead-code`)
- **Granularity:** 3-7 tags per process
- **Types:**
  - **Technical:** `api`, `reedbase`, `cache`, `csv`, `authentication`
  - **Functional:** `language`, `environment`, `routing`, `templates`
  - **Component:** `set-handlers`, `get-handlers`, `batch-operations`
  - **Issue:** `dead-code`, `performance`, `bug`, `security-fix`
  - **Layer:** `reed-02`, `reed-06` (for ticket-related work)

### Common Tags
- `api` - API endpoint changes
- `reedbase` - ReedBase core functionality
- `language` - Language handling
- `environment` - Environment system
- `dead-code` - Dead code cleanup
- `set-handlers` - SET operation handlers
- `get-handlers` - GET operation handlers
- `cache` - Cache system
- `csv` - CSV file operations
- `authentication` - Auth system
- `routing` - URL routing
- `templates` - Template system
- `security` - Security-related
- `performance` - Performance optimization

## Reference System

### Linking Related Processes

**Use `related_processes` column when:**
1. **Continuation:** Process continues/extends previous work (P005 continues P003)
2. **Conflict:** Process contradicts/reverses previous decision (P008 conflicts with P002)
3. **Related:** Process affects same system/files (P012 related to P007)

**Format:** Comma-separated process IDs
```csv
related_processes
P003,P007      # Related to both P003 and P007
P005           # Related to P005 only
n/a            # No related processes
```

### ⚠️ MANDATORY: Check Before Starting New Work

**Before implementing ANY change, ALWAYS check for existing processes:**

```bash
# Step 1: Search by tags (fastest, most reliable)
grep -E "language|environment|set-handler" _workbench/Log/INDEX.csv

# Step 2: Search by file (find all processes touching same files)
grep "batch_handlers.rs" _workbench/Log/INDEX.csv

# Step 3: Search by category + status (find in-progress work)
grep "bugfix.*in-progress" _workbench/Log/INDEX.csv

# Step 4: Follow related_processes chain
# If P007 shows "related_processes: P003,P005", check those too
grep "^P003\|^P005" _workbench/Log/INDEX.csv
```

### Research Workflow

#### Scenario 1: New Ticket - Check for Conflicts
```bash
# New ticket: "Implement environment-specific routing"
# Tags to search: environment, routing

# 1. Find all environment-related processes
grep "environment" _workbench/Log/INDEX.csv

# 2. Find all routing-related processes  
grep "routing" _workbench/Log/INDEX.csv

# 3. Check their detailed logs for decisions
cat _workbench/Log/P001-251013-0723.csv | grep -i "decision\|architecture"

# 4. Check related_processes chain
# If P001 shows related: P005, check P005 too
```

#### Scenario 2: Bug in Existing Feature
```bash
# Bug: "SET handler not saving language correctly"
# Component: set-handlers, language

# 1. Find processes that touched set-handlers AND language
grep "set-handler" _workbench/Log/INDEX.csv | grep "language"

# Result: P001 - Symmetrical language/environment in SET handlers
# 2. Read P001 to understand original implementation
cat _workbench/Log/P001-251013-0723.csv

# 3. Check if anyone modified it after (related_processes)
grep "related_processes.*P001" _workbench/Log/INDEX.csv
```

#### Scenario 3: Extending Existing System
```bash
# Task: "Add @season suffix support (christmas, easter)"
# This extends environment system

# 1. Find environment system processes
grep "environment" _workbench/Log/INDEX.csv
# Result: P001

# 2. Read architecture decisions
cat _workbench/Log/P001-251013-0723.csv | grep "architecture"

# 3. New process MUST reference P001
# In new P002:
# related_processes: P001
# tags: environment,season,suffix,extension
```

## Usage Examples

### Finding a specific fix:
```bash
# Search INDEX by tags (recommended - fastest)
grep "dead-code" _workbench/Log/INDEX.csv

# Search by file
grep "batch_handlers.rs" _workbench/Log/INDEX.csv

# Search by multiple tags (AND logic)
grep "api" _workbench/Log/INDEX.csv | grep "language"

# Once found, open the detailed log
cat _workbench/Log/P001-251013-0723.csv
```

### Token-Efficient Research:
```bash
# DON'T: Read all logs looking for language stuff (token-expensive)
# DO: Use tags to find relevant processes first
grep "language" _workbench/Log/INDEX.csv

# This returns ONLY relevant process IDs, then read those specific logs
```

### Creating a new process log:
```bash
# 1. Add entry to INDEX.csv with new process_id
# 2. Create detailed log: PXXX-YYMMDD-HHMM.csv
# 3. Document each step as you work
# 4. Update INDEX status when complete
```

### Categories:
- **bugfix** - Fixing bugs, resolving errors
- **feature** - New functionality
- **refactor** - Code restructuring without behavior change
- **architecture** - System design, structure changes
- **documentation** - Docs, comments, guides
- **performance** - Optimization work
- **security** - Security fixes, hardening

## Benefits

✅ **Audit Trail** - Complete history of why changes were made
✅ **Learning Resource** - Reference for similar problems
✅ **Onboarding** - New team members understand decisions
✅ **Debugging** - Trace back reasoning for implementations
✅ **Compliance** - Document decision-making process

## Naming Convention

Process logs use format: `PXXX-YYMMDD-HHMM.csv`
- `PXXX` - Process ID (P001, P002, etc.)
- `YYMMDD` - Date (251013 = 2025-10-13)
- `HHMM` - Start time (0723 = 07:23)

Example: `P001-251013-0723.csv` = Process 1, started October 13, 2025 at 07:23

## Maintenance

- Keep INDEX.csv sorted by date (newest first)
- Archive old logs annually to `Archive/YYYY/`
- Update status in INDEX when returning to paused processes
- Link related processes in notes column
