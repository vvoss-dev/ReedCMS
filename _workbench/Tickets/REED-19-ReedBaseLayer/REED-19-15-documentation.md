# REED-19-15: Complete Documentation System

**Status**: Not Started  
**Priority**: High  
**Estimated Effort**: 1 week  
**Layer**: ReedBase (Data Layer)  
**Dependencies**: All REED-19-01 through REED-19-14  

---

## Overview

This ticket implements comprehensive documentation for ReedBase including user guides, API documentation, examples, tutorials, and developer guides.

**Purpose**: Provide complete, accessible documentation that enables users and developers to understand, use, and extend ReedBase effectively.

**Scope**:
- User guide with CLI command reference
- Developer API documentation
- Architecture documentation
- Tutorial series with working examples
- Migration guide from other systems
- Troubleshooting guide
- Performance tuning guide
- Contributing guidelines

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

## Documentation Structure

```
manual/reedbase/
├── README.md                           # Overview and quick start
├── 01-introduction/
│   ├── overview.md                     # What is ReedBase?
│   ├── key-concepts.md                 # Core concepts explained
│   ├── architecture.md                 # System architecture
│   └── comparison.md                   # vs SQLite, PostgreSQL, Git
├── 02-getting-started/
│   ├── installation.md                 # Installation instructions
│   ├── quick-start.md                  # 5-minute quick start
│   ├── first-table.md                  # Creating your first table
│   └── basic-operations.md             # CRUD operations tutorial
├── 03-user-guide/
│   ├── cli-reference.md                # Complete CLI command reference
│   ├── tables.md                       # Working with tables
│   ├── versioning.md                   # Version control features
│   ├── queries.md                      # ReedQL query language
│   ├── concurrent-writes.md            # Concurrent operation handling
│   ├── conflict-resolution.md          # Resolving conflicts
│   ├── schemas.md                      # Schema validation
│   └── functions.md                    # Computed columns and aggregations
├── 04-developer-guide/
│   ├── api-overview.md                 # Rust API overview
│   ├── table-api.md                    # Table trait usage
│   ├── versioning-api.md               # Versioning system API
│   ├── concurrent-api.md               # Concurrent write API
│   ├── query-api.md                    # Query builder API
│   ├── function-api.md                 # Function system API
│   └── error-handling.md               # ReedError and ReedResult
├── 05-advanced-topics/
│   ├── performance-tuning.md           # Optimisation guide
│   ├── large-datasets.md               # Working with large tables
│   ├── migration-strategies.md         # Data migration patterns
│   ├── backup-recovery.md              # Backup and recovery
│   ├── custom-functions.md             # Writing custom functions
│   └── extending-reedbase.md           # Extension points
├── 06-tutorials/
│   ├── blog-database.md                # Tutorial: Simple blog database
│   ├── user-management.md              # Tutorial: User management system
│   ├── audit-log.md                    # Tutorial: Audit logging
│   ├── content-versioning.md           # Tutorial: Content versioning
│   └── data-migration.md               # Tutorial: Migrating from CSV
├── 07-reference/
│   ├── cli-commands.md                 # All CLI commands
│   ├── reedql-syntax.md                # ReedQL language reference
│   ├── schema-format.md                # Schema TOML specification
│   ├── log-format.md                   # Version log format
│   ├── file-structure.md               # Directory and file layout
│   └── performance-targets.md          # Performance specifications
├── 08-troubleshooting/
│   ├── common-errors.md                # Common error messages
│   ├── performance-issues.md           # Performance debugging
│   ├── corruption-recovery.md          # Data corruption recovery
│   ├── lock-timeouts.md                # Concurrent write issues
│   └── faq.md                          # Frequently asked questions
└── 09-contributing/
    ├── development-setup.md            # Setting up dev environment
    ├── code-style.md                   # Coding standards
    ├── testing-guide.md                # Writing tests
    ├── benchmarking-guide.md           # Adding benchmarks
    └── documentation-guide.md          # Contributing to docs
```

---

## Implementation Files

### 1. `manual/reedbase/README.md`

**Purpose**: Entry point for all ReedBase documentation.

**Content Structure**:

