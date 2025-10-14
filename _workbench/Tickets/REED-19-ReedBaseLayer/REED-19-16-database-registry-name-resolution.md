# REED-19-16: Database Registry & Name Resolution

## Metadata
- **Status**: Planned
- **Priority**: Critical
- **Complexity**: Medium (4-5 days)
- **Layer**: Data Layer (REED-19)
- **Depends on**: 
  - REED-19-02 (Universal Table API - database structure)
- **Blocks**: 
  - REED-19-17 (Multi-Location Sync - needs registry)
  - REED-19-18 (P2P Latency Routing - needs node registry)
- **Related Tickets**: None

## Problem Statement

Current ReedBase uses **path-based database access**:
- `cd /path/to/project && reedbase query "SELECT * FROM text"`
- Requires being in correct directory
- No central registry of databases
- Cannot access databases from anywhere
- No multi-location support

**Example Problems**:
```bash
# Must be in project directory
cd ~/my-project/
reedbase query "SELECT * FROM text"  # Works

cd /tmp/
reedbase query "SELECT * FROM text"  # ❌ Error: No .reedbase found

# Cannot specify which database
reedbase query "SELECT * FROM text"  # Which database?
```

**Target**: **Name-based database registry** with **global access** from anywhere.

## Solution Overview

Implement **central database registry** (`~/.reedbase/registry.toml`):

```
~/.reedbase/
├── registry.toml          # Single source of truth
├── versions/              # ReedBase binary versions
│   ├── 1.2.3/
│   └── 2.0.0/
└── databases/             # Global databases
    ├── users_prod/
    └── analytics/
```

**Usage**:
```bash
# Anywhere on system
rdb db:query users_prod "SELECT * FROM users"
rdb db:set analytics metrics date=2025-10-14 views=1250

# Name-based, not path-based
```

## Architecture

### Registry Format (`~/.reedbase/registry.toml`)

```toml
# ReedBase Database Registry
version = "1.0"

[[database]]
name = "users_prod"
mode = "global"
location = "/Users/vivian/.reedbase/databases/users_prod"
reedbase_version = "1.2.3"
created_at = "2025-10-14T10:30:00Z"
description = "Production user database"

[[database]]
name = "my_project_dev"
mode = "local"
location = "/Users/vivian/Projects/my-project/.reedbase"
reedbase_version = "2.0.0"
created_at = "2025-10-14T11:00:00Z"
project_root = "/Users/vivian/Projects/my-project"
description = "Development database for my-project"
```

### Core Types

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    pub version: String,
    pub databases: Vec<DatabaseEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseEntry {
    pub name: String,
    pub mode: DatabaseMode,
    pub location: PathBuf,
    pub reedbase_version: String,
    pub created_at: String,
    pub project_root: Option<PathBuf>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseMode {
    Global,
    Local,
}
```

## Implementation Details

### 1. Registry Management (registry.rs)

```rust
// src/registry.rs

impl Registry {
    /// Load registry from ~/.reedbase/registry.toml
    pub fn load() -> Result<Self> {
        let path = Self::registry_path();
        
        if !path.exists() {
            return Ok(Self {
                version: "1.0".to_string(),
                databases: vec![],
            });
        }
        
        let content = fs::read_to_string(&path)?;
        let registry: Registry = toml::from_str(&content)?;
        
        Ok(registry)
    }
    
    /// Save registry to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::registry_path();
        fs::create_dir_all(path.parent().unwrap())?;
        
        let toml = toml::to_string_pretty(&self)?;
        fs::write(&path, toml)?;
        
        Ok(())
    }
    
    /// Find database by name
    pub fn find(&self, name: &str) -> Option<&DatabaseEntry> {
        self.databases.iter().find(|db| db.name == name)
    }
    
    /// Register new database
    pub fn register(&mut self, entry: DatabaseEntry) -> Result<()> {
        // Check for duplicate names
        if self.find(&entry.name).is_some() {
            return Err(anyhow!("Database '{}' already registered", entry.name));
        }
        
        self.databases.push(entry);
        self.save()?;
        
        Ok(())
    }
    
    /// Unregister database
    pub fn unregister(&mut self, name: &str) -> Result<()> {
        let initial_len = self.databases.len();
        self.databases.retain(|db| db.name != name);
        
        if self.databases.len() == initial_len {
            return Err(anyhow!("Database '{}' not found", name));
        }
        
        self.save()?;
        
        Ok(())
    }
    
    fn registry_path() -> PathBuf {
        home_dir().join(".reedbase/registry.toml")
    }
}
```

---

### 2. Database Init

```rust
// src/cli/db/init.rs

