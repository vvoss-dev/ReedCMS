# REED-19-24B: CLI Tool (ReedQCommand)

**Parent**: REED-19-24 (High-Level Database API & CLI)  
**Status**: Open  
**Priority**: High  
**Complexity**: Medium  
**Depends On**: REED-19-24A (Database API)  
**Layer**: REED-19 (ReedBase)

## Overview

Implement the `reedbase` command-line tool (ReedQCommand) that provides terminal access to ReedBase operations. This is the user-facing CLI interface that mirrors ReedQL functionality and allows interactive database operations.

## Motivation

**Current State**: Database API exists but no way to use it from terminal  
**Problem**: Developers cannot interact with ReedBase without writing Rust code  
**Solution**: CLI tool with commands and interactive shell

## Goals

1. ✅ **CLI Binary** - `reedbase` command with subcommands
2. ✅ **Query Interface** - Execute SELECT queries from command line
3. ✅ **Execute Interface** - Run INSERT/UPDATE/DELETE commands
4. ✅ **Interactive Shell** - REPL for continuous operations
5. ✅ **Output Formats** - Table, JSON, CSV output modes
6. ✅ **Table Management** - Create/list/drop tables
7. ✅ **Index Management** - Create/list/drop indices

## Non-Goals

- ❌ Server mode (that's for later)
- ❌ Authentication/permissions (single-user tool for now)
- ❌ Query history persistence (nice-to-have)
- ❌ Syntax highlighting (nice-to-have)

## Architecture

```
reedbase (binary)
├── main.rs
│   ├── parse CLI args (clap)
│   └── dispatch to commands
│
├── commands/
│   ├── query.rs       # reedbase query <SQL> <PATH>
│   ├── exec.rs        # reedbase exec <SQL> <PATH>
│   ├── shell.rs       # reedbase shell <PATH>
│   ├── tables.rs      # reedbase tables <PATH>
│   ├── indices.rs     # reedbase indices <PATH>
│   ├── stats.rs       # reedbase stats <PATH>
│   └── explain.rs     # reedbase explain <SQL> <PATH>
│
├── formatters/
│   ├── table.rs       # ASCII table output
│   ├── json.rs        # JSON output
│   └── csv.rs         # CSV output
│
└── shell/
    ├── repl.rs        # Interactive shell loop
    ├── prompt.rs      # Prompt display
    └── history.rs     # Command history (in-memory)
```

## Commands

### 1. `reedbase query`

Execute a SELECT query.

```bash
reedbase query <SQL> <PATH> [OPTIONS]

Arguments:
  <SQL>     ReedQL SELECT query (quoted)
  <PATH>    Path to ReedBase directory (e.g., .reed)

Options:
  -f, --format <FORMAT>   Output format: table|json|csv [default: table]
  -o, --output <FILE>     Write output to file
  --no-header             Omit header row (CSV only)
  -h, --help              Print help

Examples:
  reedbase query "SELECT * FROM text WHERE key LIKE '%.@de'" .reed
  reedbase query "SELECT COUNT(*) FROM text" .reed --format json
  reedbase query "SELECT key, value FROM text LIMIT 10" .reed -f csv -o output.csv
```

### 2. `reedbase exec`

Execute INSERT/UPDATE/DELETE commands.

```bash
reedbase exec <SQL> <PATH> [OPTIONS]

Arguments:
  <SQL>     ReedQL command (quoted)
  <PATH>    Path to ReedBase directory

Options:
  -u, --user <USER>      Username for audit trail [default: $USER]
  -q, --quiet            Don't print affected rows
  -h, --help             Print help

Examples:
  reedbase exec "INSERT INTO text (key, value) VALUES ('page.title', 'Welcome')" .reed
  reedbase exec "UPDATE text SET value = 'Hello' WHERE key = 'page.title'" .reed --user admin
  reedbase exec "DELETE FROM text WHERE key LIKE 'temp.%'" .reed
```

### 3. `reedbase shell`

Open interactive REPL shell.

```bash
reedbase shell <PATH> [OPTIONS]

Arguments:
  <PATH>    Path to ReedBase directory

Options:
  -u, --user <USER>      Default username for exec commands [default: $USER]
  -h, --help             Print help

Examples:
  reedbase shell .reed
  reedbase shell .reed --user admin

Interactive Commands:
  SELECT ...             Execute query
  INSERT/UPDATE/DELETE   Execute command
  .tables                List all tables
  .indices               List all indices
  .stats                 Show database statistics
  .explain <SQL>         Explain query execution plan
  .format <FORMAT>       Set output format (table|json|csv)
  .exit                  Exit shell
  .help                  Show shell help
```

### 4. `reedbase tables`

List or manage tables.

```bash
reedbase tables <PATH> [OPTIONS]

Arguments:
  <PATH>    Path to ReedBase directory

Options:
  -c, --create <NAME>    Create new table
  -d, --drop <NAME>      Drop table (requires --confirm)
  --confirm              Confirm destructive operation
  -v, --verbose          Show table statistics
  -h, --help             Print help

Examples:
  reedbase tables .reed
  reedbase tables .reed --verbose
  reedbase tables .reed --create users
  reedbase tables .reed --drop temp_table --confirm
```

### 5. `reedbase indices`

List or manage indices.

```bash
reedbase indices <PATH> [OPTIONS]

Arguments:
  <PATH>    Path to ReedBase directory

Options:
  -c, --create <TABLE.COLUMN>   Create index on table.column
  -d, --drop <TABLE.COLUMN>     Drop index
  -r, --rebuild <TABLE.COLUMN>  Rebuild index
  -v, --verbose                 Show index statistics
  -h, --help                    Print help

Examples:
  reedbase indices .reed
  reedbase indices .reed --verbose
  reedbase indices .reed --create text.key
  reedbase indices .reed --rebuild text.key
  reedbase indices .reed --drop text.namespace
```

### 6. `reedbase stats`

Show database statistics.

```bash
reedbase stats <PATH> [OPTIONS]

Arguments:
  <PATH>    Path to ReedBase directory

Options:
  -f, --format <FORMAT>   Output format: table|json [default: table]
  -h, --help              Print help

Examples:
  reedbase stats .reed
  reedbase stats .reed --format json

Output:
  Tables:           5
  Total Rows:       10,523
  Indices:          8 (3 auto-created)
  Total Queries:    1,234
  Avg Query Time:   1.2ms
  Index Memory:     2.4 MB
```

### 7. `reedbase explain`

Explain query execution plan.

```bash
reedbase explain <SQL> <PATH> [OPTIONS]

Arguments:
  <SQL>     ReedQL query (quoted)
  <PATH>    Path to ReedBase directory

Options:
  -v, --verbose          Show detailed plan
  -h, --help             Print help

Examples:
  reedbase explain "SELECT * FROM text WHERE key = 'page.title'" .reed
  reedbase explain "SELECT * FROM text WHERE key LIKE 'page.%'" .reed --verbose

Output:
  Query Plan:
  ├─ Pattern: Point Lookup
  ├─ Index: text.key (hash)
  ├─ Estimated Rows: 1
  ├─ Cost: 0.1ms
  └─ Fast Path: YES
```

## Output Formats

### Table Format (Default)

```
+----------------------+----------------+
| key                  | value          |
+----------------------+----------------+
| page.header.title@de | Willkommen     |
| page.header.title@en | Welcome        |
+----------------------+----------------+
2 rows (1.2ms)
```

### JSON Format

```json
{
  "rows": [
    {"key": "page.header.title@de", "value": "Willkommen"},
    {"key": "page.header.title@en", "value": "Welcome"}
  ],
  "count": 2,
  "time_ms": 1.2
}
```

### CSV Format

```csv
key,value
page.header.title@de,Willkommen
page.header.title@en,Welcome
```

## Interactive Shell Features

### Prompt

```
reedbase> SELECT * FROM text LIMIT 3;
+-------------------+-----------+
| key               | value     |
+-------------------+-----------+
| page.title@de     | Start     |
| page.title@en     | Home      |
| page.subtitle@de  | Willkommen|
+-------------------+-----------+
3 rows (0.8ms)

reedbase> INSERT INTO text (key, value) VALUES ('test', 'value');
1 row affected (2.3ms)

reedbase> .tables
Tables:
  - text (10,234 rows)
  - routes (523 rows)
  - meta (1,034 rows)

reedbase> .exit
Goodbye!
```

### Special Commands

- `.tables` - List tables
- `.indices` - List indices
- `.stats` - Database statistics
- `.explain <SQL>` - Explain query
- `.format <FORMAT>` - Change output format
- `.clear` - Clear screen
- `.help` - Show help
- `.exit` / `.quit` - Exit shell

### History

- Arrow up/down - Navigate command history
- Ctrl+R - Reverse history search
- History stored in memory (not persisted)

## Implementation Plan

### Phase 1: Basic CLI Structure

**Files:**
- `reedbase/Cargo.toml` - Add dependencies
- `reedbase/src/bin/reedbase.rs` - Main entry point

**Dependencies:**
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
rustyline = "14.0"  # For interactive shell
anyhow = "1.0"      # Error handling
serde_json = "1.0"  # JSON output
```

**Tasks:**
1. Create binary crate structure
2. Define CLI with clap derive macros
3. Parse arguments and dispatch to commands

### Phase 2: Query & Exec Commands

**Files:**
- `reedbase/src/bin/commands/query.rs`
- `reedbase/src/bin/commands/exec.rs`
- `reedbase/src/bin/formatters/mod.rs`

**Tasks:**
1. Implement `query` command
2. Implement `exec` command
3. Create output formatters (table, JSON, CSV)
4. Error handling and user-friendly messages

### Phase 3: Interactive Shell

**Files:**
- `reedbase/src/bin/commands/shell.rs`
- `reedbase/src/bin/shell/repl.rs`

**Tasks:**
1. Implement REPL loop with rustyline
2. Parse input (SQL vs. dot-commands)
3. Execute queries/commands
4. Format and display results
5. Command history

### Phase 4: Management Commands

**Files:**
- `reedbase/src/bin/commands/tables.rs`
- `reedbase/src/bin/commands/indices.rs`
- `reedbase/src/bin/commands/stats.rs`
- `reedbase/src/bin/commands/explain.rs`

**Tasks:**
1. Implement table management
2. Implement index management
3. Implement statistics display
4. Implement query explanation

## Testing Strategy

### Manual Testing Checklist

```bash
# 1. Query command
reedbase query "SELECT * FROM text LIMIT 5" .reed
reedbase query "SELECT COUNT(*) FROM text" .reed --format json

# 2. Exec command
reedbase exec "INSERT INTO text (key, value) VALUES ('test', 'value')" .reed
reedbase exec "UPDATE text SET value = 'new' WHERE key = 'test'" .reed
reedbase exec "DELETE FROM text WHERE key = 'test'" .reed

# 3. Shell
reedbase shell .reed
# > SELECT * FROM text LIMIT 3;
# > .tables
# > .stats
# > .exit

# 4. Tables
reedbase tables .reed
reedbase tables .reed --create test_table
reedbase tables .reed --drop test_table --confirm

# 5. Indices
reedbase indices .reed
reedbase indices .reed --create text.key
reedbase indices .reed --verbose

# 6. Stats
reedbase stats .reed
reedbase stats .reed --format json

# 7. Explain
reedbase explain "SELECT * FROM text WHERE key = 'page.title'" .reed
```

### Integration Tests

Create `reedbase/tests/cli_test.rs`:
- Test each command with sample database
- Verify output formats
- Test error handling

## Performance Targets

- **Startup**: < 100ms (cold start with index loading)
- **Query**: < 10ms for indexed lookups
- **Shell responsiveness**: < 50ms prompt display
- **Large result sets**: Stream output (don't buffer everything)

## Error Handling

### User-Friendly Errors

```
❌ Error: Table 'users' not found

Available tables:
  - text (10,234 rows)
  - routes (523 rows)
  - meta (1,034 rows)

Hint: Use 'reedbase tables .reed' to list all tables
```

```
❌ Error: Invalid SQL syntax near 'FORM'

Expected: SELECT ... FROM ...
Got:      SELECT ... FORM ...
                      ^^^^

Hint: Did you mean 'FROM' instead of 'FORM'?
```

### Exit Codes

- `0` - Success
- `1` - General error
- `2` - Invalid arguments
- `3` - Database error
- `4` - I/O error

## Documentation

### Help Text

Every command must have:
- Clear description
- Arguments explanation
- Options with defaults
- Examples
- Related commands

### Man Page

Create `src/man/reedbase.1`:
- Command overview
- All subcommands
- Examples
- Configuration

## Acceptance Criteria

- [ ] `reedbase query` executes SELECT queries
- [ ] `reedbase exec` executes INSERT/UPDATE/DELETE
- [ ] `reedbase shell` provides interactive REPL
- [ ] All output formats work (table, JSON, CSV)
- [ ] Table management commands work
- [ ] Index management commands work
- [ ] Statistics display works
- [ ] Query explanation works
- [ ] Error messages are user-friendly
- [ ] Help text is complete
- [ ] Manual testing checklist passes
- [ ] Performance targets met

## Future Enhancements (Post-REED-19-24B)

- Query history persistence (~/.reedbase_history)
- Syntax highlighting in shell
- Tab completion for table/column names
- Multi-line query support
- Configuration file (~/.reedbaserc)
- Batch mode (read SQL from file)
- Progress indicators for long operations
- Color output (with --color flag)

## Related Tickets

- **REED-19-24A**: Database API (completed)
- **REED-19-24C**: Integration Tests (next)
- **REED-19-24D**: B+-Tree Integration (later)

## Notes

- Use rustyline for REPL (same as used by redis-cli, sqlite3)
- Output formatters should be reusable (also for API responses later)
- Keep binary size small (minimize dependencies)
- CLI should feel like sqlite3 / psql (familiar UX)