```markdown
# ReedBase Documentation

ReedBase is a lightweight, versioned, CSV-based database with Git-like version control and concurrent write support.

## Quick Links

- [What is ReedBase?](01-introduction/overview.md) - Start here if you're new
- [Quick Start](02-getting-started/quick-start.md) - Get up and running in 5 minutes
- [CLI Reference](03-user-guide/cli-reference.md) - Complete command documentation
- [API Documentation](04-developer-guide/api-overview.md) - Rust API reference
- [Tutorials](06-tutorials/) - Step-by-step guides

## Key Features

- **CSV-Based Storage**: Human-readable, Git-friendly data format
- **Version Control**: Git-like versioning with binary delta compression (95% smaller)
- **Concurrent Writes**: Automatic row-level merge with conflict resolution
- **Query Language**: SQL-like ReedQL for filtering and aggregation
- **Schema Validation**: Optional TOML schemas with type checking
- **Function System**: Computed columns and aggregations with caching
- **Performance**: < 100μs reads, ~10,000 rows/sec writes

## Getting Started

### Installation

```bash
cargo install reedbase
```

### Create Your First Table

```bash
# Initialise a new ReedBase database
reed init my-database
cd my-database

# Create a table
reed table:create users --schema schemas/users.toml

# Add data
reed set users username=alice email=alice@example.com
reed set users username=bob email=bob@example.com

# Query data
reed get users username=alice
```

## Documentation Sections

1. **Introduction** - Concepts, architecture, and comparisons
2. **Getting Started** - Installation and basic tutorials
3. **User Guide** - Complete CLI usage and features
4. **Developer Guide** - Rust API and integration
5. **Advanced Topics** - Performance, migration, extensions
6. **Tutorials** - Step-by-step real-world examples
7. **Reference** - Detailed specifications
8. **Troubleshooting** - Common issues and solutions
9. **Contributing** - How to contribute to ReedBase

## Support

- **Issues**: [GitHub Issues](https://github.com/vvoss-dev/reedbase/issues)
- **Discussions**: [GitHub Discussions](https://github.com/vvoss-dev/reedbase/discussions)
- **Email**: ask@vvoss.dev

## License

Apache License 2.0 - See LICENSE file for details.
```

---

### 2. `manual/reedbase/01-introduction/overview.md`

**Purpose**: Introduce ReedBase to new users.

**Content Structure**:

```markdown
# What is ReedBase?

ReedBase is a lightweight database system designed for applications that need:

- Human-readable data storage (CSV format)
- Git-like version control for all changes
- Concurrent write support without database server
- Simple CLI and Rust API
- Minimal dependencies and setup

## Design Philosophy

ReedBase follows three core principles:

1. **Simplicity**: CSV storage, no complex setup, human-readable
2. **Versioning**: Every change tracked with Git-like deltas
3. **Concurrency**: Multiple processes can write simultaneously

## When to Use ReedBase

**Good fit**:
- Configuration management systems
- Content management with version history
- Audit logs requiring complete history
- Small to medium datasets (< 1M rows per table)
- Applications requiring human-readable data
- Git-based deployment workflows

**Not ideal**:
- High-frequency writes (> 10k writes/sec)
- Complex relational queries with JOINs
- Large binary data (> 10GB)
- Real-time analytical queries

## Architecture Overview

```
┌──────────────────────────────────────────────┐
│ ReedBase CLI / Rust API                      │
├──────────────────────────────────────────────┤
│ Query Layer (ReedQL)                         │
├──────────────────────────────────────────────┤
│ Function System (Computed Columns, Cache)    │
├──────────────────────────────────────────────┤
│ Concurrent Write System (Locks, Merge)       │
├──────────────────────────────────────────────┤
│ Version Control (Binary Deltas, Compression) │
├──────────────────────────────────────────────┤
│ Table API (Universal Table Structure)        │
├──────────────────────────────────────────────┤
│ Storage Layer (CSV Files)                    │
└──────────────────────────────────────────────┘
```

## Key Features

### CSV-Based Storage
All data stored in pipe-delimited CSV files. Human-readable, Git-friendly, easily edited with any text editor.

### Git-Like Versioning
Every change creates a version. Binary deltas (bsdiff + XZ) provide 95%+ space savings compared to full snapshots.

### Concurrent Writes
Multiple processes can write simultaneously. Automatic row-level merge resolves non-conflicting changes. Configurable conflict resolution strategies.

### ReedQL Query Language
SQL-like syntax for queries:
```
reed query users "SELECT * WHERE role='admin' ORDER BY username"
```

### Schema Validation
Optional TOML schemas with type checking and constraints:
```toml
[columns.email]
type = "string"
required = true
pattern = "^[a-z0-9._%+-]+@[a-z0-9.-]+\\.[a-z]{2,}$"
```

### Function System
Computed columns and aggregations with memoization caching:
```bash
reed function users "calculate_age(birth_date)"
reed function users "count_by_role(role)"
```

## Performance Characteristics

- **Read latency**: < 100μs for single row
- **Write throughput**: ~10,000 rows/sec
- **Query performance**: < 10ms for 10k rows
- **Version creation**: < 50ms for typical table
- **Concurrent writes**: > 1,000 ops/sec with 4 writers
- **Memory usage**: < 5MB for 10k rows
- **Disk usage**: ~200 bytes per version (with compression)

## Comparison with Other Systems

See [Comparison Guide](comparison.md) for detailed comparison with SQLite, PostgreSQL, Git, and other systems.

## Next Steps

- [Key Concepts](key-concepts.md) - Understand core concepts
- [Architecture](architecture.md) - Deep dive into system design
- [Quick Start](../02-getting-started/quick-start.md) - Build your first database
```