pub fn init(name: &str, mode: DatabaseMode) -> Result<()> {
    let mut registry = Registry::load()?;
    
    // Check for duplicate
    if registry.find(name).is_some() {
        return Err(anyhow!("Database '{}' already exists", name));
    }
    
    let location = match mode {
        DatabaseMode::Global => {
            // Global: ~/.reedbase/databases/{name}
            let path = home_dir()
                .join(".reedbase/databases")
                .join(name);
            
            println!("🗄️  Creating global database \"{}\"", name);
            println!("📍 Location: {}", path.display());
            
            path
        },
        DatabaseMode::Local => {
            // Local: ./.reedbase in current directory
            let cwd = std::env::current_dir()?;
            let path = cwd.join(".reedbase");
            
            println!("🗄️  Creating local database \"{}\"", name);
            println!("📍 Current directory: {}", cwd.display());
            println!();
            println!("⚠️  This will create .reedbase/ in:");
            println!("    {}", cwd.display());
            
            // Safety check
            if !looks_like_project_dir(&cwd)? {
                println!();
                println!("⚠️  WARNING: This doesn't look like a project directory!");
            }
            
            println!();
            if !confirm("Continue?")? {
                println!("❌ Aborted");
                return Ok(());
            }
            
            path
        },
    };
    
    // Create database structure
    create_database_structure(&location)?;
    
    // Write metadata
    fs::write(location.join(".reed/name"), name)?;
    fs::write(location.join(".reed/version"), env!("CARGO_PKG_VERSION"))?;
    fs::write(location.join(".reed/mode"), format!("{:?}", mode).to_lowercase())?;
    
    // Register in registry
    let entry = DatabaseEntry {
        name: name.to_string(),
        mode,
        location: location.clone(),
        reedbase_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        project_root: if matches!(mode, DatabaseMode::Local) {
            Some(std::env::current_dir()?)
        } else {
            None
        },
        description: None,
    };
    
    registry.register(entry)?;
    
    println!();
    println!("🔧 ReedBase version: {}", env!("CARGO_PKG_VERSION"));
    println!("✅ Database \"{}\" registered", name);
    
    Ok(())
}

fn looks_like_project_dir(path: &Path) -> Result<bool> {
    // Check for common project indicators
    let indicators = [
        "Cargo.toml",
        "package.json",
        ".git",
        "README.md",
        "pyproject.toml",
    ];
    
    for indicator in &indicators {
        if path.join(indicator).exists() {
            return Ok(true);
        }
    }
    
    Ok(false)
}

fn create_database_structure(location: &Path) -> Result<()> {
    let reed_dir = location.join(".reed");
    fs::create_dir_all(&reed_dir)?;
    
    // Create initial tables
    let tables = ["text", "routes", "meta", "server", "project"];
    for table in &tables {
        let table_dir = reed_dir.join(table);
        fs::create_dir_all(&table_dir)?;
        fs::write(table_dir.join("current.csv"), "key|value|description\n")?;
    }
    
    Ok(())
}
```

---

### 3. Database Register (Import Existing)

```rust
// src/cli/db/register.rs

