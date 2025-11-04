# REED-19-24: High-Level Database API & CLI

**Status**: Open  
**Priority**: Critical  
**Estimated Effort**: 1 week  
**Layer**: ReedBase (Data Layer)  
**Dependencies**: REED-19-01 through REED-19-12, REED-19-20 through REED-19-23  

---

## Overview

This ticket implements the high-level user-facing API for ReedBase, providing both programmatic (Rust library) and command-line interfaces. Currently, ReedBase has all internal components (Tables, ReedQL, Smart Indices, Versioning) but lacks a unified interface that connects them.

**Purpose**: Create a proper database API that users actually interact with, rather than exposing low-level implementation details.

**Scope**:
- High-level `Database` struct for Rust applications
- Command-line interface (`reedbase` CLI tool)
- Query execution via ReedQL
- Table management operations
- Connection handling and lifecycle management

**Analogy**: Like PostgreSQL has `PGConnection`, ReedBase needs a `Database` struct that brings together all components.

---

## MANDATORY Development Standards

1. **Language**: All code comments and documentation in BBC English
2. **Principle**: KISS (Keep It Simple, Stupid)
3. **File Naming**: Each file has unique theme and clear responsibility
4. **Files**: One file = One responsibility (no multi-purpose files)
5. **Functions**: One function = One distinctive job (no Swiss Army knives)
6. **Testing**: Separate test files as `{name}_test.rs` (never inline `#[cfg(test)]`)
7. **Avoid**: Generic names like `handler.rs`, `middleware.rs`, `utils.rs`
8. **Templates**: Reference `service-template.md` and `service-template.test.md`

---

## Problem Statement

**Current Architecture (Broken):**
```
User Code
    ↓
    ??? (no clear entry point)
    ↓
Tables (byte-level) ← separate → ReedQL (parser) ← separate → Smart Indices
```

**Desired Architecture:**
```
User Code
    ↓
Database API (db.query("SELECT..."))
    ↓
    ├─→ ReedQL Parser
    ├─→ Query Planner (with Smart Indices)
    ├─→ Executor
    └─→ Tables (internal)
```

---

## Implementation Files

### 1. `src/lib.rs` (Update)

**Purpose**: Export high-level API, hide internal details.

**Changes**:
```rust
// High-level API (public)
pub mod database;
pub mod reedql;  // Keep parser/executor public for advanced users

// Internal modules (pub(crate) only)
pub(crate) mod tables;
pub(crate) mod version;
pub(crate) mod backup;
pub(crate) mod concurrent;
pub(crate) mod merge;
pub(crate) mod conflict;
pub(crate) mod log;

// Re-export main API
pub use database::{Database, Connection, QueryResult, ExecuteResult};
pub use reedql::{parse, QueryError};
pub use error::{ReedError, ReedResult};
```

**Quick Start Example**:
```rust
//! # Quick Start
//!
//! ```rust
//! use reedbase::Database;
//!
//! // Open database
//! let db = Database::open("./mydata")?;
//!
//! // Execute query
//! let results = db.query("SELECT * FROM users WHERE age > 25")?;
//! for row in results.rows() {
//!     println!("{:?}", row);
//! }
//! ```
```

---

### 2. `src/database/mod.rs`

**Purpose**: Main Database API - connects all ReedBase components.

```rust
/// High-level database interface.
///
/// Provides unified access to ReedBase functionality:
/// - Query execution via ReedQL
/// - Table management
/// - Transaction handling
/// - Connection lifecycle
///
/// ## Example
/// ```rust
/// use reedbase::Database;
///
/// let db = Database::open("./data")?;
/// let result = db.query("SELECT * FROM users")?;
/// ```
pub struct Database {
    /// Base directory path
    base_path: PathBuf,
    
    /// Loaded tables (lazy-loaded)
    tables: RwLock<HashMap<String, Table>>,
    
    /// Smart indices for query optimisation
    indices: RwLock<HashMap<String, Box<dyn Index<String, Vec<usize>>>>>,
    
    /// Metrics collector
    metrics: Arc<MetricsCollector>,
}