---

### 3. `manual/reedbase/02-getting-started/quick-start.md`

**Purpose**: Get users productive in 5 minutes.

**Content Structure**:

```markdown
# Quick Start Guide

Get up and running with ReedBase in 5 minutes.

## Installation

```bash
# Install ReedBase CLI
cargo install reedbase

# Verify installation
reed --version
```

## Create a Database

```bash
# Initialise new database
reed init blog
cd blog

# Directory structure created:
# blog/
# ├── .reed/
# │   ├── actions.dict
# │   ├── users.dict
# │   └── tables/
# └── schemas/
```

## Create Your First Table

```bash
# Create posts table with schema
cat > schemas/posts.toml << 'EOF'
[table]
name = "posts"
description = "Blog posts"

[columns.id]
type = "string"
primary_key = true

[columns.title]
type = "string"
required = true

[columns.content]
type = "string"

[columns.published]
type = "boolean"
default = "false"

[columns.created_at]
type = "timestamp"
EOF

# Create table from schema
reed table:create posts --schema schemas/posts.toml
```

## Add Data

```bash
# Add posts
reed set posts id=post-1 title="Hello World" content="My first post" published=true created_at=2025-01-15T10:00:00Z

reed set posts id=post-2 title="ReedBase Tutorial" content="How to use ReedBase" published=true created_at=2025-01-16T14:30:00Z

reed set posts id=post-3 title="Draft Post" content="Work in progress" published=false created_at=2025-01-17T09:15:00Z
```

## Query Data

```bash
# Get specific post
reed get posts id=post-1

# Output:
# id|title|content|published|created_at
# post-1|Hello World|My first post|true|2025-01-15T10:00:00Z

# List all posts
reed list posts

# Query published posts
reed query posts "SELECT * WHERE published=true ORDER BY created_at DESC"

# Output:
# id|title|content|published|created_at
# post-2|ReedBase Tutorial|How to use ReedBase|true|2025-01-16T14:30:00Z
# post-1|Hello World|My first post|true|2025-01-15T10:00:00Z
```

## Version Control

```bash
# Update a post
reed set posts id=post-1 title="Hello ReedBase!"

# List versions
reed version:list posts

# Output:
# Version  Time                User    Action  Rows  Size
# 1        2025-01-15 10:05:23 alice   CREATE  3     456
# 2        2025-01-15 10:12:45 alice   UPDATE  1     234

# View diff
reed version:diff posts 1 2

# Output:
# Row: post-1
#   title: "Hello World" → "Hello ReedBase!"

# Rollback to previous version
reed version:rollback posts 1
```

## Concurrent Writes

```bash
# Terminal 1: Add post
reed set posts id=post-4 title="Concurrent Post A" content="Written by user A"

# Terminal 2 (simultaneously): Add post
reed set posts id=post-5 title="Concurrent Post B" content="Written by user B"

# Both writes succeed - automatic row-level merge

# Verify both posts exist
reed list posts
```

## Next Steps

- [First Table Tutorial](first-table.md) - Detailed walkthrough
- [Basic Operations](basic-operations.md) - Learn CRUD operations
- [CLI Reference](../03-user-guide/cli-reference.md) - All commands
- [Versioning Guide](../03-user-guide/versioning.md) - Version control features

## Troubleshooting

**Command not found: reed**
```bash
# Ensure cargo bin is in PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

**Permission denied**
```bash
# Check directory permissions
ls -la .reed/
# Should be writable by your user
```

**Lock timeout**
```bash
# Another process holds lock - wait or kill process
reed status  # Shows active locks
```