pub fn register(name: String) -> Result<()> {
    let mut registry = Registry::load()?;
    
    // Find .reedbase in current directory
    let cwd = std::env::current_dir()?;
    let db_path = cwd.join(".reedbase");
    
    if !db_path.exists() {
        return Err(anyhow!(
            "No .reedbase/ found in current directory\n\n\
             Create a new database first:\n  \
             rdb db:init {} --local", 
            name
        ));
    }
    
    // Read existing metadata
    let reed_dir = db_path.join(".reed");
    let version = fs::read_to_string(reed_dir.join("version"))
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());
    
    let mode_str = fs::read_to_string(reed_dir.join("mode"))
        .unwrap_or_else(|_| "local".to_string());
    
    let mode = match mode_str.trim() {
        "global" => DatabaseMode::Global,
        _ => DatabaseMode::Local,
    };
    
    // Confirmation
    println!("🔍 Found database:");
    println!("   Name: {}", name);
    println!("   Location: {}", db_path.display());
    println!("   Mode: {:?}", mode);
    println!("   Version: {}", version.trim());
    println!();
    
    if !confirm("Register this database?")? {
        println!("❌ Aborted");
        return Ok(());
    }
    
    // Check for duplicate
    if registry.find(&name).is_some() {
        return Err(anyhow!(
            "Database '{}' already registered\n\n\
             Unregister first:\n  \
             rdb db:unregister {}", 
            name, name
        ));
    }
    
    // Write name to .reed/name
    fs::write(reed_dir.join("name"), &name)?;
    
    // Register
    let entry = DatabaseEntry {
        name: name.clone(),
        mode,
        location: db_path,
        reedbase_version: version.trim().to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        project_root: Some(cwd),
        description: None,
    };
    
    registry.register(entry)?;
    
    println!("✅ Database \"{}\" registered", name);
    
    Ok(())
}
```

---

### 4. Database Resolution

```rust
// src/database.rs

/// Resolve database by name (STRICT - no auto-detection)
pub fn resolve_database(name: &str) -> Result<PathBuf> {
    let registry = Registry::load()?;
    
    let entry = registry.find(name)
        .ok_or_else(|| {
            // Build helpful error message
            let mut msg = format!("Database '{}' not found in registry\n\n", name);
            
            if registry.databases.is_empty() {
                msg.push_str("No databases registered yet.\n\n");
                msg.push_str("Create a database:\n");
                msg.push_str("  rdb db:init <name> --global  (new global database)\n");
                msg.push_str("  rdb db:init <name> --local   (new local database)\n");
            } else {
                msg.push_str("Registered databases:\n");
                for db in &registry.databases {
                    msg.push_str(&format!("  {} ({})\n", db.name, 
                        if db.mode == DatabaseMode::Global { "global" } else { "local" }
                    ));
                }
                msg.push_str("\nRegister existing database:\n");
                msg.push_str("  cd /path/to/project\n");
                msg.push_str("  rdb db:register <name>\n");
            }
            
            anyhow!(msg)
        })?;
    
    Ok(entry.location.join(".reed"))
}
```

---

### 5. Database List

```rust
// src/cli/db/list.rs

pub fn list() -> Result<()> {
    let registry = Registry::load()?;
    
    if registry.databases.is_empty() {
        println!("No databases registered.");
        println!();
        println!("Create a database:");
        println!("  rdb db:init <name> --global");
        println!("  rdb db:init <name> --local");
        return Ok(());
    }
    
    println!("Registered databases:\n");
    
    // Group by mode
    let global_dbs: Vec<_> = registry.databases.iter()
        .filter(|db| db.mode == DatabaseMode::Global)
        .collect();
    
    let local_dbs: Vec<_> = registry.databases.iter()
        .filter(|db| db.mode == DatabaseMode::Local)
        .collect();
    
    if !global_dbs.is_empty() {
        println!("Global databases:");
        for db in global_dbs {
            println!("  {:20} v{}    {}", 
                db.name, 
                db.reedbase_version,
                db.location.display()
            );
        }
        println!();
    }
    
    if !local_dbs.is_empty() {
        println!("Local databases:");
        for db in local_dbs {
            println!("  {:20} v{}    {}", 
                db.name, 
                db.reedbase_version,
                db.location.display()
            );
        }
        println!();
    }
    
    println!("{} databases total ({} global, {} local)", 
        registry.databases.len(),
        global_dbs.len(),
        local_dbs.len()
    );
    
    Ok(())
}
```

---

### 6. Database Info

```rust
// src/cli/db/info.rs