impl Database {
    /// Open existing database or create new one.
    ///
    /// ## Input
    /// - `path`: Directory path for database (creates `.reedbase/` subdirectory)
    ///
    /// ## Output
    /// - `Result<Database>`: Database connection
    ///
    /// ## Performance
    /// - < 10ms for cold start
    /// - < 1ms for warm start (cached)
    ///
    /// ## Error Conditions
    /// - `IoError`: Cannot access directory
    /// - `PermissionDenied`: Insufficient permissions
    ///
    /// ## Example
    /// ```rust
    /// let db = Database::open("./mydata")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> ReedResult<Self>
    
    /// Execute ReedQL query.
    ///
    /// ## Input
    /// - `sql`: ReedQL query string
    ///
    /// ## Output
    /// - `Result<QueryResult>`: Query results with rows
    ///
    /// ## Performance
    /// - Simple query: < 10ms for 10k rows
    /// - Indexed query: < 1ms for 10k rows
    /// - Aggregate: < 50ms for 10k rows
    ///
    /// ## Error Conditions
    /// - `ParseError`: Invalid SQL syntax
    /// - `TableNotFound`: Table does not exist
    /// - `ColumnNotFound`: Column does not exist in table
    ///
    /// ## Example
    /// ```rust
    /// let results = db.query("SELECT name, age FROM users WHERE age > 25")?;
    /// for row in results.rows() {
    ///     println!("{}: {}", row.get("name"), row.get("age"));
    /// }
    /// ```
    pub fn query(&self, sql: &str) -> ReedResult<QueryResult>
    
    /// Execute statement (INSERT, UPDATE, DELETE).
    ///
    /// ## Input
    /// - `sql`: ReedQL statement
    ///
    /// ## Output
    /// - `Result<ExecuteResult>`: Number of affected rows
    ///
    /// ## Performance
    /// - Single row: < 5ms
    /// - Bulk insert (100 rows): < 50ms
    ///
    /// ## Example
    /// ```rust
    /// let result = db.execute("INSERT INTO users (name, age) VALUES ('Alice', 30)")?;
    /// println!("Inserted {} rows", result.affected_rows);
    /// ```
    pub fn execute(&self, sql: &str) -> ReedResult<ExecuteResult>
    
    /// Create new table.
    ///
    /// ## Input
    /// - `name`: Table name
    /// - `schema`: Column definitions (optional, inferred if None)
    ///
    /// ## Output
    /// - `Result<()>`: Success or error
    ///
    /// ## Example
    /// ```rust
    /// db.create_table("users", None)?;
    /// ```
    pub fn create_table(&self, name: &str, schema: Option<Schema>) -> ReedResult<()>
    
    /// Drop table.
    ///
    /// ## Input
    /// - `name`: Table name
    /// - `confirm`: Must be true (safety check)
    ///
    /// ## Output
    /// - `Result<()>`: Success or error
    ///
    /// ## Example
    /// ```rust
    /// db.drop_table("old_data", true)?;
    /// ```
    pub fn drop_table(&self, name: &str, confirm: bool) -> ReedResult<()>
    
    /// List all tables in database.
    ///
    /// ## Output
    /// - `Result<Vec<String>>`: Table names
    ///
    /// ## Performance
    /// - < 5ms
    ///
    /// ## Example
    /// ```rust
    /// let tables = db.list_tables()?;
    /// println!("Tables: {:?}", tables);
    /// ```
    pub fn list_tables(&self) -> ReedResult<Vec<String>>
    
    /// Create index on column for faster queries.
    ///
    /// ## Input
    /// - `table`: Table name
    /// - `column`: Column name
    ///
    /// ## Performance
    /// - < 500ms for 10k rows
    ///
    /// ## Example
    /// ```rust
    /// db.create_index("users", "age")?;
    /// ```
    pub fn create_index(&self, table: &str, column: &str) -> ReedResult<()>
    
    /// Close database connection.
    ///
    /// Flushes caches and releases resources.
    ///
    /// ## Output
    /// - `Result<()>`: Success or error
    pub fn close(self) -> ReedResult<()>
}
```

---

### 3. `src/database/query_result.rs`

**Purpose**: Query result type with row iteration.

```rust
/// Query result with rows and metadata.
pub struct QueryResult {
    /// Result rows
    rows: Vec<HashMap<String, String>>,
    
    /// Column names (in order)
    columns: Vec<String>,
    
    /// Execution time in microseconds
    execution_time_us: u64,
    