See [Troubleshooting Guide](../08-troubleshooting/common-errors.md) for more help.
```

---

### 4. `manual/reedbase/03-user-guide/cli-reference.md`

**Purpose**: Complete CLI command reference.

**Content Structure** (excerpt):

```markdown
# CLI Command Reference

Complete reference for all ReedBase CLI commands.

## Initialisation Commands

### `reed init`
Initialise a new ReedBase database.

**Usage**:
```bash
reed init [directory]
```

**Arguments**:
- `directory`: Path to create database (default: current directory)

**Options**:
- `--force`: Overwrite existing database

**Examples**:
```bash
# Initialise in current directory
reed init

# Initialise in specific directory
reed init ./my-database

# Force reinitialisation
reed init --force
```

**Output**:
```
Initialised ReedBase database in /path/to/database
Created directories:
  .reed/tables/
  schemas/
```

---

## Table Management Commands

### `reed table:create`
Create a new table.

**Usage**:
```bash
reed table:create <table_name> [--schema <schema_file>]
```

**Arguments**:
- `table_name`: Name of table to create

**Options**:
- `--schema <file>`: Path to schema TOML file
- `--columns <list>`: Comma-separated column list (if no schema)

**Examples**:
```bash
# Create with schema
reed table:create users --schema schemas/users.toml

# Create without schema
reed table:create logs --columns timestamp,level,message
```

### `reed table:list`
List all tables.

**Usage**:
```bash
reed table:list
```

**Output**:
```
Table   Rows    Size     Versions  Last Modified
users   1,234   234 KB   45        2025-01-15 10:30:15
posts   567     123 KB   23        2025-01-15 09:45:33
```

### `reed table:info`
Show table information.

**Usage**:
```bash
reed table:info <table_name>
```

**Output**:
```
Table: users
Rows: 1,234
Size: 234 KB
Versions: 45
Created: 2025-01-01 12:00:00
Last Modified: 2025-01-15 10:30:15
Schema: schemas/users.toml

Columns:
  id (string, primary_key)
  username (string, required, unique)
  email (string, required)
  created_at (timestamp)
```

---

## Data Operations

### `reed set`
Insert or update row.

**Usage**:
```bash
reed set <table> <key>=<value> [<key>=<value> ...]
```

**Arguments**:
- `table`: Table name
- `key=value`: Column assignments

**Options**:
- `--batch <file>`: Batch insert from file

**Examples**:
```bash
# Insert row
reed set users id=123 username=alice email=alice@example.com

# Update row (same syntax)
reed set users id=123 email=alice@newdomain.com

# Batch insert from CSV
reed set users --batch users.csv
```

### `reed get`
Retrieve row by key.

**Usage**:
```bash
reed get <table> <key>=<value>
```

**Examples**:
```bash
# Get by primary key
reed get users id=123

# Get by unique column
reed get users username=alice
```

### `reed delete`
Delete row.

**Usage**:
```bash
reed delete <table> <key>=<value>
```

**Examples**:
```bash
# Delete by primary key
reed delete users id=123
```

### `reed list`
List all rows.

**Usage**:
```bash
reed list <table> [--limit <n>] [--offset <n>]
```

**Options**:
- `--limit <n>`: Maximum rows to return
- `--offset <n>`: Skip first n rows
- `--format <fmt>`: Output format (table, json, csv)

**Examples**:
```bash
# List all rows
reed list users

# Paginate results
reed list users --limit 100 --offset 0

# JSON output
reed list users --format json
```

---

## Query Commands

### `reed query`
Execute ReedQL query.

**Usage**:
```bash
reed query <table> "<query>"
```

**Query Syntax**:
```
SELECT <columns|*> 
[WHERE <condition>] 
[ORDER BY <column> [ASC|DESC]] 
[LIMIT <n>]
```

**Examples**:
```bash
# Filter rows
reed query users "SELECT * WHERE role='admin'"

# Sort results
reed query posts "SELECT * WHERE published=true ORDER BY created_at DESC"

# Limit results
reed query users "SELECT username,email WHERE active=true LIMIT 10"

# Count rows
reed query users "SELECT COUNT(*) WHERE role='user'"
```

See [ReedQL Syntax](../07-reference/reedql-syntax.md) for complete language reference.

---

## Version Control Commands

### `reed version:list`
List all versions.

**Usage**:
```bash
reed version:list <table>
```