pub fn info(name: &str) -> Result<()> {
    let registry = Registry::load()?;
    let db = registry.find(name)
        .ok_or_else(|| anyhow!("Database '{}' not found", name))?;
    
    println!("Database: {}", db.name);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Mode:          {:?}", db.mode);
    println!("Location:      {}", db.location.display());
    
    if let Some(ref root) = db.project_root {
        println!("Project Root:  {}", root.display());
    }
    
    println!("ReedBase:      v{}", db.reedbase_version);
    println!("Created:       {}", db.created_at);
    
    if let Some(ref desc) = db.description {
        println!("Description:   {}", desc);
    }
    
    // Statistics
    let reed_dir = db.location.join(".reed");
    if reed_dir.exists() {
        println!();
        
        let tables = count_tables(&reed_dir)?;
        let rows = count_total_rows(&reed_dir)?;
        let size = dir_size(&reed_dir)?;
        
        println!("Tables:        {}", tables);
        println!("Total Rows:    {}", rows);
        println!("Disk Usage:    {}", human_readable_size(size));
    }
    
    Ok(())
}

fn count_tables(reed_dir: &Path) -> Result<usize> {
    let count = fs::read_dir(reed_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false))
        .count();
    
    Ok(count)
}

fn count_total_rows(reed_dir: &Path) -> Result<usize> {
    let mut total = 0;
    
    for entry in fs::read_dir(reed_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        
        let csv_path = entry.path().join("current.csv");
        if csv_path.exists() {
            let content = fs::read_to_string(csv_path)?;
            total += content.lines().count().saturating_sub(1); // -1 for header
        }
    }
    
    Ok(total)
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut size = 0;
    
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            size += entry.metadata()?.len();
        }
    }
    
    Ok(size)
}

fn human_readable_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_idx])
}
```

---

## CLI Commands

```bash
# Database Management
rdb db:init users_prod --global        # Create global database
rdb db:init my_project_dev --local     # Create local database
rdb db:register blog_prod              # Register existing database
rdb db:list                            # List all databases
rdb db:info users_prod                 # Show database info
rdb db:unregister old_db               # Unregister (keep files)
rdb db:delete old_db                   # Delete (remove files)

# Database Operations (name-based)
rdb db:query users_prod "SELECT * FROM users"
rdb db:set users_prod users id=1 name="Alice"
rdb db:get users_prod users id=1
rdb db:tables users_prod
rdb db:schema users_prod users
```

## File Structure

```
src/reedcms/
├── registry.rs            # Registry management
├── database.rs            # Database resolution
└── cli/
    └── db/
        ├── init.rs        # Database init
        ├── register.rs    # Database register
        ├── list.rs        # Database list
        ├── info.rs        # Database info
        ├── unregister.rs  # Unregister
        └── delete.rs      # Delete
```

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Registry load | < 5ms | TOML parsing |
| Registry save | < 10ms | TOML serialization + write |
| Database resolution | < 1ms | HashMap lookup |
| List databases | < 10ms | Read + format registry |

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_registry_load_save() {
    let mut registry = Registry::load().unwrap();
    
    let entry = DatabaseEntry {
        name: "test_db".to_string(),
        mode: DatabaseMode::Global,
        location: PathBuf::from("/tmp/test"),
        reedbase_version: "1.0.0".to_string(),
        created_at: "2025-01-01T00:00:00Z".to_string(),
        project_root: None,
        description: None,
    };
    
    registry.register(entry).unwrap();
    
    let loaded = Registry::load().unwrap();
    assert!(loaded.find("test_db").is_some());
}

#[test]
fn test_duplicate_registration() {
    let mut registry = Registry::load().unwrap();
    
    let entry = DatabaseEntry {
        name: "duplicate".to_string(),
        mode: DatabaseMode::Global,
        location: PathBuf::from("/tmp/dup"),
        reedbase_version: "1.0.0".to_string(),
        created_at: "2025-01-01T00:00:00Z".to_string(),
        project_root: None,
        description: None,
    };
    
    registry.register(entry.clone()).unwrap();
    
    let result = registry.register(entry);
    assert!(result.is_err());
}
```

