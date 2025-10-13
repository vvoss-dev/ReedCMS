# ReedCMS Process Log System

This directory contains detailed process logs for significant development activities, bug fixes, and architectural changes.

## Structure

### `INDEX.csv` - Quick Overview
**Purpose:** Fast lookup and search across all processes
**Columns:**
- `process_id` - Unique identifier (P001, P002, etc.)
- `date` - Date of process (YYYY-MM-DD)
- `title` - Brief descriptive title
- `category` - Type of work (bugfix, feature, refactor, architecture, documentation)
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

## Usage Examples

### Finding a specific fix:
```bash
# Search INDEX for keywords
grep -i "language" _workbench/Log/INDEX.csv

# Once found, open the detailed log
cat _workbench/Log/P001-251013-0723.csv
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