    /// Whether result was served from cache
    cached: bool,
}

impl QueryResult {
    /// Get all rows.
    pub fn rows(&self) -> &[HashMap<String, String>]
    
    /// Get column names.
    pub fn columns(&self) -> &[String]
    
    /// Get row count.
    pub fn len(&self) -> usize
    
    /// Check if empty.
    pub fn is_empty(&self) -> bool
    
    /// Get execution time.
    pub fn execution_time(&self) -> Duration
    
    /// Check if cached.
    pub fn was_cached(&self) -> bool
}
```

---

### 4. `src/database/execute_result.rs`

**Purpose**: Execute statement result.

```rust
/// Result of INSERT/UPDATE/DELETE statement.
pub struct ExecuteResult {
    /// Number of rows affected
    pub affected_rows: usize,
    
    /// Execution time
    pub execution_time_us: u64,
    
    /// Optional auto-generated ID (for INSERT)
    pub last_insert_id: Option<u64>,
}
```

---

### 5. `src/bin/reedbase.rs`

**Purpose**: CLI tool for database operations.

```rust
//! ReedBase CLI - Command-line database interface.
//!
//! ## Usage
//!
//! ```bash
//! # Open database and run query
//! reedbase query "SELECT * FROM users" ./mydata
//!
//! # Execute statement
//! reedbase exec "INSERT INTO users (name) VALUES ('Alice')" ./mydata
//!
//! # Create table
//! reedbase create-table users ./mydata
//!
//! # List tables
//! reedbase list-tables ./mydata
//!
//! # Interactive mode
//! reedbase shell ./mydata
//! ```

use clap::{Parser, Subcommand};
use reedbase::Database;

#[derive(Parser)]
#[command(name = "reedbase")]
#[command(about = "ReedBase database CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute query and print results
    Query {
        /// ReedQL query
        sql: String,
        
        /// Database path
        #[arg(default_value = ".")]
        path: String,
        
        /// Output format (table, json, csv)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Execute statement (INSERT, UPDATE, DELETE)
    Exec {
        /// ReedQL statement
        sql: String,
        
        /// Database path
        #[arg(default_value = ".")]
        path: String,
    },
    
    /// Create new table
    CreateTable {
        /// Table name
        name: String,
        
        /// Database path
        #[arg(default_value = ".")]
        path: String,
    },
    
    /// Drop table
    DropTable {
        /// Table name
        name: String,
        
        /// Database path
        #[arg(default_value = ".")]
        path: String,
        
        /// Confirm deletion
        #[arg(long)]
        confirm: bool,
    },
    
    /// List all tables
    ListTables {
        /// Database path
        #[arg(default_value = ".")]
        path: String,
    },
    
    /// Create index on column
    CreateIndex {
        /// Table name
        table: String,
        
        /// Column name
        column: String,
        
        /// Database path
        #[arg(default_value = ".")]
        path: String,
    },
    
    /// Interactive shell
    Shell {
        /// Database path
        #[arg(default_value = ".")]
        path: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Query { sql, path, format } => {
            let db = Database::open(&path)?;
            let result = db.query(&sql)?;
            
            match format.as_str() {
                "json" => print_json(&result),
                "csv" => print_csv(&result),
                _ => print_table(&result),
            }
            
            println!("\n{} rows in {:.2}ms", result.len(), result.execution_time().as_secs_f64() * 1000.0);
        }
        Commands::Exec { sql, path } => {
            let db = Database::open(&path)?;
            let result = db.execute(&sql)?;
            println!("Affected rows: {}", result.affected_rows);
        }
        Commands::CreateTable { name, path } => {
            let db = Database::open(&path)?;
            db.create_table(&name, None)?;
            println!("Created table: {}", name);
        }
        Commands::DropTable { name, path, confirm } => {
            if !confirm {
                eprintln!("Error: Must specify --confirm to drop table");
                std::process::exit(1);
            }
            let db = Database::open(&path)?;
            db.drop_table(&name, true)?;
            println!("Dropped table: {}", name);
        }
        Commands::ListTables { path } => {
            let db = Database::open(&path)?;
            let tables = db.list_tables()?;
            for table in tables {
                println!("{}", table);
            }
        }
        Commands::CreateIndex { table, column, path } => {
            let db = Database::open(&path)?;
            db.create_index(&table, &column)?;
            println!("Created index on {}.{}", table, column);
        }
        Commands::Shell { path } => {
            run_interactive_shell(&path)?;
        }
    }
    
    Ok(())
}