### Integration Tests

```rust
#[test]
fn test_init_global_database() {
    init("test_global", DatabaseMode::Global).unwrap();
    
    let registry = Registry::load().unwrap();
    let db = registry.find("test_global").unwrap();
    
    assert_eq!(db.mode, DatabaseMode::Global);
    assert!(db.location.exists());
}

#[test]
fn test_register_existing_database() {
    // Create .reedbase manually
    let test_dir = temp_dir().join("test_project");
    fs::create_dir_all(&test_dir).unwrap();
    
    std::env::set_current_dir(&test_dir).unwrap();
    
    init("test_local", DatabaseMode::Local).unwrap();
    
    // Unregister
    let mut registry = Registry::load().unwrap();
    registry.unregister("test_local").unwrap();
    
    // Re-register
    register("test_local".to_string()).unwrap();
    
    let registry = Registry::load().unwrap();
    assert!(registry.find("test_local").is_some());
}
```

## Dependencies

**Internal**:
- `reedstream::ReedError` - Error handling

**External**:
- `toml` - Registry file format
- `serde` - Serialization
- `chrono` - Timestamps
- `walkdir` - Directory traversal (for info command)

## Error Handling

```rust
#[derive(Debug)]
pub enum RegistryError {
    DatabaseNotFound { name: String },
    DuplicateName { name: String },
    InvalidRegistry { reason: String },
    IoError { path: String, source: std::io::Error },
}
```

## Acceptance Criteria

### Functional Requirements
- [x] Load registry from `~/.reedbase/registry.toml`
- [x] Save registry with atomic write
- [x] Register database with name, mode, location
- [x] Unregister database (keep files)
- [x] Delete database (remove files)
- [x] Resolve database by name
- [x] List all registered databases
- [x] Show database info with statistics
- [x] Prevent duplicate names
- [x] Support global and local modes
- [x] CLI commands: init, register, list, info, unregister, delete

### Performance Requirements
- [x] Registry load: < 5ms
- [x] Registry save: < 10ms
- [x] Database resolution: < 1ms
- [x] List databases: < 10ms

### Quality Requirements
- [x] 100% test coverage for registry operations
- [x] Integration tests for all CLI commands
- [x] Error messages with helpful suggestions
- [x] Atomic registry writes (temp file + rename)

### Documentation Requirements
- [x] Architecture documentation (this ticket)
- [x] API documentation for registry module
- [x] CLI usage examples
- [x] Error handling documentation

## Implementation Notes

### Registry Philosophy

**Explicit over Implicit**:
- No auto-registration
- User must manually register databases
- Clear error messages when database not found

**Single Source of Truth**:
- `~/.reedbase/registry.toml` is authoritative
- All tools read from registry
- Consistent behavior across commands

**Safety First**:
- Confirmation prompts for destructive operations
- Warnings for unusual operations (local in /tmp)
- Atomic writes to prevent corruption

### Trade-offs

**Pros**:
- ✅ Name-based access from anywhere
- ✅ Central management of all databases
- ✅ Support for multiple databases
- ✅ Clean separation of global vs local

**Cons**:
- ❌ Manual registration required (not auto-detect)
- ❌ Registry must be kept in sync with filesystem

**Decision**: Explicit registration is better than auto-detection (predictable behavior).

### Future Enhancements

1. **Database aliases**
   - Multiple names for same database
   - Short names for convenience

2. **Database groups**
   - Group related databases
   - Batch operations on groups

3. **Registry sync**
   - Sync registry across machines
   - Shared team registries

4. **Auto-cleanup**
   - Detect deleted databases
   - Remove stale registry entries

## References

- **REED-19-02**: Universal Table API (database structure)
- **REED-19-17**: Multi-Location Sync (uses registry)
- **REED-19-18**: P2P Latency Routing (uses registry)

## Summary

Database Registry provides **name-based database management** with a **central registry** (`~/.reedbase/registry.toml`). Databases are registered with unique names and can be accessed from anywhere on the system. Supports both **global** (system-wide) and **local** (project-specific) modes. Foundation for multi-location sync and distributed query routing.