**Output**:
```
Version  Time                User    Action  Rows  Size    Hash
1        2025-01-15 10:00:00 alice   CREATE  100   23KB    a1b2c3d
2        2025-01-15 10:05:15 bob     UPDATE  5     234B    e4f5g6h
3        2025-01-15 10:12:30 alice   DELETE  1     45B     i7j8k9l
```

### `reed version:show`
Show version details.

**Usage**:
```bash
reed version:show <table> <version>
```

**Output**:
```
Version: 2
Time: 2025-01-15 10:05:15
User: bob
Action: UPDATE
Rows modified: 5
Delta size: 234 bytes (compressed)
Hash: e4f5g6h7i8j9k0l1

Modified rows:
  id=123: username changed
  id=456: email changed
  ...
```

### `reed version:diff`
Show differences between versions.

**Usage**:
```bash
reed version:diff <table> <version1> <version2>
```

**Output**:
```
Diff: version 1 → version 2

Row id=123:
  username: "alice" → "alice2024"
  
Row id=456:
  email: "old@example.com" → "new@example.com"

2 rows changed
```

### `reed version:rollback`
Rollback to previous version.

**Usage**:
```bash
reed version:rollback <table> <version>
```

**Options**:
- `--force`: Skip confirmation prompt

**Examples**:
```bash
# Rollback with confirmation
reed version:rollback users 5

# Force rollback
reed version:rollback users 5 --force
```

---

(Continue with remaining command categories: Concurrent Operations, Schema Management, Function System, Backup/Recovery, Performance, Maintenance)

```

---

### 5. `manual/reedbase/04-developer-guide/api-overview.md`

**Purpose**: Introduction to Rust API for developers.

**Content Structure** (excerpt):

```markdown
# Rust API Overview

ReedBase provides a comprehensive Rust API for embedding in applications.

## Adding ReedBase to Your Project

```toml
# Cargo.toml
[dependencies]
reedbase = "0.1.0"
```

## Quick Start

```rust
use reedbase::{ReedBase, Table, ReedResult};

fn main() -> ReedResult<()> {
    // Initialise ReedBase database
    let db = ReedBase::init("./my-database")?;
    
    // Create table
    let users = db.table("users")?;
    
    // Insert data
    users.insert(vec![
        ("id", "123"),
        ("username", "alice"),
        ("email", "alice@example.com"),
    ])?;
    
    // Query data
    let user = users.get("id", "123")?;
    println!("Username: {}", user.get("username")?);
    
    // List all rows
    let all_users = users.list()?;
    for user in all_users {
        println!("{:?}", user);
    }
    
    Ok(())
}
```

## Core Types

### `ReedBase`
Main database handle.

```rust
pub struct ReedBase {
    path: PathBuf,
    config: Config,
}

impl ReedBase {
    /// Initialise new database
    pub fn init<P: AsRef<Path>>(path: P) -> ReedResult<Self>
    
    /// Open existing database
    pub fn open<P: AsRef<Path>>(path: P) -> ReedResult<Self>
    
    /// Get table handle
    pub fn table(&self, name: &str) -> ReedResult<Table>
    
    /// List all tables
    pub fn list_tables(&self) -> ReedResult<Vec<String>>
    
    /// Create new table
    pub fn create_table(&self, name: &str, schema: Option<Schema>) -> ReedResult<Table>
}
```

### `Table`
Table handle for operations.

```rust
pub struct Table {
    path: PathBuf,
    name: String,
    schema: Option<Schema>,
}

impl Table {
    /// Insert or update row
    pub fn insert(&self, row: Vec<(&str, &str)>) -> ReedResult<()>
    
    /// Get row by key
    pub fn get(&self, key_column: &str, key_value: &str) -> ReedResult<Row>
    
    /// Delete row
    pub fn delete(&self, key_column: &str, key_value: &str) -> ReedResult<()>
    
    /// List all rows
    pub fn list(&self) -> ReedResult<Vec<Row>>
    
    /// Query with ReedQL
    pub fn query(&self, query: &str) -> ReedResult<QueryResult>
    
    /// Create version
    pub fn create_version(&self) -> ReedResult<Version>
    
    /// List versions
    pub fn list_versions(&self) -> ReedResult<Vec<Version>>
    
    /// Rollback to version
    pub fn rollback(&self, version: u64) -> ReedResult<()>
}
```

### `Row`
Single row of data.