/// Print results as formatted table.
fn print_table(result: &QueryResult) {
    // ASCII table formatting
}

/// Print results as JSON.
fn print_json(result: &QueryResult) {
    // JSON output
}

/// Print results as CSV.
fn print_csv(result: &QueryResult) {
    // CSV output
}

/// Run interactive shell.
fn run_interactive_shell(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use rustyline::Editor;
    
    let db = Database::open(path)?;
    let mut rl = Editor::<()>::new()?;
    
    println!("ReedBase Shell - Type 'exit' to quit");
    
    loop {
        let readline = rl.readline("reedbase> ");
        match readline {
            Ok(line) => {
                if line.trim() == "exit" {
                    break;
                }
                
                // Execute query or statement
                if line.trim().to_uppercase().starts_with("SELECT") {
                    match db.query(&line) {
                        Ok(result) => print_table(&result),
                        Err(e) => eprintln!("Error: {:?}", e),
                    }
                } else {
                    match db.execute(&line) {
                        Ok(result) => println!("Affected rows: {}", result.affected_rows),
                        Err(e) => eprintln!("Error: {:?}", e),
                    }
                }
                
                rl.add_history_entry(line.as_str());
            }
            Err(_) => break,
        }
    }
    
    Ok(())
}
```

---

### 6. `src/database/connection.rs`

**Purpose**: Connection pooling and lifecycle management (future).

```rust
/// Database connection pool (for future multi-threaded access).
pub struct ConnectionPool {
    connections: Vec<Database>,
    max_connections: usize,
}

impl ConnectionPool {
    /// Create connection pool.
    pub fn new(path: &str, max_connections: usize) -> ReedResult<Self>
    
    /// Get connection from pool.
    pub fn get(&self) -> ReedResult<Database>
    
    /// Return connection to pool.
    pub fn release(&self, conn: Database)
}
```

---

## Test Files

### `src/database/database_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_open_database() {
        let temp = TempDir::new().unwrap();
        let db = Database::open(temp.path()).unwrap();
        assert!(db.base_path.exists());
    }
    
    #[test]
    fn test_create_table() {
        let temp = TempDir::new().unwrap();
        let db = Database::open(temp.path()).unwrap();
        db.create_table("users", None).unwrap();
        
        let tables = db.list_tables().unwrap();
        assert!(tables.contains(&"users".to_string()));
    }
    
    #[test]
    fn test_query_execution() {
        let temp = TempDir::new().unwrap();
        let db = Database::open(temp.path()).unwrap();
        db.create_table("users", None).unwrap();
        
        db.execute("INSERT INTO users (name, age) VALUES ('Alice', 30)").unwrap();
        
        let result = db.query("SELECT * FROM users WHERE age > 25").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.rows()[0].get("name").unwrap(), "Alice");
    }
    
    #[test]
    fn test_create_index() {
        let temp = TempDir::new().unwrap();
        let db = Database::open(temp.path()).unwrap();
        db.create_table("users", None).unwrap();
        
        db.create_index("users", "age").unwrap();
        
        // Query should use index
        let result = db.query("SELECT * FROM users WHERE age = 30").unwrap();
        assert!(result.was_cached() || result.execution_time().as_micros() < 1000);
    }
    
    #[test]
    fn test_drop_table() {
        let temp = TempDir::new().unwrap();
        let db = Database::open(temp.path()).unwrap();
        db.create_table("temp_table", None).unwrap();
        
        db.drop_table("temp_table", true).unwrap();
        
        let tables = db.list_tables().unwrap();
        assert!(!tables.contains(&"temp_table".to_string()));
    }
}
```

---

## CLI Examples

### Query Execution
```bash
# Simple SELECT
reedbase query "SELECT * FROM users" ./mydata

# With WHERE clause
reedbase query "SELECT name, age FROM users WHERE age > 25" ./mydata

# With aggregation
reedbase query "SELECT COUNT(*), AVG(age) FROM users" ./mydata

# JSON output
reedbase query "SELECT * FROM users" ./mydata --format json

# CSV output
reedbase query "SELECT * FROM users" ./mydata --format csv
```

### Table Management
```bash
# Create table
reedbase create-table users ./mydata

# List tables
reedbase list-tables ./mydata

# Drop table
reedbase drop-table old_data ./mydata --confirm
```

### Data Modification
```bash
# Insert
reedbase exec "INSERT INTO users (name, age) VALUES ('Alice', 30)" ./mydata

# Update
reedbase exec "UPDATE users SET age = 31 WHERE name = 'Alice'" ./mydata

# Delete
reedbase exec "DELETE FROM users WHERE age < 18" ./mydata
```

### Index Management
```bash
# Create index
reedbase create-index users age ./mydata

# Query will automatically use index
reedbase query "SELECT * FROM users WHERE age = 30" ./mydata
```

### Interactive Shell
```bash
reedbase shell ./mydata

# Interactive prompt
reedbase> SELECT * FROM users;
reedbase> INSERT INTO users (name) VALUES ('Bob');
reedbase> exit
```

---

## Performance Requirements

| Operation | Target | Notes |
|-----------|--------|-------|
| Database open (cold) | < 10ms | First open |
| Database open (warm) | < 1ms | Subsequent opens |
| Simple query | < 10ms | 10k rows, no index |
| Indexed query | < 1ms | 10k rows, with index |
| Aggregate query | < 50ms | COUNT/SUM/AVG over 10k rows |
| INSERT (single) | < 5ms | Including versioning |
| INSERT (bulk 100) | < 50ms | Batch operation |
| CREATE INDEX | < 500ms | 10k rows |

---

## Error Conditions

| Error | Condition | Recovery |
|-------|-----------|----------|
| `DatabaseNotFound` | Path does not exist | Create with `Database::open()` |
| `TableNotFound` | Table does not exist in query | Check `list_tables()` |
| `ColumnNotFound` | Column not in table | Check schema |
| `ParseError` | Invalid SQL syntax | Fix query syntax |
| `PermissionDenied` | Insufficient file permissions | Fix directory permissions |
| `LockTimeout` | Could not acquire lock | Retry or check for deadlock |

---

## Acceptance Criteria

- [ ] `Database::open()` creates/opens database
- [ ] `Database::query()` executes SELECT statements
- [ ] `Database::execute()` executes INSERT/UPDATE/DELETE
- [ ] `Database::create_table()` creates new tables
- [ ] `Database::drop_table()` removes tables
- [ ] `Database::list_tables()` shows all tables
- [ ] `Database::create_index()` builds smart indices
- [ ] `reedbase` CLI tool with all commands
- [ ] Interactive shell with history
- [ ] Output formatting (table, JSON, CSV)
- [ ] All tests pass with 100% coverage
- [ ] Documentation complete with examples
- [ ] Performance targets met
- [ ] Error handling comprehensive
- [ ] All code in BBC English

---

## Dependencies

- **Requires**: All REED-19 tickets (01-12, 20-23)
- **Blocks**: REED-19-13 (Migration), REED-19-14 (Benchmarks), REED-19-15 (Documentation)

---

## References

- ReedQL Documentation: `REED-19-10`
- Smart Indices: `REED-19-09`
- Table API: `REED-19-02`
- Service Template: `_workbench/Tickets/templates/service-template.md`

---

## Notes

### Why This Ticket Is Critical

ReedBase currently has all components but no way for users to access them:
- Tables exist but only work with raw bytes
- ReedQL exists but has no database connection
- Smart Indices exist but no automatic integration
- Like having a car with engine, wheels, steering separately but no way to drive

This ticket connects everything into a usable database system.

### Design Decisions

**Why `Database` not `Connection`?**
- Simpler name for single-file database
- Connection suggests network/client-server
- Database is more intuitive for file-based system

**Why separate `query()` and `execute()`?**
- Clear distinction: `query()` returns data, `execute()` modifies data
- Follows PostgreSQL/MySQL conventions
- Type-safe return values

**Why CLI in same crate?**
- Shares code with library
- Single binary installation
- Easier testing and debugging

**Why no HTTP/REST API?**
- Out of scope for this ticket
- Would be REED-19-25 or later
- CLI + Library API is sufficient for v1