```rust
pub struct Row {
    columns: HashMap<String, String>,
}

impl Row {
    /// Get column value
    pub fn get(&self, column: &str) -> ReedResult<&str>
    
    /// Set column value
    pub fn set(&mut self, column: &str, value: &str)
    
    /// Get all columns
    pub fn columns(&self) -> &HashMap<String, String>
}
```

## Error Handling

All operations return `ReedResult<T>` (alias for `Result<T, ReedError>`).

```rust
use reedbase::{ReedResult, ReedError};

fn example() -> ReedResult<()> {
    let db = ReedBase::open("./database")?;
    let table = db.table("users")?;
    
    match table.get("id", "123") {
        Ok(row) => println!("Found: {:?}", row),
        Err(ReedError::RowNotFound { .. }) => println!("User not found"),
        Err(e) => return Err(e),
    }
    
    Ok(())
}
```

See [Error Handling Guide](error-handling.md) for all error types.

## Examples

### Simple CRUD Application

```rust
use reedbase::{ReedBase, ReedResult};

struct UserDatabase {
    db: ReedBase,
}

impl UserDatabase {
    pub fn new(path: &str) -> ReedResult<Self> {
        let db = ReedBase::init(path)?;
        let users = db.create_table("users", None)?;
        Ok(Self { db })
    }
    
    pub fn create_user(&self, id: &str, username: &str, email: &str) -> ReedResult<()> {
        let users = self.db.table("users")?;
        users.insert(vec![
            ("id", id),
            ("username", username),
            ("email", email),
        ])
    }
    
    pub fn get_user(&self, id: &str) -> ReedResult<String> {
        let users = self.db.table("users")?;
        let row = users.get("id", id)?;
        Ok(row.get("username")?.to_string())
    }
    
    pub fn list_users(&self) -> ReedResult<Vec<String>> {
        let users = self.db.table("users")?;
        let rows = users.list()?;
        Ok(rows.iter()
            .map(|r| r.get("username").unwrap().to_string())
            .collect())
    }
}

fn main() -> ReedResult<()> {
    let db = UserDatabase::new("./userdb")?;
    
    db.create_user("1", "alice", "alice@example.com")?;
    db.create_user("2", "bob", "bob@example.com")?;
    
    println!("Users: {:?}", db.list_users()?);
    
    Ok(())
}
```

## Next Steps

- [Table API](table-api.md) - Detailed table operations
- [Versioning API](versioning-api.md) - Version control features
- [Query API](query-api.md) - ReedQL query builder
- [Concurrent API](concurrent-api.md) - Concurrent write handling
```

---

### 6. `manual/reedbase/06-tutorials/blog-database.md`

**Purpose**: Complete tutorial building a blog database.

**Content Structure** (excerpt):

```markdown
# Tutorial: Building a Blog Database

In this tutorial, we'll build a complete blog database with posts, authors, and comments using ReedBase.

## What We'll Build

- Authors table with user accounts
- Posts table with versioning
- Comments table with nested replies
- Full-text search on posts
- Concurrent comment submission

## Step 1: Initialise Database

```bash
mkdir blog-db && cd blog-db
reed init

# Directory structure:
# blog-db/
# ├── .reed/
# └── schemas/
```

## Step 2: Create Schemas

### Authors Schema

```bash
cat > schemas/authors.toml << 'EOF'
[table]
name = "authors"
description = "Blog authors"

[columns.id]
type = "string"
primary_key = true

[columns.username]
type = "string"
required = true
unique = true
min_length = 3
max_length = 20

[columns.email]
type = "string"
required = true
pattern = "^[a-z0-9._%+-]+@[a-z0-9.-]+\\.[a-z]{2,}$"

[columns.display_name]
type = "string"

[columns.bio]
type = "string"

[columns.created_at]
type = "timestamp"
required = true
EOF
```

### Posts Schema

```bash
cat > schemas/posts.toml << 'EOF'
[table]
name = "posts"
description = "Blog posts"

[columns.id]
type = "string"
primary_key = true

[columns.author_id]
type = "string"
required = true

[columns.title]
type = "string"
required = true
min_length = 1
max_length = 200

[columns.slug]
type = "string"
required = true
unique = true
pattern = "^[a-z0-9-]+$"

[columns.content]
type = "string"
required = true

[columns.published]
type = "boolean"
default = "false"

[columns.created_at]
type = "timestamp"
required = true

[columns.updated_at]
type = "timestamp"

[columns.view_count]
type = "integer"
default = "0"
EOF
```

### Comments Schema

```bash
cat > schemas/comments.toml << 'EOF'
[table]
name = "comments"
description = "Post comments"

[columns.id]
type = "string"
primary_key = true

[columns.post_id]
type = "string"
required = true

[columns.author_id]
type = "string"
required = true

[columns.parent_id]
type = "string"

[columns.content]
type = "string"
required = true
min_length = 1
max_length = 1000

[columns.created_at]
type = "timestamp"
required = true
EOF
```

## Step 3: Create Tables

```bash
reed table:create authors --schema schemas/authors.toml
reed table:create posts --schema schemas/posts.toml
reed table:create comments --schema schemas/comments.toml

# Verify tables
reed table:list
```

## Step 4: Add Sample Data

### Create Authors

```bash
reed set authors \
  id=author-1 \
  username=alice \
  email=alice@example.com \
  display_name="Alice Johnson" \
  bio="Software engineer and blogger" \
  created_at=2025-01-01T10:00:00Z

reed set authors \
  id=author-2 \
  username=bob \
  email=bob@example.com \
  display_name="Bob Smith" \
  bio="Tech writer" \
  created_at=2025-01-02T12:00:00Z
```

### Create Posts

```bash
reed set posts \
  id=post-1 \
  author_id=author-1 \
  title="Getting Started with ReedBase" \
  slug=getting-started-with-reedbase \
  content="ReedBase is a lightweight versioned database..." \
  published=true \
  created_at=2025-01-10T14:30:00Z \
  view_count=0

reed set posts \
  id=post-2 \
  author_id=author-1 \
  title="Advanced ReedBase Techniques" \
  slug=advanced-reedbase-techniques \
  content="In this post we explore advanced features..." \
  published=true \
  created_at=2025-01-12T09:15:00Z \
  view_count=0

reed set posts \
  id=post-3 \
  author_id=author-2 \
  title="Draft: Future Ideas" \
  slug=future-ideas \
  content="Work in progress..." \
  published=false \
  created_at=2025-01-15T08:00:00Z \
  view_count=0
```

## Step 5: Query Data

### List Published Posts

```bash
reed query posts "SELECT * WHERE published=true ORDER BY created_at DESC"
```

### Find Posts by Author

```bash
reed query posts "SELECT title,slug WHERE author_id='author-1'"
```

### Get Author with Posts

```bash
# First get author
reed get authors id=author-1

# Then get their posts
reed query posts "SELECT * WHERE author_id='author-1'"
```

## Step 6: Add Comments

```bash
# Add comment to post
reed set comments \
  id=comment-1 \
  post_id=post-1 \
  author_id=author-2 \
  content="Great post! Very helpful." \
  created_at=2025-01-11T10:30:00Z

# Add reply to comment
reed set comments \
  id=comment-2 \
  post_id=post-1 \
  author_id=author-1 \
  parent_id=comment-1 \
  content="Thanks Bob!" \
  created_at=2025-01-11T11:00:00Z

# List comments for post
reed query comments "SELECT * WHERE post_id='post-1' ORDER BY created_at"
```

## Step 7: Track Views

```bash
# Increment view count when post is read
reed set posts id=post-1 view_count=1
reed set posts id=post-1 view_count=2
reed set posts id=post-1 view_count=3

# View post with counts
reed get posts id=post-1
```

## Step 8: Version Control

```bash
# Update post content
reed set posts \
  id=post-1 \
  content="[Updated] ReedBase is a lightweight versioned database with many great features..."

# List versions
reed version:list posts

# View diff
reed version:diff posts 1 2

# Rollback if needed
reed version:rollback posts 1
```

## Step 9: Concurrent Comments

Simulate multiple users commenting simultaneously:

```bash
# Terminal 1
reed set comments \
  id=comment-3 \
  post_id=post-1 \
  author_id=author-1 \
  content="Comment from terminal 1" \
  created_at=2025-01-12T15:00:00Z &

# Terminal 2 (simultaneously)
reed set comments \
  id=comment-4 \
  post_id=post-1 \
  author_id=author-2 \
  content="Comment from terminal 2" \
  created_at=2025-01-12T15:00:01Z &

# Wait for both
wait

# Verify both comments saved
reed list comments
```

## Step 10: Build a Query Interface

Create a Rust application to query the blog:

```rust
// src/main.rs
use reedbase::{ReedBase, ReedResult};

struct BlogDB {
    db: ReedBase,
}

impl BlogDB {
    pub fn new() -> ReedResult<Self> {
        Ok(Self {
            db: ReedBase::open(".")?,
        })
    }
    
    pub fn list_published_posts(&self) -> ReedResult<Vec<Post>> {
        let posts = self.db.table("posts")?;
        let results = posts.query("SELECT * WHERE published=true ORDER BY created_at DESC")?;
        
        results.rows.iter()
            .map(|row| Post {
                id: row.get("id")?.to_string(),
                title: row.get("title")?.to_string(),
                slug: row.get("slug")?.to_string(),
                content: row.get("content")?.to_string(),
                view_count: row.get("view_count")?.parse().unwrap_or(0),
            })
            .collect()
    }
    
    pub fn get_post_with_comments(&self, slug: &str) -> ReedResult<PostWithComments> {
        let posts = self.db.table("posts")?;
        let post_row = posts.query(&format!("SELECT * WHERE slug='{}'", slug))?
            .rows.first()
            .ok_or(ReedError::RowNotFound)?;
        
        let comments = self.db.table("comments")?;
        let comment_rows = comments.query(&format!(
            "SELECT * WHERE post_id='{}' ORDER BY created_at",
            post_row.get("id")?
        ))?;
        
        Ok(PostWithComments {
            post: /* ... */,
            comments: /* ... */,
        })
    }
}

// (Continue with complete example)
```

## Summary

You've built a complete blog database with:
- ✓ Three related tables (authors, posts, comments)
- ✓ Schema validation with constraints
- ✓ CRUD operations
- ✓ Queries and filtering
- ✓ Version control for content
- ✓ Concurrent comment submission
- ✓ Rust API integration

## Next Steps

- Add full-text search with custom functions
- Implement user authentication
- Create API endpoints
- Add backup/restore functionality

See [User Management Tutorial](user-management.md) for authentication examples.
```

---

## Documentation Generation

### Automated Documentation

**`scripts/generate-docs.sh`**:
```bash
#!/usr/bin/env bash
# Generate complete documentation set

# Generate API docs from Rust code
cargo doc --no-deps --document-private-items

# Generate CLI reference from commands
reed help --markdown > manual/reedbase/03-user-guide/cli-reference.md

# Generate ReedQL syntax from parser
reed query --syntax-reference > manual/reedbase/07-reference/reedql-syntax.md

# Build documentation site
mdbook build manual/reedbase

echo "Documentation generated in manual/reedbase/book/"
```

---

## Test Files

### `tests/documentation_tests.rs`

**Test Coverage**:

```rust
#[test]
fn test_all_code_examples_compile()
// Verify: All Rust code examples in docs compile

#[test]
fn test_all_cli_examples_run()
// Verify: All CLI examples execute without errors

#[test]
fn test_documentation_links_valid()
// Verify: All internal links point to existing files

#[test]
fn test_api_docs_complete()
// Verify: All public APIs have documentation

#[test]
fn test_examples_produce_expected_output()
// Verify: Example outputs match actual command outputs
```

---

## Acceptance Criteria

- [ ] All 9 documentation sections complete
- [ ] README.md provides clear entry point
- [ ] Quick start guide works in < 5 minutes
- [ ] CLI reference documents all commands
- [ ] API documentation covers all public APIs
- [ ] Tutorials include working code examples
- [ ] All code examples compile and run
- [ ] All internal links valid
- [ ] Troubleshooting guide covers common issues
- [ ] Contributing guidelines clear
- [ ] Documentation hosted with mdbook
- [ ] Search functionality works
- [ ] Examples tested in CI
- [ ] Performance targets documented
- [ ] Error messages documented
- [ ] Migration guides complete

---

## Dependencies

- **All REED-19 tickets**: Documentation covers all implemented features

---

## Notes

### Documentation Philosophy

1. **Example-driven**: Every concept demonstrated with working code
2. **Progressive disclosure**: Simple examples first, advanced topics later
3. **Searchable**: Full-text search across all documentation
4. **Tested**: All code examples validated in CI
5. **Accessible**: Clear language, no jargon without explanation

### Documentation Tools

- **mdBook**: Static site generator for documentation
- **cargo doc**: Rust API documentation
- **doctests**: Embedded tests in documentation
- **link checker**: Validate internal and external links

### Maintenance

- Update docs with every feature change
- Review docs in every PR
- User feedback loop for improvements
- Quarterly documentation review

---

## References

- Service Template: `_workbench/Tickets/templates/service-template.md`
- mdBook Documentation: https://rust-lang.github.io/mdBook/
- Rust Doc Book: https://doc.rust-lang.org/rustdoc/
